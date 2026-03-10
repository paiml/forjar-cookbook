//! Task caching — BLAKE3 input-based build task caching.
//!
//! Demonstrates task resources with `cache: true`, `task_inputs`,
//! `output_artifacts`, and the store/pin workflow for reproducible builds.
//!
//! Usage: `cargo run --example task_caching`

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-task-cache");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    eprintln!("--- Task Caching ---\n");

    let config_path = tmp.join("forjar.yaml");
    std::fs::write(
        &config_path,
        r#"version: "1.0"
name: task-cache-demo
machines:
  build:
    hostname: build-server
    addr: 10.0.1.50
    user: ci
resources:
  compile-app:
    type: task
    machine: build
    task_mode: batch
    command: "cargo build --release"
    working_dir: /home/ci/app
    task_inputs:
      - src/**/*.rs
      - Cargo.toml
      - Cargo.lock
    output_artifacts:
      - target/release/myapp
    cache: true
    timeout: 600
  run-tests:
    type: task
    machine: build
    task_mode: batch
    command: "cargo test"
    working_dir: /home/ci/app
    task_inputs:
      - src/**/*.rs
      - tests/**/*.rs
    cache: true
    depends_on: [compile-app]
    timeout: 300
  deploy-artifact:
    type: task
    machine: build
    task_mode: batch
    command: "cp target/release/myapp /opt/bin/"
    working_dir: /home/ci/app
    depends_on: [run-tests]
"#,
    )
    .ok();

    let f = config_path.display().to_string();
    let state_dir = tmp.join("state");
    let sd = state_dir.display().to_string();
    let mut failures = 0u32;

    eprintln!("Step 1: Validate task pipeline");
    let r = run_forjar(&["validate", "-f", &f]);
    report("validate", &r, &mut failures);

    eprintln!("Step 2: Plan execution order");
    let r = run_forjar(&["plan-compact", "-f", &f]);
    report("plan", &r, &mut failures);

    eprintln!("Step 3: Pin inputs");
    let r = run_forjar(&["pin", "-f", &f, "--state-dir", &sd]);
    report("pin", &r, &mut failures);

    eprintln!("Step 4: Convert to reproducible");
    let r = run_forjar(&["convert", "-f", &f, "--reproducible"]);
    report("convert", &r, &mut failures);

    eprintln!("Step 5: Cost estimate");
    let r = run_forjar(&["cost-estimate", "-f", &f]);
    report("cost", &r, &mut failures);

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
}

fn run_forjar(args: &[&str]) -> StepResult {
    match Command::new("forjar").args(args).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            StepResult {
                success: output.status.success(),
                output: format!("{stdout}{stderr}"),
            }
        }
        Err(e) => StepResult {
            success: false,
            output: format!("failed to execute forjar: {e}"),
        },
    }
}

fn report(name: &str, r: &StepResult, failures: &mut u32) {
    if r.success {
        eprintln!("  {name}: OK");
        for line in r.output.lines().take(5) {
            if !line.trim().is_empty() {
                eprintln!("    {line}");
            }
        }
    } else {
        eprintln!(
            "  {name}: FAIL — {}",
            r.output.lines().next().unwrap_or("").trim()
        );
        *failures += 1;
    }
}
