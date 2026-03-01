//! Validate all recipe YAML files.
//!
//! Runs `forjar validate` on every `.yaml` file in `recipes/`.
//! Exit code 0 if all pass, 1 if any fail.

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

    let mut failures = 0u32;
    for file in &files {
        let result = Command::new("forjar")
            .args(["validate", "-f", &file.display().to_string()])
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

    eprintln!(
        "\n{}/{} recipes validated ({} failures)",
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
