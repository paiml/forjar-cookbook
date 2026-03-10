//! Event-driven automation — validate and test rulebooks.
//!
//! Demonstrates the event-driven automation pipeline: validate rulebook YAML,
//! check event type coverage, and trigger a dry-run. Uses `forjar rules validate`,
//! `forjar rules coverage`, and `forjar trigger`.
//!
//! Usage: `cargo run --example event_rulebook`

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-event-rulebook");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    eprintln!("--- Event-Driven Automation ---\n");

    let mut failures = 0u32;

    // Step 1: Write a test rulebook
    let rulebook_path = tmp.join("rulebook.yaml");
    std::fs::write(
        &rulebook_path,
        r#"version: "1.0"
name: cookbook-demo
rulebooks:
  - name: auto-remediate-drift
    events:
      - type: file_changed
        match:
          path: /etc/nginx/nginx.conf
    actions:
      - script: "echo 'Drift detected, re-applying config'"
    cooldown_secs: 60
  - name: notify-on-failure
    events:
      - type: process_exit
    actions:
      - script: "echo 'Process exited unexpectedly'"
  - name: metric-alert
    events:
      - type: metric_threshold
    actions:
      - script: "echo 'Metric threshold crossed'"
    cooldown_secs: 30
"#,
    )
    .ok();

    // Step 2: Write a minimal config for trigger
    let config_path = tmp.join("forjar.yaml");
    std::fs::write(
        &config_path,
        "version: '1.0'\nname: event-demo\nresources: {}\n",
    )
    .ok();

    // Step 3: Validate the rulebook
    eprintln!("Step 1: Validate rulebook");
    let validate = run_forjar(&[
        "rules",
        "validate",
        "-f",
        &rulebook_path.display().to_string(),
    ]);
    if validate.success {
        eprintln!("  OK: rulebook valid ({}ms)", validate.duration_ms);
    } else {
        eprintln!("  FAIL: {}", first_line(&validate.output));
        failures += 1;
    }

    // Step 4: Check event type coverage
    eprintln!("\nStep 2: Event type coverage");
    let coverage = run_forjar(&[
        "rules",
        "coverage",
        "-f",
        &rulebook_path.display().to_string(),
    ]);
    if coverage.success {
        eprintln!("  OK: coverage report ({}ms)", coverage.duration_ms);
        for line in coverage.output.lines().take(5) {
            if !line.trim().is_empty() {
                eprintln!("  {line}");
            }
        }
    } else {
        eprintln!("  FAIL: {}", first_line(&coverage.output));
        failures += 1;
    }

    // Step 5: Trigger a dry-run
    eprintln!("\nStep 3: Trigger dry-run (drift_detected)");
    let trigger = run_forjar(&[
        "trigger",
        "auto-remediate-drift",
        "-f",
        &rulebook_path.display().to_string(),
        "--dry-run",
    ]);
    if trigger.success {
        eprintln!("  OK: trigger completed ({}ms)", trigger.duration_ms);
    } else {
        eprintln!("  FAIL: {}", first_line(&trigger.output));
        failures += 1;
    }

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

fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or(s).trim()
}
