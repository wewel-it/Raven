//! Raven AI Agent foundational library
//!
//! Modules:
//! - intent
//! - planner
//! - memory
//! - tool
//! - llm
//! - executor

pub mod error;
pub mod intent;
pub mod planner;
pub mod memory;
pub mod tool;
pub mod llm;
pub mod executor;
pub mod workflow;
pub mod reflection;
pub mod event;

pub use error::{RavenError, RavenResult};
pub use intent::IntentAnalyzer;
pub use planner::PlannerService;
pub use memory::MemoryService;
pub use tool::ToolService;
pub use llm::SimpleLlm;
pub use executor::ExecutorService;
pub use workflow::engine::WorkflowService;
pub use workflow::state::{WorkflowState, WorkflowStatus};
pub use reflection::ReflectionService;
pub use event::{AgentEvent, EventBus};

#[cfg(test)]
mod tests {
    use crate::*;
    use crate::planner::PlannerProgress;
    use crate::llm::Llm;
    use log::LevelFilter;
    use std::sync::{Arc, Mutex};

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
        log::set_max_level(LevelFilter::Info);
    }

    #[test]
    fn integration_flow_runs() {
        init_logger();

        let analyzer = IntentAnalyzer::new();
        let intent = analyzer
            .analyze("Please summarize my notes and save to memory. Also run the echo tool with name=hello")
            .expect("analyze");

        let concrete_planner = PlannerService::new("default");
        let plan = concrete_planner.create_plan(&intent).expect("plan");
        let planner: Arc<dyn PlannerProgress + Send + Sync> = Arc::new(concrete_planner);

        let memory = Arc::new(Mutex::new(Box::new(MemoryService::new()) as Box<dyn crate::memory::MemoryStorage>));

        let tools = Arc::new(Mutex::new(Box::new(ToolService::new()) as Box<dyn crate::tool::ToolManagerService>));
        // register a simple echo tool for demonstration
        tools
            .lock()
            .unwrap()
            .register_tool(Box::new(crate::tool::tools::EchoTool::new()))
            .expect("register");

        let llm = Arc::new(SimpleLlm::new());

        let event_bus = Arc::new(EventBus::new());
        let exec = ExecutorService::new(
            Arc::clone(&memory),
            Arc::clone(&tools),
            llm,
            planner,
            Arc::clone(&event_bus),
        );
        let result = exec.execute_plan(&plan).expect("execute");

        assert!(result.contains("LLM response") || result.contains("tool_result"));
    }
}
