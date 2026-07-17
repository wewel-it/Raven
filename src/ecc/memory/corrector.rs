//! Memory entry corrector for deterministic fixes.

use crate::ecc::memory::errors::MemoryEccResult;
use crate::ecc::memory::types::MemoryValidationEntry;
use crate::ecc::report::ValidationReport;
use crate::ecc::traits::Corrector;
use std::collections::HashSet;

/// Deterministic corrector for memory entries.
///
/// Applies a staged correction pipeline with each stage performing
/// specific transformations on memory entries.
pub struct MemoryCorrector {
    stages: Vec<Box<dyn CorrectionStage>>,
}

/// A single correction stage applied to memory entries.
pub trait CorrectionStage: Send + Sync {
    fn name(&self) -> &'static str;
    fn apply(&self, entry: &mut MemoryValidationEntry) -> MemoryEccResult<bool>;
}

impl MemoryCorrector {
    /// Create a new memory corrector with all standard correction stages.
    pub fn new() -> Self {
        let stages: Vec<Box<dyn CorrectionStage>> = vec![
            Box::new(TrimTextStage),
            Box::new(NormalizeNewlinesStage),
            Box::new(NormalizeUnicodeStage),
            Box::new(RemoveEmptyMetadataStage),
            Box::new(RemoveEmptyTagsStage),
            Box::new(RemoveDuplicateTagsStage),
            Box::new(SortTagsStage),
            Box::new(NormalizeImportanceStage),
            Box::new(FixTimestampPrecisionStage),
            Box::new(RepairMetadataStructureStage),
        ];

        Self { stages }
    }

    /// Create a corrector with custom stages.
    pub fn with_stages(stages: Vec<Box<dyn CorrectionStage>>) -> Self {
        Self { stages }
    }
}

impl Default for MemoryCorrector {
    fn default() -> Self {
        Self::new()
    }
}

impl Corrector<MemoryValidationEntry> for MemoryCorrector {
    fn correct(
        &self,
        subject: &MemoryValidationEntry,
        _report: &ValidationReport,
    ) -> MemoryEccResult<MemoryValidationEntry> {
        let mut corrected = subject.clone();

        for stage in &self.stages {
            stage.apply(&mut corrected)?;
        }

        Ok(corrected)
    }
}

// ============================================================================
// CORRECTION STAGES
// ============================================================================

/// Stage: Trim whitespace from text.
struct TrimTextStage;

impl CorrectionStage for TrimTextStage {
    fn name(&self) -> &'static str {
        "trim_text"
    }

    fn apply(&self, entry: &mut MemoryValidationEntry) -> MemoryEccResult<bool> {
        let original = entry.entry.text.clone();
        entry.entry.text = entry.entry.text.trim().to_string();
        Ok(original != entry.entry.text)
    }
}

/// Stage: Normalize newlines to \n.
struct NormalizeNewlinesStage;

impl CorrectionStage for NormalizeNewlinesStage {
    fn name(&self) -> &'static str {
        "normalize_newlines"
    }

    fn apply(&self, entry: &mut MemoryValidationEntry) -> MemoryEccResult<bool> {
        let original = entry.entry.text.clone();
        entry.entry.text = entry.entry.text.replace("\r\n", "\n").replace('\r', "\n");
        Ok(original != entry.entry.text)
    }
}

/// Stage: Normalize Unicode to NFC form.
struct NormalizeUnicodeStage;

impl CorrectionStage for NormalizeUnicodeStage {
    fn name(&self) -> &'static str {
        "normalize_unicode"
    }

    fn apply(&self, entry: &mut MemoryValidationEntry) -> MemoryEccResult<bool> {
        use unicode_normalization::UnicodeNormalization;

        let original = entry.entry.text.clone();
        entry.entry.text = entry.entry.text.nfc().collect::<String>();
        Ok(original != entry.entry.text)
    }
}

/// Stage: Remove null/empty metadata values.
struct RemoveEmptyMetadataStage;

impl CorrectionStage for RemoveEmptyMetadataStage {
    fn name(&self) -> &'static str {
        "remove_empty_metadata"
    }

    fn apply(&self, entry: &mut MemoryValidationEntry) -> MemoryEccResult<bool> {
        let original_len = entry.metadata.len();

        entry.metadata.retain(|_, v| {
            !matches!(v, serde_json::Value::Null) && !(v.is_string() && v.as_str() == Some(""))
        });

        Ok(entry.metadata.len() != original_len)
    }
}

/// Stage: Remove empty strings from tags.
struct RemoveEmptyTagsStage;

impl CorrectionStage for RemoveEmptyTagsStage {
    fn name(&self) -> &'static str {
        "remove_empty_tags"
    }

    fn apply(&self, entry: &mut MemoryValidationEntry) -> MemoryEccResult<bool> {
        let original_len = entry.entry.tags.len();
        entry.entry.tags.retain(|t| !t.is_empty());
        Ok(entry.entry.tags.len() != original_len)
    }
}

/// Stage: Remove duplicate tags.
struct RemoveDuplicateTagsStage;

impl CorrectionStage for RemoveDuplicateTagsStage {
    fn name(&self) -> &'static str {
        "remove_duplicate_tags"
    }

    fn apply(&self, entry: &mut MemoryValidationEntry) -> MemoryEccResult<bool> {
        let original_len = entry.entry.tags.len();
        let mut seen = HashSet::new();
        entry.entry.tags.retain(|t| seen.insert(t.clone()));
        Ok(entry.entry.tags.len() != original_len)
    }
}

/// Stage: Sort tags alphabetically.
struct SortTagsStage;

impl CorrectionStage for SortTagsStage {
    fn name(&self) -> &'static str {
        "sort_tags"
    }

    fn apply(&self, entry: &mut MemoryValidationEntry) -> MemoryEccResult<bool> {
        let original = entry.entry.tags.clone();
        entry.entry.tags.sort();
        Ok(original != entry.entry.tags)
    }
}

/// Stage: Clamp importance to [0.0, 1.0].
struct NormalizeImportanceStage;

impl CorrectionStage for NormalizeImportanceStage {
    fn name(&self) -> &'static str {
        "normalize_importance"
    }

    fn apply(&self, entry: &mut MemoryValidationEntry) -> MemoryEccResult<bool> {
        if entry.entry.importance.is_nan() {
            entry.entry.importance = 0.5; // Default middle value
            return Ok(true);
        }

        let original = entry.entry.importance;
        entry.entry.importance = entry.entry.importance.clamp(0.0, 1.0);
        Ok(original != entry.entry.importance)
    }
}

/// Stage: Ensure timestamp has consistent precision.
struct FixTimestampPrecisionStage;

impl CorrectionStage for FixTimestampPrecisionStage {
    fn name(&self) -> &'static str {
        "fix_timestamp_precision"
    }

    fn apply(&self, _entry: &mut MemoryValidationEntry) -> MemoryEccResult<bool> {
        // Timestamps in Rust chrono are already consistent
        // This stage exists for future extensibility
        Ok(false)
    }
}

/// Stage: Repair metadata structure if partially valid.
struct RepairMetadataStructureStage;

impl CorrectionStage for RepairMetadataStructureStage {
    fn name(&self) -> &'static str {
        "repair_metadata_structure"
    }

    fn apply(&self, entry: &mut MemoryValidationEntry) -> MemoryEccResult<bool> {
        // Remove any metadata entries with null or empty values
        let original_len = entry.metadata.len();
        entry.metadata.retain(|k, v| {
            !k.is_empty()
                && !matches!(v, serde_json::Value::Null)
                && !(v.is_string() && v.as_str() == Some(""))
        });

        Ok(entry.metadata.len() != original_len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{MemoryEntry, MemoryKind};
    use chrono::Utc;

    fn create_entry_with_issues() -> MemoryValidationEntry {
        let entry = MemoryEntry {
            id: "m00000001".to_string(),
            kind: MemoryKind::Working,
            text: "  text with spaces  \r\n".to_string(),
            created_at: Utc::now(),
            tags: vec!["tag1".to_string(), "tag2".to_string(), "tag1".to_string()],
            importance: 1.5, // Out of range
        };

        MemoryValidationEntry::from_entry(entry)
    }

    #[test]
    fn test_corrector_trims_text() {
        let corrector = MemoryCorrector::new();
        let entry = create_entry_with_issues();
        let original_text = entry.entry.text.clone();

        let report = crate::ecc::report::ValidationReport::new(
            Utc::now(),
            std::time::Duration::from_secs(0),
            vec![],
        );
        let corrected = corrector.correct(&entry, &report);

        assert!(corrected.is_ok());
        let corrected_entry = corrected.unwrap();
        assert_ne!(corrected_entry.entry.text, original_text);
        assert!(!corrected_entry.entry.text.starts_with(' '));
        assert!(!corrected_entry.entry.text.ends_with(' '));
    }

    #[test]
    fn test_corrector_removes_duplicate_tags() {
        let corrector = MemoryCorrector::new();
        let entry = create_entry_with_issues();
        assert_eq!(entry.entry.tags.len(), 3);

        let report = crate::ecc::report::ValidationReport::new(
            Utc::now(),
            std::time::Duration::from_secs(0),
            vec![],
        );
        let corrected = corrector.correct(&entry, &report);

        assert!(corrected.is_ok());
        let corrected_entry = corrected.unwrap();
        assert_eq!(corrected_entry.entry.tags.len(), 2);
    }

    #[test]
    fn test_corrector_normalizes_importance() {
        let corrector = MemoryCorrector::new();
        let mut entry = create_entry_with_issues();
        entry.entry.importance = 1.5;

        let report = crate::ecc::report::ValidationReport::new(
            Utc::now(),
            std::time::Duration::from_secs(0),
            vec![],
        );
        let corrected = corrector.correct(&entry, &report);

        assert!(corrected.is_ok());
        let corrected_entry = corrected.unwrap();
        assert!(corrected_entry.entry.importance >= 0.0);
        assert!(corrected_entry.entry.importance <= 1.0);
    }
}
