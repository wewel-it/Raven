use crate::ecc::report::ValidationReport;

/// Tindakan yang direkomendasikan oleh Policy setelah validasi ECC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyAction {
    Accept,
    Correct,
    Retry,
    Reject,
    Abort,
}

/// Keputusan kebijakan ECC, berisi tindakan dan alasan.
#[derive(Debug, Clone)]
pub struct PolicyDecision {
    pub action: PolicyAction,
    pub rationale: Option<String>,
}

impl PolicyDecision {
    /// Buat keputusan baru dengan tindakan yang ditentukan.
    pub fn new(action: PolicyAction, rationale: Option<String>) -> Self {
        Self { action, rationale }
    }

    /// Keputusan default untuk menerima hasil tanpa koreksi.
    pub fn accept() -> Self {
        Self {
            action: PolicyAction::Accept,
            rationale: Some("validation indicates valid subject".into()),
        }
    }
}

/// Policy menentukan tindakan yang diambil setelah proses validasi ECC.
pub trait Policy: Send + Sync {
    /// Tentukan keputusan kebijakan berdasarkan hasil validasi.
    fn decide(&self, report: &ValidationReport) -> PolicyDecision;
}
