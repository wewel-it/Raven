use crate::knowledge::errors::{KnowledgeError, KnowledgeResult};
use crate::knowledge::loader::parse_frontmatter;
use crate::knowledge::traits::DocumentValidator;
use std::fs;
use std::path::Path;

/// Validator that checks document source file integrity and supported format.
#[derive(Debug)]
pub struct FileValidator;

impl FileValidator {
    pub fn new() -> Self {
        Self
    }
}

impl DocumentValidator for FileValidator {
    fn validate(&self, path: &Path) -> KnowledgeResult<()> {
        if !path.exists() {
            return Err(KnowledgeError::ValidationFailed(format!(
                "file does not exist: {}",
                path.display()
            )));
        }

        if !path.is_file() {
            return Err(KnowledgeError::ValidationFailed(format!(
                "path is not a file: {}",
                path.display()
            )));
        }

        let metadata = fs::metadata(path)?;
        if metadata.len() == 0 {
            return Err(KnowledgeError::ValidationFailed(format!(
                "file is empty: {}",
                path.display()
            )));
        }

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_lowercase();
        if !file_name.ends_with(".md") && !file_name.ends_with(".txt") {
            return Err(KnowledgeError::UnsupportedFormat(file_name));
        }

        let content = fs::read_to_string(path)?;
        let (frontmatter, _) = parse_frontmatter(&content)?;
        let required = [
            "title",
            "language",
            "category",
            "version",
            "difficulty",
            "source",
            "last_updated",
        ];

        for key in required {
            if frontmatter
                .get(key)
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .is_none()
            {
                return Err(KnowledgeError::ValidationFailed(format!(
                    "document is missing required metadata field: {}",
                    key
                )));
            }
        }

        if frontmatter
            .get("tags")
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .is_none()
        {
            return Err(KnowledgeError::ValidationFailed(
                "document is missing required metadata field: tags".to_string(),
            ));
        }

        Ok(())
    }
}
