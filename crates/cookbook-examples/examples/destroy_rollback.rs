//! Destroy and rollback — safe infrastructure teardown and recovery.
//!
//! Demonstrates `forjar destroy` (dry-run), `forjar undo` (dry-run),
//! and generation-based rollback for safe reversibility.
//!
//! Usage: `cargo run --example destroy_rollback`

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-destroy-rollback");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    eprintln!("--- Destroy & Rollback ---\n");

    let config_path = tmp.join("forjar.yaml");
    std::fs::write(
        &config_path,
        r#"version: "1.0"
name: destroy-demo
machines:
  local:
    hostname: localhost
    addr: 127.0.0.1
    user: demo
resources:
  app-dir:
    type: file
    machine: local
    path: /tmp/cookbook-destroy/app
    content: "application files"
  config-file:
    type: file
    machine: local
    path: /tmp/cookbook-destroy/config.yml
    content: "port: 8080"
    depends_on: [app-dir]
  web-pkg:
    type: package
    machine: local
    provider: apt
    packages: [nginx]
    depends_on: [config-file]
"#,
    )
    .ok();

    let f = config_path.display().to_string();
    let state_dir = tmp.join("state");
    let sd = state_dir.display().to_string();
    let mut failures = 0u32;

    eprintln!("Step 1: Validate");
    let r = run_forjar(&["validate", "-f", &f]);
    report("validate", &r, &mut failures);

    eprintln!("Step 2: Plan (shows what would be applied)");
    let r = run_forjar(&["plan-compact", "-f", &f]);
    report("plan", &r, &mut failures);

    eprintln!("Step 3: Lock state");
    let r = run_forjar(&["lock", "-f", &f, "--state-dir", &sd]);
    report("lock", &r, &mut failures);

    eprintln!("Step 4: Destroy dry-run (reverse order)");
    let r = run_forjar(&["destroy", "-f", &f, "--state-dir", &sd, "--yes"]);
    report("destroy", &r, &mut failures);

    eprintln!("Step 5: Undo capabilities");
    let r = run_forjar(&["undo", "--help"]);
    report("undo-help", &r, &mut failures);

    eprintln!("Step 6: Generation list");
    let r = run_forjar(&["generation", "list", "--state-dir", &sd]);
    report("generation", &r, &mut failures);

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
