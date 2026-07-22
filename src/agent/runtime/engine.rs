use super::dispatcher::Dispatcher;
use super::events::RuntimeEvents;
use super::recovery::RecoveryManager;
use super::report::RuntimeReport;
use super::retry::RetryManager;
use super::scheduler::Scheduler;
use super::traits::WorkflowFactory;
use crate::agent::runtime::context::RuntimeContext;
use crate::agent::runtime::metrics::RuntimeMetricsCollector;
use crate::error::{RavenError, RavenResult};
use crate::llm::Llm;
use crate::memory::{MemoryKind, MemoryStorage};
use crate::planner::ExecutionPlan;
use crate::planner::PlannerProgress;
use crate::reflection::ReflectionEvaluator;
use crate::tool::ToolManagerService;
use log::{error, info};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// AgentRuntimeService: orchestrates subsystems to run a plan end-to-end.
pub struct AgentRuntimeService {
    events: RuntimeEvents,
    dispatcher: Dispatcher,
    planner: Arc<dyn PlannerProgress + Send + Sync>,
    memory: Arc<Mutex<Box<dyn MemoryStorage>>>,
    tools: Arc<Mutex<Box<dyn ToolManagerService>>>,
    llm: Arc<dyn Llm + Send + Sync>,
    reflection: Arc<dyn ReflectionEvaluator>,
    workflow_factory: Arc<dyn WorkflowFactory>,
    retry: Arc<RetryManager>,
    recovery: Arc<RecoveryManager>,
    scheduler: Arc<Scheduler>,
    metrics: Arc<dyn RuntimeMetricsCollector>,
    knowledge_manager: Option<Arc<dyn crate::knowledge::KnowledgeManager>>,
}

impl AgentRuntimeService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        events: RuntimeEvents,
        dispatcher: Dispatcher,
        planner: Arc<dyn PlannerProgress + Send + Sync>,
        memory: Arc<Mutex<Box<dyn MemoryStorage>>>,
        tools: Arc<Mutex<Box<dyn ToolManagerService>>>,
        llm: Arc<dyn Llm + Send + Sync>,
        reflection: Arc<dyn ReflectionEvaluator>,
        workflow_factory: Arc<dyn WorkflowFactory>,
        retry: Arc<RetryManager>,
        recovery: Arc<RecoveryManager>,
        scheduler: Arc<Scheduler>,
        metrics: Arc<dyn RuntimeMetricsCollector>,
        knowledge_manager: Option<Arc<dyn crate::knowledge::KnowledgeManager>>,
    ) -> Self {
        Self {
            events,
            dispatcher,
            planner,
            memory,
            tools,
            llm,
            reflection,
            workflow_factory,
            retry,
            recovery,
            scheduler,
            metrics,
            knowledge_manager,
        }
    }

    /// Run an execution plan end-to-end and return a summary string report.
    pub fn run_plan(&self, plan: &ExecutionPlan) -> RavenResult<String> {
        info!("AgentRuntime: starting plan {}", plan.steps.len());
        let mut report =
            RuntimeReport::new(crate::agent::runtime::state::LifecycleState::Execution);

        let workflow_id = plan
            .steps
            .first()
            .map(|s| format!("workflow-{}", s.id))
            .unwrap_or_else(|| "workflow-unknown".to_string());

        // Metrics: mark workflow started and record planner call
        self.metrics.incr(
            "workflow_started",
            Some(&[("workflow_id", workflow_id.as_str())]),
        );
        let run_start = Instant::now();

        self.events
            .publish(crate::event::AgentEvent::WorkflowStarted {
                workflow_id: workflow_id.clone(),
            });

        let mut plan = plan.clone();

        // Build runtime context and enrich it with retrieved knowledge if available.
        let mut runtime_context = RuntimeContext::new(
            "session-unknown",
            plan.steps
                .first()
                .map(|s| s.description.clone())
                .unwrap_or_default(),
        );

        if let Some(knowledge_manager) = self.knowledge_manager.as_ref() {
            if let Some(first) = plan.steps.first() {
                let query = &first.description;
                let k_start = Instant::now();
                match knowledge_manager.retrieve(query, 5) {
                    Ok(knowledge_context) => {
                        let k_dur = k_start.elapsed();
                        self.metrics.record_duration(
                            "knowledge_retrieve_latency",
                            k_dur,
                            Some(&[("workflow_id", workflow_id.as_str())]),
                        );
                        let summary = knowledge_context.summary();
                        runtime_context = runtime_context.with_knowledge_context(knowledge_context);
                        for step in plan.steps.iter_mut() {
                            step.metadata
                                .insert("knowledge_summary".to_string(), summary.clone());
                        }
                    }
                    Err(err) => {
                        error!("knowledge retrieval failed: {}", err);
                    }
                }
            }
        }

        // Use memory to retrieve contextual items related to the plan's first step
        if let Ok(mem_guard) = self.memory.lock() {
            if let Some(first) = plan.steps.first() {
                let query = &first.description;
                let mem_start = Instant::now();
                let _ctx = mem_guard.retrieve(query, None, 5);
                let mem_dur = mem_start.elapsed();
                self.metrics.record_duration(
                    "memory_retrieve_latency",
                    mem_dur,
                    Some(&[("workflow_id", workflow_id.as_str())]),
                );
            }
        }

        // Use LLM to produce a small pre-run context (non-fatal)
        if let Some(first) = plan.steps.first() {
            let prompt = format!("Prepare execution for: {}", first.description);
            let _ = self.llm.generate(&prompt, None);
        }

        // Validate and schedule plan
        let sched_start = Instant::now();
        let scheduled = self
            .scheduler
            .schedule(&plan)
            .map_err(|e| RavenError::Planner(format!("scheduler error: {}", e)))?;
        let sched_dur = sched_start.elapsed();
        self.metrics.record_duration(
            "scheduler_latency",
            sched_dur,
            Some(&[("workflow_id", workflow_id.as_str())]),
        );

        if let Some(_) = runtime_context.knowledge_context.as_ref() {
            runtime_context = runtime_context.with_plan(scheduled.clone());
        }

        // Load the scheduled plan into progress tracking before execution.
        self.planner
            .load_plan(&scheduled)
            .map_err(|e| RavenError::Planner(format!("planner load error: {}", e)))?;

        let mut attempt = 0;
        loop {
            attempt += 1;

            // Select executor via dispatcher
            let executor = self
                .dispatcher
                .select_executor(&scheduled)
                .ok_or_else(|| RavenError::Configuration("no suitable executor found".into()))?;
            self.metrics.incr(
                "dispatcher_calls",
                Some(&[("workflow_id", workflow_id.as_str())]),
            );

            // Optionally inspect tools registry for the first step
            if let Some(first) = scheduled.steps.first() {
                if let Some(tool_name) = &first.tool_name {
                    if let Ok(tools_guard) = self.tools.lock() {
                        let _has = tools_guard.has_tool(tool_name);
                        let _ = _has; // keep value used to avoid warnings
                    }
                }
            }

            // Build workflow controller via injected factory.
            let workflow_controller = self
                .workflow_factory
                .build(executor.clone(), Some(runtime_context.clone()));
            let res = workflow_controller.start(scheduled.clone());
            match res {
                Ok(s) => {
                    report.complete();

                    // After successful run, call reflection.evaluate and commit results to memory
                    let status = workflow_controller.status();
                    let reflection_report = self.reflection.evaluate(&workflow_id, &status);
                    let summary = reflection_report.summarize();
                    if let Ok(mem_guard) = self.memory.lock() {
                        let _ = self
                            .reflection
                            .commit(mem_guard.as_ref(), reflection_report);
                        // also persist final string summary into episodic memory
                        let _ = mem_guard.add(
                            MemoryKind::Episodic,
                            &summary,
                            &["workflow", "reflection"],
                        );
                    }

                    // Metrics: success
                    let run_dur = run_start.elapsed();
                    self.metrics.record_duration(
                        "workflow_duration",
                        run_dur,
                        Some(&[("workflow_id", workflow_id.as_str())]),
                    );
                    self.metrics.incr(
                        "workflow_finished",
                        Some(&[("workflow_id", workflow_id.as_str())]),
                    );

                    self.events
                        .publish(crate::event::AgentEvent::WorkflowFinished {
                            workflow_id: workflow_id.clone(),
                            result: Ok(s.clone()),
                        });
                    return Ok(s);
                }
                Err(e) => {
                    error!("AgentRuntime plan failed attempt {}: {}", attempt, e);
                    // Metrics: failure and retry decision
                    self.metrics.incr(
                        "workflow_failed",
                        Some(&[("workflow_id", workflow_id.as_str())]),
                    );
                    // Ask RetryManager for decision
                    let ctx = crate::agent::runtime::retry::RetryContext {
                        step_id: workflow_id.clone(),
                        attempts: attempt,
                        max_attempts: self.retry.max_attempts,
                    };
                    match self.retry.decide(&ctx) {
                        crate::agent::runtime::retry::RetryDecision::Retry { next_backoff: _ } => {
                            self.metrics.incr(
                                "workflow_retry",
                                Some(&[("workflow_id", workflow_id.as_str())]),
                            );
                            self.recovery.recover(&e.to_string());
                            info!("Retrying workflow plan, attempt {}", attempt + 1);
                            continue;
                        }
                        crate::agent::runtime::retry::RetryDecision::Fail => {
                            // fallthrough to final failure handling below
                        }
                    }

                    // On final failure, still evaluate and commit reflection to memory
                    let status = workflow_controller.status();
                    let reflection_report = self.reflection.evaluate(&workflow_id, &status);
                    if let Ok(mem_guard) = self.memory.lock() {
                        let _ = self
                            .reflection
                            .commit(mem_guard.as_ref(), reflection_report);
                    }

                    // Metrics: final failure
                    self.metrics.incr(
                        "workflow_failed_final",
                        Some(&[("workflow_id", workflow_id.as_str())]),
                    );
                    let run_dur = run_start.elapsed();
                    self.metrics.record_duration(
                        "workflow_duration",
                        run_dur,
                        Some(&[("workflow_id", workflow_id.as_str())]),
                    );

                    self.events
                        .publish(crate::event::AgentEvent::WorkflowFinished {
                            workflow_id: workflow_id.clone(),
                            result: Err(e.to_string()),
                        });
                    return Err(e);
                }
            }
        }
    }

    /// Access the runtime dispatcher for executor selection and inspection.
    pub fn dispatcher(&self) -> &Dispatcher {
        &self.dispatcher
    }

    /// Retrieve the injected knowledge manager, if one is configured.
    pub fn knowledge_manager(&self) -> Option<&Arc<dyn crate::knowledge::KnowledgeManager>> {
        self.knowledge_manager.as_ref()
    }
}
