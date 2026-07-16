use crate::error::RavenResult;
use serde_json::Value;

/// Trait representing a Local LLM interface. Implementations can call external
/// services or provide deterministic/local behavior. Executor depends on this trait.
pub trait Llm: Send + Sync {
    fn generate(&self, prompt: &str, context: Option<&Value>) -> RavenResult<String>;
}

/// A deterministic, local LLM-like implementation used as a real, testable
/// component. It performs templated transformations of the prompt and context.
pub struct SimpleLlm {}

impl SimpleLlm {
    pub fn new() -> Self {
        Self {}
    }
}

impl Llm for SimpleLlm {
    fn generate(&self, prompt: &str, context: Option<&Value>) -> RavenResult<String> {
        // deterministic transformation: include context keys when present
        if let Some(ctx) = context {
            let mut parts = vec![format!("LLM response for: {}", prompt)];
            if ctx.is_object() {
                for (k, v) in ctx.as_object().unwrap().iter() {
                    let val_str = if v.is_string() { v.as_str().unwrap().to_string() } else { v.to_string() };
                    parts.push(format!("{}={}", k, val_str));
                }
            } else {
                parts.push(format!("context={}", ctx));
            }
            Ok(parts.join(" | "))
        } else {
            Ok(format!("LLM response for: {}", prompt))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn simple_llm_generates() {
        let l = SimpleLlm::new();
        let out = l.generate("hello", Some(&json!({"k":"v"}))).unwrap();
        assert!(out.contains("LLM response"));
        assert!(out.contains("k=v"));
    }
}
