//! Per-dimension scoring tests for COR, IDM, PRF, SAF, DOC, RES, CMP.

use crate::qualify::{IdempotencyClass, RecipeStatus};
use crate::score::dimensions::{
    score_cmp, score_cor, score_doc, score_idm, score_prf, score_res, score_saf,
};
use crate::score::{RecipeConfig, RuntimeData, SAFETY_CRITICAL_CAP, ScoringInput};

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

// ── COR scoring ──────────────────────────────────────────────────

#[test]
fn cor_perfect_runtime() {
    let config = minimal_config();
    let rt = full_runtime();
    let mut penalties = Vec::new();
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: "",
        budget_ms: 0,
        runtime: Some(&rt),
    };
    let score = score_cor(&input, &mut penalties);
    assert_eq!(score, 100);
    assert!(penalties.is_empty());
}

#[test]
fn cor_with_warnings() {
    let config = minimal_config();
    let mut rt = full_runtime();
    rt.warning_count = 3;
    let mut penalties = Vec::new();
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: "",
        budget_ms: 0,
        runtime: Some(&rt),
    };
    let score = score_cor(&input, &mut penalties);
    // 100 - 6 = 94
    assert_eq!(score, 94);
    assert_eq!(penalties.len(), 1);
}

#[test]
fn cor_no_runtime_returns_zero() {
    let config = minimal_config();
    let mut penalties = Vec::new();
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: "",
        budget_ms: 0,
        runtime: None,
    };
    assert_eq!(score_cor(&input, &mut penalties), 0);
}

// ── IDM scoring ──────────────────────────────────────────────────

#[test]
fn idm_perfect_strong() {
    let config = minimal_config();
    let rt = full_runtime();
    let mut penalties = Vec::new();
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: "",
        budget_ms: 0,
        runtime: Some(&rt),
    };
    assert_eq!(score_idm(&input, &mut penalties), 100);
}

#[test]
fn idm_weak_class_gets_less() {
    let config = minimal_config();
    let rt = full_runtime();
    let mut penalties = Vec::new();
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Weak,
        config: &config,
        raw_yaml: "",
        budget_ms: 0,
        runtime: Some(&rt),
    };
    // 30 + 30 + 20 + 10 = 90
    assert_eq!(score_idm(&input, &mut penalties), 90);
}

#[test]
fn idm_changed_resources_penalized() {
    let config = minimal_config();
    let mut rt = full_runtime();
    rt.changed_on_reapply = 2;
    rt.zero_changes = false;
    let mut penalties = Vec::new();
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: "",
        budget_ms: 0,
        runtime: Some(&rt),
    };
    // 30 + 0 (no zero_changes) + 20 + 20 - 20 (2 changed) = 50
    assert_eq!(score_idm(&input, &mut penalties), 50);
}

// ── PRF scoring ──────────────────────────────────────────────────

#[test]
fn prf_fast_first_apply() {
    let config = minimal_config();
    let mut rt = full_runtime();
    rt.first_apply_ms = 25000; // 25s vs 60s budget = 42%
    rt.idempotent_apply_ms = 1000;
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: "",
        budget_ms: 60000,
        runtime: Some(&rt),
    };
    let score = score_prf(&input);
    // <=50%: 50pts + <=2s: 30pts + ratio 4%: 20pts = 100
    assert_eq!(score, 100);
}

#[test]
fn prf_slow_first_apply() {
    let config = minimal_config();
    let mut rt = full_runtime();
    rt.first_apply_ms = 80000; // 80s vs 60s budget = 133%
    rt.idempotent_apply_ms = 8000;
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: "",
        budget_ms: 60000,
        runtime: Some(&rt),
    };
    let score = score_prf(&input);
    // <=150%: 15pts + <=10s: 15pts + ratio 10%: 15pts = 45
    assert_eq!(score, 45);
}

// ── SAF scoring ──────────────────────────────────────────────────

#[test]
fn saf_no_resources_is_perfect() {
    let config = minimal_config();
    let mut penalties = Vec::new();
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: "",
        budget_ms: 0,
        runtime: None,
    };
    assert_eq!(score_saf(&input, &mut penalties), 100);
}

#[test]
fn saf_file_without_mode_penalized() {
    let config = RecipeConfig::from_yaml(
        "version: '1.0'\nname: test\nresources:\n  f:\n    type: file\n    dest: /tmp/x\n",
    )
    .unwrap_or_else(|_| minimal_config());
    let mut penalties = Vec::new();
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: "",
        budget_ms: 0,
        runtime: None,
    };
    let score = score_saf(&input, &mut penalties);
    // 100 - 5 (no mode) - 3 (no owner) = 92
    assert_eq!(score, 92);
}

#[test]
fn saf_mode_0777_critical_cap() {
    let config = RecipeConfig::from_yaml(
        "version: '1.0'\nname: test\nresources:\n  f:\n    type: file\n    dest: /tmp/x\n    mode: '0777'\n    owner: root\n",
    )
    .unwrap_or_else(|_| minimal_config());
    let mut penalties = Vec::new();
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: "",
        budget_ms: 0,
        runtime: None,
    };
    let score = score_saf(&input, &mut penalties);
    assert!(score <= SAFETY_CRITICAL_CAP);
}

#[test]
fn saf_curl_bash_critical() {
    let config = minimal_config();
    let mut penalties = Vec::new();
    let yaml_with_curl = "curl -sSL https://example.com | bash";
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: yaml_with_curl,
        budget_ms: 0,
        runtime: None,
    };
    let score = score_saf(&input, &mut penalties);
    assert!(score <= SAFETY_CRITICAL_CAP);
}

// ── DOC scoring ──────────────────────────────────────────────────

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

#[test]
fn doc_full_header_and_description() {
    let config = RecipeConfig::from_yaml(SAMPLE_YAML).unwrap_or_else(|_| minimal_config());
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: SAMPLE_YAML,
        budget_ms: 0,
        runtime: None,
    };
    let score = score_doc(&input);
    // 3 comments in ~16 lines >= 15% → 40pts
    // Recipe # header → +10, Tier → +10, Idempotency → +10
    // description present → +15, name has dash → +5
    // Total: 40 + 10 + 10 + 10 + 15 + 5 = 90
    assert_eq!(score, 90);
}

#[test]
fn doc_no_comments_no_header() {
    let config = minimal_config();
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: "version: '1.0'\nname: test-recipe\n",
        budget_ms: 0,
        runtime: None,
    };
    let score = score_doc(&input);
    // No comments → 0, no header metadata → 0, no description → 0
    // name has dash → +5
    assert_eq!(score, 5);
}

// ── RES scoring ──────────────────────────────────────────────────

#[test]
fn res_with_failure_policy_and_hooks() {
    let yaml = "\
version: '1.0'
name: test-recipe
failure: continue_independent
ssh_retries: 3
pre_apply: echo pre
post_apply: echo post
resources:
  a:
    type: file
    dest: /tmp/a
    depends_on: [b]
  b:
    type: file
    dest: /tmp/b
";
    let config = RecipeConfig::from_yaml(yaml).unwrap_or_else(|_| minimal_config());
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: yaml,
        budget_ms: 0,
        runtime: None,
    };
    let score = score_res(&input);
    // failure_policy: +20, ssh_retries: +10, DAG 50%: +30, pre: +10, post: +10 = 80
    assert_eq!(score, 80);
}

// ── CMP scoring ──────────────────────────────────────────────────

#[test]
fn cmp_with_params_and_tags() {
    let yaml = "\
version: '1.0'
name: test-recipe
params:
  env: production
resources:
  a:
    type: file
    dest: /tmp/a
    tags: [web]
";
    let config = RecipeConfig::from_yaml(yaml).unwrap_or_else(|_| minimal_config());
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: yaml,
        budget_ms: 0,
        runtime: None,
    };
    let score = score_cmp(&input);
    // params: +20, tags: +15 = 35
    assert_eq!(score, 35);
}

#[test]
fn cmp_with_params_tags_and_template_interpolation() {
    let yaml = "\
version: '1.0'
name: test-recipe
params:
  base_dir: /tmp/test
resources:
  a:
    type: file
    path: \"{{params.base_dir}}/file.txt\"
    tags: [web]
    resource_group: app
";
    let config = RecipeConfig::from_yaml(yaml).unwrap_or_else(|_| minimal_config());
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: yaml,
        budget_ms: 0,
        runtime: None,
    };
    let score = score_cmp(&input);
    // params: +20, templates ({{...}}): +10, tags: +15, resource_group: +15 = 60
    assert_eq!(score, 60);
}

#[test]
fn cmp_empty_config_is_zero() {
    let config = minimal_config();
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: "",
        budget_ms: 0,
        runtime: None,
    };
    assert_eq!(score_cmp(&input), 0);
}
