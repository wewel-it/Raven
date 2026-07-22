use raven_agent::agent::runtime::builder::AgentRuntimeBuilder;
use raven_agent::agent::runtime::metrics::InMemoryMetricsCollector;
use raven_agent::event::EventBus;
use raven_agent::executor::ExecutorService;
use raven_agent::llm::SimpleLlm;
use raven_agent::memory::MemoryService;
use raven_agent::planner::PlannerService;
use raven_agent::reflection::ReflectionService;
use raven_agent::tool::ToolService;
use std::sync::{Arc, Mutex};

// This integration test runs a real runtime pipeline and asserts that the
// metrics collector observed key events. It uses real subsystems (no mocks).
#[test]
fn runtime_emits_metrics_end_to_end() {
    let bus = Arc::new(EventBus::new());
    let tools = Arc::new(Mutex::new(
        Box::new(ToolService::new()) as Box<dyn raven_agent::tool::ToolManagerService>
    ));
    let planner = Arc::new(PlannerService::new("pl"));
    let memory = Arc::new(Mutex::new(
        Box::new(MemoryService::new()) as Box<dyn raven_agent::memory::MemoryStorage>
    ));
    let llm = Arc::new(SimpleLlm::new());
    let reflection = Arc::new(ReflectionService::new());

    let executor = Arc::new(ExecutorService::new(
        memory.clone(),
        tools.clone(),
        llm.clone(),
        planner.clone(),
        bus.clone(),
    ));

    let metrics: Arc<dyn raven_agent::agent::runtime::metrics::RuntimeMetricsCollector> =
        Arc::new(InMemoryMetricsCollector::new());

    let builder = AgentRuntimeBuilder::new()
        .with_event_bus(bus.clone())
        .with_planner(planner.clone())
        .with_memory(memory.clone())
        .with_tools(tools.clone())
        .with_llm(llm.clone())
        .with_reflection(reflection.clone())
        .with_metrics(metrics.clone())
        .register_executor("default", executor.clone());

    let runtime = builder.build().expect("build ok");

    // create a simple plan via planner
    let analyzer = raven_agent::intent::IntentAnalyzer::new();
    let intent = analyzer
        .analyze("Please run echo tool name=Echo and then summarize")
        .expect("analyze ok");
    let plan = planner.create_plan(&intent).expect("create plan");

    let res = runtime.run_plan(&plan);
    assert!(res.is_ok());

    let snap = metrics.snapshot();
    // Some basic assertions about metrics being recorded (keys include labels)
    assert!(snap
        .counters
        .iter()
        .any(|(k, v)| k.starts_with("workflow_started") && *v >= 1));
    assert!(snap
        .counters
        .iter()
        .any(|(k, v)| k.starts_with("workflow_finished") && *v >= 1));
    assert!(snap
        .counters
        .iter()
        .any(|(k, v)| k.starts_with("dispatcher_calls") && *v >= 1));
    // scheduler latency histogram should exist (histograms use suffix _hist)
    assert!(snap
        .histograms
        .iter()
        .any(|(k, _)| k.starts_with("scheduler_latency_hist")));
}
