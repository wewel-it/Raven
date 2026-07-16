use chrono::{DateTime, Utc};
use std::time::Duration;

use crate::ecc::errors::EccResult;
use crate::ecc::policy::PolicyDecision;

/// Tingkat keparahan klasifikasi kesalahan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Hasil klasifikasi untuk isu tertentu dalam ECC.
#[derive(Debug, Clone)]
pub struct ErrorClassification {
    pub issue_code: String,
    pub category: String,
    pub severity: ErrorSeverity,
    pub confidence: f32,
}

/// Skor keyakinan untuk seluruh pipeline ECC.
#[derive(Debug, Clone)]
pub struct ConfidenceScore {
    pub value: f32,
    pub rationale: Option<String>,
}

impl ConfidenceScore {
    /// Buat skor kepercayaan baru.
    pub fn new(value: f32, rationale: Option<String>) -> Self {
        Self { value, rationale }
    }
}

/// Deskripsi masalah yang terdeteksi oleh validator.
#[derive(Debug, Clone)]
pub struct EccIssue {
    pub code: String,
    pub summary: String,
    pub detail: Option<String>,
    pub location: Option<String>,
}

impl EccIssue {
    /// Buat isu baru dengan kode unik dan ringkasan.
    pub fn new(
        code: String,
        summary: String,
        detail: Option<String>,
        location: Option<String>,
    ) -> Self {
        Self {
            code,
            summary,
            detail,
            location,
        }
    }
}

/// Laporan hasil validasi ECC.
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub issues: Vec<EccIssue>,
    pub is_valid: bool,
    pub rule_count: usize,
    pub timestamp: DateTime<Utc>,
    pub duration: Duration,
}

impl ValidationReport {
    /// Buat laporan validasi awal tanpa isu.
    pub fn new(timestamp: DateTime<Utc>, duration: Duration, issues: Vec<EccIssue>) -> Self {
        let is_valid = issues.is_empty();
        let rule_count = issues.len();

        Self {
            issues,
            is_valid,
            rule_count,
            timestamp,
            duration,
        }
    }
}

/// Laporan akhir dari proses ECC.
#[derive(Debug, Clone)]
pub struct EccReport {
    pub validation_result: ValidationReport,
    pub error_classification: Vec<ErrorClassification>,
    pub confidence_score: ConfidenceScore,
    pub applied_action: PolicyDecision,
    pub executed_rules: Vec<String>,
    pub applied_fixes: Vec<String>,
    pub duration: Duration,
    pub timestamp: DateTime<Utc>,
}

impl EccReport {
    /// Buat laporan ECC dari elemen-elemen pipeline final.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        validation_result: ValidationReport,
        error_classification: Vec<ErrorClassification>,
        confidence_score: ConfidenceScore,
        applied_action: PolicyDecision,
        executed_rules: Vec<String>,
        applied_fixes: Vec<String>,
        duration: Duration,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            validation_result,
            error_classification,
            confidence_score,
            applied_action,
            executed_rules,
            applied_fixes,
            duration,
            timestamp,
        }
    }
}

/// Trait reporter ECC yang menghasilkan `EccReport` dari konteks pipeline.
pub trait EccReporter<T>: Send + Sync {
    fn generate(
        &self,
        context: &crate::ecc::pipeline::PipelineContext<T>,
    ) -> EccResult<crate::ecc::report::EccReport>;
}
