use crate::ecc::errors::EccResult;
use crate::ecc::report::ValidationReport;
use crate::ecc::traits::Corrector;

/// Corrector komposit yang menerapkan beberapa corrector berurutan.
pub struct CompositeCorrector<T> {
    pub stages: Vec<Box<dyn Corrector<T>>>,
}

impl<T> CompositeCorrector<T> {
    /// Buat composite corrector dari urutan stage korektor.
    pub fn new(stages: Vec<Box<dyn Corrector<T>>>) -> Self {
        Self { stages }
    }
}

impl<T> Corrector<T> for CompositeCorrector<T>
where
    T: Clone + Send + Sync,
{
    fn correct(&self, subject: &T, report: &ValidationReport) -> EccResult<T> {
        let mut current = subject.clone();
        for stage in &self.stages {
            current = stage.correct(&current, report)?;
        }
        Ok(current)
    }
}
