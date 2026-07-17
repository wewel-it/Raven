use crate::ecc::errors::EccResult;
use crate::ecc::report::EccIssue;
use crate::ecc::tool::context::ToolEccContext;
use crate::ecc::tool::types::ToolCall;
use serde_json::Value;

/// Trait untuk rule validasi Tool ECC.
pub trait ToolRule: Send + Sync {
    /// Unique identifier for the rule.
    fn id(&self) -> &'static str;

    /// Short description for the rule.
    fn description(&self) -> &'static str;

    /// Check whether this rule should be applied for the current tool call.
    fn applies_to(&self, tool_call: &ToolCall, context: &ToolEccContext) -> bool;

    /// Evaluate the rule and produce zero or more issues.
    fn evaluate(&self, tool_call: &ToolCall, context: &ToolEccContext) -> EccResult<Vec<EccIssue>>;
}

/// Tool exists rule ensures that the requested tool is registered.
pub struct ToolExistsRule;

impl ToolExistsRule {
    pub fn new() -> Self {
        Self
    }
}

impl ToolRule for ToolExistsRule {
    fn id(&self) -> &'static str {
        "tool.exists"
    }

    fn description(&self) -> &'static str {
        "Tool must be available in the registered tool context."
    }

    fn applies_to(&self, _tool_call: &ToolCall, _context: &ToolEccContext) -> bool {
        true
    }

    fn evaluate(&self, tool_call: &ToolCall, context: &ToolEccContext) -> EccResult<Vec<EccIssue>> {
        let mut issues = Vec::new();
        if context
            .find_by_name_case_insensitive(&tool_call.tool_name)
            .is_none()
        {
            issues.push(EccIssue::new(
                self.id().to_string(),
                self.description().to_string(),
                Some(format!("tool '{}' is not registered", tool_call.tool_name)),
                Some(tool_call.tool_name.clone()),
            ));
        }
        Ok(issues)
    }
}

/// Required parameter rule ensures required tool parameters are present.
pub struct RequiredParameterRule;

impl RequiredParameterRule {
    pub fn new() -> Self {
        Self
    }
}

impl ToolRule for RequiredParameterRule {
    fn id(&self) -> &'static str {
        "tool.required_parameter"
    }

    fn description(&self) -> &'static str {
        "All required tool parameters must be provided."
    }

    fn applies_to(&self, tool_call: &ToolCall, context: &ToolEccContext) -> bool {
        context
            .find_by_name_case_insensitive(&tool_call.tool_name)
            .is_some()
    }

    fn evaluate(&self, tool_call: &ToolCall, context: &ToolEccContext) -> EccResult<Vec<EccIssue>> {
        let mut issues = Vec::new();

        if let Some(descriptor) = context.find_by_name_case_insensitive(&tool_call.tool_name) {
            if let Some(schema) = &descriptor.param_schema {
                if let Some(required) = schema.get("required") {
                    if let Some(required_array) = required.as_array() {
                        for field in required_array {
                            if let Some(field_name) = field.as_str() {
                                if !tool_call.params.get(field_name).is_some() {
                                    issues.push(EccIssue::new(
                                        self.id().to_string(),
                                        self.description().to_string(),
                                        Some(format!(
                                            "missing required parameter '{}'",
                                            field_name
                                        )),
                                        Some(field_name.to_string()),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(issues)
    }
}

/// Unknown parameter rule detects parameters not declared in the tool schema.
pub struct UnknownParameterRule;

impl UnknownParameterRule {
    pub fn new() -> Self {
        Self
    }
}

impl ToolRule for UnknownParameterRule {
    fn id(&self) -> &'static str {
        "tool.unknown_parameter"
    }

    fn description(&self) -> &'static str {
        "Tool call must not include unknown parameters."
    }

    fn applies_to(&self, tool_call: &ToolCall, context: &ToolEccContext) -> bool {
        context
            .find_by_name_case_insensitive(&tool_call.tool_name)
            .is_some()
    }

    fn evaluate(&self, tool_call: &ToolCall, context: &ToolEccContext) -> EccResult<Vec<EccIssue>> {
        let mut issues = Vec::new();

        if let Some(descriptor) = context.find_by_name_case_insensitive(&tool_call.tool_name) {
            if let Some(schema) = &descriptor.param_schema {
                if let Some(properties) = schema.get("properties") {
                    if let Some(property_map) = properties.as_object() {
                        for key in tool_call
                            .params
                            .as_object()
                            .map(|o| o.keys())
                            .into_iter()
                            .flatten()
                        {
                            if !property_map.contains_key(key) {
                                issues.push(EccIssue::new(
                                    self.id().to_string(),
                                    self.description().to_string(),
                                    Some(format!("unknown parameter '{}'", key)),
                                    Some(key.clone()),
                                ));
                            }
                        }
                    }
                }
            }
        }

        Ok(issues)
    }
}

/// Parameter type rule validates JSON parameter types against declared schema.
pub struct ParameterTypeRule;

impl ParameterTypeRule {
    pub fn new() -> Self {
        Self
    }
}

impl ToolRule for ParameterTypeRule {
    fn id(&self) -> &'static str {
        "tool.parameter_type"
    }

    fn description(&self) -> &'static str {
        "Tool parameters must match declared JSON types."
    }

    fn applies_to(&self, tool_call: &ToolCall, context: &ToolEccContext) -> bool {
        context
            .find_by_name_case_insensitive(&tool_call.tool_name)
            .is_some()
    }

    fn evaluate(&self, tool_call: &ToolCall, context: &ToolEccContext) -> EccResult<Vec<EccIssue>> {
        let mut issues = Vec::new();

        if let Some(descriptor) = context.find_by_name_case_insensitive(&tool_call.tool_name) {
            if let Some(schema) = &descriptor.param_schema {
                if let Some(properties) = schema.get("properties") {
                    if let Some(property_map) = properties.as_object() {
                        for (field, expected_schema) in property_map {
                            if let Some(value) = tool_call.params.get(field) {
                                if let Some(expected_type) = expected_schema.get("type") {
                                    if let Some(expected_type) = expected_type.as_str() {
                                        let valid = match expected_type {
                                            "string" => value.is_string(),
                                            "integer" => value.is_i64() || value.is_u64(),
                                            "number" => value.is_number(),
                                            "boolean" => value.is_boolean(),
                                            "object" => value.is_object(),
                                            "array" => value.is_array(),
                                            _ => true,
                                        };
                                        if !valid {
                                            issues.push(EccIssue::new(
                                                self.id().to_string(),
                                                self.description().to_string(),
                                                Some(format!(
                                                    "parameter '{}' has invalid type",
                                                    field
                                                )),
                                                Some(field.clone()),
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(issues)
    }
}

/// JSON format rule detects invalid raw JSON input when raw_params is provided.
pub struct JsonFormatRule;

impl JsonFormatRule {
    pub fn new() -> Self {
        Self
    }
}

impl ToolRule for JsonFormatRule {
    fn id(&self) -> &'static str {
        "tool.json_format"
    }

    fn description(&self) -> &'static str {
        "Raw tool parameters must be valid JSON."
    }

    fn applies_to(&self, tool_call: &ToolCall, _context: &ToolEccContext) -> bool {
        tool_call.raw_params.is_some()
    }

    fn evaluate(
        &self,
        tool_call: &ToolCall,
        _context: &ToolEccContext,
    ) -> EccResult<Vec<EccIssue>> {
        let mut issues = Vec::new();

        if let Some(raw) = &tool_call.raw_params {
            if serde_json::from_str::<Value>(raw).is_err() {
                issues.push(EccIssue::new(
                    self.id().to_string(),
                    self.description().to_string(),
                    Some("raw_params contains invalid JSON".to_string()),
                    None,
                ));
            }
        }

        Ok(issues)
    }
}

/// Enum rule checks that enum-like fields match allowed string values in the schema.
pub struct EnumRule;

impl EnumRule {
    pub fn new() -> Self {
        Self
    }
}

impl ToolRule for EnumRule {
    fn id(&self) -> &'static str {
        "tool.enum"
    }

    fn description(&self) -> &'static str {
        "Enum parameters must match configured allowed values."
    }

    fn applies_to(&self, tool_call: &ToolCall, context: &ToolEccContext) -> bool {
        context
            .find_by_name_case_insensitive(&tool_call.tool_name)
            .is_some()
    }

    fn evaluate(&self, tool_call: &ToolCall, context: &ToolEccContext) -> EccResult<Vec<EccIssue>> {
        let mut issues = Vec::new();
        if let Some(descriptor) = context.find_by_name_case_insensitive(&tool_call.tool_name) {
            if let Some(schema) = &descriptor.param_schema {
                if let Some(properties) = schema.get("properties") {
                    if let Some(property_map) = properties.as_object() {
                        for (field, expected_schema) in property_map {
                            if let Some(value) = tool_call.params.get(field) {
                                if let Some(enum_values) = expected_schema.get("enum") {
                                    if let Some(allowed) = enum_values.as_array() {
                                        if let Some(actual) = value.as_str() {
                                            if !allowed.iter().any(|v| v.as_str() == Some(actual)) {
                                                issues.push(EccIssue::new(
                                                    self.id().to_string(),
                                                    self.description().to_string(),
                                                    Some(format!(
                                                        "parameter '{}' has invalid enum value",
                                                        field
                                                    )),
                                                    Some(field.clone()),
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(issues)
    }
}

/// Duplicate parameter rule detects repeated parameter keys in object payloads.
pub struct DuplicateParameterRule;

impl DuplicateParameterRule {
    pub fn new() -> Self {
        Self
    }
}

impl ToolRule for DuplicateParameterRule {
    fn id(&self) -> &'static str {
        "tool.duplicate_parameter"
    }

    fn description(&self) -> &'static str {
        "Duplicate parameters are not allowed in a tool call."
    }

    fn applies_to(&self, tool_call: &ToolCall, _context: &ToolEccContext) -> bool {
        tool_call.raw_params.is_some()
    }

    fn evaluate(
        &self,
        tool_call: &ToolCall,
        _context: &ToolEccContext,
    ) -> EccResult<Vec<EccIssue>> {
        let mut issues = Vec::new();
        if let Some(raw) = &tool_call.raw_params {
            let mut seen = std::collections::HashMap::new();
            let mut chars = raw.chars().peekable();

            fn skip_whitespace<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>) {
                while let Some(&ch) = chars.peek() {
                    if ch.is_whitespace() {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }

            fn parse_string<I: Iterator<Item = char>>(
                chars: &mut std::iter::Peekable<I>,
            ) -> Option<String> {
                let mut result = String::new();
                if chars.next()? != '"' {
                    return None;
                }
                while let Some(ch) = chars.next() {
                    match ch {
                        '"' => return Some(result),
                        '\\' => {
                            if let Some(escaped) = chars.next() {
                                result.push(escaped);
                            }
                        }
                        _ => result.push(ch),
                    }
                }
                None
            }

            fn skip_value<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>) {
                skip_whitespace(chars);
                if let Some(ch) = chars.peek().copied() {
                    match ch {
                        '{' => {
                            chars.next();
                            loop {
                                skip_whitespace(chars);
                                if let Some('}') = chars.peek().copied() {
                                    chars.next();
                                    break;
                                }
                                parse_string(chars);
                                skip_whitespace(chars);
                                if chars.next() != Some(':') {
                                    break;
                                }
                                skip_value(chars);
                                skip_whitespace(chars);
                                if let Some(',') = chars.peek().copied() {
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                        }
                        '[' => {
                            chars.next();
                            loop {
                                skip_whitespace(chars);
                                if let Some(']') = chars.peek().copied() {
                                    chars.next();
                                    break;
                                }
                                skip_value(chars);
                                skip_whitespace(chars);
                                if let Some(',') = chars.peek().copied() {
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                        }
                        '"' => {
                            parse_string(chars);
                        }
                        't' | 'f' | 'n' => {
                            while let Some(&c) = chars.peek() {
                                if c.is_alphanumeric() {
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                        }
                        '-' | '0'..='9' => {
                            chars.next();
                            while let Some(&c) = chars.peek() {
                                if c.is_digit(10)
                                    || c == '.'
                                    || c == 'e'
                                    || c == 'E'
                                    || c == '+'
                                    || c == '-'
                                {
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                        }
                        _ => {
                            chars.next();
                        }
                    }
                }
            }

            skip_whitespace(&mut chars);
            if chars.peek() == Some(&'{') {
                chars.next();
                loop {
                    skip_whitespace(&mut chars);
                    if chars.peek() == Some(&'}') {
                        break;
                    }
                    if let Some(key) = parse_string(&mut chars) {
                        skip_whitespace(&mut chars);
                        if chars.next() != Some(':') {
                            break;
                        }
                        let count = seen.entry(key.clone()).or_insert(0);
                        *count += 1;
                        skip_value(&mut chars);
                        skip_whitespace(&mut chars);
                        if chars.peek() == Some(&',') {
                            chars.next();
                        }
                    } else {
                        break;
                    }
                }
            }

            for (key, count) in seen {
                if count > 1 {
                    issues.push(EccIssue::new(
                        self.id().to_string(),
                        self.description().to_string(),
                        Some(format!("duplicate parameter '{}' detected", key)),
                        Some(key.clone()),
                    ));
                }
            }
        }
        Ok(issues)
    }
}

/// Permission rule checks whether required tool permission is granted.
pub struct PermissionRule;

impl PermissionRule {
    pub fn new() -> Self {
        Self
    }
}

impl ToolRule for PermissionRule {
    fn id(&self) -> &'static str {
        "tool.permission"
    }

    fn description(&self) -> &'static str {
        "Tool calls require permission if the tool declares one."
    }

    fn applies_to(&self, tool_call: &ToolCall, context: &ToolEccContext) -> bool {
        context
            .find_by_name_case_insensitive(&tool_call.tool_name)
            .is_some()
    }

    fn evaluate(&self, tool_call: &ToolCall, context: &ToolEccContext) -> EccResult<Vec<EccIssue>> {
        let mut issues = Vec::new();
        if let Some(descriptor) = context.find_by_name_case_insensitive(&tool_call.tool_name) {
            if let Some(required_permission) = &descriptor.required_permission {
                if !tool_call
                    .permissions
                    .iter()
                    .any(|p| p == required_permission)
                {
                    issues.push(EccIssue::new(
                        self.id().to_string(),
                        self.description().to_string(),
                        Some(format!(
                            "missing required permission '{}'",
                            required_permission
                        )),
                        Some(descriptor.name.clone()),
                    ));
                }
            }
        }
        Ok(issues)
    }
}

/// Dangerous argument rule detects offensive or unsafe parameter content.
pub struct DangerousArgumentRule;

impl DangerousArgumentRule {
    pub fn new() -> Self {
        Self
    }
}

impl ToolRule for DangerousArgumentRule {
    fn id(&self) -> &'static str {
        "tool.dangerous_argument"
    }

    fn description(&self) -> &'static str {
        "Tool arguments must not contain dangerous or restricted content."
    }

    fn applies_to(&self, tool_call: &ToolCall, _context: &ToolEccContext) -> bool {
        tool_call.params.is_object()
    }

    fn evaluate(
        &self,
        tool_call: &ToolCall,
        _context: &ToolEccContext,
    ) -> EccResult<Vec<EccIssue>> {
        let mut issues = Vec::new();
        if let Some(params) = tool_call.params.as_object() {
            for (key, value) in params {
                if let Some(text) = value.as_str() {
                    if text.contains("rm -rf") || text.contains("sudo") {
                        issues.push(EccIssue::new(
                            self.id().to_string(),
                            self.description().to_string(),
                            Some(format!(
                                "dangerous argument detected in parameter '{}'",
                                key
                            )),
                            Some(key.clone()),
                        ));
                    }
                }
            }
        }
        Ok(issues)
    }
}

/// Dependency rule validates referenced tool dependencies exist in context.
pub struct DependencyRule;

impl DependencyRule {
    pub fn new() -> Self {
        Self
    }
}

impl ToolRule for DependencyRule {
    fn id(&self) -> &'static str {
        "tool.dependency"
    }

    fn description(&self) -> &'static str {
        "Tool dependencies must reference registered tools."
    }

    fn applies_to(&self, tool_call: &ToolCall, context: &ToolEccContext) -> bool {
        context
            .find_by_name_case_insensitive(&tool_call.tool_name)
            .is_some()
    }

    fn evaluate(&self, tool_call: &ToolCall, context: &ToolEccContext) -> EccResult<Vec<EccIssue>> {
        let mut issues = Vec::new();
        if let Some(descriptor) = context.find_by_name_case_insensitive(&tool_call.tool_name) {
            for dependency in &descriptor.dependencies {
                if !context.has_tool(dependency) {
                    issues.push(EccIssue::new(
                        self.id().to_string(),
                        self.description().to_string(),
                        Some(format!("dependency '{}' is not registered", dependency)),
                        Some(dependency.clone()),
                    ));
                }
            }
        }
        Ok(issues)
    }
}

/// Timeout rule for tool calls defined by the tool descriptor.
pub struct TimeoutRule;

impl TimeoutRule {
    pub fn new() -> Self {
        Self
    }
}

impl ToolRule for TimeoutRule {
    fn id(&self) -> &'static str {
        "tool.timeout"
    }

    fn description(&self) -> &'static str {
        "Tool calls must not exceed configured timeout constraints."
    }

    fn applies_to(&self, tool_call: &ToolCall, _context: &ToolEccContext) -> bool {
        tool_call.params.is_object()
    }

    fn evaluate(
        &self,
        tool_call: &ToolCall,
        _context: &ToolEccContext,
    ) -> EccResult<Vec<EccIssue>> {
        let mut issues = Vec::new();
        if let Some(timeout) = tool_call.params.get("timeout") {
            if !timeout.is_u64() {
                issues.push(EccIssue::new(
                    self.id().to_string(),
                    self.description().to_string(),
                    Some("timeout parameter must be an unsigned integer".to_string()),
                    Some("timeout".to_string()),
                ));
            }
        }
        Ok(issues)
    }
}

/// Sandbox rule checks whether sandbox-related tool calls are allowed.
pub struct SandboxRule;

impl SandboxRule {
    pub fn new() -> Self {
        Self
    }
}

impl ToolRule for SandboxRule {
    fn id(&self) -> &'static str {
        "tool.sandbox"
    }

    fn description(&self) -> &'static str {
        "Sandbox tools must be executed in allowed environments."
    }

    fn applies_to(&self, tool_call: &ToolCall, _context: &ToolEccContext) -> bool {
        tool_call.tool_name.to_lowercase().contains("sandbox")
    }

    fn evaluate(
        &self,
        tool_call: &ToolCall,
        _context: &ToolEccContext,
    ) -> EccResult<Vec<EccIssue>> {
        let mut issues = Vec::new();
        if !tool_call.permissions.iter().any(|p| p == "sandbox:execute") {
            issues.push(EccIssue::new(
                self.id().to_string(),
                self.description().to_string(),
                Some("sandbox tool call missing sandbox execute permission".to_string()),
                Some(tool_call.tool_name.clone()),
            ));
        }
        Ok(issues)
    }
}

/// Reserved parameter rule detects reserved parameter names.
pub struct ReservedParameterRule;

impl ReservedParameterRule {
    pub fn new() -> Self {
        Self
    }
}

impl ToolRule for ReservedParameterRule {
    fn id(&self) -> &'static str {
        "tool.reserved_parameter"
    }

    fn description(&self) -> &'static str {
        "Reserved parameter names must not be present in tool calls."
    }

    fn applies_to(&self, tool_call: &ToolCall, _context: &ToolEccContext) -> bool {
        tool_call.params.is_object()
    }

    fn evaluate(
        &self,
        tool_call: &ToolCall,
        _context: &ToolEccContext,
    ) -> EccResult<Vec<EccIssue>> {
        let mut issues = Vec::new();
        let reserved_names = ["_internal", "debug", "meta"];
        if let Some(object) = tool_call.params.as_object() {
            for name in reserved_names {
                if object.contains_key(name) {
                    issues.push(EccIssue::new(
                        self.id().to_string(),
                        self.description().to_string(),
                        Some(format!("reserved parameter '{}' detected", name)),
                        Some(name.to_string()),
                    ));
                }
            }
        }
        Ok(issues)
    }
}

/// Empty parameter rule detects empty or null parameters that may be invalid.
pub struct EmptyParameterRule;

impl EmptyParameterRule {
    pub fn new() -> Self {
        Self
    }
}

impl ToolRule for EmptyParameterRule {
    fn id(&self) -> &'static str {
        "tool.empty_parameter"
    }

    fn description(&self) -> &'static str {
        "Empty or null parameter values should be removed."
    }

    fn applies_to(&self, tool_call: &ToolCall, _context: &ToolEccContext) -> bool {
        tool_call.params.is_object()
    }

    fn evaluate(
        &self,
        tool_call: &ToolCall,
        _context: &ToolEccContext,
    ) -> EccResult<Vec<EccIssue>> {
        let mut issues = Vec::new();
        if let Some(object) = tool_call.params.as_object() {
            for (key, value) in object {
                let is_empty = match value {
                    Value::Null => true,
                    Value::String(s) => s.trim().is_empty(),
                    Value::Array(arr) => arr.is_empty(),
                    Value::Object(map) => map.is_empty(),
                    _ => false,
                };
                if is_empty {
                    issues.push(EccIssue::new(
                        self.id().to_string(),
                        self.description().to_string(),
                        Some(format!("empty parameter '{}' detected", key)),
                        Some(key.clone()),
                    ));
                }
            }
        }
        Ok(issues)
    }
}
