use serde_json::Value;
use std::collections::HashMap;

/// Execution context data provided to tools at runtime.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub caller_id: Option<String>,
    pub permissions: Vec<String>,
    pub metadata: HashMap<String, Value>,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        ExecutionContext::new()
    }
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            caller_id: None,
            permissions: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_caller(mut self, caller_id: impl Into<String>) -> Self {
        self.caller_id = Some(caller_id.into());
        self
    }

    pub fn with_permission(mut self, permission: impl Into<String>) -> Self {
        self.permissions.push(permission.into());
        self
    }

    pub fn insert_metadata(&mut self, key: impl Into<String>, value: Value) {
        self.metadata.insert(key.into(), value);
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission)
    }
}
