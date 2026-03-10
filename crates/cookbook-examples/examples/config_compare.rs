//! Config comparison — diff, merge, and stack-diff operations.
//!
//! Demonstrates `forjar compare`, `forjar config-merge`, `forjar stack-diff`,
//! and `forjar fmt` for managing config evolution.
//!
//! Usage: `cargo run --example config_compare`

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-compare");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    eprintln!("--- Config Comparison ---\n");

    let config_a = tmp.join("config-a.yaml");
    std::fs::write(
        &config_a,
        r#"version: "1.0"
name: config-a
machines:
  web:
    hostname: web-01
    addr: 10.0.1.10
    user: deploy
resources:
  nginx:
    type: package
    machine: web
    provider: apt
    packages: [nginx]
  app-config:
    type: file
    machine: web
    path: /etc/app.conf
    content: "port: 8080"
"#,
    )
    .ok();

    let config_b = tmp.join("config-b.yaml");
    std::fs::write(
        &config_b,
        r#"version: "1.0"
name: config-b
machines:
  web:
    hostname: web-01
    addr: 10.0.1.10
    user: deploy
resources:
  nginx:
    type: package
    machine: web
    provider: apt
    packages: [nginx, certbot]
  app-config:
    type: file
    machine: web
    path: /etc/app.conf
    content: "port: 9090"
  redis:
    type: package
    machine: web
    provider: apt
    packages: [redis-server]
"#,
    )
    .ok();

    let fa = config_a.display().to_string();
    let fb = config_b.display().to_string();
    let mut failures = 0u32;

    eprintln!("Step 1: Compare two configs");
    let r = run_forjar(&["compare", &fa, &fb]);
    report("compare", &r, &mut failures);

    eprintln!("Step 2: Stack diff");
    let r = run_forjar(&["stack-diff", &fa, &fb]);
    report("stack-diff", &r, &mut failures);

    eprintln!("Step 3: Merge configs");
    let r = run_forjar(&["config-merge", &fa, &fb, "--allow-collisions"]);
    report("merge", &r, &mut failures);

    eprintln!("Step 4: Format config A");
    let r = run_forjar(&["fmt", "-f", &fa]);
    report("fmt", &r, &mut failures);

    eprintln!("Step 5: Lint config B");
    let r = run_forjar(&["lint", "-f", &fb]);
    report("lint", &r, &mut failures);

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
