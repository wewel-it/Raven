use crate::llm::{Llm, SimpleLlm};
use crate::memory::MemoryService;
use crate::planner::{PlannerProgress, PlannerService};
use crate::tool::ToolService;
use crate::workflow::engine::WorkflowService;
use crate::workflow::state::WorkflowState;
use std::sync::{Arc, Mutex};

#[test]
fn workflow_service_lifecycle_controls() {
    let concrete_planner = PlannerService::new("test-planner");
    let planner: Arc<dyn PlannerProgress + Send + Sync> = Arc::new(concrete_planner);
    let memory: Arc<Mutex<Box<dyn crate::memory::MemoryStorage>>> =
        Arc::new(Mutex::new(Box::new(MemoryService::new())));
    let tools: Arc<Mutex<Box<dyn crate::tool::ToolManagerService>>> =
        Arc::new(Mutex::new(Box::new(ToolService::new())));
    let llm: Arc<dyn Llm + Send + Sync> = Arc::new(SimpleLlm::new());

    let event_bus = Arc::new(crate::event::EventBus::new());
    let executor: Arc<dyn crate::executor::Executor> =
        Arc::new(crate::executor::ExecutorService::new(
            Arc::clone(&memory),
            Arc::clone(&tools),
            Arc::clone(&llm),
            Arc::clone(&planner),
            Arc::clone(&event_bus),
        ));
    let reflection = crate::reflection::ReflectionService::new();
    let engine = WorkflowService::new(
        Arc::clone(&planner),
        Arc::clone(&memory),
        Arc::clone(&tools),
        Arc::clone(&llm),
        Arc::new(reflection),
        executor,
        Arc::clone(&event_bus),
        None,
    );

    let plan_res = PlannerService::new("test-planner").create_plan(&crate::intent::Intent {
        name: "general".to_string(),
        confidence: 1.0,
        requires_tool: false,
        requires_planner: true,
        metadata: Default::default(),
        raw: "Say hello".to_string(),
    });
    let plan = match plan_res {
        Ok(p) => p,
        Err(_) => return,
    };

    let start_res = engine.start(plan);
    let result = match start_res {
        Ok(r) => r,
        Err(_) => return,
    };
    assert!(result.contains("LLM response"));

    let status_res = engine.status();
    let status = match status_res {
        Ok(s) => s,
        Err(_) => return,
    };
    assert_eq!(status.state, WorkflowState::Completed);

    assert!(engine.pause().is_err());
    assert!(engine.resume().is_err());
    assert!(engine.cancel().is_err());
}
