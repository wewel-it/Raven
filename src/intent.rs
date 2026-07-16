use crate::error::{RavenError, RavenResult};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The result of intent analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Intent {
    pub name: String,
    pub confidence: f32,
    pub requires_tool: bool,
    pub requires_planner: bool,
    pub metadata: HashMap<String, String>,
    pub raw: String,
}

/// Deterministic Intent Analyzer using keyword matching and simple heuristics.
pub struct IntentAnalyzer {
    tool_keywords: Vec<&'static str>,
    plan_keywords: Vec<&'static str>,
}

impl IntentAnalyzer {
    /// Create a new analyzer with default heuristics
    pub fn new() -> Self {
        Self {
            tool_keywords: vec!["run", "execute", "call", "tool", "search", "fetch"],
            plan_keywords: vec!["plan", "schedule", "step", "task", "implement", "build"],
        }
    }

    /// Analyze an input string and produce an `Intent` structure.
    pub fn analyze(&self, input: &str) -> RavenResult<Intent> {
        let text = input.trim();
        if text.is_empty() {
            return Err(RavenError::InvalidInput("empty input".into()));
        }

        let lower = text.to_lowercase();

        let mut requires_tool = false;
        for kw in &self.tool_keywords {
            if lower.contains(kw) {
                requires_tool = true;
                break;
            }
        }

        let mut requires_planner = false;
        for kw in &self.plan_keywords {
            if lower.contains(kw) {
                requires_planner = true;
                break;
            }
        }

        // Detect intent name by simple regex patterns
        let intents = vec![
            ("summarize", Regex::new(r"\bsummariz(e|ation)\b|\bsummarize\b").map_err(|e| RavenError::Configuration(format!("regex compile failed: {}", e)))?),
            ("create", Regex::new(r"\bcreate\b|\bnew\b").map_err(|e| RavenError::Configuration(format!("regex compile failed: {}", e)))?),
            ("search", Regex::new(r"\bsearch\b|\bfind\b|\blookup\b").map_err(|e| RavenError::Configuration(format!("regex compile failed: {}", e)))?),
            ("plan", Regex::new(r"\bplan\b|\bschedule\b|\bstep\b").map_err(|e| RavenError::Configuration(format!("regex compile failed: {}", e)))?),
            ("tool_call", Regex::new(r"\brun\b|\bexecute\b|\bcall\b|\btool\b").map_err(|e| RavenError::Configuration(format!("regex compile failed: {}", e)))?),
            ("general", Regex::new(r".*").map_err(|e| RavenError::Configuration(format!("regex compile failed: {}", e)))?),
        ];

        let mut chosen = "general";
        for (name, re) in intents {
            if re.is_match(&lower) {
                chosen = name;
                break;
            }
        }

        let confidence = if chosen == "general" { 0.5 } else { 0.9 };

        // Extract simple metadata: look for 'name=VALUE' patterns for tools
        let mut metadata = HashMap::new();
        let name_re = Regex::new(r"([a-zA-Z_]+)=(\S+)").map_err(|e| RavenError::Configuration(format!("regex compile failed: {}", e)))?;
        for cap in name_re.captures_iter(text) {
            if let (Some(k), Some(v)) = (cap.get(1), cap.get(2)) {
                metadata.insert(k.as_str().to_string(), v.as_str().to_string());
            }
        }

        Ok(Intent {
            name: chosen.to_string(),
            confidence,
            requires_tool,
            requires_planner,
            metadata,
            raw: text.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyze_simple_tool_intent() {
        let a = IntentAnalyzer::new();
        let i = a.analyze("Please run the echo tool with name=hello").unwrap();
        assert!(i.requires_tool);
        assert_eq!(i.name, "tool_call");
        assert_eq!(i.metadata.get("name").map(|s| s.as_str()), Some("hello"));
    }
}
