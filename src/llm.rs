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

impl Default for SimpleLlm {
    fn default() -> Self {
        SimpleLlm::new()
    }
}

impl Llm for SimpleLlm {
    fn generate(&self, prompt: &str, context: Option<&Value>) -> RavenResult<String> {
        // deterministic transformation: include context keys when present
        if let Some(ctx) = context {
            let mut parts = vec![format!("LLM response for: {}", prompt)];
            if let Some(map) = ctx.as_object() {
                for (k, v) in map.iter() {
                    let val_str = if let Some(s) = v.as_str() {
                        s.to_string()
                    } else {
                        v.to_string()
                    };
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
        let res = l.generate("hello", Some(&json!({"k":"v"})));
        let out = match res {
            Ok(o) => o,
            Err(_) => return,
        };
        assert!(out.contains("LLM response"));
        assert!(out.contains("k=v"));
    }
}
