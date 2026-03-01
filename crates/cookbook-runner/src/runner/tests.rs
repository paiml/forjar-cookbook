//! Tests for recipe runner.

use super::*;

#[test]
fn run_outcome_defaults() {
    let outcome = RunOutcome {
        exit_code: 0,
        output: "success".to_string(),
        duration_ms: 100,
    };
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(outcome.duration_ms, 100);
}

#[test]
fn run_outcome_failure() {
    let outcome = RunOutcome {
        exit_code: 1,
        output: "error: something failed".to_string(),
        duration_ms: 50,
    };
    assert_eq!(outcome.exit_code, 1);
    assert!(outcome.output.contains("error"));
}

#[test]
fn recipe_runner_from_path() {
    let runner = RecipeRunner::from_path();
    assert_eq!(runner.forjar_bin, "forjar");
}

#[test]
fn recipe_runner_custom_path() {
    let runner = RecipeRunner::new("/usr/local/bin/forjar");
    assert_eq!(runner.forjar_bin, "/usr/local/bin/forjar");
}

#[test]
fn qualify_result_validate_failure() {
    let result = QualifyResult {
        validate: RunOutcome {
            exit_code: 1,
            output: "parse error".to_string(),
            duration_ms: 10,
        },
        plan: None,
        first_apply: None,
        idempotent_apply: None,
        idempotent: false,
    };
    assert!(result.plan.is_none());
    assert!(result.first_apply.is_none());
    assert!(!result.idempotent);
}

#[test]
fn qualify_result_idempotent() {
    let result = QualifyResult {
        validate: RunOutcome {
            exit_code: 0,
            output: String::new(),
            duration_ms: 10,
        },
        plan: Some(RunOutcome {
            exit_code: 0,
            output: String::new(),
            duration_ms: 20,
        }),
        first_apply: Some(RunOutcome {
            exit_code: 0,
            output: "3 changed".to_string(),
            duration_ms: 45000,
        }),
        idempotent_apply: Some(RunOutcome {
            exit_code: 0,
            output: "0 changed".to_string(),
            duration_ms: 1200,
        }),
        idempotent: true,
    };
    assert!(result.idempotent);
    assert_eq!(
        result.first_apply.as_ref().map(|o| o.duration_ms),
        Some(45000)
    );
}

#[test]
fn runner_validate_missing_binary() {
    let runner = RecipeRunner::new("/nonexistent/forjar");
    let outcome = runner.validate(Path::new("/tmp/test.yaml"));
    assert_eq!(outcome.exit_code, -1);
    assert!(outcome.output.contains("failed to execute"));
}

#[test]
fn runner_plan_missing_binary() {
    let runner = RecipeRunner::new("/nonexistent/forjar");
    let outcome = runner.plan(Path::new("/tmp/test.yaml"), Path::new("/tmp/state"));
    assert_eq!(outcome.exit_code, -1);
}

#[test]
fn runner_apply_missing_binary() {
    let runner = RecipeRunner::new("/nonexistent/forjar");
    let outcome = runner.apply(Path::new("/tmp/test.yaml"), Path::new("/tmp/state"));
    assert_eq!(outcome.exit_code, -1);
}

#[test]
fn runner_qualify_stops_on_validate_failure() {
    let runner = RecipeRunner::new("/nonexistent/forjar");
    let result = runner.qualify(Path::new("/tmp/test.yaml"), Path::new("/tmp/state"));
    assert_eq!(result.validate.exit_code, -1);
    assert!(result.plan.is_none());
    assert!(result.first_apply.is_none());
    assert!(result.idempotent_apply.is_none());
    assert!(!result.idempotent);
}

#[test]
fn run_forjar_captures_output() {
    // Use /bin/echo as a "forjar" substitute to test output capture
    let runner = RecipeRunner::new("/bin/echo");
    let outcome = runner.validate(Path::new("test.yaml"));
    // echo will succeed (exit 0) and output the args
    assert_eq!(outcome.exit_code, 0);
    assert!(outcome.output.contains("validate"));
    assert!(outcome.output.contains("test.yaml"));
    assert!(outcome.duration_ms < 5000);
}

#[test]
fn run_forjar_plan_captures_args() {
    let runner = RecipeRunner::new("/bin/echo");
    let outcome = runner.plan(Path::new("r.yaml"), Path::new("/tmp/st"));
    assert_eq!(outcome.exit_code, 0);
    assert!(outcome.output.contains("plan"));
    assert!(outcome.output.contains("r.yaml"));
    assert!(outcome.output.contains("--state-dir"));
}

#[test]
fn run_forjar_apply_captures_args() {
    let runner = RecipeRunner::new("/bin/echo");
    let outcome = runner.apply(Path::new("r.yaml"), Path::new("/tmp/st"));
    assert_eq!(outcome.exit_code, 0);
    assert!(outcome.output.contains("apply"));
    assert!(outcome.output.contains("--yes"));
}

#[test]
fn qualify_full_cycle_with_echo() {
    // Using /bin/echo as forjar means all steps succeed (exit 0)
    // but idempotent_apply won't contain "0 changed", so idempotent=false
    let runner = RecipeRunner::new("/bin/echo");
    let result = runner.qualify(Path::new("test.yaml"), Path::new("/tmp/state"));
    assert_eq!(result.validate.exit_code, 0);
    assert!(result.plan.is_some());
    assert!(result.first_apply.is_some());
    assert!(result.idempotent_apply.is_some());
    // echo output won't contain "0 changed"
    assert!(!result.idempotent);
}

#[test]
fn run_outcome_debug() {
    let outcome = RunOutcome {
        exit_code: 42,
        output: "test output".to_string(),
        duration_ms: 123,
    };
    let debug = format!("{outcome:?}");
    assert!(debug.contains("42"));
    assert!(debug.contains("test output"));
}

#[test]
fn qualify_result_debug() {
    let result = QualifyResult {
        validate: RunOutcome {
            exit_code: 0,
            output: String::new(),
            duration_ms: 0,
        },
        plan: None,
        first_apply: None,
        idempotent_apply: None,
        idempotent: false,
    };
    let debug = format!("{result:?}");
    assert!(debug.contains("QualifyResult"));
}

/// Generate a unique temp dir path for tests (avoids parallel test collisions).
fn unique_temp_dir(prefix: &str) -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = std::process::id();
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}-{id}-{seq}"))
}

/// Helper: write a script that succeeds for given subcommands, fails otherwise.
fn write_test_script(path: &std::path::Path, succeed_on: &[&str]) {
    let conditions: Vec<String> = succeed_on
        .iter()
        .map(|cmd| format!("[ \"$1\" = \"{cmd}\" ]"))
        .collect();
    let cond_str = conditions.join(" || ");
    let script = format!(
        "#!/bin/bash\nif {cond_str}; then\n  echo ok\n  exit 0\nelse\n  echo fail\n  exit 1\nfi\n"
    );
    std::fs::write(path, script).ok();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).ok();
    }
}

#[test]
fn qualify_stops_on_plan_failure() {
    let dir = unique_temp_dir("plan-fail");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).ok();
    let script = dir.join("fake-forjar-plan");
    write_test_script(&script, &["validate"]);

    let runner = RecipeRunner::new(&script.display().to_string());
    let result = runner.qualify(Path::new("test.yaml"), Path::new("/tmp/state"));

    assert_eq!(result.validate.exit_code, 0, "validate should succeed");
    assert!(result.plan.is_some(), "plan should have been attempted");
    assert!(
        result.plan.as_ref().is_some_and(|p| p.exit_code != 0),
        "plan should fail"
    );
    assert!(result.first_apply.is_none());
    assert!(result.idempotent_apply.is_none());
    assert!(!result.idempotent);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn qualify_stops_on_apply_failure() {
    let dir = unique_temp_dir("apply-fail");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).ok();
    let script = dir.join("fake-forjar-apply");
    write_test_script(&script, &["validate", "plan"]);

    let runner = RecipeRunner::new(&script.display().to_string());
    let result = runner.qualify(Path::new("test.yaml"), Path::new("/tmp/state"));

    assert_eq!(result.validate.exit_code, 0, "validate should succeed");
    assert!(
        result.plan.as_ref().is_some_and(|p| p.exit_code == 0),
        "plan should succeed"
    );
    assert!(result.first_apply.is_some(), "apply should be attempted");
    assert!(
        result
            .first_apply
            .as_ref()
            .is_some_and(|a| a.exit_code != 0),
        "apply should fail"
    );
    assert!(result.idempotent_apply.is_none());
    assert!(!result.idempotent);

    let _ = std::fs::remove_dir_all(&dir);
}
