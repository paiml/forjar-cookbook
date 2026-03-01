//! Additional SAF and OBS dimension scoring tests.

use crate::qualify::{IdempotencyClass, RecipeStatus};
use crate::score::dimensions::{score_obs, score_saf};
use crate::score::{RecipeConfig, ScoringInput};

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

// ── OBS scoring with full policy ────────────────────────────────

#[test]
fn obs_full_policy_with_notify() {
    let yaml = "\
version: '1.0'
name: test-recipe
resources:
  f:
    type: file
    dest: /tmp/x
    mode: '0644'
    owner: root
policy:
  tripwire: true
  lock_file: true
  notify:
    on_success: echo ok
    on_failure: echo fail
    on_drift: echo drift
outputs:
  test:
    value: ok
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
    // tripwire: +15, lock_file: +15, outputs: +10, mode 100%: +15, owner 100%: +15, notify 3/3: +20 = 90
    assert_eq!(score, 90);
}

#[test]
fn obs_full_policy_without_notify() {
    let yaml = "\
version: '1.0'
name: test-recipe
resources:
  f:
    type: file
    dest: /tmp/x
    mode: '0644'
    owner: root
policy:
  tripwire: true
  lock_file: true
outputs:
  test:
    value: ok
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
    // tripwire: +15, lock_file: +15, outputs: +10, mode 100%: +15, owner 100%: +15 = 70
    assert_eq!(score, 70);
}

#[test]
fn obs_no_file_resources_full_mode_owner_credit() {
    let yaml = "\
version: '1.0'
name: test-recipe
resources:
  gpu:
    type: gpu
    backend: rocm
policy:
  tripwire: true
  lock_file: true
  notify:
    on_success: echo ok
    on_failure: echo fail
    on_drift: echo drift
outputs:
  backend:
    value: rocm
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
    // tripwire(15) + lock(15) + outputs(10) + no-files bonus(30) + notify(20) = 90
    assert_eq!(score, 90);
}
