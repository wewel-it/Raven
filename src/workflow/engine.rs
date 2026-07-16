use crate::error::{RavenError, RavenResult};
use crate::event::{AgentEvent, EventBus};
use crate::executor::Executor;
use crate::memory::MemoryStorage;
use crate::planner::{ExecutionPlan, PlannerProgress};
use crate::reflection::ReflectionEvaluator;
use crate::workflow::state::{WorkflowState, WorkflowStatus};
use std::sync::{Arc, Mutex};

/// Workflow controller trait for orchestration management.
pub trait WorkflowController: Send + Sync {
    fn start(&self, plan: ExecutionPlan) -> RavenResult<String>;
    fn pause(&self) -> RavenResult<()>;
    fn resume(&self) -> RavenResult<()>;
    fn cancel(&self) -> RavenResult<()>;
    fn retry(&self) -> RavenResult<String>;
    fn continue_workflow(&self) -> RavenResult<String>;
    fn status(&self) -> WorkflowStatus;
}

/// The workflow service manages execution of plans and exposes lifecycle controls.
pub struct WorkflowService {
    planner: Arc<dyn PlannerProgress + Send + Sync>,
    memory: Arc<Mutex<Box<dyn MemoryStorage>>>,
    tools: Arc<Mutex<Box<dyn crate::tool::ToolManagerService>>>,
    llm: Arc<dyn crate::llm::Llm + Send + Sync>,
    reflection: Arc<dyn ReflectionEvaluator>,
    executor: Arc<dyn Executor>,
    event_bus: Arc<EventBus>,
    status: Arc<Mutex<WorkflowStatus>>,
    /// Persistent execution plan snapshot.
    plan: Arc<Mutex<Option<ExecutionPlan>>>,
}

impl WorkflowService {
    fn workflow_id_for_plan(plan: &ExecutionPlan) -> String {
        plan.steps
            .first()
            .map(|step| format!("workflow-{}", step.id))
            .unwrap_or_else(|| "workflow-unknown".to_string())
    }

    pub fn new(
        planner: Arc<dyn PlannerProgress + Send + Sync>,
        memory: Arc<Mutex<Box<dyn MemoryStorage>>>,
        tools: Arc<Mutex<Box<dyn crate::tool::ToolManagerService>>>,
        llm: Arc<dyn crate::llm::Llm + Send + Sync>,
        reflection: Arc<dyn ReflectionEvaluator>,
        executor: Arc<dyn Executor>,
        event_bus: Arc<EventBus>,
    ) -> Self {
        Self {
            planner,
            memory,
            tools,
            llm,
            reflection,
            executor,
            event_bus,
            status: Arc::new(Mutex::new(WorkflowStatus::new())),
            plan: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start(&self, plan: ExecutionPlan) -> RavenResult<String> {
        // reference injected but not-yet-used components to avoid unused-field warnings
        let _ = &self.planner;
        let _ = &self.tools;
        let _ = &self.llm;

        let workflow_id = Self::workflow_id_for_plan(&plan);

        let mut plan_guard = self
            .plan
            .lock()
            .map_err(|e| RavenError::LockPoisoned(e.to_string()))?;
        *plan_guard = Some(plan.clone());

        let mut status_guard = self
            .status
            .lock()
            .map_err(|e| RavenError::LockPoisoned(e.to_string()))?;
        status_guard.update_state(WorkflowState::Pending);
        status_guard.mark_current_step(None, None);
        drop(status_guard);

        let _ = self.event_bus.publish(AgentEvent::WorkflowStarted {
            workflow_id: workflow_id.clone(),
        });
        let result = self.execute(plan);
        let _ = self.event_bus.publish(AgentEvent::WorkflowFinished {
            workflow_id,
            result: result
                .as_ref()
                .map(|s| s.clone())
                .map_err(|e| e.to_string()),
        });
        result
    }

    pub fn pause(&self) -> RavenResult<()> {
        let mut status_guard = self
            .status
            .lock()
            .map_err(|e| RavenError::LockPoisoned(e.to_string()))?;
        if status_guard.state == WorkflowState::Running
            || status_guard.state == WorkflowState::Waiting
        {
            status_guard.update_state(WorkflowState::Waiting);
            Ok(())
        } else {
            Err(RavenError::Workflow(format!(
                "cannot pause workflow in state {:?}",
                status_guard.state
            )))
        }
    }

    pub fn resume(&self) -> RavenResult<()> {
        let mut status_guard = self
            .status
            .lock()
            .map_err(|e| RavenError::LockPoisoned(e.to_string()))?;
        if status_guard.state == WorkflowState::Waiting {
            status_guard.update_state(WorkflowState::Running);
            Ok(())
        } else {
            Err(RavenError::Workflow(format!(
                "cannot resume workflow in state {:?}",
                status_guard.state
            )))
        }
    }

    pub fn cancel(&self) -> RavenResult<()> {
        let mut status_guard = self
            .status
            .lock()
            .map_err(|e| RavenError::LockPoisoned(e.to_string()))?;
        if status_guard.state != WorkflowState::Completed
            && status_guard.state != WorkflowState::Cancelled
        {
            status_guard.update_state(WorkflowState::Cancelled);
            Ok(())
        } else {
            Err(RavenError::Workflow(format!(
                "cannot cancel workflow in state {:?}",
                status_guard.state
            )))
        }
    }

    pub fn retry(&self) -> RavenResult<String> {
        let current_plan = self
            .plan
            .lock()
            .map_err(|e| RavenError::LockPoisoned(e.to_string()))?
            .clone();
        if let Some(plan) = current_plan {
            let mut status_guard = self
                .status
                .lock()
                .map_err(|e| RavenError::LockPoisoned(e.to_string()))?;
            status_guard.update_state(WorkflowState::Retrying);
            status_guard.failed_step = None;
            status_guard.last_error = None;
            drop(status_guard);
            self.execute(plan)
        } else {
            Err(RavenError::Workflow(
                "no workflow plan available to retry".into(),
            ))
        }
    }

    pub fn continue_workflow(&self) -> RavenResult<String> {
        let current_plan = self
            .plan
            .lock()
            .map_err(|e| RavenError::LockPoisoned(e.to_string()))?
            .clone();
        if let Some(plan) = current_plan {
            let workflow_id = Self::workflow_id_for_plan(&plan);
            let _ = self.event_bus.publish(AgentEvent::WorkflowStarted {
                workflow_id: workflow_id.clone(),
            });
            let result = self.execute(plan);
            let _ = self.event_bus.publish(AgentEvent::WorkflowFinished {
                workflow_id,
                result: result
                    .as_ref()
                    .map(|s| s.clone())
                    .map_err(|e| e.to_string()),
            });
            result
        } else {
            Err(RavenError::Workflow(
                "no workflow plan available to continue".into(),
            ))
        }
    }

    pub fn status(&self) -> RavenResult<WorkflowStatus> {
        let guard = self
            .status
            .lock()
            .map_err(|e| RavenError::LockPoisoned(e.to_string()))?;
        Ok(guard.clone())
    }

    fn execute(&self, plan: ExecutionPlan) -> RavenResult<String> {
        {
            let mut status_guard = self
                .status
                .lock()
                .map_err(|e| RavenError::LockPoisoned(e.to_string()))?;
            status_guard.update_state(WorkflowState::Running);
            status_guard.mark_current_step(None, None);
        }

        let result = self.executor.execute_plan(&plan);

        let mut status_guard = self
            .status
            .lock()
            .map_err(|e| RavenError::LockPoisoned(e.to_string()))?;
        match &result {
            Ok(_) => {
                status_guard.update_state(WorkflowState::Completed);
            }
            Err(err) => {
                status_guard.set_failed("workflow".to_string(), err.to_string());
            }
        }

        let status_snapshot = status_guard.clone();
        drop(status_guard);

        let report = self.reflection.evaluate("workflow", &status_snapshot);
        let manager = self
            .memory
            .lock()
            .map_err(|e| RavenError::LockPoisoned(e.to_string()))?;
        let _ = self.reflection.commit(manager.as_ref(), report);

        result
    }
}
