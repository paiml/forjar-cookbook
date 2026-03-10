//! Environment promotion with rollback and history.
//!
//! Demonstrates the environment pipeline: list environments, promote with
//! quality gates, view promotion history, and rollback. Requires a config
//! with `environments:` and `promotion:` blocks.
//!
//! Usage: `cargo run --example environment_promotion -- recipes/94-canary-deployment.yaml`
//! (or any config with environment definitions)

use std::path::PathBuf;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let file = args.get(1).map_or_else(
        || {
            // Default: find a config with environment definitions
            let root = cookbook_examples::find_project_root().unwrap_or_default();
            root.join("recipes/94-canary-deployment.yaml")
        },
        PathBuf::from,
    );

    if !file.exists() {
        eprintln!("error: config not found: {}", file.display());
        return ExitCode::FAILURE;
    }

    let state_dir = std::env::temp_dir().join("cookbook-env-promotion");
    let _ = std::fs::remove_dir_all(&state_dir);
    std::fs::create_dir_all(&state_dir).ok();

    let mut failures = 0u32;

    eprintln!("--- Environment Promotion ---");
    eprintln!("  config: {}\n", file.display());

    // Step 1: List environments
    eprintln!("Step 1: List defined environments");
    let list = run_forjar(&["environments", "list", "-f", &file.display().to_string()]);
    report_step("list", &list, &mut failures);
    if list.success {
        eprintln!("  output: {}", list.output.trim());
    }

    // Step 2: Promote (dry-run) to staging
    eprintln!("Step 2: Promote to staging (dry-run)");
    let promote = run_forjar(&[
        "promote",
        "-f",
        &file.display().to_string(),
        "--target",
        "staging",
        "--dry-run",
    ]);
    report_step("promote-dry-run", &promote, &mut failures);

    // Step 3: View promotion history
    eprintln!("Step 3: View environment history");
    let history = run_forjar(&[
        "environments",
        "history",
        "staging",
        "--state-dir",
        &state_dir.display().to_string(),
        "--limit",
        "10",
    ]);
    report_step("history", &history, &mut failures);

    // Step 4: Environment diff (dev vs staging)
    eprintln!("Step 4: Diff environments (dev vs staging)");
    let diff = run_forjar(&[
        "environments",
        "diff",
        "dev",
        "staging",
        "-f",
        &file.display().to_string(),
    ]);
    report_step("diff", &diff, &mut failures);

    // Step 5: Rollback
    eprintln!("Step 5: Rollback environment");
    let rollback = run_forjar(&[
        "environments",
        "rollback",
        "staging",
        "--state-dir",
        &state_dir.display().to_string(),
        "--generations",
        "1",
        "--yes",
    ]);
    report_step("rollback", &rollback, &mut failures);

    let _ = std::fs::remove_dir_all(&state_dir);

    eprintln!("\n--- Result: {failures} failure(s) ---");
    if failures > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

struct StepResult {
    success: bool,
    output: String,
    duration_ms: u64,
}

fn run_forjar(args: &[&str]) -> StepResult {
    let start = std::time::Instant::now();
    let result = Command::new("forjar").args(args).output();
    let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            StepResult {
                success: output.status.success(),
                output: format!("{stdout}{stderr}"),
                duration_ms,
            }
        }
        Err(e) => StepResult {
            success: false,
            output: format!("failed to execute forjar: {e}"),
            duration_ms,
        },
    }
}

fn report_step(name: &str, result: &StepResult, failures: &mut u32) {
    if result.success {
        eprintln!("  {name}: OK ({}ms)", result.duration_ms);
    } else {
        eprintln!(
            "  {name}: FAIL ({}ms) — {}",
            result.duration_ms,
            first_line(&result.output)
        );
        *failures += 1;
    }
}

fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or(s).trim()
}
