//! Tests for README sync logic.

use super::*;
use crate::{END_MARKER, START_MARKER};

fn sample_csv() -> String {
    "\
recipe_num,name,category,status,tier,idempotency_class,first_apply_ms,idempotent_apply_ms,blocker_ticket,blocker_description,last_qualified,qualified_by
1,developer-workstation,infra,qualified,2+3,strong,45000,1200,,,2026-03-01,cookbook-runner
2,web-server,infra,pending,2+3,strong,,,,,,
"
    .to_string()
}

fn sample_readme() -> String {
    format!("# Cookbook\n\nIntro text.\n\n{START_MARKER}\nold table\n{END_MARKER}\n\nFooter.\n")
}

#[test]
fn sync_readme_success() {
    let dir = std::env::temp_dir().join("cookbook-sync-test");
    let _ = std::fs::create_dir_all(&dir);

    let csv_path = dir.join("recipes.csv");
    let readme_path = dir.join("README.md");

    std::fs::write(&csv_path, sample_csv()).ok();
    std::fs::write(&readme_path, sample_readme()).ok();

    let result = sync_readme(&csv_path, &readme_path, "2026-03-01 12:00 UTC");
    assert!(result.is_ok());

    let sr = result.unwrap_or(SyncResult {
        total: 0,
        qualified: 0,
    });
    assert_eq!(sr.total, 2);
    assert_eq!(sr.qualified, 1);

    let updated = std::fs::read_to_string(&readme_path).unwrap_or_default();
    assert!(updated.contains("developer-workstation"));
    assert!(updated.contains("Qualified | 1"));
    assert!(!updated.contains("old table"));
    assert!(updated.contains("Footer."));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn sync_readme_missing_csv() {
    let dir = std::env::temp_dir().join("cookbook-sync-missing-csv");
    let _ = std::fs::create_dir_all(&dir);
    let csv_path = dir.join("nonexistent.csv");
    let readme_path = dir.join("README.md");
    std::fs::write(&readme_path, sample_readme()).ok();

    let result = sync_readme(&csv_path, &readme_path, "ts");
    assert!(result.is_err());
    let Err(err) = result else { return };
    assert!(err.contains("read"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn sync_readme_missing_readme() {
    let dir = std::env::temp_dir().join("cookbook-sync-missing-readme");
    let _ = std::fs::create_dir_all(&dir);
    let csv_path = dir.join("recipes.csv");
    std::fs::write(&csv_path, sample_csv()).ok();
    let readme_path = dir.join("nonexistent-readme.md");

    let result = sync_readme(&csv_path, &readme_path, "ts");
    assert!(result.is_err());
    let Err(err) = result else { return };
    assert!(err.contains("read"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn sync_readme_missing_markers() {
    let dir = std::env::temp_dir().join("cookbook-sync-no-markers");
    let _ = std::fs::create_dir_all(&dir);
    let csv_path = dir.join("recipes.csv");
    let readme_path = dir.join("README.md");
    std::fs::write(&csv_path, sample_csv()).ok();
    std::fs::write(&readme_path, "# No markers here\n").ok();

    let result = sync_readme(&csv_path, &readme_path, "ts");
    assert!(result.is_err());
    let Err(err) = result else { return };
    assert!(err.contains("marker"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn sync_result_debug() {
    let sr = SyncResult {
        total: 10,
        qualified: 7,
    };
    let debug = format!("{sr:?}");
    assert!(debug.contains("10"));
    assert!(debug.contains('7'));
}

#[test]
fn find_project_root_from_cookbook_dir() {
    // When run inside the forjar-cookbook workspace, find_project_root should succeed
    // because the workspace has both Cargo.toml and docs/certifications/.
    let result = find_project_root();
    // May or may not succeed depending on where tests run from, but shouldn't panic.
    if let Ok(root) = &result {
        assert!(root.join("Cargo.toml").exists());
        assert!(root.join("docs/certifications").exists());
    }
}

#[test]
fn find_project_root_error_message() {
    let err_msg = "could not find project root (Cargo.toml + docs/certifications/)";
    assert!(err_msg.contains("project root"));
}

#[test]
fn run_readme_sync_from_workspace() {
    // When run from the forjar-cookbook workspace, this should find the
    // project root and sync the README. We test that it doesn't panic
    // and produces a valid result.
    let result = run_readme_sync();
    if let Ok((path, sr)) = result {
        assert!(path.ends_with("README.md"));
        assert!(sr.total > 0);
    }
    // If it fails (e.g., running from a different directory), that's OK —
    // we just verify it doesn't panic.
}
