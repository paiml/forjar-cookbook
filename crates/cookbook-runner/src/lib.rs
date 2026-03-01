//! Forjar cookbook recipe execution harness.
//!
//! Runs forjar commands (validate, plan, apply) against recipe configs,
//! captures timing and results, and updates the qualification CSV.

mod commands;
mod report;
mod runner;

pub use commands::{
    grade_passes_threshold, run_qualify, run_validate, score_after_qualify, score_recipe_file,
};
pub use report::{
    QualifyVerdict, format_qualify_report, format_score_report, format_validate_report,
    runtime_data_from_qualify, verdict,
};
pub use runner::{QualifyResult, RecipeRunner, RunOutcome};
