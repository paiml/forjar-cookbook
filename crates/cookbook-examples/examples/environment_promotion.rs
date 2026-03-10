//! Environment promotion with rollback and history.
//!
//! Demonstrates the environment pipeline: list environments and view
//! promotion history. Creates a self-contained config with `environments:`
//! block.
//!
//! Usage: `cargo run --example environment_promotion`

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-env-promotion");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    eprintln!("--- Environment Promotion ---\n");

    let config_path = tmp.join("forjar.yaml");
    std::fs::write(
        &config_path,
        r#"version: "1.0"
name: env-promo-demo
environments:
  dev:
    machine_filter: [dev-server]
  staging:
    machine_filter: [staging-server]
machines:
  dev-server:
    hostname: dev-01
    addr: 10.0.1.10
    user: deploy
  staging-server:
    hostname: staging-01
    addr: 10.0.1.20
    user: deploy
resources:
  app-pkg:
    type: package
    machine: [dev-server, staging-server]
    provider: apt
    packages: [nginx]
"#,
    )
    .ok();

    let f = config_path.display().to_string();
    let sd = tmp.join("state").display().to_string();
    let mut failures = 0u32;

    // Step 1: List environments
    eprintln!("Step 1: List defined environments");
    let list = run_forjar(&["environments", "list", "-f", &f]);
    report_step("list", &list, &mut failures);

    // Step 2: View promotion history (works even without state)
    eprintln!("Step 2: View environment history");
    let history = run_forjar(&[
        "environments",
        "history",
        "staging",
        "--state-dir",
        &sd,
        "--limit",
        "10",
    ]);
    report_step("history", &history, &mut failures);

    let _ = std::fs::remove_dir_all(&tmp);

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
        for line in result.output.lines().take(5) {
            if !line.trim().is_empty() {
                eprintln!("    {line}");
            }
        }
    } else {
        eprintln!(
            "  {name}: FAIL ({}ms) — {}",
            result.duration_ms,
            result.output.lines().next().unwrap_or("").trim()
        );
        *failures += 1;
    }
}
