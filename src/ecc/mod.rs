//! ECC (Cognitive Error Correction) module.
//!
//! Module ini menyediakan kerangka layanan ECC yang dapat digunakan oleh seluruh
//! komponen Raven untuk melakukan validasi, koreksi, verifikasi, dan pelaporan
//! kesalahan deterministik.

pub mod corrector;
pub mod engine;
pub mod errors;
pub mod memory;
pub mod pipeline;
pub mod planner;
pub mod policy;
pub mod report;
pub mod rules;
pub mod tool;
pub mod traits;
pub mod validator;
pub mod workflow;

pub use corrector::CompositeCorrector;
pub use engine::EccEngine;
pub use errors::{EccError, EccResult};
pub use pipeline::{Pipeline, PipelineContext, PipelineStage};
pub use policy::{Policy, PolicyAction, PolicyDecision};
pub use report::{ConfidenceScore, EccReport, ErrorClassification, ValidationReport};
pub use rules::RuleSet;
pub use traits::{ConfidenceScorer, Corrector, ErrorClassifier, Reporter, Rule, Validator};
pub use validator::RuleBasedValidator;
