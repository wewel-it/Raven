use crate::ecc::errors::EccResult;
use crate::ecc::report::ValidationReport;
use crate::ecc::tool::context::ToolEccContext;
use crate::ecc::tool::types::ToolCall;
use crate::ecc::traits::Corrector;
use serde_json::Value;

/// Deterministic corrector for tool calls.
pub struct ToolCorrector {
    context: ToolEccContext,
}

impl ToolCorrector {
    /// Create a new tool call corrector using the registered tool context.
    pub fn new(context: ToolEccContext) -> Self {
        Self { context }
    }

    fn normalize_string_values(&self, params: &Value) -> Value {
        match params {
            Value::String(s) => Value::String(s.trim().to_string()),
            Value::Array(array) => Value::Array(
                array
                    .iter()
                    .map(|item| self.normalize_string_values(item))
                    .collect(),
            ),
            Value::Object(map) => {
                let mut normalized = serde_json::Map::new();
                for (key, value) in map {
                    let normalized_value = self.normalize_string_values(value);
                    if !matches!(normalized_value, Value::Null)
                        && !(matches!(normalized_value, Value::String(ref s) if s.trim().is_empty()))
                    {
                        normalized.insert(key.clone(), normalized_value);
                    }
                }
                Value::Object(normalized)
            }
            other => other.clone(),
        }
    }

    fn normalize_tool_name(&self, tool_name: &str) -> String {
        if let Some(descriptor) = self.context.find_by_name_case_insensitive(tool_name) {
            descriptor.name.clone()
        } else {
            tool_name.trim().to_string()
        }
    }

    fn correct_value_type(&self, value: &Value) -> Value {
        if let Value::String(s) = value {
            let trimmed = s.trim();
            if trimmed.eq_ignore_ascii_case("true") {
                return Value::Bool(true);
            }
            if trimmed.eq_ignore_ascii_case("false") {
                return Value::Bool(false);
            }
            if let Ok(int_value) = trimmed.parse::<i64>() {
                return Value::Number(int_value.into());
            }
            if let Ok(float_value) = trimmed.parse::<f64>() {
                if let Some(number) = serde_json::Number::from_f64(float_value) {
                    return Value::Number(number);
                }
            }
        }
        value.clone()
    }
}

impl Corrector<ToolCall> for ToolCorrector {
    fn correct(&self, subject: &ToolCall, _report: &ValidationReport) -> EccResult<ToolCall> {
        let corrected_tool_name = self.normalize_tool_name(&subject.tool_name);
        let corrected_params = self.normalize_string_values(&subject.params);

        let corrected_params = match corrected_params {
            Value::Object(mut map) => {
                for (key, value) in map.clone() {
                    if let Value::String(_) = &value {
                        map.insert(key.clone(), self.correct_value_type(&value));
                    }
                }
                Value::Object(map)
            }
            other => other,
        };

        Ok(ToolCall {
            tool_name: corrected_tool_name,
            params: corrected_params,
            raw_params: subject.raw_params.clone(),
            permissions: subject.permissions.clone(),
        })
    }
}
