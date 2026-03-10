//! CIS compliance — validate infrastructure against CIS benchmarks.
//!
//! Demonstrates `forjar compliance`, `forjar policy`, `forjar policy-coverage`,
//! and `forjar policy-install` for compliance-as-code workflows.
//!
//! Usage: `cargo run --example cis_compliance`

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-cis");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    eprintln!("--- CIS Compliance ---\n");

    let config_path = tmp.join("forjar.yaml");
    std::fs::write(
        &config_path,
        r#"version: "1.0"
name: cis-demo
machines:
  server:
    hostname: prod-01
    addr: 10.0.1.5
    user: root
resources:
  ssh-config:
    type: file
    machine: server
    path: /etc/ssh/sshd_config
    content: |
      PermitRootLogin no
      PasswordAuthentication no
      MaxAuthTries 3
      Protocol 2
    owner: root
    group: root
    mode: "0600"
    sudo: true
  firewall:
    type: network
    machine: server
    protocol: tcp
    port: 22
    action: allow
    from: "10.0.0.0/8"
  audit-pkg:
    type: package
    machine: server
    provider: apt
    packages: [auditd, aide]
    sudo: true
  sysctl:
    type: file
    machine: server
    path: /etc/sysctl.d/99-hardening.conf
    content: |
      net.ipv4.ip_forward=0
      net.ipv4.conf.all.send_redirects=0
      kernel.randomize_va_space=2
    sudo: true
policies:
  - type: require
    resource_type: file
    field: mode
    message: "All files must have explicit permissions"
  - type: deny
    resource_type: file
    condition_field: mode
    condition_value: "0777"
    message: "World-writable files are prohibited"
"#,
    )
    .ok();

    let f = config_path.display().to_string();
    let mut failures = 0u32;

    eprintln!("Step 1: Validate");
    let r = run_forjar(&["validate", "-f", &f]);
    report("validate", &r, &mut failures);

    eprintln!("Step 2: Policy evaluation");
    let r = run_forjar(&["policy", "-f", &f]);
    report("policy", &r, &mut failures);

    eprintln!("Step 3: Policy coverage");
    let r = run_forjar(&["policy-coverage", "-f", &f]);
    report("coverage", &r, &mut failures);

    eprintln!("Step 4: Compliance check");
    let r = run_forjar(&["compliance", "-f", &f]);
    report("compliance", &r, &mut failures);

    eprintln!("Step 5: Security scan");
    let r = run_forjar(&["security-scan", "-f", &f]);
    report("security", &r, &mut failures);

    eprintln!("Step 6: Privilege analysis");
    let r = run_forjar(&["privilege-analysis", "-f", &f]);
    report("privilege", &r, &mut failures);

    eprintln!("Step 7: SBOM");
    let r = run_forjar(&["sbom", "-f", &f]);
    report("sbom", &r, &mut failures);

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
