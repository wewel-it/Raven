use crate::ecc::pipeline::{Pipeline, PipelineContext};
use crate::planner::ExecutionPlan;

/// Alias untuk pipeline planner.
pub type PlannerPipeline = Pipeline<ExecutionPlan>;
pub type PlannerPipelineContext = PipelineContext<ExecutionPlan>;
