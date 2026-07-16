use crate::tool::{Tool, ToolError};
use serde_json::json;
use serde_json::Value;

/// Echo tool returns the `input` parameter as output under key `tool_result.echoed`.
pub struct EchoTool;

impl EchoTool {
    pub fn new() -> Self {
        Self {}
    }
}

impl Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }

    fn param_schema(&self) -> Option<Value> {
        Some(json!({ "required": ["input"], "type": "object" }))
    }

    fn validate(&self, params: &Value) -> Result<(), ToolError> {
        if !params.is_object() {
            return Err(ToolError::invalid_params("params must be object"));
        }
        if params.get("input").is_none() {
            return Err(ToolError::invalid_params("missing 'input'"));
        }
        Ok(())
    }

    fn call(&self, params: &Value) -> Result<Value, ToolError> {
        let input = params.get("input").and_then(|v| v.as_str()).unwrap_or("");
        Ok(json!({"tool_result": {"echoed": input}}))
    }
}
