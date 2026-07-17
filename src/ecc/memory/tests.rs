//! Comprehensive tests for Memory ECC subsystem.

#[cfg(test)]
mod integration_tests {
    use crate::ecc::memory::engine::MemoryEccEngine;
    use crate::ecc::memory::types::MemoryValidationEntry;
    use crate::memory::{MemoryEntry, MemoryKind};
    use chrono::Utc;

    fn create_entry(
        id: &str,
        text: &str,
        tags: Vec<String>,
        importance: f32,
    ) -> MemoryValidationEntry {
        let entry = MemoryEntry {
            id: id.to_string(),
            kind: MemoryKind::Working,
            text: text.to_string(),
            created_at: Utc::now(),
            tags,
            importance,
        };

        MemoryValidationEntry::from_entry(entry)
    }

    #[test]
    fn test_ecc_valid_memory_entry() {
        let engine = MemoryEccEngine::new();
        let entry = create_entry(
            "m00000001",
            "Valid memory content",
            vec!["important".to_string()],
            0.8,
        );

        let result = engine.execute(entry);
        assert!(result.is_ok());

        let (_final_entry, report) = result.unwrap();
        assert!(report.is_valid());
        assert!(report.accepted);
        assert!(!report.corrected);
    }

    #[test]
    fn test_ecc_memory_with_empty_text() {
        let engine = MemoryEccEngine::new();
        let entry = create_entry("m00000001", "", vec!["tag".to_string()], 0.5);

        let result = engine.execute(entry);
        assert!(result.is_ok());

        let (_final_entry, report) = result.unwrap();
        assert!(!report.is_valid());
        assert!(!report.accepted);
    }

    #[test]
    fn test_ecc_memory_with_empty_tags() {
        let engine = MemoryEccEngine::new();
        let entry = create_entry("m00000001", "text", vec![], 0.5);

        let result = engine.execute(entry);
        assert!(result.is_ok());

        let (_final_entry, report) = result.unwrap();
        assert!(!report.is_valid());
    }

    #[test]
    fn test_ecc_memory_with_invalid_id_format() {
        let engine = MemoryEccEngine::new();
        let entry = create_entry("invalid_id", "text", vec!["tag".to_string()], 0.5);

        let result = engine.execute(entry);
        assert!(result.is_ok());

        let (_final_entry, report) = result.unwrap();
        assert!(!report.is_valid());
    }

    #[test]
    fn test_ecc_memory_with_out_of_range_importance() {
        let engine = MemoryEccEngine::new();
        let entry = create_entry("m00000001", "text", vec!["tag".to_string()], 1.5);

        let result = engine.execute(entry);
        assert!(result.is_ok());

        let (_final_entry, report) = result.unwrap();
        assert!(!report.is_valid());
    }

    #[test]
    fn test_ecc_memory_with_duplicate_tags() {
        let engine = MemoryEccEngine::new();
        let entry = create_entry(
            "m00000001",
            "text",
            vec!["tag".to_string(), "tag".to_string()],
            0.5,
        );

        let result = engine.execute(entry);
        assert!(result.is_ok());

        let (_final_entry, report) = result.unwrap();
        assert!(!report.is_valid());
    }

    #[test]
    fn test_ecc_memory_with_correctable_issues() {
        let engine = MemoryEccEngine::new();
        let entry = create_entry(
            "m00000001",
            "   text with leading/trailing spaces   ",
            vec!["tag1".to_string(), "tag1".to_string()],
            0.5,
        );

        let result = engine.execute(entry);
        assert!(result.is_ok());

        let (final_entry, report) = result.unwrap();
        assert!(!report.is_valid()); // Has issues
        assert!(report.corrected); // But was corrected

        // Check that corrections were applied
        assert_eq!(final_entry.entry.text.trim(), final_entry.entry.text);
        assert_eq!(final_entry.entry.tags.len(), 1); // Duplicates removed
    }

    #[test]
    fn test_ecc_memory_importance_normalization() {
        let engine = MemoryEccEngine::new();
        let mut entry = create_entry("m00000001", "text", vec!["tag".to_string()], 0.5);
        entry.entry.importance = f32::NAN;

        let result = engine.execute(entry);
        assert!(result.is_ok());

        let (final_entry, _report) = result.unwrap();
        assert!(!final_entry.entry.importance.is_nan());
    }

    #[test]
    fn test_ecc_report_summary() {
        let engine = MemoryEccEngine::new();
        let entry = create_entry("m00000001", "text", vec!["tag".to_string()], 0.5);

        let result = engine.execute(entry);
        assert!(result.is_ok());

        let (_final_entry, report) = result.unwrap();
        let summary = report.summary();
        assert!(summary.contains("m00000001"));
        assert!(summary.contains("ACCEPTED"));
    }

    #[test]
    fn test_ecc_multiple_issues() {
        let engine = MemoryEccEngine::new();
        let entry = create_entry("invalid", "  ", vec![], 2.0);

        let result = engine.execute(entry);
        assert!(result.is_ok());

        let (_final_entry, report) = result.unwrap();
        assert!(report.issue_count() > 0); // Multiple issues
    }

    #[test]
    fn test_ecc_confidence_scoring() {
        let engine = MemoryEccEngine::new();
        let entry = create_entry("m00000001", "text", vec!["tag".to_string()], 0.5);

        let result = engine.execute(entry);
        assert!(result.is_ok());

        let (_final_entry, report) = result.unwrap();
        let confidence = report.confidence();
        assert!((0.0..=1.0).contains(&confidence));
        assert!(confidence > 0.9); // Valid entry should have high confidence
    }

    #[test]
    fn test_ecc_memory_with_metadata() {
        let engine = MemoryEccEngine::new();
        let mut entry = create_entry("m00000001", "text", vec!["tag".to_string()], 0.5);

        // Add metadata
        entry.metadata.insert(
            "source".to_string(),
            serde_json::Value::String("test_source".to_string()),
        );
        entry
            .metadata
            .insert("version".to_string(), serde_json::Value::Number(1.into()));

        let result = engine.execute(entry);
        assert!(result.is_ok());

        let (_final_entry, report) = result.unwrap();
        assert!(report.is_valid());
    }

    #[test]
    fn test_ecc_validate_only() {
        let engine = MemoryEccEngine::new();
        let entry = create_entry("m00000001", "text", vec!["tag".to_string()], 0.5);

        let result = engine.validate_only(&entry);
        assert!(result.is_ok());

        let report = result.unwrap();
        assert!(report.is_valid);
    }
}
