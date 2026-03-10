//! User management — create users, groups, SSH keys.
//!
//! Demonstrates the `user` resource type: system users, login shells,
//! home directories, group membership, and SSH authorized keys.
//!
//! Usage: `cargo run --example user_management`

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-user-mgmt");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    eprintln!("--- User Management ---\n");

    let config_path = tmp.join("forjar.yaml");
    std::fs::write(
        &config_path,
        r#"version: "1.0"
name: user-management-demo
machines:
  server:
    hostname: srv-01
    addr: 10.0.1.5
    user: root
resources:
  ops-group:
    type: user
    machine: server
    name: ops
    groups: [sudo]
    sudo: true
  deploy-user:
    type: user
    machine: server
    name: deploy
    shell: /bin/bash
    home: /home/deploy
    groups: [ops, www-data]
    ssh_authorized_keys:
      - "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIExample deploy@laptop"
    depends_on: [ops-group]
    sudo: true
  service-account:
    type: user
    machine: server
    name: apprunner
    shell: /usr/sbin/nologin
    home: /opt/apprunner
    system_user: true
    sudo: true
  deploy-ssh-dir:
    type: file
    machine: server
    path: /home/deploy/.ssh
    owner: deploy
    group: deploy
    mode: "0700"
    content: ""
    depends_on: [deploy-user]
    sudo: true
"#,
    )
    .ok();

    let f = config_path.display().to_string();
    let mut failures = 0u32;

    eprintln!("Step 1: Validate");
    let r = run_forjar(&["validate", "-f", &f]);
    report("validate", &r, &mut failures);

    eprintln!("Step 2: Plan");
    let r = run_forjar(&["plan", "-f", &f]);
    report("plan", &r, &mut failures);

    eprintln!("Step 3: Lint (validate scripts)");
    let r = run_forjar(&["lint", "-f", &f]);
    report("lint", &r, &mut failures);

    eprintln!("Step 4: Privilege analysis");
    let r = run_forjar(&["privilege-analysis", "-f", &f]);
    report("privilege", &r, &mut failures);

    eprintln!("Step 5: Security scan");
    let r = run_forjar(&["security-scan", "-f", &f]);
    report("security", &r, &mut failures);

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
