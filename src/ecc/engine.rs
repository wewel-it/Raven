use crate::ecc::errors::EccResult;
use crate::ecc::pipeline::{Pipeline, PipelineContext};
use crate::ecc::policy::Policy;
use crate::ecc::report::ValidationReport;
use crate::ecc::traits::{Corrector, ErrorClassifier, Reporter, Validator};

/// ECC Engine orchestrates the entire ECC pipeline.
pub struct EccEngine<T> {
    pub validator: Box<dyn Validator<T>>,
    pub corrector: Box<dyn Corrector<T>>,
    pub classifier: Box<dyn ErrorClassifier<T>>,
    pub scorer: Box<dyn crate::ecc::traits::ConfidenceScorer<T>>,
    pub reporter: Box<dyn Reporter<T>>,
    pub policy: Box<dyn Policy>,
    pub pipeline: Pipeline<T>,
}

impl<T> EccEngine<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Buat engine ECC baru dengan dependensi utama.
    pub fn new(
        validator: Box<dyn Validator<T>>,
        corrector: Box<dyn Corrector<T>>,
        classifier: Box<dyn ErrorClassifier<T>>,
        scorer: Box<dyn crate::ecc::traits::ConfidenceScorer<T>>,
        reporter: Box<dyn Reporter<T>>,
        policy: Box<dyn Policy>,
        pipeline: Pipeline<T>,
    ) -> Self {
        Self {
            validator,
            corrector,
            classifier,
            scorer,
            reporter,
            policy,
            pipeline,
        }
    }

    /// Jalankan ECC terhadap objek input melalui pipeline.
    pub fn execute(&self, subject: T) -> EccResult<crate::ecc::report::EccReport> {
        let mut context = PipelineContext::new(subject);
        self.pipeline.run(&mut context)
    }

    /// Validasi input menggunakan validator yang terpasang.
    pub fn validate(&self, subject: &T) -> EccResult<ValidationReport> {
        self.validator.validate(subject)
    }

    /// Perbaiki data menggunakan corrector yang terpasang.
    pub fn correct(&self, subject: &T, validation: &ValidationReport) -> EccResult<T> {
        self.corrector.correct(subject, validation)
    }

    /// Putuskan tindakan kebijakan berdasarkan laporan validasi.
    pub fn decide_policy(&self, report: &ValidationReport) -> crate::ecc::policy::PolicyDecision {
        self.policy.decide(report)
    }
}
