use raven_agent::agent::runtime::builder::AgentRuntimeBuilder;
use raven_agent::event::EventBus;
use raven_agent::executor::ExecutorService;
use raven_agent::knowledge::{KnowledgeManagerImpl, KnowledgePipelineBuilder};
use raven_agent::llm::{Llm, SimpleLlm};
use raven_agent::memory::MemoryService;
use raven_agent::planner::{PlannerProgress, PlannerService};
use raven_agent::reflection::{ReflectionEvaluator, ReflectionService};
use raven_agent::tool::ToolService;
use raven_agent::KnowledgeManager;

use std::collections::HashSet;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[test]
fn knowledge_pipeline_integration_loads_and_retrieves_documents() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("knowledge/programming");
    let pipeline = KnowledgePipelineBuilder::new()
        .with_storage(Box::new(
            raven_agent::knowledge::storage::InMemoryKnowledgeStorage::new(),
        ))
        .build();
    let manager = KnowledgeManagerImpl::new_with_default_engine(pipeline);

    let processed = manager
        .process_directory(&root)
        .expect("process knowledge directory");

    let unique_ids: HashSet<_> = processed.iter().collect();
    assert_eq!(
        unique_ids.len(),
        processed.len(),
        "document ids must be unique"
    );
    assert!(
        processed.len() >= 5,
        "expect at least one document per language"
    );

    let result = manager
        .retrieve("borrow", 5)
        .expect("retrieve knowledge by query");

    assert!(
        result.document_count >= 1,
        "should retrieve at least one document"
    );
    assert!(result
        .documents
        .iter()
        .any(|doc| doc.metadata().language() == "rust"));
    assert!(result.candidate_count >= 1);
}

#[test]
fn runtime_can_access_knowledge_manager_and_run_plan() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("knowledge/programming");
    let pipeline = KnowledgePipelineBuilder::new()
        .with_storage(Box::new(
            raven_agent::knowledge::storage::InMemoryKnowledgeStorage::new(),
        ))
        .build();
    let mut knowledge_manager_impl = KnowledgeManagerImpl::new_with_default_engine(pipeline);
    let _ = knowledge_manager_impl
        .process_directory(&root)
        .expect("process knowledge directory");

    let knowledge_manager: Arc<dyn raven_agent::knowledge::KnowledgeManager> =
        Arc::new(knowledge_manager_impl);

    let bus = Arc::new(EventBus::new());
    let tools = Arc::new(Mutex::new(
        Box::new(ToolService::new()) as Box<dyn raven_agent::tool::ToolManagerService>
    ));
    let planner: Arc<dyn PlannerProgress + Send + Sync> = Arc::new(PlannerService::new("pl"));
    let memory = Arc::new(Mutex::new(
        Box::new(MemoryService::new()) as Box<dyn raven_agent::memory::MemoryStorage>
    ));
    let llm: Arc<dyn Llm + Send + Sync> = Arc::new(SimpleLlm::new());
    let reflection: Arc<dyn ReflectionEvaluator> = Arc::new(ReflectionService::new());

    let executor = Arc::new(ExecutorService::new(
        Arc::clone(&memory),
        Arc::clone(&tools),
        Arc::clone(&llm),
        Arc::clone(&planner),
        Arc::clone(&bus),
    ));

    let runtime = AgentRuntimeBuilder::new()
        .with_event_bus(Arc::clone(&bus))
        .with_planner(Arc::clone(&planner))
        .with_memory(Arc::clone(&memory))
        .with_tools(Arc::clone(&tools))
        .with_llm(Arc::clone(&llm))
        .with_reflection(Arc::clone(&reflection))
        .register_executor("default", executor.clone())
        .with_knowledge_manager(Arc::clone(&knowledge_manager))
        .build()
        .expect("build runtime");

    assert!(
        runtime.knowledge_manager().is_some(),
        "runtime should have knowledge manager"
    );

    let plan = PlannerService::new("test-planner")
        .create_plan(&raven_agent::intent::Intent {
            name: "knowledge-query".to_string(),
            confidence: 1.0,
            requires_tool: false,
            requires_planner: true,
            metadata: Default::default(),
            raw: "Explain lifetimes".to_string(),
        })
        .expect("create plan");

    let result = runtime.run_plan(&plan).expect("runtime should run plan");
    assert!(!result.is_empty(), "runtime response should not be empty");
}
