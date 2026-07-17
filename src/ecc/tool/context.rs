use serde_json::Value;
use std::collections::HashMap;

/// Descriptor for a tool available to the Tool ECC pipeline.
#[derive(Debug, Clone)]
pub struct ToolDescriptor {
    pub name: String,
    pub param_schema: Option<Value>,
    pub required_permission: Option<String>,
    pub dependencies: Vec<String>,
}

impl ToolDescriptor {
    /// Create a new descriptor for a registered tool.
    pub fn new(
        name: impl Into<String>,
        param_schema: Option<Value>,
        required_permission: Option<String>,
        dependencies: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            param_schema,
            required_permission,
            dependencies,
        }
    }
}

/// Context that stores metadata about available tools and their schemas.
#[derive(Debug, Clone)]
pub struct ToolEccContext {
    pub tools: HashMap<String, ToolDescriptor>,
}

impl ToolEccContext {
    /// Create a new Tool ECC context from a list of tool descriptors.
    pub fn new(tools: Vec<ToolDescriptor>) -> Self {
        let mut map = HashMap::new();
        for descriptor in tools {
            map.insert(descriptor.name.clone(), descriptor);
        }
        Self { tools: map }
    }

    /// Look up a tool descriptor by exact name.
    pub fn get(&self, name: &str) -> Option<&ToolDescriptor> {
        self.tools.get(name)
    }

    /// Find a tool by case-insensitive name match.
    pub fn find_by_name_case_insensitive(&self, name: &str) -> Option<&ToolDescriptor> {
        let lower = name.to_lowercase();
        self.tools
            .values()
            .find(|descriptor| descriptor.name.to_lowercase() == lower)
    }

    /// Check whether a tool exists in the context.
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }
}
