use crate::ecc::tool::builder::ToolEccBuilder;
use crate::ecc::tool::context::{ToolDescriptor, ToolEccContext};
use crate::ecc::tool::rules::{RequiredParameterRule, ToolExistsRule, UnknownParameterRule};
use crate::ecc::tool::types::ToolCall;
use crate::ecc::traits::Validator;
use serde_json::json;

#[test]
fn test_tool_exists_rule_detects_missing_tool() {
    let context = ToolEccContext::new(vec![]);
    let tool_call = ToolCall::new("missing_tool", json!({}));

    let validator = crate::ecc::tool::validator::ToolValidator::new(
        context,
        vec![Box::new(ToolExistsRule::new())],
    );

    let report = validator.validate(&tool_call).unwrap();
    assert!(!report.is_valid);
    assert_eq!(report.issues[0].code, "tool.exists");
}

#[test]
fn test_required_parameter_rule_detects_missing_required_field() {
    let schema = json!({
        "required": ["input"],
        "properties": {"input": {"type": "string"}},
    });
    let descriptor = ToolDescriptor::new("echo", Some(schema), None, vec![]);
    let context = ToolEccContext::new(vec![descriptor]);
    let tool_call = ToolCall::new("echo", json!({}));

    let validator = crate::ecc::tool::validator::ToolValidator::new(
        context,
        vec![Box::new(RequiredParameterRule::new())],
    );

    let report = validator.validate(&tool_call).unwrap();
    assert!(!report.is_valid);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == "tool.required_parameter"));
}

#[test]
fn test_unknown_parameter_rule_detects_extra_field() {
    let schema = json!({
        "properties": {"input": {"type": "string"}},
    });
    let descriptor = ToolDescriptor::new("echo", Some(schema), None, vec![]);
    let context = ToolEccContext::new(vec![descriptor]);
    let tool_call = ToolCall::new("echo", json!({"bad": "value"}));

    let validator = crate::ecc::tool::validator::ToolValidator::new(
        context,
        vec![Box::new(UnknownParameterRule::new())],
    );

    let report = validator.validate(&tool_call).unwrap();
    assert!(!report.is_valid);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == "tool.unknown_parameter"));
}

#[test]
fn test_tool_builder_default_pipeline_runs() {
    let schema = json!({
        "required": ["input"],
        "properties": {"input": {"type": "string"}},
    });
    let descriptor = ToolDescriptor::new("echo", Some(schema), None, vec![]);
    let context = ToolEccContext::new(vec![descriptor]);

    let engine = ToolEccBuilder::new(context).build();
    let tool_call = ToolCall::new("echo", json!({"input": "hello"}));

    let report = engine.execute(tool_call).unwrap();
    assert!(report.validation_result.is_valid);
    assert_eq!(
        report.applied_action.action,
        crate::ecc::policy::PolicyAction::Accept
    );
}
