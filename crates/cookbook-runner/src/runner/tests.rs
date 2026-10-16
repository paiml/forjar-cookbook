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

/// Helper: write a script that succeeds for given subcommands, fails otherwise.
/// Returns (`TempDir` guard, script `PathBuf`). The `TempDir` keeps the dir alive.
#[allow(clippy::expect_used)]
fn write_test_script(succeed_on: &[&str]) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("create temp dir");
    let script_path = dir.path().join("fake-forjar.sh");
    let conditions: Vec<String> = succeed_on
        .iter()
        .map(|cmd| format!("[ \"$1\" = \"{cmd}\" ]"))
        .collect();
    let cond_str = conditions.join(" || ");
    let content = format!(
        "#!/bin/bash\nif {cond_str}; then\n  echo ok\n  exit 0\nelse\n  echo fail\n  exit 1\nfi\n"
    );
    // Write with explicit fsync to ensure visibility under parallel execution.
    {
        use std::io::Write;
        let mut f = std::fs::File::create(&script_path).expect("create script");
        f.write_all(content.as_bytes()).expect("write script");
        f.sync_all().expect("sync script");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755))
            .expect("set permissions");
        // Sync parent directory so the entry is visible to other threads.
        let parent = std::fs::File::open(dir.path()).expect("open dir");
        parent.sync_all().expect("sync dir");
        wait_until_executable(&script_path);
    }
    (dir, script_path)
}

/// Block until `path` can be exec'd without ETXTBSY (GH-26).
///
/// fsync does not prevent "Text file busy". While the write fd above was
/// open, a test on another thread may have forked; its child inherits that
/// fd until it execs, and the kernel refuses to exec an inode that any
/// process holds open for writing. Our own fd is closed by now, so no NEW
/// holder can appear: once one exec gets past ETXTBSY, every later exec
/// does too. Probe until that happens, so the runner under test never
/// sees the race.
#[cfg(unix)]
#[allow(clippy::expect_used, clippy::panic)]
fn wait_until_executable(path: &std::path::Path) {
    const ETXTBSY: i32 = 26;
    for _ in 0..500 {
        match std::process::Command::new(path)
            .arg("__probe__")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
        {
            Err(e) if e.raw_os_error() == Some(ETXTBSY) => {
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            other => {
                other.expect("probe exec of fake forjar");
                return;
            }
        }
    }
    panic!("{} stayed ETXTBSY for 1s", path.display());
}

/// GH-26 fixture: a writer that outlives our own close (the inherited fd of a
/// forked sibling) makes exec fail with ETXTBSY; `wait_until_executable`
/// must not return until it is gone. With the helper's body removed, the
/// exec after it fails here deterministically.
#[cfg(unix)]
#[test]
#[allow(clippy::expect_used)]
fn wait_until_executable_rides_out_a_lingering_writer() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().expect("create temp dir");
    let path = dir.path().join("busy.sh");
    std::fs::write(&path, "#!/bin/sh\nexit 0\n").expect("write script");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    let writer = std::fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .expect("hold a write fd");
    let busy = std::process::Command::new(&path).status();
    assert_eq!(
        busy.err().and_then(|e| e.raw_os_error()),
        Some(26),
        "an open write fd must make exec fail with ETXTBSY"
    );
    let release = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(50));
        drop(writer);
    });
    wait_until_executable(&path);
    let after = std::process::Command::new(&path).status();
    release.join().expect("join writer thread");
    assert!(
        after.is_ok_and(|s| s.success()),
        "exec right after wait_until_executable must succeed"
    );
}

#[test]
fn qualify_stops_on_plan_failure() {
    let (_dir, script) = write_test_script(&["validate"]);
    let script_path = script.display().to_string();

    let runner = RecipeRunner::new(&script_path);
    let result = runner.qualify(Path::new("test.yaml"), Path::new("/tmp/state"));

    assert_eq!(
        result.validate.exit_code, 0,
        "validate should succeed (output: {})",
        result.validate.output
    );
    assert!(result.plan.is_some(), "plan should have been attempted");
    assert!(
        result.plan.as_ref().is_some_and(|p| p.exit_code != 0),
        "plan should fail"
    );
    assert!(result.first_apply.is_none());
    assert!(result.idempotent_apply.is_none());
    assert!(!result.idempotent);
}

#[test]
fn qualify_stops_on_apply_failure() {
    let (_dir, script) = write_test_script(&["validate", "plan"]);
    let script_path = script.display().to_string();

    let runner = RecipeRunner::new(&script_path);
    let result = runner.qualify(Path::new("test.yaml"), Path::new("/tmp/state"));

    assert_eq!(
        result.validate.exit_code, 0,
        "validate should succeed (output: {})",
        result.validate.output
    );
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
}
