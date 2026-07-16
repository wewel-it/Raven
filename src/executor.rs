use crate::error::{RavenError, RavenResult};
use crate::event::{AgentEvent, EventBus};
use crate::llm::Llm;
use crate::memory::{MemoryKind, MemoryStorage};
use crate::planner::{ExecutionPlan, PlannerProgress, Step};
use crate::tool::ToolManagerService;
use log::{error, info};
use serde_json::Value;
use std::sync::{Arc, Mutex};

/// Executor service interface.
pub trait Executor: Send + Sync {
    fn execute_plan(&self, plan: &ExecutionPlan) -> RavenResult<String>;
}

/// ExecutorService runs an ExecutionPlan using injected services and an LLM.
pub struct ExecutorService {
    memory: Arc<Mutex<Box<dyn MemoryStorage>>>,
    tools: Arc<Mutex<Box<dyn ToolManagerService>>>,
    llm: Arc<dyn Llm + Send + Sync>,
    planner: Arc<dyn PlannerProgress + Send + Sync>,
    event_bus: Arc<EventBus>,
}

impl ExecutorService {
    pub fn new(
        memory: Arc<Mutex<Box<dyn MemoryStorage>>>,
        tools: Arc<Mutex<Box<dyn ToolManagerService>>>,
        llm: Arc<dyn Llm + Send + Sync>,
        planner: Arc<dyn PlannerProgress + Send + Sync>,
        event_bus: Arc<EventBus>,
    ) -> Self {
        Self {
            memory,
            tools,
            llm,
            planner,
            event_bus,
        }
    }

    /// Execute an entire plan and return an aggregated string result.
    pub fn execute_plan(&self, plan: &ExecutionPlan) -> RavenResult<String> {
        let mut outputs: Vec<String> = Vec::new();

        for step in &plan.steps {
            info!("Executing step {}", step.id);
            let _ = self.event_bus.publish(AgentEvent::TaskStarted {
                task_id: step.id.clone(),
                description: step.description.clone(),
            });

            // attempt loop with retry policy decided by planner
            loop {
                if let Err(e) = self.planner.mark_step_started(&step.id) {
                    error!("failed to mark step started {}: {}", step.id, e);
                }

                let res = self.execute_step(step);
                match res {
                    Ok(s) => {
                        if let Err(e) = self.planner.mark_step_completed(&step.id) {
                            error!("failed to mark step completed {}: {}", step.id, e);
                        }
                        let memory_id = match self
                            .memory
                            .lock()
                            .map_err(|e| RavenError::LockPoisoned(e.to_string()))?
                            .add(MemoryKind::Working, &s, &[step.id.as_str()])
                        {
                            Ok(id) => {
                                let _ = self.event_bus.publish(AgentEvent::MemoryUpdated {
                                    memory_id: id.clone(),
                                    kind: MemoryKind::Working,
                                    tags: vec![step.id.clone()],
                                    text: s.clone(),
                                });
                                Some(id)
                            }
                            Err(e) => {
                                error!("failed to store memory: {}", e);
                                None
                            }
                        };
                        outputs.push(s.clone());
                        let _ = self.event_bus.publish(AgentEvent::TaskCompleted {
                            task_id: step.id.clone(),
                            output: s.clone(),
                            memory_id,
                        });
                        break;
                    }
                    Err(err) => {
                        error!("step {} failed attempt: {}", step.id, err);
                        let err_msg = err.to_string();
                        if let Err(e) = self.planner.mark_step_failed(&step.id, &err_msg) {
                            error!("failed to mark step failed {}: {}", step.id, e);
                        }

                        // consult planner whether to retry
                        match self.planner.should_retry(&step.id) {
                            Ok(true) => {
                                if let Ok((attempt_no, backoff)) =
                                    self.planner.next_retry_backoff(&step.id)
                                {
                                    info!(
                                        "Retrying step {} attempt {} backoff {}",
                                        step.id, attempt_no, backoff
                                    );
                                } else {
                                    info!("Retrying step {} (attempt info unavailable)", step.id);
                                }
                                // loop to retry (deterministic; no sleeping)
                                continue;
                            }
                            Ok(false) => {
                                let _ = self.event_bus.publish(AgentEvent::TaskFailed {
                                    task_id: step.id.clone(),
                                    error: err_msg.clone(),
                                });
                                // exceeded retries => treat as failure
                                // if critical, stop workflow
                                if step.priority >= 8 {
                                    error!("fatal failure on critical step {}", step.id);
                                    // attempt replanning for record-keeping
                                    let mut cloned_plan = plan.clone();
                                    let _ =
                                        self.planner.replan_on_failure(&mut cloned_plan, &step.id);
                                    return Err(RavenError::Executor(format!(
                                        "fatal failure on step {}: {}",
                                        step.id, err_msg
                                    )));
                                } else {
                                    info!(
                                        "Non-critical step {} failed and will be skipped",
                                        step.id
                                    );
                                    outputs.push(format!("[error:{}]", err_msg));
                                    break;
                                }
                            }
                            Err(e) => {
                                error!("planner error while checking retry for {}: {}", step.id, e);
                                let _ = self.event_bus.publish(AgentEvent::TaskFailed {
                                    task_id: step.id.clone(),
                                    error: e.to_string(),
                                });
                                return Err(e);
                            }
                        }
                    }
                }
            }
        }

        // Optionally consolidate memory after execution
        if let Err(e) = self
            .memory
            .lock()
            .map_err(|e| RavenError::LockPoisoned(e.to_string()))?
            .consolidate()
        {
            error!("memory consolidation failed: {}", e);
        }

        Ok(outputs.join("\n"))
    }

    fn execute_step(&self, step: &Step) -> RavenResult<String> {
        if step.needs_tool {
            let tool_name = step.tool_name.as_deref().unwrap_or("echo");
            let params = step.params.as_ref().unwrap_or(&Value::Null);
            let context = crate::tool::ExecutionContext::new().with_caller("executor");
            let _ = self.event_bus.publish(AgentEvent::ToolCalled {
                tool_name: tool_name.to_string(),
                params: params.clone(),
            });
            let outcome = match self
                .tools
                .lock()
                .map_err(|e| RavenError::LockPoisoned(e.to_string()))?
                .invoke(tool_name, params, &context)
            {
                Ok(result) => {
                    let _ = self.event_bus.publish(AgentEvent::ToolCompleted {
                        tool_name: tool_name.to_string(),
                        result: result.data.clone(),
                    });
                    Ok(format!("tool_result: {}", result.data))
                }
                Err(e) => Err(RavenError::Tool(e)),
            };
            outcome
        } else {
            // Use LLM to handle content generation or processing
            let prompt = &step.description;
            let context = step.params.as_ref();
            self.llm.generate(prompt, context)
        }
    }
}

impl Executor for ExecutorService {
    fn execute_plan(&self, plan: &ExecutionPlan) -> RavenResult<String> {
        ExecutorService::execute_plan(self, plan)
    }
}
