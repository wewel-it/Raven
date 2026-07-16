use crate::ecc::traits::Validator;
use crate::planner::ExecutionPlan;

use crate::ecc::planner::builder::default_planner_pipeline;
use crate::ecc::planner::classifier::PlannerErrorClassifier;
use crate::ecc::planner::corrector::PlannerCorrector;
use crate::ecc::planner::policy::PlannerPolicy;
use crate::ecc::planner::reporter::PlannerReporter;
use crate::ecc::planner::scorer::PlannerConfidenceScorer;
use crate::ecc::planner::validator::PlannerValidator;

/// Helper untuk mem-build engine ECC planner.
pub fn build_planner_ecc_engine() -> crate::ecc::engine::EccEngine<ExecutionPlan> {
    let validator: Box<dyn Validator<ExecutionPlan>> = Box::new(PlannerValidator::new(vec![
        Box::new(crate::ecc::planner::rules::PlanNotEmptyRule),
        Box::new(crate::ecc::planner::rules::StepIdNotEmptyRule),
        Box::new(crate::ecc::planner::rules::UniqueStepIdsRule),
        Box::new(crate::ecc::planner::rules::DependencyExistsRule),
        Box::new(crate::ecc::planner::rules::DuplicateDependencyRule),
        Box::new(crate::ecc::planner::rules::AcyclicDependencyRule),
        Box::new(crate::ecc::planner::rules::ReachabilityRule),
        Box::new(crate::ecc::planner::rules::PlanStartEndRule),
    ]));

    crate::ecc::engine::EccEngine::new(
        validator,
        Box::new(PlannerCorrector::new()),
        Box::new(PlannerErrorClassifier),
        Box::new(PlannerConfidenceScorer::new()),
        Box::new(PlannerReporter::new()),
        Box::new(PlannerPolicy::new()),
        default_planner_pipeline(),
    )
}
