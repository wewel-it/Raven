//! Workflow ECC module - Error Correction & Consistency for Workflows.
//!
//! Workflow ECC memastikan integritas dan konsistensi workflow sebelum, selama, dan setelah eksekusi.

pub mod engine;
pub mod errors;
pub mod rules;
pub mod types;
pub mod validator;

// Re-exports
pub use engine::WorkflowEccEngine;
pub use errors::{WorkflowEccError, WorkflowEccResult};
pub use types::{RecoveryConfig, Workflow, WorkflowAnalysisOptions, WorkflowStep};
pub use validator::WorkflowValidator;
