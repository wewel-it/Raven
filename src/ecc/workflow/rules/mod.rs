//! Workflow validation rules.
//!
//! Setiap rule memeriksa aspek spesifik dari workflow untuk memastikan validitas.

use crate::ecc::traits::Rule;
use crate::ecc::workflow::types::Workflow;

mod deadlock;
mod dependency_cycle;
mod dependency_exists;
mod disconnected_graph;
mod duplicate_step;
mod end_state;
mod orphan_node;
mod reachability;
mod retry_config;
mod start_state;
mod terminal_state;
mod timeout;
mod unique_step_id;
mod workflow_id;
mod workflow_not_empty;

pub use deadlock::DeadlockRule;
pub use dependency_cycle::DependencyCycleRule;
pub use dependency_exists::DependencyExistsRule;
pub use disconnected_graph::DisconnectedGraphRule;
pub use duplicate_step::DuplicateStepRule;
pub use end_state::EndStateRule;
pub use orphan_node::OrphanNodeRule;
pub use reachability::ReachabilityRule;
pub use retry_config::RetryConfigRule;
pub use start_state::StartStateRule;
pub use terminal_state::TerminalStateRule;
pub use timeout::TimeoutRule;
pub use unique_step_id::UniqueStepIdRule;
pub use workflow_id::WorkflowIdRule;
pub use workflow_not_empty::WorkflowNotEmptyRule;

/// Dapatkan semua rule standar untuk workflow.
pub fn get_all_rules() -> Vec<Box<dyn Rule<Workflow>>> {
    vec![
        Box::new(WorkflowIdRule),
        Box::new(WorkflowNotEmptyRule),
        Box::new(UniqueStepIdRule),
        Box::new(DependencyExistsRule),
        Box::new(DependencyCycleRule),
        Box::new(ReachabilityRule),
        Box::new(TerminalStateRule),
        Box::new(StartStateRule),
        Box::new(EndStateRule),
        Box::new(DuplicateStepRule),
        Box::new(OrphanNodeRule),
        Box::new(DeadlockRule),
        Box::new(DisconnectedGraphRule),
        Box::new(TimeoutRule),
        Box::new(RetryConfigRule),
    ]
}
