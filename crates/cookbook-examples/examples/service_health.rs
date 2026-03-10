//! Service health — health checks, restart policies, and monitoring.
//!
//! Demonstrates service resources with health checks, restart policies,
//! and the `forjar doctor` pre-flight diagnostics.
//!
//! Usage: `cargo run --example service_health`

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-service-health");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    eprintln!("--- Service Health ---\n");

    let config_path = tmp.join("forjar.yaml");
    std::fs::write(
        &config_path,
        r#"version: "1.0"
name: service-health-demo
machines:
  app:
    hostname: app-01
    addr: 10.0.1.10
    user: deploy
resources:
  web-server:
    type: service
    machine: app
    name: nginx
    enabled: true
    health_check: "curl -sf http://localhost/health"
    restart_policy: on-failure
    restart_delay: 5
  api-service:
    type: service
    machine: app
    name: api
    enabled: true
    health_check: "curl -sf http://localhost:8080/ready"
    restart_policy: always
    restart_delay: 10
    depends_on: [web-server]
  worker:
    type: service
    machine: app
    name: worker
    enabled: true
    health_check: "pgrep -f worker"
    depends_on: [api-service]
"#,
    )
    .ok();

    let f = config_path.display().to_string();
    let mut failures = 0u32;

    eprintln!("Step 1: Validate");
    let r = run_forjar(&["validate", "-f", &f]);
    report("validate", &r, &mut failures);

    eprintln!("Step 2: Doctor pre-flight checks");
    let r = run_forjar(&["doctor"]);
    report("doctor", &r, &mut failures);

    eprintln!("Step 3: Dependency graph");
    let r = run_forjar(&["graph", "-f", &f]);
    report("graph", &r, &mut failures);

    eprintln!("Step 4: Plan");
    let r = run_forjar(&["plan-compact", "-f", &f]);
    report("plan", &r, &mut failures);

    eprintln!("Step 5: Test (check scripts)");
    let r = run_forjar(&["test", "-f", &f]);
    report("test", &r, &mut failures);

    eprintln!("Step 6: Suggest improvements");
    let r = run_forjar(&["suggest", "-f", &f]);
    report("suggest", &r, &mut failures);

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
