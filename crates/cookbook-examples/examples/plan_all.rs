//! Plan all recipe YAML files (dry-run).
//!
//! Runs `forjar plan` on every `.yaml` file in `recipes/`.
//! Uses a temporary state directory that is cleaned up after each recipe.

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let root = match cookbook_examples::find_project_root() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    let recipes_dir = root.join("recipes");
    let files = match cookbook_examples::collect_yaml_files(&recipes_dir) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    if files.is_empty() {
        eprintln!("no recipe files found in {}", recipes_dir.display());
        return ExitCode::FAILURE;
    }

    let state_dir = std::env::temp_dir().join("cookbook-plan-all");
    let mut failures = 0u32;

    for file in &files {
        // Clean state between recipes
        let _ = std::fs::remove_dir_all(&state_dir);

        let result = Command::new("forjar")
            .args([
                "plan",
                "-f",
                &file.display().to_string(),
                "--state-dir",
                &state_dir.display().to_string(),
            ])
            .output();

        match result {
            Ok(output) if output.status.success() => {
                eprintln!("  OK: {}", file.display());
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("  FAIL: {} — {}", file.display(), stderr.trim());
                failures += 1;
            }
            Err(e) => {
                eprintln!("  ERROR: {} — {e}", file.display());
                failures += 1;
            }
        }
    }

    let _ = std::fs::remove_dir_all(&state_dir);

    eprintln!(
        "\n{}/{} recipes planned ({} failures)",
        files.len() - failures as usize,
        files.len(),
        failures
    );

    if failures > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
