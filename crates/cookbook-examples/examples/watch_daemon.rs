//! Watch daemon — continuous monitoring and auto-remediation.
//!
//! Demonstrates `forjar watch` (immediate exit), `forjar agent` modes,
//! and `forjar data-freshness` for continuous infrastructure monitoring.
//!
//! Usage: `cargo run --example watch_daemon`

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-watch");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    eprintln!("--- Watch Daemon ---\n");

    let config_path = tmp.join("forjar.yaml");
    std::fs::write(
        &config_path,
        r#"version: "1.0"
name: watch-demo
machines:
  local:
    hostname: localhost
    addr: 127.0.0.1
    user: demo
resources:
  app-config:
    type: file
    machine: local
    path: /tmp/cookbook-watch/app.conf
    content: "port: 8080"
  app-pkg:
    type: package
    machine: local
    provider: apt
    packages: [curl]
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

    eprintln!("Step 2: Lock state");
    let r = run_forjar(&["lock", "-f", &f, "--state-dir", &sd]);
    report("lock", &r, &mut failures);

    eprintln!("Step 3: Watch help (shows daemon options)");
    let r = run_forjar(&["watch", "--help"]);
    report("watch-help", &r, &mut failures);

    eprintln!("Step 4: Agent help (push/pull modes)");
    let r = run_forjar(&["agent", "--help"]);
    report("agent-help", &r, &mut failures);

    eprintln!("Step 5: Data freshness check");
    let r = run_forjar(&["data-freshness", "-f", &f, "--state-dir", &sd]);
    report("freshness", &r, &mut failures);

    eprintln!("Step 6: Data validation");
    let r = run_forjar(&["data-validate", "-f", &f, "--state-dir", &sd]);
    report("data-validate", &r, &mut failures);

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
