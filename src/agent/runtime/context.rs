use crate::knowledge::KnowledgeContext;
use crate::planner::ExecutionPlan;
use serde_json::Value;
use std::sync::Arc;

/// RuntimeContext is built for each execution and is immutable afterwards.
#[derive(Debug, Clone)]
pub struct RuntimeContext {
    pub session_id: String,
    pub conversation_id: Option<String>,
    pub user_input: String,
    pub plan: Option<ExecutionPlan>,
    pub metadata: Arc<Vec<(String, Value)>>,
    pub knowledge_context: Option<KnowledgeContext>,
    pub tool_names: Vec<String>,
    pub llm_prompt: Option<String>,
    pub llm_response: Option<String>,
    pub reflection_summary: Option<String>,
    pub prompt_context: Option<Value>,
}

impl RuntimeContext {
    pub fn new(session_id: impl Into<String>, user_input: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            conversation_id: None,
            user_input: user_input.into(),
            plan: None,
            metadata: Arc::new(Vec::new()),
            knowledge_context: None,
            tool_names: Vec::new(),
            llm_prompt: None,
            llm_response: None,
            reflection_summary: None,
            prompt_context: None,
        }
    }

    pub fn with_plan(mut self, plan: ExecutionPlan) -> Self {
        self.plan = Some(plan);
        self
    }

    pub fn with_knowledge_context(mut self, knowledge_context: KnowledgeContext) -> Self {
        self.knowledge_context = Some(knowledge_context);
        self
    }

    pub fn with_tool_names(mut self, tool_names: Vec<String>) -> Self {
        self.tool_names = tool_names;
        self
    }

    pub fn with_llm_prompt(mut self, prompt: String) -> Self {
        self.llm_prompt = Some(prompt);
        self
    }

    pub fn with_llm_response(mut self, response: String) -> Self {
        self.llm_response = Some(response);
        self
    }

    pub fn with_reflection_summary(mut self, summary: String) -> Self {
        self.reflection_summary = Some(summary);
        self
    }

    pub fn with_prompt_context(mut self, context: Value) -> Self {
        self.prompt_context = Some(context);
        self
    }
}
