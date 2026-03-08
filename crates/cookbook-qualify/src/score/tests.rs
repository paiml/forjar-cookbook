//! Tests for `ForjarScore` v2 bridge to forjar's scoring engine.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::qualify::{IdempotencyClass, RecipeStatus};

fn sample_yaml() -> &'static str {
    "\
# Recipe: Test Recipe
# Tier: 2+3
# Idempotency: Strong
version: '1.0'
name: test-recipe
description: A test recipe
resources:
  my-file:
    type: file
    path: /tmp/test
    mode: '0644'
    owner: root
    content: hello
"
}

// ── Grade tests ──────────────────────────────────────────────────

#[test]
fn grade_from_csv_all_variants() {
    assert_eq!(Grade::from_csv("A").unwrap(), Grade::A);
    assert_eq!(Grade::from_csv("b").unwrap(), Grade::B);
    assert_eq!(Grade::from_csv(" C ").unwrap(), Grade::C);
    assert_eq!(Grade::from_csv("d").unwrap(), Grade::D);
    assert_eq!(Grade::from_csv("F").unwrap(), Grade::F);
    assert_eq!(Grade::from_csv("").unwrap(), Grade::F);
    assert!(Grade::from_csv("X").is_err());
}

#[test]
fn grade_as_str() {
    assert_eq!(Grade::A.as_str(), "A");
    assert_eq!(Grade::F.as_str(), "F");
}

#[test]
fn grade_badge() {
    assert!(Grade::A.badge().contains("brightgreen"));
    assert!(Grade::F.badge().contains("red"));
}

#[test]
fn grade_from_char() {
    assert_eq!(Grade::from_char('A'), Grade::A);
    assert_eq!(Grade::from_char('B'), Grade::B);
    assert_eq!(Grade::from_char('C'), Grade::C);
    assert_eq!(Grade::from_char('D'), Grade::D);
    assert_eq!(Grade::from_char('F'), Grade::F);
    assert_eq!(Grade::from_char('Z'), Grade::F);
}

// ── DimensionScores tests ────────────────────────────────────────

#[test]
fn dimension_scores_min() {
    let d = DimensionScores {
        cor: 100,
        idm: 90,
        prf: 80,
        saf: 70,
        obs: 60,
        doc: 50,
        res: 40,
        cmp: 30,
    };
    assert_eq!(d.min_score(), 30);
}

// ── ForjarScore compute tests ────────────────────────────────────

#[test]
fn compute_invalid_yaml() {
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        raw_yaml: "{{invalid yaml!!",
        budget_ms: 0,
        runtime: None,
    };
    let score = ForjarScore::compute(&input);
    assert_eq!(score.grade, Grade::F);
    assert_eq!(score.composite, 0);
    assert!(!score.penalties.is_empty());
}

#[test]
fn compute_qualified_recipe() {
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        raw_yaml: sample_yaml(),
        budget_ms: 0,
        runtime: None,
    };
    let score = ForjarScore::compute(&input);
    assert!(score.dimensions.saf > 0, "SAF should be > 0");
    assert!(score.dimensions.obs > 0, "OBS should be > 0");
}

#[test]
fn compute_pending_gets_real_grade() {
    let input = ScoringInput {
        status: &RecipeStatus::Pending,
        idempotency_class: &IdempotencyClass::Strong,
        raw_yaml: sample_yaml(),
        budget_ms: 0,
        runtime: None,
    };
    let score = ForjarScore::compute(&input);
    // v2: pending recipes get a real static grade, not automatic F
    assert!(score.dimensions.saf > 0);
    assert!(score.dimensions.doc > 0);
}

#[test]
fn compute_blocked_gets_real_dimensions() {
    let input = ScoringInput {
        status: &RecipeStatus::Blocked,
        idempotency_class: &IdempotencyClass::Strong,
        raw_yaml: sample_yaml(),
        budget_ms: 0,
        runtime: None,
    };
    let score = ForjarScore::compute(&input);
    assert!(
        score.dimensions.saf > 0,
        "blocked should still compute static dims"
    );
}

#[test]
fn compute_with_runtime_data() {
    let rt = RuntimeData {
        validate_pass: true,
        plan_pass: true,
        first_apply_pass: true,
        second_apply_pass: true,
        zero_changes: true,
        hash_stable: true,
        changed_on_reapply: 0,
        warning_count: 0,
        first_apply_ms: 5000,
        idempotent_apply_ms: 200,
        state_lock_written: true,
        all_resources_converged: true,
    };
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        raw_yaml: sample_yaml(),
        budget_ms: 60000,
        runtime: Some(&rt),
    };
    let score = ForjarScore::compute(&input);
    assert!(score.dimensions.cor > 0, "COR should score with runtime");
    assert!(score.dimensions.idm > 0, "IDM should score with runtime");
    assert!(
        score.dimensions.prf > 0,
        "PRF should score with budget+runtime"
    );
}

#[test]
fn score_version_is_v2() {
    assert_eq!(SCORE_VERSION, "2.0");
}

// ── RuntimeData mapping ──────────────────────────────────────────

#[test]
fn runtime_data_to_forjar_maps_fields() {
    let rt = RuntimeData {
        validate_pass: true,
        plan_pass: false,
        first_apply_pass: true,
        second_apply_pass: true,
        zero_changes: true,
        hash_stable: false,
        changed_on_reapply: 3,
        warning_count: 2,
        first_apply_ms: 5000,
        idempotent_apply_ms: 200,
        state_lock_written: true,
        all_resources_converged: false,
    };
    let fj = rt.to_forjar();
    assert!(fj.validate_pass);
    assert!(!fj.plan_pass);
    assert!(fj.zero_changes_on_reapply);
    assert_eq!(fj.changed_on_reapply, 3);
    assert_eq!(fj.second_apply_ms, 200);
    assert!(!fj.all_resources_converged);
}

// ── Grade min ────────────────────────────────────────────────────

#[test]
fn grade_min_char_tests() {
    assert_eq!(grade_min_char('A', 'A'), 'A');
    assert_eq!(grade_min_char('A', 'B'), 'B');
    assert_eq!(grade_min_char('B', 'A'), 'B');
    assert_eq!(grade_min_char('A', 'F'), 'F');
    assert_eq!(grade_min_char('C', 'D'), 'D');
}
