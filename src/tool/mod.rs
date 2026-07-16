pub mod context;
pub mod errors;
pub mod manage;
pub mod registry;
pub mod result;
pub mod schema;
pub mod tools;

use serde_json::Value;

/// Tool trait defines the interface for tool implementations.
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;

    fn param_schema(&self) -> Option<Value> {
        None
    }

    fn required_permission(&self) -> Option<&str> {
        None
    }

    fn validate(&self, params: &Value) -> Result<(), ToolError>;

    fn call(&self, params: &Value) -> Result<Value, ToolError>;
}

/// Tool service interface for managing tool registration and invocation.
pub trait ToolManagerService: Send + Sync {
    fn register_tool(&self, tool: Box<dyn Tool>) -> Result<(), ToolError>;
    fn unregister_tool(&self, name: &str) -> Option<std::sync::Arc<dyn Tool>>;
    fn invoke(
        &self,
        name: &str,
        params: &Value,
        context: &ExecutionContext,
    ) -> Result<ToolResult, ToolError>;
    fn has_tool(&self, name: &str) -> bool;
}

pub use context::ExecutionContext;
pub use errors::ToolError;
pub use manage::ToolService;
pub use result::ToolResult;
pub use schema::ToolSchema;
