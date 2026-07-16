use crate::ecc::pipeline::Pipeline;
use crate::ecc::traits::Validator;
use crate::planner::ExecutionPlan;

use crate::ecc::planner::classifier::PlannerErrorClassifier;
use crate::ecc::planner::corrector::PlannerCorrector;
use crate::ecc::planner::policy::PlannerPolicy;
use crate::ecc::planner::reporter::PlannerReporter;
use crate::ecc::planner::rules::{
    AcyclicDependencyRule, DependencyExistsRule, DuplicateDependencyRule, PlanNotEmptyRule,
    PlanStartEndRule, ReachabilityRule, StepIdNotEmptyRule, UniqueStepIdsRule,
};
use crate::ecc::planner::scorer::PlannerConfidenceScorer;
use crate::ecc::planner::stages::{
    PlannerClassificationStage, PlannerCorrectionStage, PlannerReportingStage, PlannerScoringStage,
    PlannerValidationStage, PlannerVerificationStage,
};
use crate::ecc::planner::validator::PlannerValidator;

/// Buat pipeline ECC planner default.
pub fn default_planner_pipeline() -> Pipeline<ExecutionPlan> {
    let rules: Vec<Box<dyn crate::ecc::traits::Rule<ExecutionPlan>>> = vec![
        Box::new(PlanNotEmptyRule),
        Box::new(StepIdNotEmptyRule),
        Box::new(UniqueStepIdsRule),
        Box::new(DependencyExistsRule),
        Box::new(DuplicateDependencyRule),
        Box::new(AcyclicDependencyRule),
        Box::new(ReachabilityRule),
        Box::new(PlanStartEndRule),
    ];

    let validator = PlannerValidator::new(rules);

    Pipeline::new(vec![
        Box::new(PlannerValidationStage::new(Box::new(validator))),
        Box::new(PlannerClassificationStage::new(Box::new(
            PlannerErrorClassifier,
        ))),
        Box::new(PlannerCorrectionStage::new(Box::new(
            PlannerCorrector::new(),
        ))),
        Box::new(PlannerVerificationStage::new(Box::new(
            PlannerValidator::new(vec![
                Box::new(PlanNotEmptyRule),
                Box::new(UniqueStepIdsRule),
                Box::new(DependencyExistsRule),
                Box::new(AcyclicDependencyRule),
                Box::new(ReachabilityRule),
                Box::new(PlanStartEndRule),
            ]),
        ))),
        Box::new(PlannerScoringStage::new(
            Box::new(PlannerConfidenceScorer::new()),
            Box::new(PlannerPolicy::new()),
        )),
        Box::new(PlannerReportingStage::new(Box::new(PlannerReporter::new()))),
    ])
}

/// Helper untuk mem-build engine ECC planner.
pub fn build_planner_ecc_engine() -> crate::ecc::engine::EccEngine<ExecutionPlan> {
    let validator: Box<dyn Validator<ExecutionPlan>> = Box::new(PlannerValidator::new(vec![
        Box::new(PlanNotEmptyRule),
        Box::new(StepIdNotEmptyRule),
        Box::new(UniqueStepIdsRule),
        Box::new(DependencyExistsRule),
        Box::new(DuplicateDependencyRule),
        Box::new(AcyclicDependencyRule),
        Box::new(ReachabilityRule),
        Box::new(PlanStartEndRule),
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
