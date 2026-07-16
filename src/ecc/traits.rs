use crate::ecc::errors::EccResult;
use crate::ecc::policy::PolicyDecision;
use crate::ecc::report::{ConfidenceScore, ErrorClassification, ValidationReport};

/// Validator hanya bertanggung jawab mendeteksi kondisi yang tidak valid.
pub trait Validator<T>: Send + Sync {
    /// Validasi subjek ECC dan hasilkan laporan validasi.
    fn validate(&self, subject: &T) -> EccResult<ValidationReport>;

    /// Daftar `Rule` yang dijalankan selama validasi.
    fn rule_ids(&self) -> Vec<&'static str> {
        Vec::new()
    }
}

/// Corrector bertugas menghasilkan bentuk data yang telah diperbaiki.
pub trait Corrector<T>: Send + Sync {
    /// Perbaiki subjek berdasarkan hasil validasi.
    fn correct(&self, subject: &T, report: &ValidationReport) -> EccResult<T>;
}

/// Rule modular untuk evaluasi kondisi tertentu.
pub trait Rule<T>: Send + Sync {
    /// Identifier unik rule.
    fn id(&self) -> &'static str;

    /// Deskripsi ringkas tentang rule ini.
    fn description(&self) -> &'static str;

    /// Tentukan apakah rule ini relevan untuk subjek yang diberikan.
    fn applies_to(&self, subject: &T) -> bool;

    /// Evaluasi subjek dan kembalikan isu yang ditemukan.
    fn evaluate(&self, subject: &T) -> EccResult<Vec<crate::ecc::report::EccIssue>>;
}

/// Policy menentukan tindakan ECC setelah validasi selesai.
pub trait Policy: Send + Sync {
    fn decide(&self, report: &ValidationReport) -> PolicyDecision;
}

/// Tahap pipeline ECC generik yang memproses konteks bersama.
pub trait PipelineStage<T>: Send + Sync {
    /// Nama tahap untuk pelacakan dan debugging.
    fn name(&self) -> &'static str;

    /// Jalankan tahap terhadap konteks pipeline.
    fn execute(&self, context: &mut crate::ecc::pipeline::PipelineContext<T>) -> EccResult<()>;
}

/// Reporter bertanggung jawab membuat laporan akhir ECC.
pub trait Reporter<T>: Send + Sync {
    fn generate(
        &self,
        context: &crate::ecc::pipeline::PipelineContext<T>,
    ) -> EccResult<crate::ecc::report::EccReport>;
}

/// Mengklasifikasikan isu ECC ke dalam kategori dan tingkat keparahan.
pub trait ErrorClassifier<T>: Send + Sync {
    fn classify(
        &self,
        issue: &crate::ecc::report::EccIssue,
        context: &crate::ecc::pipeline::PipelineContext<T>,
    ) -> EccResult<ErrorClassification>;
}

/// Menentukan skor kepercayaan untuk hasil ECC.
pub trait ConfidenceScorer<T>: Send + Sync {
    fn score(
        &self,
        context: &crate::ecc::pipeline::PipelineContext<T>,
    ) -> EccResult<ConfidenceScore>;
}
