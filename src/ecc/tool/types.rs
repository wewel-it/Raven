use serde_json::Value;

/// Represents a tool call that will be evaluated by Tool ECC.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolCall {
    pub tool_name: String,
    pub params: Value,
    pub raw_params: Option<String>,
    pub permissions: Vec<String>,
}

impl ToolCall {
    /// Create a new tool call with the given name and parsed parameters.
    pub fn new(tool_name: impl Into<String>, params: Value) -> Self {
        Self {
            tool_name: tool_name.into(),
            params,
            raw_params: None,
            permissions: Vec::new(),
        }
    }

    /// Set raw parameter text in addition to parsed JSON.
    pub fn with_raw_params(mut self, raw_params: impl Into<String>) -> Self {
        self.raw_params = Some(raw_params.into());
        self
    }

    /// Set permissions available for this tool call.
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = permissions;
        self
    }
}

/// Result of Tool ECC verification plus the final verified tool call.
#[derive(Debug, Clone)]
pub struct VerifiedToolCall {
    pub tool_call: ToolCall,
    pub corrected: bool,
}

impl VerifiedToolCall {
    /// Create a new verified tool call wrapper.
    pub fn new(tool_call: ToolCall, corrected: bool) -> Self {
        Self {
            tool_call,
            corrected,
        }
    }
}
