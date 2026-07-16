use crate::ecc::errors::{EccError, EccResult};
use crate::ecc::policy::PolicyDecision;
use crate::ecc::report::{ConfidenceScore, EccReport, ValidationReport};
use chrono::Utc;

/// Konteks yang dibawa sepanjang pipeline ECC.
pub struct PipelineContext<T> {
    pub subject: T,
    pub validation_report: Option<ValidationReport>,
    pub corrected_subject: Option<T>,
    pub error_classification: Vec<crate::ecc::report::ErrorClassification>,
    pub confidence_score: Option<ConfidenceScore>,
    pub applied_action: Option<PolicyDecision>,
    pub executed_rules: Vec<String>,
    pub applied_fixes: Vec<String>,
    pub report: Option<EccReport>,
}

impl<T> PipelineContext<T> {
    /// Buat konteks pipeline awal dengan subjek input.
    pub fn new(subject: T) -> Self {
        Self {
            subject,
            validation_report: None,
            corrected_subject: None,
            error_classification: Vec::new(),
            confidence_score: None,
            applied_action: None,
            executed_rules: Vec::new(),
            applied_fixes: Vec::new(),
            report: None,
        }
    }
}

/// Tahap pipeline yang dapat dipanggil oleh `EccEngine`.
pub trait PipelineStage<T>: Send + Sync {
    /// Nama tahap untuk pelacakan dan debugging.
    fn name(&self) -> &'static str;
    /// Jalankan tahap terhadap konteks pipeline.
    fn execute(&self, context: &mut PipelineContext<T>) -> EccResult<()>;
}

/// Abstraksi pipeline ECC yang dapat dikonfigurasikan.
pub struct Pipeline<T> {
    pub stages: Vec<Box<dyn PipelineStage<T>>>,
}

impl<T> Pipeline<T> {
    /// Buat pipeline baru dengan urutan stage tertentu.
    pub fn new(stages: Vec<Box<dyn PipelineStage<T>>>) -> Self {
        Self { stages }
    }

    /// Jalankan pipeline penuh terhadap konteks input.
    pub fn run(&self, context: &mut PipelineContext<T>) -> EccResult<EccReport> {
        let start = std::time::Instant::now();
        for stage in &self.stages {
            stage.execute(context)?;
        }

        let report = crate::ecc::report::EccReport::new(
            context
                .validation_report
                .clone()
                .ok_or(EccError::Pipeline {
                    details: "missing validation report".into(),
                })?,
            context.error_classification.clone(),
            context.confidence_score.clone().ok_or(EccError::Pipeline {
                details: "missing confidence score".into(),
            })?,
            context.applied_action.clone().ok_or(EccError::Pipeline {
                details: "missing policy decision".into(),
            })?,
            context.executed_rules.clone(),
            context.applied_fixes.clone(),
            start.elapsed(),
            Utc::now(),
        );

        Ok(report)
    }
}
