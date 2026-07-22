use crate::agent::runtime::context::RuntimeContext;
use crate::error::RavenResult;
use crate::executor::Executor;
use crate::planner::ExecutionPlan;
use crate::workflow::engine::WorkflowController;
use std::sync::{Arc, Mutex};
/// PlannerRuntime combines planner creation and progress tracking for runtime.
pub trait PlannerRuntime: crate::planner::Planner + crate::planner::PlannerProgress {}

impl<T> PlannerRuntime for T where T: crate::planner::Planner + crate::planner::PlannerProgress {}
/// Minimal runtime trait to execute a plan and return a report string.
pub trait AgentRuntime: Send + Sync {
    fn run_plan(&self, plan: &ExecutionPlan) -> RavenResult<String>;
}

/// Factory trait for building workflow controllers with a selected executor.
pub trait WorkflowFactory: Send + Sync {
    fn build(
        &self,
        executor: Arc<dyn Executor>,
        runtime_context: Option<RuntimeContext>,
    ) -> Arc<dyn WorkflowController>;
}

/// Default runtime workflow factory implementation.
pub struct WorkflowFactoryImpl {
    planner: Arc<dyn crate::planner::PlannerProgress + Send + Sync>,
    memory: Arc<Mutex<Box<dyn crate::memory::MemoryStorage>>>,
    tools: Arc<Mutex<Box<dyn crate::tool::ToolManagerService>>>,
    llm: Arc<dyn crate::llm::Llm + Send + Sync>,
    reflection: Arc<dyn crate::reflection::ReflectionEvaluator>,
    event_bus: Arc<crate::event::EventBus>,
}

impl WorkflowFactoryImpl {
    pub fn new(
        planner: Arc<dyn crate::planner::PlannerProgress + Send + Sync>,
        memory: Arc<Mutex<Box<dyn crate::memory::MemoryStorage>>>,
        tools: Arc<Mutex<Box<dyn crate::tool::ToolManagerService>>>,
        llm: Arc<dyn crate::llm::Llm + Send + Sync>,
        reflection: Arc<dyn crate::reflection::ReflectionEvaluator>,
        event_bus: Arc<crate::event::EventBus>,
    ) -> Self {
        Self {
            planner,
            memory,
            tools,
            llm,
            reflection,
            event_bus,
        }
    }
}

impl WorkflowFactory for WorkflowFactoryImpl {
    fn build(
        &self,
        executor: Arc<dyn Executor>,
        runtime_context: Option<RuntimeContext>,
    ) -> Arc<dyn WorkflowController> {
        Arc::new(crate::workflow::engine::WorkflowService::new(
            self.planner.clone(),
            self.memory.clone(),
            self.tools.clone(),
            self.llm.clone(),
            self.reflection.clone(),
            executor,
            self.event_bus.clone(),
            runtime_context,
        ))
    }
}
