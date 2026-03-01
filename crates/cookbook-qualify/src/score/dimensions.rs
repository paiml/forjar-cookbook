//! Per-dimension scoring functions for the 8 Forjar Score dimensions.

use super::{
    IdempotencyClass, MAX_POINTS, Penalty, SAFETY_CRITICAL_CAP, ScoringInput, has_depends_on,
    has_resource_group, has_tags, has_template, resource_str,
};

/// Clamp a signed score to 0–100.
pub(super) fn clamp_score(pts: i32) -> u32 {
    u32::try_from(pts.clamp(0, i32::try_from(MAX_POINTS).unwrap_or(100))).unwrap_or(0)
}

/// COR — Correctness (20%).
pub(super) fn score_cor(input: &ScoringInput<'_>, penalties: &mut Vec<Penalty>) -> u32 {
    let Some(rt) = input.runtime else {
        return 0;
    };
    let mut pts: i32 = 0;
    if rt.validate_pass {
        pts += 20;
    }
    if rt.plan_pass {
        pts += 20;
    }
    if rt.first_apply_pass {
        pts += 40;
    }
    if rt.all_resources_converged {
        pts += 10;
    }
    if rt.state_lock_written {
        pts += 10;
    }

    // Penalty: -2 per warning, max -10
    let warn_penalty = i32::try_from(rt.warning_count.min(5)).unwrap_or(5) * 2;
    if warn_penalty > 0 {
        penalties.push(Penalty {
            dimension: "COR".to_string(),
            points: warn_penalty,
            reason: format!("{} warning(s) during apply", rt.warning_count),
        });
        pts -= warn_penalty;
    }

    clamp_score(pts)
}

/// IDM — Idempotency (20%).
pub(super) fn score_idm(input: &ScoringInput<'_>, penalties: &mut Vec<Penalty>) -> u32 {
    let Some(rt) = input.runtime else {
        return 0;
    };
    let mut pts: i32 = 0;
    if rt.second_apply_pass {
        pts += 30;
    }
    if rt.zero_changes {
        pts += 30;
    }
    if rt.hash_stable {
        pts += 20;
    }

    // Idempotency class bonus
    match input.idempotency_class {
        IdempotencyClass::Strong => pts += 20,
        IdempotencyClass::Weak => pts += 10,
        IdempotencyClass::Eventual => {}
    }

    // Penalty: -10 per changed resource on 2nd apply
    if rt.changed_on_reapply > 0 {
        let deduction = i32::try_from(rt.changed_on_reapply).unwrap_or(10) * 10;
        penalties.push(Penalty {
            dimension: "IDM".to_string(),
            points: deduction,
            reason: format!("{} resource(s) changed on re-apply", rt.changed_on_reapply),
        });
        pts -= deduction;
    }

    clamp_score(pts)
}

/// PRF — Performance (15%).
#[allow(clippy::cast_precision_loss)] // timing values well within f64
pub(super) fn score_prf(input: &ScoringInput<'_>) -> u32 {
    let Some(rt) = input.runtime else {
        return 0;
    };
    let mut pts: i32 = 0;

    // First apply vs budget
    if input.budget_ms > 0 && rt.first_apply_ms > 0 {
        let ratio = rt.first_apply_ms as f64 / input.budget_ms as f64;
        if ratio <= 0.50 {
            pts += 50;
        } else if ratio <= 0.75 {
            pts += 40;
        } else if ratio <= 1.00 {
            pts += 30;
        } else if ratio <= 1.50 {
            pts += 15;
        }
    } else if rt.first_apply_ms > 0 {
        // No budget → assume on-budget
        pts += 30;
    }

    // Idempotent timing
    if rt.idempotent_apply_ms > 0 {
        if rt.idempotent_apply_ms <= 2000 {
            pts += 30;
        } else if rt.idempotent_apply_ms <= 5000 {
            pts += 25;
        } else if rt.idempotent_apply_ms <= 10000 {
            pts += 15;
        }
    }

    // Efficiency ratio
    if rt.first_apply_ms > 0 && rt.idempotent_apply_ms > 0 {
        let ratio = rt.idempotent_apply_ms as f64 / rt.first_apply_ms as f64;
        if ratio <= 0.05 {
            pts += 20;
        } else if ratio <= 0.10 {
            pts += 15;
        } else if ratio <= 0.25 {
            pts += 10;
        }
    }

    clamp_score(pts)
}

/// SAF — Safety (15%). Starts at 100, deductions applied.
pub(super) fn score_saf(input: &ScoringInput<'_>, penalties: &mut Vec<Penalty>) -> u32 {
    let mut pts: i32 = 100;
    let mut has_critical = false;

    let file_resources: Vec<_> = input
        .config
        .resources
        .values()
        .filter(|v| resource_str(v, "type").as_deref() == Some("file"))
        .collect();

    for res in &file_resources {
        let mode = resource_str(res, "mode");
        let owner = resource_str(res, "owner");

        // Critical: mode 0777
        if mode.as_deref() == Some("0777") {
            pts -= 30;
            has_critical = true;
            penalties.push(Penalty {
                dimension: "SAF".to_string(),
                points: 30,
                reason: "file with mode 0777".to_string(),
            });
        }

        // Moderate: no explicit mode
        if mode.is_none() {
            pts -= 5;
            penalties.push(Penalty {
                dimension: "SAF".to_string(),
                points: 5,
                reason: "file without explicit mode".to_string(),
            });
        }

        // Moderate: no explicit owner
        if owner.is_none() {
            pts -= 3;
            penalties.push(Penalty {
                dimension: "SAF".to_string(),
                points: 3,
                reason: "file without explicit owner".to_string(),
            });
        }
    }

    // Check for curl|bash in raw YAML
    if input.raw_yaml.contains("curl") && input.raw_yaml.contains("bash") {
        pts -= 30;
        has_critical = true;
        penalties.push(Penalty {
            dimension: "SAF".to_string(),
            points: 30,
            reason: "curl|bash pattern detected".to_string(),
        });
    }

    // Check for version pins on packages
    let pkg_resources: Vec<_> = input
        .config
        .resources
        .values()
        .filter(|v| resource_str(v, "type").as_deref() == Some("package"))
        .collect();
    for res in &pkg_resources {
        let version = resource_str(res, "version");
        if version.is_none() {
            pts -= 3;
            penalties.push(Penalty {
                dimension: "SAF".to_string(),
                points: 3,
                reason: "package without version pin".to_string(),
            });
        }
    }

    // Hard cap if critical violation
    let score = clamp_score(pts);
    if has_critical {
        score.min(SAFETY_CRITICAL_CAP)
    } else {
        score
    }
}

/// OBS — Observability (10%).
pub(super) fn score_obs(input: &ScoringInput<'_>) -> u32 {
    let mut pts: i32 = 0;

    if input.config.tripwire.is_some() {
        pts += 15;
    }
    if input.config.lock_file.is_some() {
        pts += 15;
    }
    if input.config.outputs.is_some() {
        pts += 10;
    }

    // File mode coverage
    let file_resources: Vec<_> = input
        .config
        .resources
        .values()
        .filter(|v| resource_str(v, "type").as_deref() == Some("file"))
        .collect();

    if !file_resources.is_empty() {
        let with_mode = file_resources
            .iter()
            .filter(|v| resource_str(v, "mode").is_some())
            .count();
        #[allow(clippy::cast_precision_loss)]
        let ratio = with_mode as f64 / file_resources.len() as f64;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let mode_pts = (ratio * 15.0) as i32;
        pts += mode_pts;

        // Owner coverage
        let with_owner = file_resources
            .iter()
            .filter(|v| resource_str(v, "owner").is_some())
            .count();
        #[allow(clippy::cast_precision_loss)]
        let owner_ratio = with_owner as f64 / file_resources.len() as f64;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let owner_pts = (owner_ratio * 15.0) as i32;
        pts += owner_pts;
    }

    // Notify hooks
    if let Some(ref notify) = input.config.notify {
        if let Some(map) = notify.as_mapping() {
            let hook_count = ["success", "failure", "drift"]
                .iter()
                .filter(|k| {
                    map.get(serde_yaml_ng::Value::String((**k).to_owned()))
                        .is_some()
                })
                .count();
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            let notify_pts = ((hook_count * 20) / 3) as i32;
            pts += notify_pts;
        }
    }

    clamp_score(pts)
}

/// DOC — Documentation (8%).
pub(super) fn score_doc(input: &ScoringInput<'_>) -> u32 {
    let mut pts: i32 = 0;

    // Comment ratio
    let total_lines = input.raw_yaml.lines().count();
    if total_lines > 0 {
        let comment_lines = input
            .raw_yaml
            .lines()
            .filter(|l| l.trim_start().starts_with('#'))
            .count();
        #[allow(clippy::cast_precision_loss)]
        let ratio = comment_lines as f64 / total_lines as f64;
        if ratio >= 0.15 {
            pts += 40;
        } else if ratio >= 0.10 {
            pts += 30;
        } else if ratio >= 0.05 {
            pts += 20;
        }
    }

    // Header metadata: recipe#, tier, idempotency
    let first_lines: String = input
        .raw_yaml
        .lines()
        .take(5)
        .collect::<Vec<_>>()
        .join("\n");
    if first_lines.contains("Recipe #") || first_lines.contains("Recipe:") {
        pts += 10;
    }
    if first_lines.contains("Tier:") || first_lines.contains("Tier ") {
        pts += 10;
    }
    if first_lines.contains("Idempotency:") || first_lines.contains("idempotency") {
        pts += 10;
    }

    // Description field
    if input.config.description.is_some() {
        pts += 15;
    }

    // Descriptive name (not empty, not single word)
    if !input.config.name.is_empty() && input.config.name.contains('-') {
        pts += 5;
    }

    clamp_score(pts)
}

/// RES — Resilience (7%).
pub(super) fn score_res(input: &ScoringInput<'_>) -> u32 {
    let mut pts: i32 = 0;

    // Failure policy
    if input.config.failure.as_deref() == Some("continue_independent") {
        pts += 20;
    }

    // SSH retries
    if input.config.ssh_retries.is_some_and(|r| r > 1) {
        pts += 10;
    }

    // Dependency DAG ratio
    let total = input.config.resources.len();
    if total > 0 {
        let with_deps = input
            .config
            .resources
            .values()
            .filter(|v| has_depends_on(v))
            .count();
        #[allow(clippy::cast_precision_loss)]
        let ratio = with_deps as f64 / total as f64;
        if ratio >= 0.50 {
            pts += 30;
        } else if ratio >= 0.30 {
            pts += 20;
        }
    }

    // Lifecycle hooks
    if input.config.pre_apply.is_some() {
        pts += 10;
    }
    if input.config.post_apply.is_some() {
        pts += 10;
    }

    clamp_score(pts)
}

/// CMP — Composability (5%).
pub(super) fn score_cmp(input: &ScoringInput<'_>) -> u32 {
    let mut pts: i32 = 0;

    // Params
    if input.config.params.is_some() {
        pts += 20;
    }

    // Templates
    let has_templates = input.config.resources.values().any(has_template);
    if has_templates {
        pts += 10;
    }

    // Includes
    if input
        .config
        .includes
        .as_ref()
        .is_some_and(|inc| !inc.is_empty())
    {
        pts += 10;
    }

    // Tags
    let has_any_tags = input.config.resources.values().any(has_tags);
    if has_any_tags {
        pts += 15;
    }

    // Resource groups
    let has_groups = input.config.resources.values().any(has_resource_group);
    if has_groups {
        pts += 15;
    }

    // Multi-machine
    if input.config.machines.len() > 1 {
        pts += 10;
    }

    // Recipe nesting (includes count > 0 already handled, but nested = > 1)
    if input
        .config
        .includes
        .as_ref()
        .is_some_and(|inc| inc.len() > 1)
    {
        pts += 15;
    }

    clamp_score(pts)
}
