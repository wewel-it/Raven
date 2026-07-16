use crate::tool::errors::ToolError;
use crate::tool::Tool;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// A thread-safe registry for tool implementations.
pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub fn register(&self, tool: Arc<dyn Tool>) -> Result<(), ToolError> {
        let mut guard = self.tools.write().map_err(|_| ToolError::Registration("registry lock poisoned".to_string()))?;
        let name = tool.name().to_string();
        if guard.contains_key(&name) {
            return Err(ToolError::Registration(format!("tool already registered: {}", name)));
        }
        guard.insert(name, tool);
        Ok(())
    }

    pub fn unregister(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.write().ok().and_then(|mut guard| guard.remove(name))
    }

    pub fn get(&self, name: &str) -> Result<Arc<dyn Tool>, ToolError> {
        let guard = self.tools.read().map_err(|_| ToolError::NotFound(name.to_string()))?;
        guard
            .get(name)
            .cloned()
            .ok_or_else(|| ToolError::NotFound(name.to_string()))
    }

    pub fn contains(&self, name: &str) -> bool {
        self.tools.read().map(|guard| guard.contains_key(name)).unwrap_or(false)
    }

    pub fn list(&self) -> Vec<String> {
        self.tools.read().map(|guard| guard.keys().cloned().collect()).unwrap_or_default()
    }
}
