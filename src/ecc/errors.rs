use thiserror::Error;

/// Hasil operasi ECC yang dapat gagal dengan kesalahan spesifik ECC.
pub type EccResult<T> = Result<T, EccError>;

/// Hierarki error ECC yang terstruktur.
#[derive(Error, Debug)]
pub enum EccError {
    #[error("validation failed: {details}")]
    Validation { details: String },

    #[error("correction failed: {details}")]
    Correction { details: String },

    #[error("policy resolution failed: {details}")]
    Policy { details: String },

    #[error("pipeline execution failed: {details}")]
    Pipeline { details: String },

    #[error("report generation failed: {details}")]
    Reporting { details: String },

    #[error("integration failed: {details}")]
    Integration { details: String },
}
