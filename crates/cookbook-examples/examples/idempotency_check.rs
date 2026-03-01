//! Two-apply idempotency test for a recipe.
//!
//! Usage: `cargo run --example idempotency_check -- recipes/01-developer-workstation.yaml`
//! Applies twice and verifies the second apply produces zero changes.

use std::path::PathBuf;
use std::process::ExitCode;

use cookbook_runner::RecipeRunner;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let Some(path) = args.get(1) else {
        eprintln!("usage: idempotency_check <recipe.yaml>");
        return ExitCode::FAILURE;
    };
    let file = PathBuf::from(path);

    if !file.exists() {
        eprintln!("error: file not found: {}", file.display());
        return ExitCode::FAILURE;
    }

    let state_dir = std::env::temp_dir().join("cookbook-idempotency-check");
    let _ = std::fs::remove_dir_all(&state_dir);

    let runner = RecipeRunner::from_path();

    // Validate first
    let validate = runner.validate(&file);
    if validate.exit_code != 0 {
        eprintln!("FAIL: validation failed for {}", file.display());
        eprintln!("{}", validate.output);
        return ExitCode::FAILURE;
    }
    eprintln!("  validate: OK ({}ms)", validate.duration_ms);

    // First apply — converge from clean state
    let apply1 = runner.apply(&file, &state_dir);
    if apply1.exit_code != 0 {
        eprintln!("FAIL: first apply failed for {}", file.display());
        eprintln!("{}", apply1.output);
        let _ = std::fs::remove_dir_all(&state_dir);
        return ExitCode::FAILURE;
    }
    eprintln!("  apply #1: OK ({}ms)", apply1.duration_ms);

    // Second apply — must produce zero changes
    let apply2 = runner.apply(&file, &state_dir);
    if apply2.exit_code != 0 {
        eprintln!("FAIL: second apply failed for {}", file.display());
        eprintln!("{}", apply2.output);
        let _ = std::fs::remove_dir_all(&state_dir);
        return ExitCode::FAILURE;
    }
    eprintln!("  apply #2: OK ({}ms)", apply2.duration_ms);

    let _ = std::fs::remove_dir_all(&state_dir);

    let idempotent = apply2.output.contains("0 changed");
    if idempotent {
        eprintln!(
            "IDEMPOTENT: {} (first: {}ms, second: {}ms)",
            file.display(),
            apply1.duration_ms,
            apply2.duration_ms
        );
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "NOT IDEMPOTENT: {} — second apply made changes",
            file.display()
        );
        eprintln!("output: {}", apply2.output);
        ExitCode::FAILURE
    }
}
