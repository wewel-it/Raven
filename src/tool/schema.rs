use serde_json::Value;

/// Tool schema defines expected parameter shape for validation.
#[derive(Debug, Clone)]
pub struct ToolSchema {
    pub schema: Value,
}

impl ToolSchema {
    pub fn new(schema: Value) -> Self {
        Self { schema }
    }
}
