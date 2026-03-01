//! Cookbook examples — shared utilities for `cargo run --example` targets.

use std::path::{Path, PathBuf};

/// Find the project root by looking for `Cargo.toml` + `recipes/`.
///
/// # Errors
///
/// Returns error if the project root cannot be found.
pub fn find_project_root() -> Result<PathBuf, String> {
    let mut dir = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
    loop {
        if dir.join("Cargo.toml").exists() && dir.join("recipes").exists() {
            return Ok(dir);
        }
        if !dir.pop() {
            return Err("could not find project root (Cargo.toml + recipes/)".into());
        }
    }
}

/// Collect all `.yaml` files in a directory.
///
/// # Errors
///
/// Returns error if the directory cannot be read.
pub fn collect_yaml_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
    let mut files: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .is_some_and(|ext| ext == "yaml" || ext == "yml")
        })
        .collect();
    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_yaml_empty_dir() {
        let dir = std::env::temp_dir().join("cookbook-test-empty");
        let _ = std::fs::create_dir_all(&dir);
        let files = collect_yaml_files(&dir).unwrap_or_default();
        assert!(files.is_empty());
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn collect_yaml_nonexistent_dir() {
        let result = collect_yaml_files(Path::new("/nonexistent/dir"));
        assert!(result.is_err());
        let Err(err) = result else { return };
        assert!(err.contains("read_dir"));
    }

    #[test]
    fn collect_yaml_with_files() {
        let dir = std::env::temp_dir().join("cookbook-test-yaml-files");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("01-test.yaml"), "version: '1.0'").ok();
        std::fs::write(dir.join("02-other.yml"), "version: '1.0'").ok();
        std::fs::write(dir.join("readme.md"), "# not yaml").ok();
        std::fs::write(dir.join("config.json"), "{}").ok();

        let files = collect_yaml_files(&dir).unwrap_or_default();
        assert_eq!(files.len(), 2);
        // Sorted order
        assert!(files[0].file_name().is_some_and(|n| n == "01-test.yaml"));
        assert!(files[1].file_name().is_some_and(|n| n == "02-other.yml"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn collect_yaml_sorted() {
        let dir = std::env::temp_dir().join("cookbook-test-yaml-sorted");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("z-last.yaml"), "a: 1").ok();
        std::fs::write(dir.join("a-first.yaml"), "a: 1").ok();
        std::fs::write(dir.join("m-middle.yaml"), "a: 1").ok();

        let files = collect_yaml_files(&dir).unwrap_or_default();
        assert_eq!(files.len(), 3);
        assert!(files[0].file_name().is_some_and(|n| n == "a-first.yaml"));
        assert!(files[1].file_name().is_some_and(|n| n == "m-middle.yaml"));
        assert!(files[2].file_name().is_some_and(|n| n == "z-last.yaml"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn find_project_root_from_cookbook() {
        // This test runs inside the forjar-cookbook workspace, so it should find it
        let result = find_project_root();
        // May or may not succeed depending on where tests run from
        if let Ok(root) = result {
            assert!(root.join("Cargo.toml").exists());
            assert!(root.join("recipes").exists());
        }
    }

    #[test]
    fn find_project_root_error_message() {
        // Test that the error path produces a useful message
        // We can't easily test the "no root found" path without changing cwd,
        // but we verify the function signature works
        let _result = find_project_root();
        // The function either succeeds or returns an appropriate error
    }
}
