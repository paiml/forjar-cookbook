//! Cross-compilation — build on one machine, deploy to another.
//!
//! Demonstrates the `build_machine` field, architecture filtering with
//! `arch:`, and the `forjar build` workflow for cross-compiled binaries.
//!
//! Usage: `cargo run --example build_crosscompile`

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-crosscompile");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    eprintln!("--- Cross-Compilation ---\n");

    let config_path = tmp.join("forjar.yaml");
    std::fs::write(
        &config_path,
        r#"version: "1.0"
name: crosscompile-demo
machines:
  build-x86:
    hostname: build-server
    addr: 10.0.1.50
    user: ci
    arch: x86_64
  jetson:
    hostname: jetson-01
    addr: 10.0.1.100
    user: deploy
    arch: aarch64
resources:
  build-tools:
    type: package
    machine: build-x86
    provider: apt
    packages: [gcc-aarch64-linux-gnu, binutils-aarch64-linux-gnu]
    arch: [x86_64]
  compile-app:
    type: task
    machine: build-x86
    task_mode: batch
    command: "cargo build --release --target aarch64-unknown-linux-gnu"
    working_dir: /home/ci/app
    depends_on: [build-tools]
  deploy-binary:
    type: github_release
    machine: jetson
    repo: myorg/myapp
    tag: nightly
    asset_pattern: "*aarch64-unknown-linux-gnu*"
    binary: myapp
    install_dir: /usr/local/bin
    arch: [aarch64]
"#,
    )
    .ok();

    let f = config_path.display().to_string();
    let mut failures = 0u32;

    eprintln!("Step 1: Validate");
    let r = run_forjar(&["validate", "-f", &f]);
    report("validate", &r, &mut failures);

    eprintln!("Step 2: Plan");
    let r = run_forjar(&["plan-compact", "-f", &f]);
    report("plan", &r, &mut failures);

    eprintln!("Step 3: Cross-machine dependencies");
    let r = run_forjar(&["cross-deps", "-f", &f]);
    report("cross-deps", &r, &mut failures);

    eprintln!("Step 4: Inventory");
    let r = run_forjar(&["inventory", "-f", &f]);
    report("inventory", &r, &mut failures);

    eprintln!("Step 5: Cost estimate");
    let r = run_forjar(&["cost-estimate", "-f", &f]);
    report("cost", &r, &mut failures);

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
