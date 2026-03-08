//! Extracted CLI subcommand handlers for testability.

use std::path::Path;

use cookbook_qualify::{ForjarScore, Grade, IdempotencyClass, RecipeStatus, ScoringInput};

use crate::{
    QualifyResult, RecipeRunner, format_qualify_report, format_score_report,
    format_validate_report, runtime_data_from_qualify, verdict,
};

/// Run the validate subcommand: validate a recipe and return the report.
///
/// # Errors
///
/// Returns error if validation fails (non-zero exit code).
pub fn run_validate(forjar: &str, file: &Path) -> Result<String, String> {
    let runner = RecipeRunner::new(forjar);
    let outcome = runner.validate(file);
    let report = format_validate_report(file, outcome.exit_code, outcome.duration_ms);
    if outcome.exit_code != 0 {
        return Err(format!(
            "{report}\n{}\nvalidation failed: {}",
            outcome.output,
            file.display()
        ));
    }
    Ok(report)
}

/// Run the qualify subcommand: full qualification cycle with scoring.
///
/// # Errors
///
/// Returns error if any qualification step fails.
pub fn run_qualify(forjar: &str, file: &Path, state_dir: &Path) -> Result<String, String> {
    let runner = RecipeRunner::new(forjar);
    let result = runner.qualify(file, state_dir);
    let mut report = format_qualify_report(file, &result);

    if let Some((score_report, _)) = score_after_qualify(file, &result) {
        report.push('\n');
        report.push_str(&score_report);
    }

    let v = verdict(&result);
    if let Some(msg) = v.error_message() {
        return Err(format!("{report}\n{msg}"));
    }
    Ok(report)
}

/// Score a recipe from a YAML file (static analysis, no apply).
///
/// Returns `(report_text, score)`.
///
/// # Errors
///
/// Returns error if the file cannot be read, YAML is invalid, or
/// the status/idempotency strings are unrecognised.
pub fn score_recipe_file(
    file: &Path,
    status: &str,
    idempotency: &str,
    budget_ms: u64,
) -> Result<(String, ForjarScore), String> {
    let raw_yaml = std::fs::read_to_string(file)
        .map_err(|e| format!("cannot read {}: {e}", file.display()))?;
    let recipe_status = RecipeStatus::from_csv(status)?;
    let idem_class = IdempotencyClass::from_csv(idempotency)?;
    let input = ScoringInput {
        status: &recipe_status,
        idempotency_class: &idem_class,
        raw_yaml: &raw_yaml,
        budget_ms,
        runtime: None,
    };
    let score = ForjarScore::compute(&input);
    let report = format_score_report(&score);
    Ok((report, score))
}

/// Check whether a score grade passes the threshold (A-C pass, D-F fail).
#[must_use]
pub const fn grade_passes_threshold(grade: &Grade) -> bool {
    matches!(grade, Grade::A | Grade::B | Grade::C)
}

/// Compute and format a score after a qualification run.
///
/// Returns the score report text if scoring succeeds, or `None` if
/// the recipe file cannot be read or parsed.
#[must_use]
pub fn score_after_qualify(file: &Path, result: &QualifyResult) -> Option<(String, ForjarScore)> {
    let raw_yaml = std::fs::read_to_string(file).ok()?;
    let rt = runtime_data_from_qualify(result);
    let v = verdict(result);
    let status = if v.is_qualified() {
        RecipeStatus::Qualified
    } else {
        RecipeStatus::Pending
    };
    let input = ScoringInput {
        status: &status,
        idempotency_class: &IdempotencyClass::Strong,
        raw_yaml: &raw_yaml,
        budget_ms: 0,
        runtime: Some(&rt),
    };
    let score = ForjarScore::compute(&input);
    let report = format_score_report(&score);
    Some((report, score))
}

#[cfg(test)]
mod tests;
