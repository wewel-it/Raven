use serde_json::Value;

/// Execution result wrapper for tool outcomes.
#[derive(Debug, Clone)]
pub struct ToolResult {
    pub data: Value,
}

impl ToolResult {
    pub fn normalize(value: Value) -> Self {
        Self { data: value }
    }
}
