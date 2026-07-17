use crate::ecc::errors::{EccError, EccResult};

/// Result type for Tool ECC operations.
pub type ToolEccResult<T> = EccResult<T>;

/// Error type for Tool ECC operations.
pub type ToolEccError = EccError;
