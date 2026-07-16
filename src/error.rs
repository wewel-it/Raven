use crate::tool::ToolError;
use serde_json::Error as SerdeError;
use std::sync::PoisonError;
use thiserror::Error;

pub type RavenResult<T> = Result<T, RavenError>;

#[derive(Error, Debug)]
pub enum RavenError {
    #[error("lock poisoned: {0}")]
    LockPoisoned(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("configuration error: {0}")]
    Configuration(String),
    #[error("event bus error: {0}")]
    EventBus(String),
    #[error("memory error: {0}")]
    Memory(String),
    #[error("planner error: {0}")]
    Planner(String),
    #[error("executor error: {0}")]
    Executor(String),
    #[error("workflow error: {0}")]
    Workflow(String),
    #[error("tool error: {0}")]
    Tool(#[from] ToolError),
    #[error("llm error: {0}")]
    Llm(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("unsupported operation: {0}")]
    Unsupported(String),
}

impl<T> From<PoisonError<T>> for RavenError {
    fn from(err: PoisonError<T>) -> Self {
        RavenError::LockPoisoned(err.to_string())
    }
}

impl From<SerdeError> for RavenError {
    fn from(err: SerdeError) -> Self {
        RavenError::Serialization(err.to_string())
    }
}
