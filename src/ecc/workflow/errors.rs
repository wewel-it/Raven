//! Error types for Workflow ECC.

use thiserror::Error;

/// Hasil operasi Workflow ECC.
pub type WorkflowEccResult<T> = Result<T, WorkflowEccError>;

/// Error types untuk Workflow ECC.
#[derive(Error, Debug)]
pub enum WorkflowEccError {
    #[error("workflow validation failed: {details}")]
    ValidationFailed { details: String },

    #[error("workflow correction failed: {details}")]
    CorrectionFailed { details: String },

    #[error("workflow classification failed: {details}")]
    ClassificationFailed { details: String },

    #[error("workflow policy failed: {details}")]
    PolicyFailed { details: String },

    #[error("workflow confidence scoring failed: {details}")]
    ScoringFailed { details: String },

    #[error("workflow reporting failed: {details}")]
    ReportingFailed { details: String },

    #[error("workflow pipeline failed: {details}")]
    PipelineFailed { details: String },

    #[error("workflow recovery failed: {details}")]
    RecoveryFailed { details: String },

    #[error("workflow not found: {workflow_id}")]
    WorkflowNotFound { workflow_id: String },

    #[error("step not found: {step_id}")]
    StepNotFound { step_id: String },

    #[error("invalid workflow structure: {details}")]
    InvalidStructure { details: String },

    #[error("integrity error: {details}")]
    IntegrityError { details: String },
}
