//! Multi-machine deployment — coordinated cross-machine infrastructure.
//!
//! Demonstrates deploying resources across multiple machines with dependency
//! ordering, inventory checking, and cross-machine analysis.
//!
//! Usage: `cargo run --example multi_machine`

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-multi-machine");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    eprintln!("--- Multi-Machine Deployment ---\n");

    let config_path = tmp.join("forjar.yaml");
    std::fs::write(
        &config_path,
        r#"version: "1.0"
name: multi-machine-demo
machines:
  web:
    hostname: web-01
    addr: 10.0.1.10
    user: deploy
  db:
    hostname: db-01
    addr: 10.0.1.20
    user: deploy
  cache:
    hostname: cache-01
    addr: 10.0.1.30
    user: deploy
resources:
  db-pkg:
    type: package
    machine: db
    provider: apt
    packages: [postgresql-16]
  db-config:
    type: file
    machine: db
    path: /etc/postgresql/16/main/pg_hba.conf
    content: "host all all 10.0.1.0/24 scram-sha-256"
    depends_on: [db-pkg]
    sudo: true
  cache-pkg:
    type: package
    machine: cache
    provider: apt
    packages: [redis-server]
  web-pkg:
    type: package
    machine: web
    provider: apt
    packages: [nginx]
  web-config:
    type: file
    machine: web
    path: /etc/nginx/conf.d/app.conf
    content: "upstream db { server 10.0.1.20:5432; }"
    depends_on: [web-pkg]
    sudo: true
"#,
    )
    .ok();

    let f = config_path.display().to_string();
    let mut failures = 0u32;

    // Step 1: Validate
    eprintln!("Step 1: Validate multi-machine config");
    let r = run_forjar(&["validate", "-f", &f]);
    report("validate", &r, &mut failures);

    // Step 2: Inventory
    eprintln!("Step 2: Machine inventory");
    let r = run_forjar(&["inventory", "-f", &f]);
    report("inventory", &r, &mut failures);

    // Step 3: Dependency graph
    eprintln!("Step 3: Resource dependency graph");
    let r = run_forjar(&["graph", "-f", &f]);
    report("graph", &r, &mut failures);

    // Step 4: Cross-machine dependency analysis
    eprintln!("Step 4: Cross-machine dependencies");
    let r = run_forjar(&["cross-deps", "-f", &f]);
    report("cross-deps", &r, &mut failures);

    // Step 5: Plan
    eprintln!("Step 5: Execution plan");
    let r = run_forjar(&["plan-compact", "-f", &f]);
    report("plan-compact", &r, &mut failures);

    // Step 6: Complexity analysis
    eprintln!("Step 6: Configuration complexity");
    let r = run_forjar(&["complexity", "-f", &f]);
    report("complexity", &r, &mut failures);

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
