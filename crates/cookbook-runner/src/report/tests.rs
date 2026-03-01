//! Tests for qualification report formatting and verdict logic.

use super::*;
use crate::RunOutcome;
use std::path::Path;

fn ok_outcome(ms: u64) -> RunOutcome {
    RunOutcome {
        exit_code: 0,
        output: String::new(),
        duration_ms: ms,
    }
}

fn fail_outcome(ms: u64) -> RunOutcome {
    RunOutcome {
        exit_code: 1,
        output: "error".to_string(),
        duration_ms: ms,
    }
}

fn qualified_result() -> QualifyResult {
    QualifyResult {
        validate: ok_outcome(10),
        plan: Some(ok_outcome(20)),
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
    }
}

// --- QualifyVerdict tests ---

#[test]
fn verdict_qualified() {
    let result = qualified_result();
    assert_eq!(verdict(&result), QualifyVerdict::Qualified);
}

#[test]
fn verdict_validation_failed() {
    let result = QualifyResult {
        validate: fail_outcome(5),
        plan: None,
        first_apply: None,
        idempotent_apply: None,
        idempotent: false,
    };
    assert_eq!(verdict(&result), QualifyVerdict::ValidationFailed);
}

#[test]
fn verdict_plan_failed() {
    let result = QualifyResult {
        validate: ok_outcome(10),
        plan: Some(fail_outcome(20)),
        first_apply: None,
        idempotent_apply: None,
        idempotent: false,
    };
    assert_eq!(verdict(&result), QualifyVerdict::PlanFailed);
}

#[test]
fn verdict_apply_failed() {
    let result = QualifyResult {
        validate: ok_outcome(10),
        plan: Some(ok_outcome(20)),
        first_apply: Some(fail_outcome(30)),
        idempotent_apply: None,
        idempotent: false,
    };
    assert_eq!(verdict(&result), QualifyVerdict::ApplyFailed);
}

#[test]
fn verdict_idempotency_failed() {
    let result = QualifyResult {
        validate: ok_outcome(10),
        plan: Some(ok_outcome(20)),
        first_apply: Some(ok_outcome(30)),
        idempotent_apply: Some(RunOutcome {
            exit_code: 0,
            output: "1 changed".to_string(),
            duration_ms: 40,
        }),
        idempotent: false,
    };
    assert_eq!(verdict(&result), QualifyVerdict::IdempotencyFailed);
}

// --- QualifyVerdict methods ---

#[test]
fn verdict_is_qualified() {
    assert!(QualifyVerdict::Qualified.is_qualified());
    assert!(!QualifyVerdict::ValidationFailed.is_qualified());
    assert!(!QualifyVerdict::PlanFailed.is_qualified());
    assert!(!QualifyVerdict::ApplyFailed.is_qualified());
    assert!(!QualifyVerdict::IdempotencyFailed.is_qualified());
}

#[test]
fn verdict_error_message() {
    assert!(QualifyVerdict::Qualified.error_message().is_none());
    assert_eq!(
        QualifyVerdict::ValidationFailed.error_message(),
        Some("validation failed")
    );
    assert_eq!(
        QualifyVerdict::PlanFailed.error_message(),
        Some("plan failed")
    );
    assert_eq!(
        QualifyVerdict::ApplyFailed.error_message(),
        Some("first apply failed")
    );
    assert_eq!(
        QualifyVerdict::IdempotencyFailed.error_message(),
        Some("idempotency check failed")
    );
}

// --- format_validate_report tests ---

#[test]
fn validate_report_success() {
    let report = format_validate_report(Path::new("test.yaml"), 0, 42);
    assert!(report.contains("OK"));
    assert!(report.contains("test.yaml"));
    assert!(report.contains("42ms"));
}

#[test]
fn validate_report_failure() {
    let report = format_validate_report(Path::new("bad.yaml"), 1, 5);
    assert!(report.contains("FAIL"));
    assert!(report.contains("bad.yaml"));
    assert!(report.contains("exit 1"));
}

// --- format_qualify_report tests ---

#[test]
fn qualify_report_full_success() {
    let result = qualified_result();
    let report = format_qualify_report(Path::new("recipe.yaml"), &result);
    assert!(report.contains("Qualifying: recipe.yaml"));
    assert!(report.contains("validate: exit=0"));
    assert!(report.contains("plan:"));
    assert!(report.contains("apply:"));
    assert!(report.contains("idempotent:"));
    assert!(report.contains("zero_changes=true"));
    assert!(report.contains("QUALIFIED: recipe.yaml"));
}

#[test]
fn qualify_report_validation_failure() {
    let result = QualifyResult {
        validate: fail_outcome(5),
        plan: None,
        first_apply: None,
        idempotent_apply: None,
        idempotent: false,
    };
    let report = format_qualify_report(Path::new("bad.yaml"), &result);
    assert!(report.contains("validate: exit=1"));
    assert!(!report.contains("plan:"));
    assert!(report.contains("FAILED: validation failed"));
}

#[test]
fn qualify_report_plan_failure() {
    let result = QualifyResult {
        validate: ok_outcome(10),
        plan: Some(fail_outcome(20)),
        first_apply: None,
        idempotent_apply: None,
        idempotent: false,
    };
    let report = format_qualify_report(Path::new("r.yaml"), &result);
    assert!(report.contains("plan:     exit=1"));
    assert!(report.contains("FAILED: plan failed"));
}

#[test]
fn qualify_report_idempotency_failure() {
    let result = QualifyResult {
        validate: ok_outcome(10),
        plan: Some(ok_outcome(20)),
        first_apply: Some(ok_outcome(30)),
        idempotent_apply: Some(RunOutcome {
            exit_code: 0,
            output: "1 changed".to_string(),
            duration_ms: 40,
        }),
        idempotent: false,
    };
    let report = format_qualify_report(Path::new("r.yaml"), &result);
    assert!(report.contains("zero_changes=false"));
    assert!(report.contains("FAILED: idempotency check failed"));
}

// --- Debug/Clone/PartialEq ---

#[test]
fn verdict_debug() {
    let v = QualifyVerdict::Qualified;
    let debug = format!("{v:?}");
    assert!(debug.contains("Qualified"));
}

#[test]
fn verdict_clone_eq() {
    let v = QualifyVerdict::PlanFailed;
    let v2 = v.clone();
    assert_eq!(v, v2);
}
