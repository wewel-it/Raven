use crate::ecc::errors::EccResult;
use crate::ecc::pipeline::PipelineContext;
use crate::ecc::report::{ErrorClassification, ErrorSeverity};
use crate::ecc::traits::ErrorClassifier;

/// Classifier for tool call validation issues.
pub struct ToolErrorClassifier;

impl ToolErrorClassifier {
    /// Create a new tool error classifier.
    pub fn new() -> Self {
        Self
    }

    fn classify_issue(&self, code: &str) -> (String, ErrorSeverity, f32) {
        let lower = code.to_lowercase();
        if lower.contains("dangerous") {
            ("security".to_string(), ErrorSeverity::Critical, 1.0)
        } else if lower.contains("missing") || lower.contains("required") {
            ("validation".to_string(), ErrorSeverity::High, 0.95)
        } else if lower.contains("unknown") || lower.contains("type") {
            ("validation".to_string(), ErrorSeverity::Medium, 0.8)
        } else if lower.contains("empty") {
            ("sanitization".to_string(), ErrorSeverity::Low, 0.7)
        } else {
            ("general".to_string(), ErrorSeverity::Medium, 0.75)
        }
    }
}

impl Default for ToolErrorClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorClassifier<crate::ecc::tool::types::ToolCall> for ToolErrorClassifier {
    fn classify(
        &self,
        issue: &crate::ecc::report::EccIssue,
        _context: &PipelineContext<crate::ecc::tool::types::ToolCall>,
    ) -> EccResult<ErrorClassification> {
        let (category, severity, confidence) = self.classify_issue(&issue.code);
        Ok(ErrorClassification {
            issue_code: issue.code.clone(),
            category,
            severity,
            confidence,
        })
    }
}
