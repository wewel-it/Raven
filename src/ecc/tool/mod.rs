//! Tool ECC subsystem for deterministic validation, correction, classification,
//! policy enforcement, confidence scoring, and reporting of tool calls.

pub mod builder;
pub mod classifier;
pub mod confidence;
pub mod context;
pub mod corrector;
pub mod engine;
pub mod errors;
pub mod pipeline;
pub mod policy;
pub mod report;
pub mod rules;
pub mod tests;
pub mod types;
pub mod validator;

pub use builder::ToolEccBuilder;
pub use classifier::ToolErrorClassifier;
pub use confidence::ToolConfidenceScorer;
pub use context::{ToolDescriptor, ToolEccContext};
pub use corrector::ToolCorrector;
pub use engine::ToolEccEngine;
pub use errors::{ToolEccError, ToolEccResult};
pub use policy::ToolPolicy;
pub use report::ToolReporter;
pub use rules::{ToolExistsRule, ToolRule};
pub use types::{ToolCall, VerifiedToolCall};
