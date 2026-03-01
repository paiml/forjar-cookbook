//! Extended per-dimension scoring tests — additional PRF, IDM, OBS, DOC, RES, CMP, SAF.

use crate::qualify::{IdempotencyClass, RecipeStatus};
use crate::score::dimensions::{
    score_cmp, score_doc, score_idm, score_obs, score_prf, score_res, score_saf,
};
use crate::score::{RecipeConfig, RuntimeData, ScoringInput};

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

// ── Additional PRF scoring ──────────────────────────────────────

#[test]
fn prf_no_runtime_returns_zero() {
    let config = minimal_config();
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: "",
        budget_ms: 60000,
        runtime: None,
    };
    assert_eq!(score_prf(&input), 0);
}

#[test]
fn prf_no_budget_assumes_on_budget() {
    let config = minimal_config();
    let mut rt = full_runtime();
    rt.first_apply_ms = 45000;
    rt.idempotent_apply_ms = 1500;
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: "",
        budget_ms: 0, // no budget
        runtime: Some(&rt),
    };
    let score = score_prf(&input);
    // no budget: 30pts + <=2s: 30pts + ratio 3.3%: 20pts = 80
    assert_eq!(score, 80);
}

#[test]
fn prf_medium_budget_ratio_75() {
    let config = minimal_config();
    let mut rt = full_runtime();
    rt.first_apply_ms = 42000; // 70% of budget
    rt.idempotent_apply_ms = 3500; // between 2s and 5s
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: "",
        budget_ms: 60000,
        runtime: Some(&rt),
    };
    let score = score_prf(&input);
    // <=75%: 40pts + <=5s: 25pts + ratio 8.3%: 15pts = 80
    assert_eq!(score, 80);
}

#[test]
fn prf_on_budget_ratio_100() {
    let config = minimal_config();
    let mut rt = full_runtime();
    rt.first_apply_ms = 55000; // 91.7% of budget
    rt.idempotent_apply_ms = 7000; // between 5s and 10s
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: "",
        budget_ms: 60000,
        runtime: Some(&rt),
    };
    let score = score_prf(&input);
    // <=100%: 30pts + <=10s: 15pts + ratio 12.7%: 10pts = 55
    assert_eq!(score, 55);
}

#[test]
fn prf_over_budget_no_points() {
    let config = minimal_config();
    let mut rt = full_runtime();
    rt.first_apply_ms = 100_000; // 167% of budget
    rt.idempotent_apply_ms = 12000; // over 10s
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: "",
        budget_ms: 60000,
        runtime: Some(&rt),
    };
    let score = score_prf(&input);
    // >150%: 0pts + >10s: 0pts + ratio 12%: 10pts = 10
    assert_eq!(score, 10);
}

// ── Additional IDM scoring ──────────────────────────────────────

#[test]
fn idm_eventual_class_no_bonus() {
    let config = minimal_config();
    let rt = full_runtime();
    let mut penalties = Vec::new();
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Eventual,
        config: &config,
        raw_yaml: "",
        budget_ms: 0,
        runtime: Some(&rt),
    };
    // 30 + 30 + 20 + 0 (eventual) = 80
    assert_eq!(score_idm(&input, &mut penalties), 80);
}

#[test]
fn idm_no_runtime_returns_zero() {
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
    assert_eq!(score_idm(&input, &mut penalties), 0);
}

// ── OBS scoring ──────────────────────────────────────────────────

#[test]
fn obs_empty_config_is_zero() {
    let config = minimal_config();
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: "",
        budget_ms: 0,
        runtime: None,
    };
    assert_eq!(score_obs(&input), 0);
}

#[test]
fn obs_with_tripwire_and_lock() {
    let yaml = "\
version: '1.0'
name: test-recipe
tripwire:
  policy: warn
lock_file: state.lock.yaml
outputs:
  db_host: test.example.com
resources:
  f:
    type: file
    dest: /tmp/x
    mode: '0644'
    owner: root
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
    let score = score_obs(&input);
    // tripwire: +15, lock_file: +15, outputs: +10, mode coverage 100%: +15, owner 100%: +15 = 70
    assert_eq!(score, 70);
}

#[test]
fn obs_with_notify_hooks() {
    let yaml = "\
version: '1.0'
name: test-recipe
notify:
  on_success: slack://channel
  on_failure: email://admin
  on_drift: webhook://url
resources: {}
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
    let score = score_obs(&input);
    // No tripwire/lock/outputs, no files → 0 + notify 3/3: 20pts = 20
    assert_eq!(score, 20);
}

#[test]
fn obs_partial_file_coverage() {
    let yaml = "\
version: '1.0'
name: test-recipe
resources:
  a:
    type: file
    dest: /tmp/a
    mode: '0644'
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
    let score = score_obs(&input);
    // mode coverage 50% → 7pts, owner coverage 0% → 0pts = 7
    assert_eq!(score, 7);
}

// ── Additional DOC scoring ──────────────────────────────────────

#[test]
fn doc_moderate_comments_10_percent() {
    // 2 comments in 10 lines = 20% → 40pts threshold
    let yaml = "\
# Recipe config
# Managed by forjar
version: '1.0'
name: test-recipe
machines:
  target:
    hostname: target
    addr: localhost
resources:
  f:
    type: file
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
    let score = score_doc(&input);
    // 2/12 lines = 16.7% → 40pts (>= 15%), name has dash: +5 = 45
    assert_eq!(score, 45);
}

#[test]
fn doc_name_without_dash() {
    let mut config = minimal_config();
    config.name = "simple".to_string();
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: "version: '1.0'\nname: simple\n",
        budget_ms: 0,
        runtime: None,
    };
    let score = score_doc(&input);
    // No comments, no header, no description, no dash in name = 0
    assert_eq!(score, 0);
}

// ── Additional RES scoring ──────────────────────────────────────

#[test]
fn res_empty_config_is_zero() {
    let config = minimal_config();
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: "",
        budget_ms: 0,
        runtime: None,
    };
    assert_eq!(score_res(&input), 0);
}

#[test]
fn res_low_dag_ratio() {
    let yaml = "\
version: '1.0'
name: test-recipe
resources:
  a:
    type: file
    dest: /tmp/a
    depends_on: [b]
  b:
    type: file
    dest: /tmp/b
  c:
    type: file
    dest: /tmp/c
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
    // DAG 33% (1/3): +20 = 20
    assert_eq!(score, 20);
}

// ── Additional CMP scoring ──────────────────────────────────────

#[test]
fn cmp_multi_machine() {
    let yaml = "\
version: '1.0'
name: test-recipe
machines:
  web:
    hostname: web
    addr: 10.0.0.1
  db:
    hostname: db
    addr: 10.0.0.2
resources:
  f:
    type: file
    dest: /tmp/f
    template: true
    resource_group: core
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
    // template: +10, resource_group: +15, multi-machine: +10 = 35
    assert_eq!(score, 35);
}

#[test]
fn cmp_with_includes() {
    let yaml = "\
version: '1.0'
name: test-recipe
includes:
  - base.yaml
  - security.yaml
resources: {}
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
    // includes: +10, recipe nesting (>1): +15 = 25
    assert_eq!(score, 25);
}

// ── SAF additional ──────────────────────────────────────────────

#[test]
fn saf_package_without_version_pin() {
    let yaml = "\
version: '1.0'
name: test-recipe
resources:
  pkg:
    type: package
    name: nginx
";
    let config = RecipeConfig::from_yaml(yaml).unwrap_or_else(|_| minimal_config());
    let mut penalties = Vec::new();
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: yaml,
        budget_ms: 0,
        runtime: None,
    };
    let score = score_saf(&input, &mut penalties);
    // 100 - 3 (no version pin) = 97
    assert_eq!(score, 97);
    assert_eq!(penalties.len(), 1);
    assert!(penalties[0].reason.contains("version pin"));
}

#[test]
fn saf_multiple_files_all_penalized() {
    let yaml = "\
version: '1.0'
name: test-recipe
resources:
  a:
    type: file
    dest: /tmp/a
  b:
    type: file
    dest: /tmp/b
";
    let config = RecipeConfig::from_yaml(yaml).unwrap_or_else(|_| minimal_config());
    let mut penalties = Vec::new();
    let input = ScoringInput {
        status: &RecipeStatus::Qualified,
        idempotency_class: &IdempotencyClass::Strong,
        config: &config,
        raw_yaml: yaml,
        budget_ms: 0,
        runtime: None,
    };
    let score = score_saf(&input, &mut penalties);
    // 100 - 5×2 (no mode) - 3×2 (no owner) = 84
    assert_eq!(score, 84);
    assert_eq!(penalties.len(), 4);
}
