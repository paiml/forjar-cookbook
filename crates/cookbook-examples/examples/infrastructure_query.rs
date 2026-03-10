//! Infrastructure query — search and filter resources.
//!
//! Demonstrates `forjar query` with type, machine, tag, and pattern
//! filters for discovering and inspecting managed infrastructure.
//!
//! Usage: `cargo run --example infrastructure_query`

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-query");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    eprintln!("--- Infrastructure Query ---\n");

    let config_path = tmp.join("forjar.yaml");
    std::fs::write(
        &config_path,
        r#"version: "1.0"
name: query-demo
machines:
  web:
    hostname: web-01
    addr: 10.0.1.10
    user: deploy
  db:
    hostname: db-01
    addr: 10.0.1.20
    user: deploy
resources:
  nginx-pkg:
    type: package
    machine: web
    provider: apt
    packages: [nginx]
    tags: [tier:frontend, env:prod]
  nginx-config:
    type: file
    machine: web
    path: /etc/nginx/nginx.conf
    content: "worker_processes auto;"
    depends_on: [nginx-pkg]
    sudo: true
    tags: [tier:frontend, env:prod]
  nginx-svc:
    type: service
    machine: web
    name: nginx
    enabled: true
    depends_on: [nginx-config]
    tags: [tier:frontend, env:prod]
  pg-pkg:
    type: package
    machine: db
    provider: apt
    packages: [postgresql-16]
    tags: [tier:backend, env:prod]
  pg-config:
    type: file
    machine: db
    path: /etc/postgresql/16/main/pg_hba.conf
    content: "host all all 10.0.1.0/24 scram-sha-256"
    depends_on: [pg-pkg]
    sudo: true
    tags: [tier:backend, env:prod]
"#,
    )
    .ok();

    let f = config_path.display().to_string();
    let mut failures = 0u32;

    eprintln!("Step 1: Query all resources");
    let r = run_forjar(&["query", "-f", &f]);
    report("all", &r, &mut failures);

    eprintln!("Step 2: Query by type (package)");
    let r = run_forjar(&["query", "-f", &f, "--type", "package"]);
    report("by-type", &r, &mut failures);

    eprintln!("Step 3: Query by machine (web)");
    let r = run_forjar(&["query", "-f", &f, "--machine", "web"]);
    report("by-machine", &r, &mut failures);

    eprintln!("Step 4: Query by tag");
    let r = run_forjar(&["query", "-f", &f, "--tag", "tier:backend"]);
    report("by-tag", &r, &mut failures);

    eprintln!("Step 5: Query by pattern");
    let r = run_forjar(&["query", "-f", &f, "--pattern", "nginx"]);
    report("by-pattern", &r, &mut failures);

    eprintln!("Step 6: Query with details");
    let r = run_forjar(&["query", "-f", &f, "--type", "file", "--details"]);
    report("details", &r, &mut failures);

    eprintln!("Step 7: Extract by tag");
    let r = run_forjar(&["extract", "-f", &f, "--tags", "tier:backend"]);
    report("extract", &r, &mut failures);

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
