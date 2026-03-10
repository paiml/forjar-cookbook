//! Progressive rollout — canary and rolling deployment strategies.
//!
//! Demonstrates `forjar canary`, `forjar rolling`, and `forjar parallel-apply`
//! for staged deployments across machine fleets.
//!
//! Usage: `cargo run --example progressive_rollout`

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-rollout");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    eprintln!("--- Progressive Rollout ---\n");

    let config_path = tmp.join("forjar.yaml");
    std::fs::write(
        &config_path,
        r#"version: "1.0"
name: rollout-demo
machines:
  canary:
    hostname: web-canary
    addr: 10.0.1.10
    user: deploy
    roles: [web, canary]
  web-01:
    hostname: web-01
    addr: 10.0.1.11
    user: deploy
    roles: [web]
  web-02:
    hostname: web-02
    addr: 10.0.1.12
    user: deploy
    roles: [web]
  web-03:
    hostname: web-03
    addr: 10.0.1.13
    user: deploy
    roles: [web]
resources:
  app-pkg:
    type: package
    machine: [canary, web-01, web-02, web-03]
    provider: apt
    packages: [nginx]
  app-config:
    type: file
    machine: [canary, web-01, web-02, web-03]
    path: /var/www/app/config.yaml
    content: "server { listen 80; root /var/www/app; }"
    depends_on: [app-pkg]
"#,
    )
    .ok();

    let f = config_path.display().to_string();
    let mut failures = 0u32;

    eprintln!("Step 1: Validate fleet config");
    let r = run_forjar(&["validate", "-f", &f]);
    report("validate", &r, &mut failures);

    eprintln!("Step 2: Canary deployment plan (canary machine first)");
    let r = run_forjar(&["canary", "--machine", "canary", "-f", &f, "--help"]);
    report("canary-help", &r, &mut failures);

    eprintln!("Step 3: Rolling deployment plan (batch size 2)");
    let r = run_forjar(&["rolling", "-f", &f, "--batch-size", "2", "--help"]);
    report("rolling-help", &r, &mut failures);

    eprintln!("Step 4: Inventory check");
    let r = run_forjar(&["inventory", "-f", &f]);
    report("inventory", &r, &mut failures);

    eprintln!("Step 5: Impact analysis");
    let r = run_forjar(&["impact", "-f", &f, "--resource", "app-config"]);
    report("impact", &r, &mut failures);

    eprintln!("Step 6: Preservation check");
    let r = run_forjar(&["preservation", "-f", &f]);
    report("preservation", &r, &mut failures);

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
