//! Apply Tier 2 recipes inside a Docker container.
//!
//! Usage: `cargo run --example apply_container`
//! Iterates over `configs/` directory for container-testable variants.
//! Falls back to `recipes/` if no container configs exist.

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let root = match cookbook_examples::find_project_root() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    // Prefer container-specific configs, fall back to recipes
    let configs_dir = root.join("configs");
    let recipes_dir = root.join("recipes");
    let target_dir = if configs_dir.exists() {
        &configs_dir
    } else {
        &recipes_dir
    };

    let files = match cookbook_examples::collect_yaml_files(target_dir) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    if files.is_empty() {
        eprintln!("no config files found in {}", target_dir.display());
        return ExitCode::FAILURE;
    }

    let state_dir = std::env::temp_dir().join("cookbook-apply-container");
    let mut pass = 0u32;
    let mut fail = 0u32;
    let mut skip = 0u32;

    for file in &files {
        // Clean state between recipes
        let _ = std::fs::remove_dir_all(&state_dir);

        // First validate
        let validate = Command::new("forjar")
            .args(["validate", "-f", &file.display().to_string()])
            .output();
        match validate {
            Ok(output) if !output.status.success() => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!(
                    "  SKIP: {} — validation failed: {}",
                    file.display(),
                    stderr.trim()
                );
                skip += 1;
                continue;
            }
            Err(_) => {
                eprintln!("  SKIP: {} — forjar not found", file.display());
                skip += 1;
                continue;
            }
            _ => {}
        }

        // Apply with --yes
        let result = Command::new("forjar")
            .args([
                "apply",
                "-f",
                &file.display().to_string(),
                "--state-dir",
                &state_dir.display().to_string(),
                "--yes",
            ])
            .output();

        match result {
            Ok(output) if output.status.success() => {
                eprintln!("  OK: {}", file.display());
                pass += 1;
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("  FAIL: {} — {}", file.display(), stderr.trim());
                fail += 1;
            }
            Err(e) => {
                eprintln!("  ERROR: {} — {e}", file.display());
                fail += 1;
            }
        }
    }

    let _ = std::fs::remove_dir_all(&state_dir);

    eprintln!(
        "\nContainer apply: {} passed, {} failed, {} skipped (of {})",
        pass,
        fail,
        skip,
        files.len()
    );

    if fail > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
