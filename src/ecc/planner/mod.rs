pub mod builder;
pub mod classifier;
pub mod context;
pub mod corrector;
pub mod engine;
pub mod errors;
pub mod pipeline;
pub mod policy;
pub mod reporter;
pub mod rules;
pub mod scorer;
pub mod stages;
pub mod types;
pub mod validator;

pub use builder::{build_planner_ecc_engine, default_planner_pipeline};
pub use classifier::PlannerErrorClassifier;
pub use corrector::PlannerCorrector;
pub use pipeline::{PlannerPipeline, PlannerPipelineContext};
pub use policy::PlannerPolicy;
pub use reporter::PlannerReporter;
pub use rules::*;
pub use scorer::PlannerConfidenceScorer;
pub use stages::{
    PlannerClassificationStage, PlannerCorrectionStage, PlannerReportingStage, PlannerScoringStage,
    PlannerValidationStage, PlannerVerificationStage,
};
pub use validator::PlannerValidator;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecc::policy::Policy;
    use crate::ecc::traits::{ConfidenceScorer, Corrector, Rule};
    use crate::planner::{ExecutionPlan, Step, StepStatus};

    fn make_step(id: &str, description: &str, depends_on: Vec<&str>) -> Step {
        let mut step = Step::new(id.to_string(), description.to_string());
        step.depends_on = depends_on.into_iter().map(|s| s.to_string()).collect();
        step
    }

    fn valid_execution_plan() -> ExecutionPlan {
        ExecutionPlan {
            steps: vec![
                make_step("A", "start task", vec![]),
                make_step("B", "follow-up task", vec!["A"]),
            ],
        }
    }

    #[test]
    fn duplicate_dependency_rule_detects_duplicate_entries() {
        let plan = ExecutionPlan {
            steps: vec![
                make_step("A", "start", vec![]),
                make_step("B", "duplicate dep", vec!["A", "A"]),
            ],
        };

        let result = DuplicateDependencyRule.evaluate(&plan);
        assert!(result.is_ok());
        let issues = match result {
            Ok(issues) => issues,
            Err(_) => return,
        };

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, DuplicateDependencyRule.id());
    }

    #[test]
    fn acyclic_dependency_rule_detects_cycle() {
        let plan = ExecutionPlan {
            steps: vec![
                make_step("A", "step a", vec!["B"]),
                make_step("B", "step b", vec!["A"]),
            ],
        };

        let result = AcyclicDependencyRule.evaluate(&plan);
        assert!(result.is_ok());
        let issues = match result {
            Ok(issues) => issues,
            Err(_) => return,
        };

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, AcyclicDependencyRule.id());
    }

    #[test]
    fn reachability_rule_detects_unreachable_step() {
        let plan = ExecutionPlan {
            steps: vec![
                make_step("A", "root", vec![]),
                make_step("B", "unreachable", vec!["C"]),
            ],
        };

        let result = ReachabilityRule.evaluate(&plan);
        assert!(result.is_ok());
        let issues = match result {
            Ok(issues) => issues,
            Err(_) => return,
        };

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, ReachabilityRule.id());
    }

    #[test]
    fn step_id_not_empty_rule_detects_empty_step_id() {
        let plan = ExecutionPlan {
            steps: vec![make_step("", "invalid step", vec![])],
        };

        let result = StepIdNotEmptyRule.evaluate(&plan);
        assert!(result.is_ok());
        let issues = match result {
            Ok(issues) => issues,
            Err(_) => return,
        };

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, StepIdNotEmptyRule.id());
    }

    #[test]
    fn planner_corrector_deduplicates_dependencies_and_normalizes_status() {
        let mut step = Step::new("A".to_string(), "task".to_string());
        step.depends_on = vec!["B".to_string(), "B".to_string()];
        step.status = StepStatus::Failed;
        let plan = ExecutionPlan {
            steps: vec![make_step("B", "root", vec![]), step],
        };

        let report = crate::ecc::report::ValidationReport::new(
            chrono::Utc::now(),
            std::time::Duration::ZERO,
            vec![],
        );
        let corrector = PlannerCorrector::new();
        let result = corrector.correct(&plan, &report);
        assert!(result.is_ok());
        let corrected = match result {
            Ok(corrected) => corrected,
            Err(_) => return,
        };

        assert_eq!(corrected.steps[1].depends_on, vec!["B".to_string()]);
        assert_eq!(corrected.steps[1].status, StepStatus::Failed);
    }

    #[test]
    fn planner_policy_decides_accept_correct_retry_reject() {
        let zero_issues_report = crate::ecc::report::ValidationReport::new(
            chrono::Utc::now(),
            std::time::Duration::ZERO,
            vec![],
        );
        let one_issue_report = crate::ecc::report::ValidationReport::new(
            chrono::Utc::now(),
            std::time::Duration::ZERO,
            vec![crate::ecc::report::EccIssue::new(
                "x".into(),
                "x".into(),
                None,
                None,
            )],
        );
        let three_issues_report = crate::ecc::report::ValidationReport::new(
            chrono::Utc::now(),
            std::time::Duration::ZERO,
            vec![
                crate::ecc::report::EccIssue::new("1".into(), "1".into(), None, None),
                crate::ecc::report::EccIssue::new("2".into(), "2".into(), None, None),
                crate::ecc::report::EccIssue::new("3".into(), "3".into(), None, None),
            ],
        );
        let five_issues_report = crate::ecc::report::ValidationReport::new(
            chrono::Utc::now(),
            std::time::Duration::ZERO,
            vec![
                crate::ecc::report::EccIssue::new("1".into(), "1".into(), None, None),
                crate::ecc::report::EccIssue::new("2".into(), "2".into(), None, None),
                crate::ecc::report::EccIssue::new("3".into(), "3".into(), None, None),
                crate::ecc::report::EccIssue::new("4".into(), "4".into(), None, None),
                crate::ecc::report::EccIssue::new("5".into(), "5".into(), None, None),
            ],
        );

        let policy = PlannerPolicy::new();
        assert_eq!(
            policy.decide(&zero_issues_report).action,
            crate::ecc::policy::PolicyAction::Accept
        );
        assert_eq!(
            policy.decide(&one_issue_report).action,
            crate::ecc::policy::PolicyAction::Correct
        );
        assert_eq!(
            policy.decide(&three_issues_report).action,
            crate::ecc::policy::PolicyAction::Retry
        );
        assert_eq!(
            policy.decide(&five_issues_report).action,
            crate::ecc::policy::PolicyAction::Reject
        );
    }

    #[test]
    fn planner_confidence_scorer_returns_expected_values() {
        let scorer = PlannerConfidenceScorer::new();
        let mut context =
            crate::ecc::planner::pipeline::PlannerPipelineContext::new(valid_execution_plan());

        context.validation_report = Some(crate::ecc::report::ValidationReport::new(
            chrono::Utc::now(),
            std::time::Duration::ZERO,
            vec![],
        ));
        let score = scorer.score(&context);
        assert!(score.is_ok());
        let score = match score {
            Ok(score) => score,
            Err(_) => return,
        };
        assert_eq!(score.value, 100.0);

        context.validation_report = Some(crate::ecc::report::ValidationReport::new(
            chrono::Utc::now(),
            std::time::Duration::ZERO,
            vec![crate::ecc::report::EccIssue::new(
                "1".into(),
                "1".into(),
                None,
                None,
            )],
        ));
        let score = scorer.score(&context);
        assert!(score.is_ok());
        let score = match score {
            Ok(score) => score,
            Err(_) => return,
        };
        assert_eq!(score.value, 85.0);

        context.validation_report = Some(crate::ecc::report::ValidationReport::new(
            chrono::Utc::now(),
            std::time::Duration::ZERO,
            vec![
                crate::ecc::report::EccIssue::new("1".into(), "1".into(), None, None),
                crate::ecc::report::EccIssue::new("2".into(), "2".into(), None, None),
                crate::ecc::report::EccIssue::new("3".into(), "3".into(), None, None),
            ],
        ));
        let score = scorer.score(&context);
        assert!(score.is_ok());
        let score = match score {
            Ok(score) => score,
            Err(_) => return,
        };
        assert_eq!(score.value, 70.0);

        context.validation_report = Some(crate::ecc::report::ValidationReport::new(
            chrono::Utc::now(),
            std::time::Duration::ZERO,
            vec![
                crate::ecc::report::EccIssue::new("1".into(), "1".into(), None, None),
                crate::ecc::report::EccIssue::new("2".into(), "2".into(), None, None),
                crate::ecc::report::EccIssue::new("3".into(), "3".into(), None, None),
                crate::ecc::report::EccIssue::new("4".into(), "4".into(), None, None),
                crate::ecc::report::EccIssue::new("5".into(), "5".into(), None, None),
            ],
        ));
        let score = scorer.score(&context);
        assert!(score.is_ok());
        let score = match score {
            Ok(score) => score,
            Err(_) => return,
        };
        assert_eq!(score.value, 45.0);
    }

    #[test]
    fn planner_pipeline_end_to_end_validates_and_reports() {
        let engine = build_planner_ecc_engine();
        let plan = valid_execution_plan();

        let result = engine.execute(plan);
        assert!(result.is_ok());
        let report = match result {
            Ok(report) => report,
            Err(_) => return,
        };

        assert!(report.validation_result.is_valid);
        assert_eq!(report.error_classification.len(), 0);
        assert_eq!(
            report.applied_action.action,
            crate::ecc::policy::PolicyAction::Accept
        );
    }

    #[test]
    fn planner_pipeline_integration_flow_classifies_initial_issues() {
        let engine = build_planner_ecc_engine();
        let plan = ExecutionPlan {
            steps: vec![
                make_step("A", "root", vec![]),
                make_step("B", "duplicate dependency", vec!["A", "A"]),
            ],
        };

        let result = engine.execute(plan);
        assert!(result.is_ok());
        let report = match result {
            Ok(report) => report,
            Err(_) => return,
        };

        assert!(report.validation_result.is_valid);
        assert_eq!(report.error_classification.len(), 1);
        assert_eq!(
            report.error_classification[0].issue_code,
            DuplicateDependencyRule.id()
        );
        assert_eq!(
            report.applied_action.action,
            crate::ecc::policy::PolicyAction::Accept
        );
    }
}
