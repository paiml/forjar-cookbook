#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Tests for Forjar Score — integration, grade boundaries, hard-fails,
//! composite calculations, and type roundtrips.

mod tests_dimensions;
mod tests_dimensions_ext;

use super::*;
use crate::qualify::{IdempotencyClass, RecipeStatus};

// ── Helpers ──────────────────────────────────────────────────────

fn minimal_config() -> RecipeConfig {
    RecipeConfig {
        name: "test-recipe".to_string(),
        version: "1.0".to_string(),
        description: None,
        machines: std::collections::HashMap::new(),
        resources: std::collections::HashMap::new(),
        params: None,
        failure: None,
        outputs: None,
        tripwire: None,
        includes: None,
        pre_apply: None,
        post_apply: None,
        notify: None,
        ssh_retries: None,
        lock_file: None,
        policy: None,
    }
}

fn full_runtime() -> RuntimeData {
    RuntimeData {
        validate_pass: true,
        plan_pass: true,
        first_apply_pass: true,
        second_apply_pass: true,
        zero_changes: true,
        hash_stable: true,
        changed_on_reapply: 0,
        warning_count: 0,
        first_apply_ms: 30000,
        idempotent_apply_ms: 1200,
        state_lock_written: true,
        all_resources_converged: true,
    }
}

const SAMPLE_YAML: &str = "\
# Recipe #1: Test Recipe
# Tier: 2+3 — container + bare-metal
# Idempotency: Strong
version: \"1.0\"
name: test-recipe
description: A test recipe for scoring
machines:
  target:
    hostname: target
    addr: localhost
resources:
  my-file:
    type: file
    machine: target
    dest: /tmp/test
    mode: \"0644\"
    owner: root
";

// ── Hard-fail tests ──────────────────────────────────────────────

#[test]
fn blocked_recipe_always_f() {
    let config = minimal_config();
    let rt = full_runtime();
    let input = ScoringInput {
        status: &RecipeStatus::Blocked,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: SAMPLE_YAML,
        budget_ms: 60000,
        runtime: Some(&rt),
    };
    let score = ForjarScore::compute(&input);
    assert_eq!(score.grade, Grade::F);
    assert_eq!(score.composite, 0);
}

#[test]
fn pending_recipe_always_f() {
    let config = minimal_config();
    let input = ScoringInput {
        status: &RecipeStatus::Pending,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: SAMPLE_YAML,
        budget_ms: 0,
        runtime: None,
    };
    let score = ForjarScore::compute(&input);
    assert_eq!(score.grade, Grade::F);
    assert_eq!(score.composite, 0);
}

#[test]
fn validation_failure_hard_fail() {
    let config = minimal_config();
    let mut rt = full_runtime();
    rt.validate_pass = false;
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: SAMPLE_YAML,
        budget_ms: 60000,
        runtime: Some(&rt),
    };
    let score = ForjarScore::compute(&input);
    assert_eq!(score.grade, Grade::F);
    assert_eq!(score.composite, 0);
}

#[test]
fn plan_failure_hard_fail() {
    let config = minimal_config();
    let mut rt = full_runtime();
    rt.plan_pass = false;
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: SAMPLE_YAML,
        budget_ms: 60000,
        runtime: Some(&rt),
    };
    let score = ForjarScore::compute(&input);
    assert_eq!(score.grade, Grade::F);
}

#[test]
fn apply_failure_hard_fail() {
    let config = minimal_config();
    let mut rt = full_runtime();
    rt.first_apply_pass = false;
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: SAMPLE_YAML,
        budget_ms: 60000,
        runtime: Some(&rt),
    };
    let score = ForjarScore::compute(&input);
    assert_eq!(score.grade, Grade::F);
}

// ── Grade boundary tests ─────────────────────────────────────────

#[test]
fn grade_a_requires_composite_90_and_min_80() {
    assert_eq!(compute_grade(90, 80), Grade::A);
    assert_eq!(compute_grade(95, 85), Grade::A);
    assert_eq!(compute_grade(100, 100), Grade::A);
}

#[test]
fn grade_a_denied_if_min_dim_below_80() {
    assert_eq!(compute_grade(95, 79), Grade::B);
}

#[test]
fn grade_a_denied_if_composite_below_90() {
    assert_eq!(compute_grade(89, 85), Grade::B);
}

#[test]
fn grade_b_boundaries() {
    assert_eq!(compute_grade(75, 60), Grade::B);
    assert_eq!(compute_grade(89, 79), Grade::B);
    assert_eq!(compute_grade(80, 59), Grade::C);
}

#[test]
fn grade_c_boundaries() {
    assert_eq!(compute_grade(60, 40), Grade::C);
    assert_eq!(compute_grade(74, 59), Grade::C);
    assert_eq!(compute_grade(65, 39), Grade::D);
}

#[test]
fn grade_d_boundaries() {
    assert_eq!(compute_grade(40, 10), Grade::D);
    assert_eq!(compute_grade(59, 39), Grade::D);
}

#[test]
fn grade_f_below_40() {
    assert_eq!(compute_grade(39, 39), Grade::F);
    assert_eq!(compute_grade(0, 0), Grade::F);
}

// ── Determinism test ─────────────────────────────────────────────

#[test]
fn same_input_produces_same_score() {
    let config = RecipeConfig::from_yaml(SAMPLE_YAML).unwrap_or_else(|_| minimal_config());
    let rt = full_runtime();
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: SAMPLE_YAML,
        budget_ms: 60000,
        runtime: Some(&rt),
    };
    let score1 = ForjarScore::compute(&input);
    let score2 = ForjarScore::compute(&input);
    assert_eq!(score1.composite, score2.composite);
    assert_eq!(score1.grade, score2.grade);
    assert_eq!(score1.dimensions, score2.dimensions);
}

// ── A-grade impossibility: weak dimension blocks A ───────────────

#[test]
fn weak_cmp_blocks_a_grade() {
    let config = minimal_config(); // no params, no tags → CMP = 0
    let rt = full_runtime();
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: SAMPLE_YAML,
        budget_ms: 60000,
        runtime: Some(&rt),
    };
    let score = ForjarScore::compute(&input);
    assert_ne!(score.grade, Grade::A);
    assert_eq!(score.dimensions.cmp, 0);
}

// ── Composite calculation ────────────────────────────────────────

#[test]
fn composite_all_100() {
    let d = DimensionScores {
        cor: 100,
        idm: 100,
        prf: 100,
        saf: 100,
        obs: 100,
        doc: 100,
        res: 100,
        cmp: 100,
    };
    assert_eq!(compute_composite(&d), 100);
}

#[test]
fn composite_all_zero() {
    let d = DimensionScores {
        cor: 0,
        idm: 0,
        prf: 0,
        saf: 0,
        obs: 0,
        doc: 0,
        res: 0,
        cmp: 0,
    };
    assert_eq!(compute_composite(&d), 0);
}

#[test]
fn composite_weighted_correctly() {
    let d = DimensionScores {
        cor: 100,
        idm: 0,
        prf: 0,
        saf: 0,
        obs: 0,
        doc: 0,
        res: 0,
        cmp: 0,
    };
    assert_eq!(compute_composite(&d), 20);
}

// ── Grade from_csv / as_str roundtrip ────────────────────────────

#[test]
fn grade_roundtrip() {
    for grade in &[Grade::A, Grade::B, Grade::C, Grade::D, Grade::F] {
        let s = grade.as_str();
        let parsed = Grade::from_csv(s);
        assert_eq!(parsed, Ok(*grade));
    }
}

#[test]
fn grade_from_csv_empty_is_f() {
    assert_eq!(Grade::from_csv(""), Ok(Grade::F));
}

#[test]
fn grade_badge_contains_color() {
    assert!(Grade::A.badge().contains("brightgreen"));
    assert!(Grade::B.badge().contains("blue"));
    assert!(Grade::C.badge().contains("yellow"));
    assert!(Grade::D.badge().contains("orange"));
    assert!(Grade::F.badge().contains("red"));
}

// ── DimensionScores min_score ────────────────────────────────────

#[test]
fn min_score_finds_minimum() {
    let d = DimensionScores {
        cor: 90,
        idm: 85,
        prf: 80,
        saf: 75,
        obs: 70,
        doc: 65,
        res: 60,
        cmp: 55,
    };
    assert_eq!(d.min_score(), 55);
}

// ── RecipeConfig parsing ─────────────────────────────────────────

#[test]
fn recipe_config_parse_sample() {
    let config = RecipeConfig::from_yaml(SAMPLE_YAML);
    assert!(config.is_ok());
    let config = config.unwrap_or_else(|_| minimal_config());
    assert_eq!(config.name, "test-recipe");
    assert!(config.description.is_some());
    assert_eq!(config.resources.len(), 1);
}

#[test]
fn recipe_config_invalid_yaml() {
    let result = RecipeConfig::from_yaml("{{invalid yaml!!");
    assert!(result.is_err());
}

// ── Policy section parsing ───────────────────────────────────────

const POLICY_YAML: &str = "\
version: '1.0'
name: policy-test
resources:
  my-file:
    type: file
    machine: target
    path: /tmp/test
policy:
  failure: continue_independent
  tripwire: true
  lock_file: true
  ssh_retries: 3
  pre_apply: \"echo before\"
  post_apply: \"echo after\"
  notify:
    on_success: \"echo ok\"
";

#[test]
fn policy_section_parsed() {
    let config = RecipeConfig::from_yaml(POLICY_YAML).expect("parse");
    assert!(config.policy.is_some());
    let p = config.policy.as_ref().unwrap();
    assert_eq!(p.failure.as_deref(), Some("continue_independent"));
    assert_eq!(p.ssh_retries, Some(3));
    assert!(p.pre_apply.is_some());
    assert!(p.post_apply.is_some());
}

#[test]
fn eff_failure_from_policy() {
    let config = RecipeConfig::from_yaml(POLICY_YAML).expect("parse");
    assert_eq!(config.eff_failure(), Some("continue_independent"));
}

#[test]
fn eff_failure_from_top_level() {
    let mut config = minimal_config();
    config.failure = Some("continue_independent".to_string());
    assert_eq!(config.eff_failure(), Some("continue_independent"));
}

#[test]
fn eff_tripwire_from_policy() {
    let config = RecipeConfig::from_yaml(POLICY_YAML).expect("parse");
    assert!(config.eff_tripwire().is_some());
}

#[test]
fn eff_lock_file_from_policy() {
    let config = RecipeConfig::from_yaml(POLICY_YAML).expect("parse");
    assert!(config.eff_lock_file());
}

#[test]
fn eff_ssh_retries_from_policy() {
    let config = RecipeConfig::from_yaml(POLICY_YAML).expect("parse");
    assert_eq!(config.eff_ssh_retries(), Some(3));
}

#[test]
fn eff_pre_post_apply_from_policy() {
    let config = RecipeConfig::from_yaml(POLICY_YAML).expect("parse");
    assert_eq!(config.eff_pre_apply(), Some("echo before"));
    assert_eq!(config.eff_post_apply(), Some("echo after"));
}

#[test]
fn eff_notify_from_policy() {
    let config = RecipeConfig::from_yaml(POLICY_YAML).expect("parse");
    assert!(config.eff_notify().is_some());
}

#[test]
fn eff_methods_none_when_no_policy() {
    let config = minimal_config();
    assert!(config.eff_failure().is_none());
    assert!(config.eff_tripwire().is_none());
    assert!(!config.eff_lock_file());
    assert!(config.eff_ssh_retries().is_none());
    assert!(config.eff_pre_apply().is_none());
    assert!(config.eff_post_apply().is_none());
    assert!(config.eff_notify().is_none());
}

// ── Blocked/pending still score static dimensions ────────────────

#[test]
fn blocked_recipe_scores_static_dimensions() {
    let config = RecipeConfig::from_yaml(SAMPLE_YAML).unwrap_or_else(|_| minimal_config());
    let input = ScoringInput {
        status: &RecipeStatus::Blocked,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: SAMPLE_YAML,
        budget_ms: 0,
        runtime: None,
    };
    let score = ForjarScore::compute(&input);
    assert_eq!(score.grade, Grade::F);
    assert!(score.dimensions.doc > 0);
    assert!(score.dimensions.saf > 0);
    assert_eq!(score.dimensions.cor, 0);
    assert_eq!(score.dimensions.idm, 0);
    assert_eq!(score.dimensions.prf, 0);
}

// ── Score version ────────────────────────────────────────────────

#[test]
fn score_version_matches_const() {
    let config = minimal_config();
    let rt = full_runtime();
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: "",
        budget_ms: 60000,
        runtime: Some(&rt),
    };
    let score = ForjarScore::compute(&input);
    assert_eq!(score.version, SCORE_VERSION);
}
