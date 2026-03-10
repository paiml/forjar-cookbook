//! Full-stack application deployment — all resource types together.
//!
//! Demonstrates a real-world deployment: packages, users, files, services,
//! cron jobs, and firewall rules with proper dependency ordering.
//!
//! Usage: `cargo run --example full_stack_deploy`

use std::path::Path;
use std::process::{Command, ExitCode};

fn write_config(path: &Path) {
    std::fs::write(
        path,
        r##"version: "1.0"
name: full-stack-demo
machines:
  web:
    hostname: web-prod
    addr: 10.0.1.10
    user: deploy
resources:
  base-pkgs:
    type: package
    machine: web
    provider: apt
    packages: [nginx, certbot, fail2ban, ufw]
  app-user:
    type: user
    machine: web
    name: webapp
    shell: /bin/bash
    home: /opt/webapp
    groups: [www-data]
    system_user: true
    depends_on: [base-pkgs]
  app-dir:
    type: file
    machine: web
    path: /opt/webapp/current
    content: "# placeholder for app deployment"
    owner: webapp
    group: www-data
    mode: "0755"
    depends_on: [app-user]
    sudo: true
  nginx-config:
    type: file
    machine: web
    path: /etc/nginx/sites-available/webapp
    content: |
      server {
        listen 80;
        server_name webapp.example.com;
        root /opt/webapp/current/public;
      }
    depends_on: [base-pkgs]
    sudo: true
  nginx-svc:
    type: service
    machine: web
    name: nginx
    enabled: true
    restart_on: [nginx-config]
    depends_on: [nginx-config]
  certbot-cron:
    type: cron
    machine: web
    schedule: "0 3 * * 1"
    command: "certbot renew --quiet"
    depends_on: [base-pkgs]
    sudo: true
  firewall:
    type: network
    machine: web
    protocol: tcp
    port: 443
    action: allow
    from: "0.0.0.0/0"
    depends_on: [nginx-svc]
"##,
    )
    .ok();
}

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-full-stack");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    eprintln!("--- Full-Stack Deployment ---\n");

    let config_path = tmp.join("forjar.yaml");
    write_config(&config_path);

    let f = config_path.display().to_string();
    let mut failures = 0u32;

    eprintln!("Step 1: Validate");
    let r = run_forjar(&["validate", "-f", &f]);
    report("validate", &r, &mut failures);

    eprintln!("Step 2: Dependency graph");
    let r = run_forjar(&["graph", "-f", &f]);
    report("graph", &r, &mut failures);

    eprintln!("Step 3: Plan");
    let r = run_forjar(&["plan-compact", "-f", &f]);
    report("plan", &r, &mut failures);

    eprintln!("Step 4: Privilege analysis");
    let r = run_forjar(&["privilege-analysis", "-f", &f]);
    report("privilege", &r, &mut failures);

    eprintln!("Step 5: Cost estimate");
    let r = run_forjar(&["cost-estimate", "-f", &f]);
    report("cost", &r, &mut failures);

    eprintln!("Step 6: Compliance check");
    let r = run_forjar(&["compliance", "-f", &f]);
    report("compliance", &r, &mut failures);

    eprintln!("Step 7: Score");
    let r = run_forjar(&["score", "-f", &f]);
    report("score", &r, &mut failures);

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
