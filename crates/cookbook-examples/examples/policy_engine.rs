//! Policy-as-code engine — evaluate and install compliance packs.
//!
//! Demonstrates the policy-as-code pipeline: `forjar policy` evaluation,
//! `forjar policy-coverage` analysis, and `forjar policy-install` for
//! compliance packs. Works with any config containing policy rules.
//!
//! Usage: `cargo run --example policy_engine`

use std::path::Path;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-policy-engine");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    eprintln!("--- Policy-as-Code Engine ---\n");

    let config_path = tmp.join("forjar.yaml");
    write_demo_config(&config_path);

    let mut failures = 0u32;
    failures += run_evaluate_policy(&config_path);
    failures += run_coverage(&config_path);
    failures += run_json_output(&config_path);
    failures += run_install_pack(&tmp);

    let _ = std::fs::remove_dir_all(&tmp);
    eprintln!("\n--- Result: {failures} failure(s) ---");
    if failures > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn write_demo_config(config_path: &Path) {
    std::fs::write(
        config_path,
        r#"version: "1.0"
name: policy-demo
machines:
  web-01:
    hostname: web-01
    addr: 10.0.0.1
    user: deploy
resources:
  nginx-config:
    type: file
    machine: web-01
    path: /etc/nginx/nginx.conf
    owner: root
    mode: "0644"
    content: "server { listen 80; }"
    sudo: true
  ssh-config:
    type: file
    machine: web-01
    path: /etc/ssh/sshd_config
    owner: root
    mode: "0600"
    content: "PermitRootLogin no"
    sudo: true
policies:
  - name: require-file-owner
    type: require
    resource_type: file
    field: owner
    message: "All files must have an explicit owner"
  - name: require-secure-mode
    type: require
    resource_type: file
    field: mode
    message: "All files must have explicit permissions"
"#,
    )
    .ok();
}

fn run_evaluate_policy(config_path: &Path) -> u32 {
    eprintln!("Step 1: Evaluate policies");
    let policy = run_forjar(&["policy", "-f", &config_path.display().to_string()]);
    if policy.success {
        eprintln!("  OK: policy evaluation passed ({}ms)", policy.duration_ms);
    } else {
        eprintln!("  WARN: {}", first_line(&policy.output));
    }
    0
}

fn run_coverage(config_path: &Path) -> u32 {
    eprintln!("\nStep 2: Policy coverage analysis");
    let coverage = run_forjar(&["policy-coverage", "-f", &config_path.display().to_string()]);
    if coverage.success {
        eprintln!("  OK: coverage complete ({}ms)", coverage.duration_ms);
        for line in coverage.output.lines().take(5) {
            if !line.trim().is_empty() {
                eprintln!("  {line}");
            }
        }
        0
    } else {
        eprintln!("  FAIL: {}", first_line(&coverage.output));
        1
    }
}

fn run_json_output(config_path: &Path) -> u32 {
    eprintln!("\nStep 3: JSON policy output (for CI)");
    let json_policy = run_forjar(&["policy", "-f", &config_path.display().to_string(), "--json"]);
    if json_policy.success {
        eprintln!("  OK: JSON output ({}ms)", json_policy.duration_ms);
        for line in json_policy.output.lines().take(3) {
            eprintln!("  {line}");
        }
        0
    } else {
        eprintln!("  FAIL: {}", first_line(&json_policy.output));
        1
    }
}

fn run_install_pack(tmp: &Path) -> u32 {
    let pack_dir = tmp.join("packs");
    std::fs::create_dir_all(&pack_dir).ok();
    eprintln!("\nStep 4: Install compliance pack");
    let install = run_forjar(&[
        "policy-install",
        "cis-ubuntu-22",
        "--output-dir",
        &pack_dir.display().to_string(),
    ]);
    if install.success {
        eprintln!("  OK: pack installed ({}ms)", install.duration_ms);
        0
    } else {
        eprintln!("  FAIL: {}", first_line(&install.output));
        1
    }
}

struct StepResult {
    success: bool,
    output: String,
    duration_ms: u64,
}

fn run_forjar(args: &[&str]) -> StepResult {
    let start = std::time::Instant::now();
    let result = Command::new("forjar").args(args).output();
    let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            StepResult {
                success: output.status.success(),
                output: format!("{stdout}{stderr}"),
                duration_ms,
            }
        }
        Err(e) => StepResult {
            success: false,
            output: format!("failed to execute forjar: {e}"),
            duration_ms,
        },
    }
}

fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or(s).trim()
}
