//! Validation rules for Memory ECC.
//!
//! This module contains all 18 validation rules for memory entries,
//! each checking specific aspects of memory structure and content validity.

use crate::ecc::memory::errors::MemoryEccResult;
use crate::ecc::memory::types::{
    MemoryIssueKind, MemoryValidationEntry, MEMORY_MAX_TAGS, MEMORY_TAG_MAX_BYTES,
    MEMORY_TEXT_MAX_BYTES,
};
use crate::ecc::report::EccIssue;
use crate::ecc::traits::Rule;

// ============================================================================
// MEMORY ID RULES
// ============================================================================

/// Rule: Memory ID format must be valid (m followed by 8 digits).
pub struct MemoryIdFormatRule;

impl Rule<MemoryValidationEntry> for MemoryIdFormatRule {
    fn id(&self) -> &'static str {
        "structure.memory_id_format"
    }

    fn description(&self) -> &'static str {
        "Memory ID must have format m followed by 8 digits"
    }

    fn applies_to(&self, _: &MemoryValidationEntry) -> bool {
        true
    }

    fn evaluate(&self, entry: &MemoryValidationEntry) -> MemoryEccResult<Vec<EccIssue>> {
        let id = &entry.entry.id;

        // Check format: m followed by 8 digits
        if id.len() != 9 || !id.starts_with('m') || !id[1..].chars().all(|c| c.is_ascii_digit()) {
            return Ok(vec![EccIssue::new(
                MemoryIssueKind::MemoryIdFormat.category().to_string(),
                "Memory ID format invalid".to_string(),
                Some(format!("Expected format 'm' + 8 digits, got '{}'", id)),
                Some(format!("field: id")),
            )]);
        }

        Ok(Vec::new())
    }
}

/// Rule: Memory ID must not be empty.
pub struct MemoryIdNotEmptyRule;

impl Rule<MemoryValidationEntry> for MemoryIdNotEmptyRule {
    fn id(&self) -> &'static str {
        "structure.memory_id_not_empty"
    }

    fn description(&self) -> &'static str {
        "Memory ID must not be empty"
    }

    fn applies_to(&self, _: &MemoryValidationEntry) -> bool {
        true
    }

    fn evaluate(&self, entry: &MemoryValidationEntry) -> MemoryEccResult<Vec<EccIssue>> {
        if entry.entry.id.is_empty() {
            return Ok(vec![EccIssue::new(
                MemoryIssueKind::MemoryIdEmpty.category().to_string(),
                "Memory ID is empty".to_string(),
                None,
                Some(format!("field: id")),
            )]);
        }

        Ok(Vec::new())
    }
}

// ============================================================================
// TIMESTAMP RULES
// ============================================================================

/// Rule: Timestamp must be valid.
pub struct TimestampValidRule;

impl Rule<MemoryValidationEntry> for TimestampValidRule {
    fn id(&self) -> &'static str {
        "structure.timestamp_valid"
    }

    fn description(&self) -> &'static str {
        "Timestamp must be a valid datetime"
    }

    fn applies_to(&self, _: &MemoryValidationEntry) -> bool {
        true
    }

    fn evaluate(&self, entry: &MemoryValidationEntry) -> MemoryEccResult<Vec<EccIssue>> {
        use chrono::Utc;

        let timestamp = entry.entry.created_at;
        let now = Utc::now();

        // Check if timestamp is too far in the past (more than 100 years)
        let duration = (now - timestamp).num_days().abs();
        if duration > 36500 {
            return Ok(vec![EccIssue::new(
                MemoryIssueKind::TimestampInvalid.category().to_string(),
                "Timestamp is too far in the past".to_string(),
                Some(format!("Timestamp: {}", timestamp)),
                Some("field: created_at".to_string()),
            )]);
        }

        Ok(Vec::new())
    }
}

// ============================================================================
// MEMORY KIND RULES
// ============================================================================

/// Rule: Memory kind must be a valid enum variant.
pub struct MemoryKindValidRule;

impl Rule<MemoryValidationEntry> for MemoryKindValidRule {
    fn id(&self) -> &'static str {
        "structure.memory_kind_valid"
    }

    fn description(&self) -> &'static str {
        "Memory kind must be a valid MemoryKind variant"
    }

    fn applies_to(&self, _: &MemoryValidationEntry) -> bool {
        true
    }

    fn evaluate(&self, _entry: &MemoryValidationEntry) -> MemoryEccResult<Vec<EccIssue>> {
        // Since MemoryKind is an enum, the type system ensures it's always valid
        // This rule exists for completeness and future extensibility
        Ok(Vec::new())
    }
}

// ============================================================================
// TEXT RULES
// ============================================================================

/// Rule: Text content must not be empty.
pub struct TextNotEmptyRule;

impl Rule<MemoryValidationEntry> for TextNotEmptyRule {
    fn id(&self) -> &'static str {
        "structure.text_not_empty"
    }

    fn description(&self) -> &'static str {
        "Text content must not be empty"
    }

    fn applies_to(&self, _: &MemoryValidationEntry) -> bool {
        true
    }

    fn evaluate(&self, entry: &MemoryValidationEntry) -> MemoryEccResult<Vec<EccIssue>> {
        if entry.entry.text.trim().is_empty() {
            return Ok(vec![EccIssue::new(
                MemoryIssueKind::TextEmpty.category().to_string(),
                "Text content is empty".to_string(),
                None,
                Some("field: text".to_string()),
            )]);
        }

        Ok(Vec::new())
    }
}

/// Rule: Text must be valid UTF-8 (always true in Rust, but kept for consistency).
pub struct TextUtf8ValidRule;

impl Rule<MemoryValidationEntry> for TextUtf8ValidRule {
    fn id(&self) -> &'static str {
        "structure.text_utf8_valid"
    }

    fn description(&self) -> &'static str {
        "Text must be valid UTF-8"
    }

    fn applies_to(&self, _: &MemoryValidationEntry) -> bool {
        true
    }

    fn evaluate(&self, _entry: &MemoryValidationEntry) -> MemoryEccResult<Vec<EccIssue>> {
        // In Rust, String is always valid UTF-8 by construction
        Ok(Vec::new())
    }
}

/// Rule: Text must not exceed maximum length.
pub struct TextMaxLengthRule;

impl Rule<MemoryValidationEntry> for TextMaxLengthRule {
    fn id(&self) -> &'static str {
        "structure.text_max_length"
    }

    fn description(&self) -> &'static str {
        "Text content must not exceed maximum length"
    }

    fn applies_to(&self, _: &MemoryValidationEntry) -> bool {
        true
    }

    fn evaluate(&self, entry: &MemoryValidationEntry) -> MemoryEccResult<Vec<EccIssue>> {
        let text_len = entry.entry.text.len();

        if text_len > MEMORY_TEXT_MAX_BYTES {
            return Ok(vec![EccIssue::new(
                MemoryIssueKind::TextTooLong.category().to_string(),
                "Text exceeds maximum length".to_string(),
                Some(format!(
                    "Text is {} bytes, max is {} bytes",
                    text_len, MEMORY_TEXT_MAX_BYTES
                )),
                Some("field: text".to_string()),
            )]);
        }

        Ok(Vec::new())
    }
}

// ============================================================================
// IMPORTANCE RULES
// ============================================================================

/// Rule: Importance score must be in range [0.0, 1.0].
pub struct ImportanceInRangeRule;

impl Rule<MemoryValidationEntry> for ImportanceInRangeRule {
    fn id(&self) -> &'static str {
        "structure.importance_in_range"
    }

    fn description(&self) -> &'static str {
        "Importance score must be in range [0.0, 1.0]"
    }

    fn applies_to(&self, _: &MemoryValidationEntry) -> bool {
        true
    }

    fn evaluate(&self, entry: &MemoryValidationEntry) -> MemoryEccResult<Vec<EccIssue>> {
        let importance = entry.entry.importance;

        if !(0.0..=1.0).contains(&importance) {
            return Ok(vec![EccIssue::new(
                MemoryIssueKind::ImportanceOutOfRange.category().to_string(),
                "Importance score out of range".to_string(),
                Some(format!("Importance {} not in [0.0, 1.0]", importance)),
                Some("field: importance".to_string()),
            )]);
        }

        Ok(Vec::new())
    }
}

/// Rule: Importance score must not be NaN.
pub struct ImportanceNotNaNRule;

impl Rule<MemoryValidationEntry> for ImportanceNotNaNRule {
    fn id(&self) -> &'static str {
        "structure.importance_not_nan"
    }

    fn description(&self) -> &'static str {
        "Importance score must not be NaN"
    }

    fn applies_to(&self, _: &MemoryValidationEntry) -> bool {
        true
    }

    fn evaluate(&self, entry: &MemoryValidationEntry) -> MemoryEccResult<Vec<EccIssue>> {
        if entry.entry.importance.is_nan() {
            return Ok(vec![EccIssue::new(
                MemoryIssueKind::ImportanceNaN.category().to_string(),
                "Importance score is NaN".to_string(),
                None,
                Some("field: importance".to_string()),
            )]);
        }

        Ok(Vec::new())
    }
}

// ============================================================================
// TAG RULES
// ============================================================================

/// Rule: Memory must have at least one tag.
pub struct TagsNotEmptyRule;

impl Rule<MemoryValidationEntry> for TagsNotEmptyRule {
    fn id(&self) -> &'static str {
        "structure.tags_not_empty"
    }

    fn description(&self) -> &'static str {
        "Memory must have at least one tag"
    }

    fn applies_to(&self, _: &MemoryValidationEntry) -> bool {
        true
    }

    fn evaluate(&self, entry: &MemoryValidationEntry) -> MemoryEccResult<Vec<EccIssue>> {
        if entry.entry.tags.is_empty() {
            return Ok(vec![EccIssue::new(
                MemoryIssueKind::TagsEmpty.category().to_string(),
                "Memory has no tags".to_string(),
                None,
                Some("field: tags".to_string()),
            )]);
        }

        Ok(Vec::new())
    }
}

/// Rule: Memory must not have duplicate tags.
pub struct TagsNoDuplicatesRule;

impl Rule<MemoryValidationEntry> for TagsNoDuplicatesRule {
    fn id(&self) -> &'static str {
        "structure.tags_no_duplicates"
    }

    fn description(&self) -> &'static str {
        "Memory must not have duplicate tags"
    }

    fn applies_to(&self, _: &MemoryValidationEntry) -> bool {
        true
    }

    fn evaluate(&self, entry: &MemoryValidationEntry) -> MemoryEccResult<Vec<EccIssue>> {
        let mut seen = std::collections::HashSet::new();

        for tag in &entry.entry.tags {
            if !seen.insert(tag.clone()) {
                return Ok(vec![EccIssue::new(
                    MemoryIssueKind::TagsDuplicate.category().to_string(),
                    "Duplicate tags detected".to_string(),
                    Some(format!("Duplicate tag: '{}'", tag)),
                    Some("field: tags".to_string()),
                )]);
            }
        }

        Ok(Vec::new())
    }
}

/// Rule: All tags must be valid strings (non-empty, within length limits).
pub struct TagsValidStringsRule;

impl Rule<MemoryValidationEntry> for TagsValidStringsRule {
    fn id(&self) -> &'static str {
        "structure.tags_valid_strings"
    }

    fn description(&self) -> &'static str {
        "All tags must be valid non-empty strings within length limits"
    }

    fn applies_to(&self, _: &MemoryValidationEntry) -> bool {
        true
    }

    fn evaluate(&self, entry: &MemoryValidationEntry) -> MemoryEccResult<Vec<EccIssue>> {
        for (idx, tag) in entry.entry.tags.iter().enumerate() {
            if tag.is_empty() {
                return Ok(vec![EccIssue::new(
                    MemoryIssueKind::TagsInvalid.category().to_string(),
                    "Tag is empty string".to_string(),
                    Some(format!("Tag at index {} is empty", idx)),
                    Some("field: tags".to_string()),
                )]);
            }

            if tag.len() > MEMORY_TAG_MAX_BYTES {
                return Ok(vec![EccIssue::new(
                    MemoryIssueKind::TagsInvalid.category().to_string(),
                    "Tag exceeds maximum length".to_string(),
                    Some(format!(
                        "Tag at index {} is {} bytes, max is {}",
                        idx,
                        tag.len(),
                        MEMORY_TAG_MAX_BYTES
                    )),
                    Some("field: tags".to_string()),
                )]);
            }
        }

        // Check total tag count
        if entry.entry.tags.len() > MEMORY_MAX_TAGS {
            return Ok(vec![EccIssue::new(
                MemoryIssueKind::TagsInvalid.category().to_string(),
                "Too many tags".to_string(),
                Some(format!(
                    "Memory has {} tags, max is {}",
                    entry.entry.tags.len(),
                    MEMORY_MAX_TAGS
                )),
                Some("field: tags".to_string()),
            )]);
        }

        Ok(Vec::new())
    }
}

// ============================================================================
// METADATA RULES
// ============================================================================

/// Rule: JSON metadata must be valid (no checking needed, it's already parsed).
pub struct MetadataJsonValidRule;

impl Rule<MemoryValidationEntry> for MetadataJsonValidRule {
    fn id(&self) -> &'static str {
        "structure.metadata_json_valid"
    }

    fn description(&self) -> &'static str {
        "JSON metadata must be valid and parseable"
    }

    fn applies_to(&self, _: &MemoryValidationEntry) -> bool {
        true
    }

    fn evaluate(&self, _entry: &MemoryValidationEntry) -> MemoryEccResult<Vec<EccIssue>> {
        // Metadata is a serde_json::Map, already validated by Rust type system
        Ok(Vec::new())
    }
}

/// Rule: Metadata keys must not be empty.
pub struct MetadataKeysValidRule;

impl Rule<MemoryValidationEntry> for MetadataKeysValidRule {
    fn id(&self) -> &'static str {
        "structure.metadata_keys_valid"
    }

    fn description(&self) -> &'static str {
        "Metadata keys must not be empty"
    }

    fn applies_to(&self, _: &MemoryValidationEntry) -> bool {
        true
    }

    fn evaluate(&self, entry: &MemoryValidationEntry) -> MemoryEccResult<Vec<EccIssue>> {
        for key in entry.metadata.keys() {
            if key.is_empty() {
                return Ok(vec![EccIssue::new(
                    MemoryIssueKind::MetadataKeysInvalid.category().to_string(),
                    "Metadata key is empty".to_string(),
                    None,
                    Some("field: metadata".to_string()),
                )]);
            }
        }

        Ok(Vec::new())
    }
}

/// Rule: Embedding metadata must be valid if present.
pub struct EmbeddingMetadataValidRule;

impl Rule<MemoryValidationEntry> for EmbeddingMetadataValidRule {
    fn id(&self) -> &'static str {
        "structure.embedding_metadata_valid"
    }

    fn description(&self) -> &'static str {
        "Embedding metadata structure must be valid if present"
    }

    fn applies_to(&self, entry: &MemoryValidationEntry) -> bool {
        entry.embedding_metadata.is_some()
    }

    fn evaluate(&self, entry: &MemoryValidationEntry) -> MemoryEccResult<Vec<EccIssue>> {
        if let Some(embedding) = &entry.embedding_metadata {
            if embedding.dimension == 0 {
                return Ok(vec![EccIssue::new(
                    MemoryIssueKind::EmbeddingMetadataInvalid
                        .category()
                        .to_string(),
                    "Embedding dimension is zero".to_string(),
                    None,
                    Some("field: embedding_metadata".to_string()),
                )]);
            }

            if embedding.model.is_empty() {
                return Ok(vec![EccIssue::new(
                    MemoryIssueKind::EmbeddingMetadataInvalid
                        .category()
                        .to_string(),
                    "Embedding model name is empty".to_string(),
                    None,
                    Some("field: embedding_metadata".to_string()),
                )]);
            }
        }

        Ok(Vec::new())
    }
}

// ============================================================================
// SOURCE & CHECKSUM RULES
// ============================================================================

/// Rule: Source field must be valid if present (non-empty string).
pub struct SourceValidRule;

impl Rule<MemoryValidationEntry> for SourceValidRule {
    fn id(&self) -> &'static str {
        "structure.source_valid"
    }

    fn description(&self) -> &'static str {
        "Source field must be non-empty if present"
    }

    fn applies_to(&self, entry: &MemoryValidationEntry) -> bool {
        entry.source.is_some()
    }

    fn evaluate(&self, entry: &MemoryValidationEntry) -> MemoryEccResult<Vec<EccIssue>> {
        if let Some(source) = &entry.source {
            if source.is_empty() {
                return Ok(vec![EccIssue::new(
                    MemoryIssueKind::SourceInvalid.category().to_string(),
                    "Source is empty string".to_string(),
                    None,
                    Some("field: source".to_string()),
                )]);
            }
        }

        Ok(Vec::new())
    }
}

/// Rule: Checksum format must be valid if present (hex string, typical length).
pub struct ChecksumFormatValidRule;

impl Rule<MemoryValidationEntry> for ChecksumFormatValidRule {
    fn id(&self) -> &'static str {
        "structure.checksum_format_valid"
    }

    fn description(&self) -> &'static str {
        "Checksum must be valid hex format if present"
    }

    fn applies_to(&self, entry: &MemoryValidationEntry) -> bool {
        entry.checksum.is_some()
    }

    fn evaluate(&self, entry: &MemoryValidationEntry) -> MemoryEccResult<Vec<EccIssue>> {
        if let Some(checksum) = &entry.checksum {
            // Check if it's a valid hex string (32 or 64 chars for MD5/SHA256)
            if !checksum.chars().all(|c| c.is_ascii_hexdigit()) {
                return Ok(vec![EccIssue::new(
                    MemoryIssueKind::ChecksumFormatInvalid
                        .category()
                        .to_string(),
                    "Checksum is not valid hex".to_string(),
                    Some(format!("Checksum: {}", checksum)),
                    Some("field: checksum".to_string()),
                )]);
            }

            if checksum.len() != 32 && checksum.len() != 64 {
                return Ok(vec![EccIssue::new(
                    MemoryIssueKind::ChecksumFormatInvalid
                        .category()
                        .to_string(),
                    "Checksum length invalid".to_string(),
                    Some(format!(
                        "Expected 32 or 64 hex chars, got {}",
                        checksum.len()
                    )),
                    Some("field: checksum".to_string()),
                )]);
            }
        }

        Ok(Vec::new())
    }
}

// ============================================================================
// STRUCTURAL RULES
// ============================================================================

/// Rule: No duplicate references between different fields.
pub struct NoDuplicateReferencesRule;

impl Rule<MemoryValidationEntry> for NoDuplicateReferencesRule {
    fn id(&self) -> &'static str {
        "structure.no_duplicate_references"
    }

    fn description(&self) -> &'static str {
        "No duplicate references between entry fields"
    }

    fn applies_to(&self, _: &MemoryValidationEntry) -> bool {
        true
    }

    fn evaluate(&self, _entry: &MemoryValidationEntry) -> MemoryEccResult<Vec<EccIssue>> {
        // This rule checks for logical consistency across fields
        // For now, always passes as a placeholder for future extensibility
        Ok(Vec::new())
    }
}

/// Rule: Required fields must all be present and non-empty.
pub struct RequiredFieldsPresentRule;

impl Rule<MemoryValidationEntry> for RequiredFieldsPresentRule {
    fn id(&self) -> &'static str {
        "structure.required_fields_present"
    }

    fn description(&self) -> &'static str {
        "All required fields must be present and valid"
    }

    fn applies_to(&self, _: &MemoryValidationEntry) -> bool {
        true
    }

    fn evaluate(&self, entry: &MemoryValidationEntry) -> MemoryEccResult<Vec<EccIssue>> {
        // Check all required fields
        if entry.entry.id.is_empty() {
            return Ok(vec![EccIssue::new(
                MemoryIssueKind::RequiredFieldsMissing
                    .category()
                    .to_string(),
                "Required field 'id' is missing or empty".to_string(),
                None,
                Some("field: id".to_string()),
            )]);
        }

        if entry.entry.text.is_empty() {
            return Ok(vec![EccIssue::new(
                MemoryIssueKind::RequiredFieldsMissing
                    .category()
                    .to_string(),
                "Required field 'text' is missing or empty".to_string(),
                None,
                Some("field: text".to_string()),
            )]);
        }

        if entry.entry.tags.is_empty() {
            return Ok(vec![EccIssue::new(
                MemoryIssueKind::RequiredFieldsMissing
                    .category()
                    .to_string(),
                "Required field 'tags' is missing or empty".to_string(),
                None,
                Some("field: tags".to_string()),
            )]);
        }

        Ok(Vec::new())
    }
}
