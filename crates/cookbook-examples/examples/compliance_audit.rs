//! Compliance pack audit — validate CIS hardening recipes.
//!
//! Runs `forjar validate` with deep validation on compliance-related recipes,
//! then checks policy evaluation results. Demonstrates how forjar's built-in
//! compliance packs (CIS, STIG, SOC2) integrate with the recipe pipeline.
//!
//! Usage: `cargo run --example compliance_audit`

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let root = match cookbook_examples::find_project_root() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    let recipes_dir = root.join("recipes");

    // Compliance-related recipes to audit
    let compliance_recipes = [
        "92-cis-hardening.yaml",
        "09-secure-baseline.yaml",
        "81-security-hardening.yaml",
    ];

    eprintln!("--- Compliance Audit ---\n");

    let mut total = 0u32;
    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut skipped = 0u32;

    for recipe_name in &compliance_recipes {
        let recipe_path = recipes_dir.join(recipe_name);
        total += 1;

        if !recipe_path.exists() {
            eprintln!("  SKIP: {recipe_name} — not found");
            skipped += 1;
            continue;
        }

        // Step 1: Validate with deep checks
        let validate = run_forjar(&[
            "validate",
            "-f",
            &recipe_path.display().to_string(),
            "--deep",
        ]);

        if !validate.success {
            eprintln!(
                "  FAIL: {recipe_name} — validate: {}",
                first_line(&validate.output)
            );
            failed += 1;
            continue;
        }
        eprintln!("  validate: {recipe_name} OK ({}ms)", validate.duration_ms);

        // Step 2: Check policy evaluation
        let policy = run_forjar(&[
            "validate",
            "-f",
            &recipe_path.display().to_string(),
            "--policy",
        ]);

        if policy.success {
            eprintln!("  policy:   {recipe_name} OK ({}ms)", policy.duration_ms);
        } else {
            eprintln!(
                "  policy:   {recipe_name} WARN ({}ms) — {}",
                policy.duration_ms,
                first_line(&policy.output)
            );
        }
        // Policy warnings are not hard failures
        passed += 1;
    }

    eprintln!(
        "\n--- Compliance Audit: {passed}/{total} passed, {failed} failed, {skipped} skipped ---"
    );

    if failed > 0 {
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

fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or(s).trim()
}
