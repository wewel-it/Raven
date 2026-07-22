use crate::knowledge::document::Document;
use crate::knowledge::errors::{KnowledgeError, KnowledgeResult};
use crate::knowledge::metadata::DocumentMetadata;
use crate::knowledge::traits::DocumentLoader;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Loader that reads supported document files into Document objects.
#[derive(Debug)]
pub struct FileLoader;

impl FileLoader {
    pub fn new() -> Self {
        Self
    }

    fn language_from_extension(path: &Path) -> String {
        match path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
        {
            "md" => "markdown".to_string(),
            "txt" => "text".to_string(),
            other => other.to_string(),
        }
    }

    fn title_from_path(path: &Path) -> String {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_else(|| "document")
            .to_string()
    }

    fn parse_list(value: &str) -> Vec<String> {
        let trimmed = value.trim();
        let cleaned = trimmed.trim_start_matches('[').trim_end_matches(']').trim();
        if cleaned.is_empty() {
            return Vec::new();
        }

        cleaned
            .split(',')
            .map(|item| item.trim().trim_matches('"').to_string())
            .filter(|item| !item.is_empty())
            .collect()
    }

    fn parse_date(value: &str) -> KnowledgeResult<DateTime<Utc>> {
        if value.trim().is_empty() {
            return Ok(Utc::now());
        }

        DateTime::parse_from_rfc3339(value)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|err| {
                KnowledgeError::ValidationFailed(format!(
                    "invalid last_updated value '{}': {}",
                    value, err
                ))
            })
    }
}

pub fn parse_frontmatter(content: &str) -> KnowledgeResult<(HashMap<String, String>, String)> {
    let mut lines = content.lines();
    let first_line = lines.next().unwrap_or_default();
    if first_line.trim() != "---" {
        return Err(KnowledgeError::ValidationFailed(
            "document metadata frontmatter is missing".to_string(),
        ));
    }

    let mut metadata = HashMap::new();
    let mut body_lines = Vec::new();
    let mut in_frontmatter = true;

    for line in lines {
        let trimmed = line.trim();
        if in_frontmatter {
            if trimmed == "---" {
                in_frontmatter = false;
                continue;
            }
            if trimmed.is_empty() {
                continue;
            }
            let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
            if parts.len() != 2 {
                continue;
            }
            metadata.insert(
                parts[0].trim().to_lowercase(),
                parts[1].trim().trim_matches('"').to_string(),
            );
        } else {
            body_lines.push(line);
        }
    }

    if in_frontmatter {
        return Err(KnowledgeError::ValidationFailed(
            "document metadata frontmatter is not closed".to_string(),
        ));
    }

    Ok((metadata, body_lines.join("\n")))
}

impl DocumentLoader for FileLoader {
    fn load(&self, path: &Path) -> KnowledgeResult<Document> {
        let raw_content = fs::read_to_string(path).map_err(|err| {
            KnowledgeError::Io(format!("failed to read file {}: {}", path.display(), err))
        })?;

        if raw_content.trim().is_empty() {
            return Err(KnowledgeError::ValidationFailed(format!(
                "document content is empty: {}",
                path.display()
            )));
        }

        let (frontmatter, content) = parse_frontmatter(&raw_content)?;
        let metadata = fs::metadata(path)?;
        let size = metadata.len();
        let now = Utc::now();

        let language = frontmatter
            .get("language")
            .cloned()
            .unwrap_or_else(|| Self::language_from_extension(path));
        let title = frontmatter
            .get("title")
            .cloned()
            .unwrap_or_else(|| Self::title_from_path(path));
        let category = frontmatter
            .get("category")
            .cloned()
            .unwrap_or_else(|| "general".to_string());
        let topic = frontmatter.get("topic").cloned();
        let tags = Self::parse_list(
            frontmatter
                .get("tags")
                .map(String::as_str)
                .unwrap_or_default(),
        );
        let version = frontmatter
            .get("version")
            .cloned()
            .unwrap_or_else(|| "1.0".to_string());
        let difficulty = frontmatter
            .get("difficulty")
            .cloned()
            .unwrap_or_else(|| "intermediate".to_string());
        let source = frontmatter
            .get("source")
            .cloned()
            .unwrap_or_else(|| path.display().to_string());
        let updated_at = if let Some(last_updated) = frontmatter.get("last_updated") {
            Self::parse_date(last_updated)?
        } else {
            now
        };

        let id = format!("{}:{}", path.display(), size);
        let hash = id.clone();

        let document_metadata = DocumentMetadata::new(
            title.clone(),
            None,
            language.clone(),
            category,
            topic,
            tags,
            difficulty,
            version,
            source.clone(),
            hash,
            size,
            now,
            updated_at,
        );

        let document = Document::new(
            id.clone(),
            PathBuf::from(path),
            title,
            language,
            Vec::new(),
            source,
            document_metadata,
            content,
        );
        Ok(document)
    }
}
