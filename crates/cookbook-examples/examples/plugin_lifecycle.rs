//! Plugin install/verify/remove lifecycle.
//!
//! Demonstrates the full plugin lifecycle: scaffold, verify, list, and remove.
//! Runs `forjar plugin init`, `forjar plugin list`, `forjar plugin verify`,
//! and `forjar plugin remove` against a temporary plugin directory.
//!
//! Usage: `cargo run --example plugin_lifecycle`

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let tmp = std::env::temp_dir().join("cookbook-plugin-lifecycle");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).ok();

    let plugin_dir = tmp.join("plugins");
    std::fs::create_dir_all(&plugin_dir).ok();

    let mut failures = 0u32;

    // Step 1: Scaffold a plugin
    eprintln!("--- Plugin Lifecycle ---\n");
    eprintln!("Step 1: Scaffold plugin 'example-monitor'");
    let scaffold = run_forjar(&[
        "plugin",
        "init",
        "example-monitor",
        "--output",
        &plugin_dir.display().to_string(),
    ]);
    report_step("scaffold", &scaffold, &mut failures);

    // Step 2: List plugins
    eprintln!("Step 2: List installed plugins");
    let list = run_forjar(&[
        "plugin",
        "list",
        "--plugin-dir",
        &plugin_dir.display().to_string(),
    ]);
    report_step("list", &list, &mut failures);
    if list.success {
        eprintln!("  output: {}", list.output.trim());
    }

    // Step 3: Verify plugin (if scaffold created a manifest)
    let manifest_path = plugin_dir.join("example-monitor").join("plugin.yaml");
    if manifest_path.exists() {
        eprintln!("Step 3: Verify plugin manifest");
        let verify = run_forjar(&["plugin", "verify", &manifest_path.display().to_string()]);
        report_step("verify", &verify, &mut failures);
    } else {
        eprintln!("Step 3: SKIP — no manifest at {}", manifest_path.display());
    }

    // Step 4: Remove plugin
    eprintln!("Step 4: Remove plugin 'example-monitor'");
    let remove = run_forjar(&[
        "plugin",
        "remove",
        "example-monitor",
        "--plugin-dir",
        &plugin_dir.display().to_string(),
        "--yes",
    ]);
    report_step("remove", &remove, &mut failures);

    // Step 5: Confirm empty list
    eprintln!("Step 5: Verify plugin list is empty after removal");
    let list_after = run_forjar(&[
        "plugin",
        "list",
        "--plugin-dir",
        &plugin_dir.display().to_string(),
    ]);
    report_step("list-after-remove", &list_after, &mut failures);

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

fn report_step(name: &str, result: &StepResult, failures: &mut u32) {
    if result.success {
        eprintln!("  {name}: OK ({}ms)", result.duration_ms);
    } else {
        eprintln!(
            "  {name}: FAIL ({}ms) — {}",
            result.duration_ms,
            first_line(&result.output)
        );
        *failures += 1;
    }
}

fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or(s).trim()
}
