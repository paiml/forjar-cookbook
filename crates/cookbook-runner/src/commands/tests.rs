#![allow(clippy::expect_used)]

use super::*;
use crate::RunOutcome;
use cookbook_qualify::Grade;

fn sample_yaml() -> &'static str {
    "\
# Recipe #1: Test Recipe
# Tier: 2+3
# Idempotency: Strong
version: '1.0'
name: test-recipe
description: A test recipe
machines:
  target:
    hostname: target
    addr: localhost
resources:
  my-file:
    type: file
    machine: target
    dest: /tmp/test
    mode: '0644'
    owner: root
"
}

fn ok_outcome(ms: u64) -> RunOutcome {
    RunOutcome {
        exit_code: 0,
        output: String::new(),
        duration_ms: ms,
    }
}

fn fail_outcome() -> RunOutcome {
    RunOutcome {
        exit_code: 1,
        output: "error".to_string(),
        duration_ms: 50,
    }
}

#[test]
fn score_recipe_file_pending() {
    let dir = tempfile::tempdir().expect("tempdir");
    let yaml_path = dir.path().join("recipe.yaml");
    std::fs::write(&yaml_path, sample_yaml()).expect("write yaml");

    let (report, score) =
        score_recipe_file(&yaml_path, "pending", "strong", 0).expect("score should succeed");
    assert_eq!(score.grade, Grade::F);
    assert_eq!(score.composite, 0);
    assert!(report.contains("grade"));
}

#[test]
fn score_recipe_file_qualified() {
    let dir = tempfile::tempdir().expect("tempdir");
    let yaml_path = dir.path().join("recipe.yaml");
    std::fs::write(&yaml_path, sample_yaml()).expect("write yaml");

    let (report, score) =
        score_recipe_file(&yaml_path, "qualified", "strong", 0).expect("score should succeed");
    assert!(score.dimensions.saf > 0);
    assert!(report.contains("grade"));
}

#[test]
fn score_recipe_file_missing_file() {
    let result = score_recipe_file(
        Path::new("/nonexistent/recipe.yaml"),
        "pending",
        "strong",
        0,
    );
    let err = result.expect_err("should fail for missing file");
    assert!(err.contains("cannot read"));
}

#[test]
fn score_recipe_file_invalid_yaml() {
    let dir = tempfile::tempdir().expect("tempdir");
    let yaml_path = dir.path().join("recipe.yaml");
    std::fs::write(&yaml_path, "{{invalid yaml!!").expect("write");

    let result = score_recipe_file(&yaml_path, "pending", "strong", 0);
    assert!(result.is_err());
}

#[test]
fn score_recipe_file_invalid_status() {
    let dir = tempfile::tempdir().expect("tempdir");
    let yaml_path = dir.path().join("recipe.yaml");
    std::fs::write(&yaml_path, sample_yaml()).expect("write");

    let result = score_recipe_file(&yaml_path, "unknown_status", "strong", 0);
    assert!(result.is_err());
}

#[test]
fn score_recipe_file_invalid_idempotency() {
    let dir = tempfile::tempdir().expect("tempdir");
    let yaml_path = dir.path().join("recipe.yaml");
    std::fs::write(&yaml_path, sample_yaml()).expect("write");

    let result = score_recipe_file(&yaml_path, "pending", "bogus", 0);
    assert!(result.is_err());
}

#[test]
fn score_recipe_file_with_budget() {
    let dir = tempfile::tempdir().expect("tempdir");
    let yaml_path = dir.path().join("recipe.yaml");
    std::fs::write(&yaml_path, sample_yaml()).expect("write");

    let (_, score) =
        score_recipe_file(&yaml_path, "qualified", "strong", 60000).expect("score should succeed");
    assert_eq!(score.dimensions.prf, 0);
}

#[test]
fn grade_passes_threshold_abc() {
    assert!(grade_passes_threshold(&Grade::A));
    assert!(grade_passes_threshold(&Grade::B));
    assert!(grade_passes_threshold(&Grade::C));
}

#[test]
fn grade_passes_threshold_df() {
    assert!(!grade_passes_threshold(&Grade::D));
    assert!(!grade_passes_threshold(&Grade::F));
}

#[test]
fn score_after_qualify_all_pass() {
    let dir = tempfile::tempdir().expect("tempdir");
    let yaml_path = dir.path().join("recipe.yaml");
    std::fs::write(&yaml_path, sample_yaml()).expect("write");

    let result = QualifyResult {
        validate: ok_outcome(100),
        plan: Some(ok_outcome(200)),
        first_apply: Some(ok_outcome(5000)),
        idempotent_apply: Some(ok_outcome(1000)),
        idempotent: true,
    };

    let (report, score) =
        score_after_qualify(&yaml_path, &result).expect("scoring after qualify should succeed");
    assert!(report.contains("grade"));
    assert!(score.composite > 0);
}

#[test]
fn score_after_qualify_missing_file() {
    let result = QualifyResult {
        validate: ok_outcome(100),
        plan: None,
        first_apply: None,
        idempotent_apply: None,
        idempotent: false,
    };
    let out = score_after_qualify(Path::new("/nonexistent/recipe.yaml"), &result);
    assert!(out.is_none());
}

#[test]
fn score_after_qualify_failed_validate() {
    let dir = tempfile::tempdir().expect("tempdir");
    let yaml_path = dir.path().join("recipe.yaml");
    std::fs::write(&yaml_path, sample_yaml()).expect("write");

    let result = QualifyResult {
        validate: fail_outcome(),
        plan: None,
        first_apply: None,
        idempotent_apply: None,
        idempotent: false,
    };

    let (_, score) = score_after_qualify(&yaml_path, &result)
        .expect("scoring should succeed even on failed qualify");
    assert_eq!(score.grade, Grade::F);
}

#[test]
fn score_after_qualify_invalid_yaml() {
    let dir = tempfile::tempdir().expect("tempdir");
    let yaml_path = dir.path().join("recipe.yaml");
    std::fs::write(&yaml_path, "{{invalid yaml").expect("write");

    let result = QualifyResult {
        validate: ok_outcome(100),
        plan: None,
        first_apply: None,
        idempotent_apply: None,
        idempotent: false,
    };
    let out = score_after_qualify(&yaml_path, &result);
    assert!(out.is_none());
}

/// Path to a binary that always succeeds (no script creation needed).
fn ok_forjar() -> &'static str {
    "/bin/true"
}

/// Path to a binary that always fails (no script creation needed).
fn fail_forjar() -> &'static str {
    "/bin/false"
}

#[test]
fn run_validate_success() {
    let dir = tempfile::tempdir().expect("tempdir");
    let yaml_path = dir.path().join("recipe.yaml");
    std::fs::write(&yaml_path, sample_yaml()).expect("write");

    let report = run_validate(ok_forjar(), &yaml_path);
    assert!(report.is_ok());
    let text = report.expect("should succeed");
    assert!(text.contains("OK:"));
}

#[test]
fn run_validate_failure() {
    let dir = tempfile::tempdir().expect("tempdir");
    let yaml_path = dir.path().join("recipe.yaml");
    std::fs::write(&yaml_path, sample_yaml()).expect("write");

    let result = run_validate(fail_forjar(), &yaml_path);
    assert!(result.is_err());
    let err = result.expect_err("should fail");
    assert!(err.contains("validation failed"));
}

#[test]
fn run_qualify_with_ok_forjar() {
    let dir = tempfile::tempdir().expect("tempdir");
    let yaml_path = dir.path().join("recipe.yaml");
    std::fs::write(&yaml_path, sample_yaml()).expect("write");
    let state_dir = dir.path().join("state");

    let result = run_qualify(ok_forjar(), &yaml_path, &state_dir);
    // /bin/true exits 0 but output doesn't contain "0 changed" → idempotency fails
    assert!(result.is_err());
    let err = result.expect_err("idempotency should fail");
    assert!(err.contains("idempotency"));
}

#[test]
fn run_qualify_with_fail_forjar() {
    let dir = tempfile::tempdir().expect("tempdir");
    let yaml_path = dir.path().join("recipe.yaml");
    std::fs::write(&yaml_path, sample_yaml()).expect("write");
    let state_dir = dir.path().join("state");

    let result = run_qualify(fail_forjar(), &yaml_path, &state_dir);
    assert!(result.is_err());
}
