//! Cron management — scheduled tasks and automation.
//!
//! Demonstrates the `cron` resource type with schedule expressions,
//! validation, and integration with the rules engine.
//!
//! Usage: `cargo run --example cron_management`

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-cron");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    eprintln!("--- Cron Management ---\n");

    let config_path = tmp.join("forjar.yaml");
    std::fs::write(
        &config_path,
        r#"version: "1.0"
name: cron-demo
machines:
  server:
    hostname: srv-01
    addr: 10.0.1.5
    user: deploy
resources:
  backup-cron:
    type: cron
    machine: server
    schedule: "0 2 * * *"
    command: "/usr/local/bin/backup.sh"
    sudo: true
  log-rotate:
    type: cron
    machine: server
    schedule: "0 0 * * 0"
    command: "logrotate /etc/logrotate.conf"
    sudo: true
  health-check:
    type: cron
    machine: server
    schedule: "*/5 * * * *"
    command: "curl -sf http://localhost/health || systemctl restart app"
  cert-renew:
    type: cron
    machine: server
    schedule: "0 3 * * 1"
    command: "certbot renew --quiet"
    sudo: true
    depends_on: [health-check]
"#,
    )
    .ok();

    let f = config_path.display().to_string();
    let mut failures = 0u32;

    eprintln!("Step 1: Validate");
    let r = run_forjar(&["validate", "-f", &f]);
    report("validate", &r, &mut failures);

    eprintln!("Step 2: Plan");
    let r = run_forjar(&["plan-compact", "-f", &f]);
    report("plan", &r, &mut failures);

    eprintln!("Step 3: Test (check scripts)");
    let r = run_forjar(&["test", "-f", &f]);
    report("test", &r, &mut failures);

    eprintln!("Step 4: Lint");
    let r = run_forjar(&["lint", "-f", &f]);
    report("lint", &r, &mut failures);

    eprintln!("Step 5: Privilege analysis");
    let r = run_forjar(&["privilege-analysis", "-f", &f]);
    report("privilege", &r, &mut failures);

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
