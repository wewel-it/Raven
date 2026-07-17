use crate::ecc::tool::builder::ToolEccBuilder;
use crate::ecc::tool::context::{ToolDescriptor, ToolEccContext};
use crate::ecc::tool::rules::{DangerousArgumentRule, DuplicateParameterRule, ToolExistsRule};
use crate::ecc::tool::types::ToolCall;
use serde_json::json;

#[test]
fn integration_valid_tool_call_accepts() {
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

#[test]
fn integration_missing_tool_rejects() {
    let context = ToolEccContext::new(vec![]);
    let engine = ToolEccBuilder::new(context)
        .register_rule(Box::new(ToolExistsRule::new()))
        .build();

    let tool_call = ToolCall::new("missing", json!({"input": "hello"}));
    let report = engine.execute(tool_call).unwrap();

    assert!(!report.validation_result.is_valid);
    assert_eq!(
        report.applied_action.action,
        crate::ecc::policy::PolicyAction::Reject
    );
}

#[test]
fn integration_duplicate_parameter_rejects() {
    let schema = json!({
        "properties": {"input": {"type": "string"}},
    });
    let descriptor = ToolDescriptor::new("echo", Some(schema), None, vec![]);
    let context = ToolEccContext::new(vec![descriptor]);
    let engine = ToolEccBuilder::new(context)
        .register_rule(Box::new(DuplicateParameterRule::new()))
        .build();

    let tool_call = ToolCall::new("echo", json!({"input": "hello"}))
        .with_raw_params(r#"{"input": "hello", "input": "again"}"#);
    let report = engine.execute(tool_call).unwrap();

    assert!(!report.validation_result.is_valid);
    assert_eq!(
        report.applied_action.action,
        crate::ecc::policy::PolicyAction::Reject
    );
}

#[test]
fn integration_dangerous_argument_rejects() {
    let schema = json!({
        "properties": {"command": {"type": "string"}},
    });
    let descriptor = ToolDescriptor::new("shell", Some(schema), None, vec![]);
    let context = ToolEccContext::new(vec![descriptor]);
    let engine = ToolEccBuilder::new(context)
        .register_rule(Box::new(DangerousArgumentRule::new()))
        .build();

    let tool_call = ToolCall::new("shell", json!({"command": "rm -rf /"}));
    let report = engine.execute(tool_call).unwrap();

    assert!(!report.validation_result.is_valid);
    assert_eq!(
        report.applied_action.action,
        crate::ecc::policy::PolicyAction::Reject
    );
}
