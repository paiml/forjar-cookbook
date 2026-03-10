//! GitHub Release — install binaries from GitHub Releases.
//!
//! Demonstrates the `github_release` resource type: validate, plan, check,
//! and show absent state. Uses `forjar validate`, `forjar plan`, and
//! `forjar check` with JSON output.
//!
//! Usage: `cargo run --example github_release`

use std::path::Path;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-github-release");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    eprintln!("--- GitHub Release Resource ---\n");

    let mut failures = 0u32;
    failures += run_validate(&tmp);
    failures += run_plan(&tmp);
    failures += run_check_json(&tmp);
    failures += run_absent(&tmp);

    let _ = std::fs::remove_dir_all(&tmp);
    eprintln!("\n--- Result: {failures} failure(s) ---");
    if failures > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn write_present_config(dir: &Path) -> std::path::PathBuf {
    let path = dir.join("forjar.yaml");
    std::fs::write(
        &path,
        r#"version: "1.0"
name: github-release-demo
machines:
  local:
    hostname: localhost
    addr: 127.0.0.1
    user: demo
resources:
  install-bat:
    type: github_release
    machine: local
    repo: sharkdp/bat
    tag: v0.24.0
    asset_pattern: "*x86_64-unknown-linux-gnu*"
    binary: bat
    install_dir: /tmp/cookbook-gh-demo/bin
"#,
    )
    .ok();
    path
}

fn run_validate(dir: &Path) -> u32 {
    let config = write_present_config(dir);
    eprintln!("Step 1: Validate github_release config");
    let r = run_forjar(&["validate", "-f", &config.display().to_string()]);
    if r.success {
        eprintln!("  OK: {}", first_line(&r.output));
        0
    } else {
        eprintln!("  FAIL: {}", first_line(&r.output));
        1
    }
}

fn run_plan(dir: &Path) -> u32 {
    let config = write_present_config(dir);
    eprintln!("\nStep 2: Plan github_release installation");
    let r = run_forjar(&["plan", "-f", &config.display().to_string()]);
    if r.success {
        eprintln!("  OK: plan generated");
        for line in r.output.lines().take(5) {
            if !line.trim().is_empty() {
                eprintln!("  {line}");
            }
        }
        0
    } else {
        eprintln!("  FAIL: {}", first_line(&r.output));
        1
    }
}

fn run_check_json(dir: &Path) -> u32 {
    let config = write_present_config(dir);
    eprintln!("\nStep 3: Check status (JSON)");
    let r = run_forjar(&["check", "-f", &config.display().to_string(), "--json"]);
    if r.success {
        eprintln!("  OK: check completed");
        for line in r.output.lines().take(3) {
            eprintln!("  {line}");
        }
        0
    } else {
        eprintln!("  FAIL: {}", first_line(&r.output));
        1
    }
}

fn run_absent(dir: &Path) -> u32 {
    let path = dir.join("absent.yaml");
    std::fs::write(
        &path,
        r#"version: "1.0"
name: github-release-absent
machines:
  local:
    hostname: localhost
    addr: 127.0.0.1
    user: demo
resources:
  remove-bat:
    type: github_release
    machine: local
    repo: sharkdp/bat
    binary: bat
    state: absent
"#,
    )
    .ok();
    eprintln!("\nStep 4: Validate absent state");
    let r = run_forjar(&["validate", "-f", &path.display().to_string()]);
    if r.success {
        eprintln!("  OK: absent config valid");
        0
    } else {
        eprintln!("  FAIL: {}", first_line(&r.output));
        1
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

fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or(s).trim()
}
