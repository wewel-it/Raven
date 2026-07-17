//! Workflow ECC engine implementation.

use crate::ecc::errors::EccResult;
use crate::ecc::report::ValidationReport;
use crate::ecc::traits::Validator;
use crate::ecc::workflow::types::Workflow;
use crate::ecc::workflow::validator::WorkflowValidator;

/// Workflow ECC engine yang menjalankan seluruh pipeline.
pub struct WorkflowEccEngine {
    validator: WorkflowValidator,
}

impl WorkflowEccEngine {
    /// Buat engine baru dengan default configuration.
    pub fn new() -> Self {
        let validator = WorkflowValidator::with_default_rules();

        Self { validator }
    }

    /// Jalankan validasi untuk workflow.
    pub fn process(&self, workflow: &Workflow) -> EccResult<ValidationReport> {
        self.validator.validate(workflow)
    }

    /// Validasi saja tanpa correction.
    pub fn validate_only(&self, workflow: &Workflow) -> EccResult<ValidationReport> {
        self.validator.validate(workflow)
    }
}

impl Default for WorkflowEccEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecc::workflow::types::WorkflowStep;

    #[test]
    fn test_valid_workflow_processing() {
        let engine = WorkflowEccEngine::new();
        let workflow = Workflow::new("test".to_string())
            .with_start_step("step1".to_string())
            .add_step(WorkflowStep::new("step1".to_string()))
            .add_step(WorkflowStep::new("step2".to_string()).add_dependency("step1".to_string()))
            .add_end_step("step2".to_string());

        let result = engine.process(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_workflow_processing() {
        let engine = WorkflowEccEngine::new();
        let workflow = Workflow::new("test".to_string());

        let result = engine.process(&workflow);
        assert!(result.is_ok());
    }
}
