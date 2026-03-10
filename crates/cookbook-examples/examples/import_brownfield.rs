//! Brownfield import — adopt existing infrastructure into forjar.
//!
//! Demonstrates `forjar import` and `forjar import-brownfield` for
//! discovering and importing existing system state into managed configs.
//!
//! Usage: `cargo run --example import_brownfield`

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-import");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    eprintln!("--- Brownfield Import ---\n");

    let mut failures = 0u32;

    eprintln!("Step 1: Import from localhost (packages)");
    let r = run_forjar(&[
        "import",
        "--addr",
        "localhost",
        "--name",
        "local",
        "--scan",
        "packages",
        "-o",
        &tmp.join("imported.yaml").display().to_string(),
    ]);
    report("import", &r, &mut failures);

    eprintln!("Step 2: Validate imported config");
    let imported = tmp.join("imported.yaml");
    if imported.exists() {
        let r = run_forjar(&["validate", "-f", &imported.display().to_string()]);
        report("validate", &r, &mut failures);
    } else {
        eprintln!("  SKIP: no imported config");
    }

    eprintln!("Step 3: Import-brownfield help");
    let r = run_forjar(&["import-brownfield", "--help"]);
    report("brownfield-help", &r, &mut failures);

    eprintln!("Step 4: Doctor (check system readiness)");
    let r = run_forjar(&["doctor"]);
    report("doctor", &r, &mut failures);

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
