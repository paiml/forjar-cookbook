//! Qualify a single recipe — full cycle with scoring.
//!
//! Usage: `cargo run --example qualify_recipe -- recipes/01-developer-workstation.yaml`
//! Runs validate → plan → apply → idempotency → score.

use std::path::PathBuf;
use std::process::ExitCode;

use cookbook_qualify::{ForjarScore, IdempotencyClass, RecipeStatus, ScoringInput};
use cookbook_runner::{
    RecipeRunner, format_qualify_report, format_score_report, runtime_data_from_qualify, verdict,
};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let Some(path) = args.get(1) else {
        eprintln!("usage: qualify_recipe <recipe.yaml>");
        return ExitCode::FAILURE;
    };
    let file = PathBuf::from(path);

    if !file.exists() {
        eprintln!("error: file not found: {}", file.display());
        return ExitCode::FAILURE;
    }

    let state_dir = std::env::temp_dir().join("cookbook-qualify-recipe");
    let _ = std::fs::remove_dir_all(&state_dir);

    let runner = RecipeRunner::from_path();
    let result = runner.qualify(&file, &state_dir);
    let report = format_qualify_report(&file, &result);
    eprintln!("{report}");

    // Score the recipe with runtime data
    if let Ok(raw_yaml) = std::fs::read_to_string(&file) {
        let rt = runtime_data_from_qualify(&result);
        let v = verdict(&result);
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
        eprintln!("{}", format_score_report(&score));
    }

    let _ = std::fs::remove_dir_all(&state_dir);

    let v = verdict(&result);
    if v.is_qualified() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
