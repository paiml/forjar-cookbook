//! Forjar cookbook recipe execution harness.
//!
//! Runs forjar commands (validate, plan, apply) against recipe configs,
//! captures timing and results, and updates the qualification CSV.

mod report;
mod runner;

pub use report::{QualifyVerdict, format_qualify_report, format_validate_report, verdict};
pub use runner::{QualifyResult, RecipeRunner, RunOutcome};
