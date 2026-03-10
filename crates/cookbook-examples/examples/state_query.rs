//! State querying — inspect applied state with rich queries.
//!
//! Demonstrates `forjar state-query` with health, timing, churn,
//! reversibility, and history views for operational insight.
//!
//! Usage: `cargo run --example state_query`

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-state-query");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    eprintln!("--- State Querying ---\n");

    let config_path = tmp.join("forjar.yaml");
    std::fs::write(
        &config_path,
        r#"version: "1.0"
name: state-query-demo
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
    packages: [nginx, curl]
  app-config:
    type: file
    machine: local
    path: /tmp/cookbook-sq/app.conf
    content: "port: 8080"
  web-svc:
    type: service
    machine: local
    name: nginx
    enabled: true
    depends_on: [web-pkg, app-config]
"#,
    )
    .ok();

    let f = config_path.display().to_string();
    let state_dir = tmp.join("state");
    let sd = state_dir.display().to_string();
    let mut failures = 0u32;

    eprintln!("Step 1: Lock state");
    let r = run_forjar(&["lock", "-f", &f, "--state-dir", &sd]);
    report("lock", &r, &mut failures);

    eprintln!("Step 2: State health summary");
    let r = run_forjar(&["state-query", "--health", "--state-dir", &sd]);
    report("health", &r, &mut failures);

    eprintln!("Step 3: Query by pattern");
    let r = run_forjar(&["state-query", "nginx", "--state-dir", &sd]);
    report("query", &r, &mut failures);

    eprintln!("Step 4: State list");
    let r = run_forjar(&["state-list", "--state-dir", &sd]);
    report("state-list", &r, &mut failures);

    eprintln!("Step 5: Lock info");
    let r = run_forjar(&["lock-info", "--state-dir", &sd]);
    report("lock-info", &r, &mut failures);

    eprintln!("Step 6: Lock stats");
    let r = run_forjar(&["lock-stats", "--state-dir", &sd]);
    report("lock-stats", &r, &mut failures);

    eprintln!("Step 7: Lock verify");
    let r = run_forjar(&["lock-verify", "--state-dir", &sd]);
    report("lock-verify", &r, &mut failures);

    eprintln!("Step 8: Lock export (JSON)");
    let r = run_forjar(&["lock-export", "--state-dir", &sd, "--format", "json"]);
    report("lock-export", &r, &mut failures);

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
