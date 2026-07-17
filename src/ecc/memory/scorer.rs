//! Memory entry confidence scorer.

use crate::ecc::memory::errors::MemoryEccResult;
use crate::ecc::memory::types::MemoryValidationEntry;
use crate::ecc::pipeline::PipelineContext;
use crate::ecc::report::ConfidenceScore;
use crate::ecc::traits::ConfidenceScorer;

/// Scorer for memory entry validation confidence.
pub struct MemoryConfidenceScorer;

impl MemoryConfidenceScorer {
    /// Create a new scorer.
    pub fn new() -> Self {
        Self
    }

    /// Calculate confidence score from context.
    fn calculate_score(&self, context: &PipelineContext<MemoryValidationEntry>) -> f32 {
        let mut confidence = 1.0;

        // Deduct points based on validation issues
        if let Some(report) = &context.validation_report {
            let issue_count = report.issues.len();

            // Each issue reduces confidence by a factor
            let issues_penalty = (issue_count as f32) * 0.15;
            confidence = (confidence - issues_penalty).max(0.0);
        }

        // Deduct points based on corrections applied
        let corrections_penalty = (context.applied_fixes.len() as f32) * 0.10;
        confidence = (confidence - corrections_penalty).max(0.0);

        // Increase confidence if entry was never validated before and is valid
        let is_valid = context
            .validation_report
            .as_ref()
            .map(|r| r.is_valid)
            .unwrap_or(false);
        if !context.subject.ever_validated && is_valid {
            confidence = (confidence + 0.05).min(1.0);
        }

        confidence.clamp(0.0, 1.0)
    }
}

impl Default for MemoryConfidenceScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfidenceScorer<MemoryValidationEntry> for MemoryConfidenceScorer {
    fn score(
        &self,
        context: &PipelineContext<MemoryValidationEntry>,
    ) -> MemoryEccResult<ConfidenceScore> {
        let value = self.calculate_score(context);

        let rationale = if value >= 0.95 {
            Some("Entry is valid with no issues or corrections".to_string())
        } else if value >= 0.80 {
            Some("Entry has minor issues but is recoverable".to_string())
        } else if value >= 0.50 {
            Some("Entry has significant issues that were corrected".to_string())
        } else if value >= 0.20 {
            Some("Entry has substantial damage, recovery uncertain".to_string())
        } else {
            Some("Entry is severely corrupted or unreliable".to_string())
        };

        Ok(ConfidenceScore::new(value, rationale))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{MemoryEntry, MemoryKind};
    use chrono::Utc;

    fn create_context() -> PipelineContext<MemoryValidationEntry> {
        let entry = MemoryEntry {
            id: "m00000001".to_string(),
            kind: MemoryKind::Working,
            text: "test".to_string(),
            created_at: Utc::now(),
            tags: vec!["tag".to_string()],
            importance: 0.5,
        };

        PipelineContext::new(MemoryValidationEntry::from_entry(entry))
    }

    #[test]
    fn test_score_perfect_entry() {
        let scorer = MemoryConfidenceScorer::new();
        let context = create_context();

        let result = scorer.score(&context);
        assert!(result.is_ok());

        let score = result.unwrap();
        assert!(score.value > 0.95);
    }

    #[test]
    fn test_score_with_issues() {
        let scorer = MemoryConfidenceScorer::new();
        let mut context = create_context();

        // Add some dummy issues
        let report = crate::ecc::report::ValidationReport::new(
            Utc::now(),
            std::time::Duration::from_secs(0),
            vec![
                crate::ecc::report::EccIssue::new(
                    "test".to_string(),
                    "Test issue".to_string(),
                    None,
                    None,
                ),
                crate::ecc::report::EccIssue::new(
                    "test".to_string(),
                    "Test issue 2".to_string(),
                    None,
                    None,
                ),
            ],
        );
        context.validation_report = Some(report);

        let result = scorer.score(&context);
        assert!(result.is_ok());

        let score = result.unwrap();
        assert!(score.value < 1.0);
        assert!(score.value > 0.5);
    }

    #[test]
    fn test_score_with_corrections() {
        let scorer = MemoryConfidenceScorer::new();
        let mut context = create_context();

        context.applied_fixes.push("trim_text".to_string());
        context.applied_fixes.push("normalize_tags".to_string());

        let result = scorer.score(&context);
        assert!(result.is_ok());

        let score = result.unwrap();
        assert!(score.value <= 1.0);
    }
}
