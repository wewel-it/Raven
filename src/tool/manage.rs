use crate::tool::context::ExecutionContext;
use crate::tool::errors::ToolError;
use crate::tool::registry::ToolRegistry;
use crate::tool::result::ToolResult;
use crate::tool::{Tool, ToolManagerService};
use serde_json::Value;
use std::sync::Arc;

/// ToolService owns a thread-safe registry and manages tool invocation.
pub struct ToolService {
    registry: ToolRegistry,
}

impl Default for ToolService {
    fn default() -> Self {
        ToolService::new()
    }
}

impl ToolService {
    pub fn new() -> Self {
        Self {
            registry: ToolRegistry::new(),
        }
    }

    pub fn register_tool(&self, tool: Box<dyn Tool>) -> Result<(), ToolError> {
        self.registry.register(Arc::from(tool))
    }

    pub fn unregister_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.registry.unregister(name)
    }

    pub fn invoke(
        &self,
        name: &str,
        params: &Value,
        context: &ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        let tool = self.registry.get(name)?;
        let tool_ref: &dyn Tool = tool.as_ref();
        self.check_permissions(tool_ref, context)?;
        self.validate_params(tool_ref, params)?;
        let result = tool_ref.call(params)?;
        Ok(ToolResult::normalize(result))
    }

    pub fn has_tool(&self, name: &str) -> bool {
        self.registry.contains(name)
    }

    fn validate_params(&self, tool: &dyn Tool, params: &Value) -> Result<(), ToolError> {
        tool.validate(params)
    }

    fn check_permissions(
        &self,
        tool: &dyn Tool,
        context: &ExecutionContext,
    ) -> Result<(), ToolError> {
        if let Some(required) = tool.required_permission() {
            if !context.has_permission(required) {
                return Err(ToolError::permission_denied(format!(
                    "missing permission {} for tool {}",
                    required,
                    tool.name()
                )));
            }
        }
        Ok(())
    }
}

impl ToolManagerService for ToolService {
    fn register_tool(&self, tool: Box<dyn Tool>) -> Result<(), ToolError> {
        ToolService::register_tool(self, tool)
    }

    fn unregister_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        ToolService::unregister_tool(self, name)
    }

    fn invoke(
        &self,
        name: &str,
        params: &Value,
        context: &ExecutionContext,
    ) -> Result<ToolResult, ToolError> {
        ToolService::invoke(self, name, params, context)
    }

    fn has_tool(&self, name: &str) -> bool {
        ToolService::has_tool(self, name)
    }
}
