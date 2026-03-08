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

// --- format_score_report tests ---

#[test]
fn score_report_shows_grade() {
    let score = cookbook_qualify::ForjarScore {
        composite: 83,
        grade: cookbook_qualify::Grade::B,
        static_grade: cookbook_qualify::Grade::B,
        runtime_grade: Some(cookbook_qualify::Grade::A),
        dimensions: cookbook_qualify::DimensionScores {
            cor: 100,
            idm: 100,
            prf: 85,
            saf: 82,
            obs: 60,
            doc: 90,
            res: 50,
            cmp: 35,
        },
        penalties: vec![],
        version: "2.0".to_string(),
    };
    let report = format_score_report(&score);
    assert!(report.contains("score: 83 (grade B)"));
    assert!(report.contains("COR=100"));
    assert!(report.contains("CMP= 35"));
}

// --- runtime_data_from_qualify tests ---

#[test]
fn runtime_data_from_qualified_result() {
    let result = qualified_result();
    let rt = runtime_data_from_qualify(&result);
    assert!(rt.validate_pass);
    assert!(rt.plan_pass);
    assert!(rt.first_apply_pass);
    assert!(rt.second_apply_pass);
    assert!(rt.zero_changes);
    assert!(rt.hash_stable);
    assert_eq!(rt.first_apply_ms, 45000);
    assert_eq!(rt.idempotent_apply_ms, 1200);
    assert!(rt.state_lock_written);
    assert!(rt.all_resources_converged);
}

#[test]
fn runtime_data_from_failed_result() {
    let result = QualifyResult {
        validate: ok_outcome(10),
        plan: Some(fail_outcome(20)),
        first_apply: None,
        idempotent_apply: None,
        idempotent: false,
    };
    let rt = runtime_data_from_qualify(&result);
    assert!(rt.validate_pass);
    assert!(!rt.plan_pass);
    assert!(!rt.first_apply_pass);
    assert!(!rt.second_apply_pass);
    assert!(!rt.zero_changes);
    assert_eq!(rt.first_apply_ms, 0);
    assert_eq!(rt.idempotent_apply_ms, 0);
}

#[test]
fn runtime_data_from_validate_only_failure() {
    let result = QualifyResult {
        validate: fail_outcome(5),
        plan: None,
        first_apply: None,
        idempotent_apply: None,
        idempotent: false,
    };
    let rt = runtime_data_from_qualify(&result);
    assert!(!rt.validate_pass);
    assert!(!rt.plan_pass);
    assert!(!rt.first_apply_pass);
}

#[test]
fn score_report_shows_penalties() {
    let score = cookbook_qualify::ForjarScore {
        composite: 70,
        grade: cookbook_qualify::Grade::C,
        static_grade: cookbook_qualify::Grade::C,
        runtime_grade: Some(cookbook_qualify::Grade::C),
        dimensions: cookbook_qualify::DimensionScores {
            cor: 94,
            idm: 80,
            prf: 60,
            saf: 70,
            obs: 40,
            doc: 50,
            res: 30,
            cmp: 20,
        },
        penalties: vec![cookbook_qualify::Penalty {
            dimension: "SAF".to_string(),
            points: 5,
            reason: "file without explicit mode".to_string(),
        }],
        version: "2.0".to_string(),
    };
    let report = format_score_report(&score);
    assert!(report.contains("penalties: 1"));
    assert!(report.contains("-5 SAF: file without explicit mode"));
}
