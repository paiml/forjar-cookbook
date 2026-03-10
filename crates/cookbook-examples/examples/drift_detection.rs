//! Drift detection — detect unauthorized changes with BLAKE3 tripwire.
//!
//! Demonstrates `forjar drift`, `forjar drift-predict`, `forjar anomaly`,
//! and `forjar invariants` for comprehensive drift monitoring.
//!
//! Usage: `cargo run --example drift_detection`

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-drift");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    eprintln!("--- Drift Detection ---\n");

    let config_path = tmp.join("forjar.yaml");
    std::fs::write(
        &config_path,
        r#"version: "1.0"
name: drift-demo
machines:
  local:
    hostname: localhost
    addr: 127.0.0.1
    user: demo
resources:
  web-pkg:
    type: package
    machine: local
    provider: apt
    packages: [nginx]
  config-file:
    type: file
    machine: local
    path: /tmp/cookbook-drift/nginx.conf
    content: "worker_processes auto;"
    depends_on: [web-pkg]
  web-svc:
    type: service
    machine: local
    name: nginx
    enabled: true
    depends_on: [config-file]
policy:
  tripwire: true
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

    eprintln!("Step 2: Lock state (baseline)");
    let r = run_forjar(&["lock", "-f", &f, "--state-dir", &sd]);
    report("lock", &r, &mut failures);

    eprintln!("Step 3: Check for drift");
    let r = run_forjar(&["drift", "-f", &f, "--state-dir", &sd]);
    report("drift", &r, &mut failures);

    eprintln!("Step 4: Drift prediction");
    let r = run_forjar(&["drift-predict", "--state-dir", &sd]);
    report("predict", &r, &mut failures);

    eprintln!("Step 5: Anomaly detection");
    let r = run_forjar(&["anomaly", "--state-dir", &sd]);
    report("anomaly", &r, &mut failures);

    eprintln!("Step 6: Runtime invariants");
    let r = run_forjar(&["invariants", "-f", &f]);
    report("invariants", &r, &mut failures);

    eprintln!("Step 7: Lineage (Merkle DAG)");
    let r = run_forjar(&["lineage", "-f", &f]);
    report("lineage", &r, &mut failures);

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
