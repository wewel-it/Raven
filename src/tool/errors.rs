use thiserror::Error;

/// Tool manager error variants for registration, permission, validation, and execution.
#[derive(Error, Debug)]
pub enum ToolError {
    #[error("tool not found: {0}")]
    NotFound(String),
    #[error("invalid parameters: {0}")]
    InvalidParams(String),
    #[error("execution failed: {0}")]
    Execution(String),
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("registration failed: {0}")]
    Registration(String),
}

impl ToolError {
    pub fn invalid_params<T: Into<String>>(message: T) -> Self {
        ToolError::InvalidParams(message.into())
    }

    pub fn execution<T: Into<String>>(message: T) -> Self {
        ToolError::Execution(message.into())
    }

    pub fn permission_denied<T: Into<String>>(message: T) -> Self {
        ToolError::PermissionDenied(message.into())
    }

    pub fn registration<T: Into<String>>(message: T) -> Self {
        ToolError::Registration(message.into())
    }
}
