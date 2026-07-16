//! Raven AI Agent foundational library
//!
//! Modules:
//! - intent
//! - planner
//! - memory
//! - tool
//! - llm
//! - executor

pub mod ecc;
pub mod error;
pub mod event;
pub mod executor;
pub mod intent;
pub mod llm;
pub mod memory;
pub mod planner;
pub mod reflection;
pub mod tool;
pub mod workflow;

pub use error::{RavenError, RavenResult};
pub use event::{AgentEvent, EventBus};
pub use executor::ExecutorService;
pub use intent::IntentAnalyzer;
pub use llm::SimpleLlm;
pub use memory::MemoryService;
pub use planner::PlannerService;
pub use reflection::ReflectionService;
pub use tool::ToolService;
pub use workflow::engine::WorkflowService;
pub use workflow::state::{WorkflowState, WorkflowStatus};

#[cfg(test)]
mod tests {
    use crate::planner::PlannerProgress;
    use crate::*;
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
        let intent_res = analyzer.analyze(
            "Please summarize my notes and save to memory. Also run the echo tool with name=hello",
        );
        let intent = match intent_res {
            Ok(i) => i,
            Err(_) => return,
        };

        let concrete_planner = PlannerService::new("default");
        let plan_res = concrete_planner.create_plan(&intent);
        let plan = match plan_res {
            Ok(p) => p,
            Err(_) => return,
        };
        let planner: Arc<dyn PlannerProgress + Send + Sync> = Arc::new(concrete_planner);

        let memory = Arc::new(Mutex::new(
            Box::new(MemoryService::new()) as Box<dyn crate::memory::MemoryStorage>
        ));

        let tools = Arc::new(Mutex::new(
            Box::new(ToolService::new()) as Box<dyn crate::tool::ToolManagerService>
        ));
        // register a simple echo tool for demonstration
        if let Ok(guard) = tools.lock() {
            let reg_res = guard.register_tool(Box::new(crate::tool::tools::EchoTool::new()));
            assert!(reg_res.is_ok());
        } else {
            return;
        }

        let llm = Arc::new(SimpleLlm::new());

        let event_bus = Arc::new(EventBus::new());
        let exec = ExecutorService::new(
            Arc::clone(&memory),
            Arc::clone(&tools),
            llm,
            planner,
            Arc::clone(&event_bus),
        );
        let exec_res = exec.execute_plan(&plan);
        assert!(exec_res.is_ok());
        let result = match exec_res {
            Ok(r) => r,
            Err(_) => return,
        };
        assert!(result.contains("LLM response") || result.contains("tool_result"));
    }
}
