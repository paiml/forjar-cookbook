//! Qualification report formatting and verdict logic.

use crate::QualifyResult;
use cookbook_qualify::ForjarScore;
use std::path::Path;

/// Verdict from a full qualification cycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QualifyVerdict {
    /// All steps passed and idempotency confirmed.
    Qualified,
    /// Validation step failed.
    ValidationFailed,
    /// Plan step failed.
    PlanFailed,
    /// First apply step failed.
    ApplyFailed,
    /// Second apply was not idempotent.
    IdempotencyFailed,
}

impl QualifyVerdict {
    /// Whether this verdict represents a successful qualification.
    #[must_use]
    pub const fn is_qualified(&self) -> bool {
        matches!(self, Self::Qualified)
    }

    /// Human-readable error message, if not qualified.
    #[must_use]
    pub const fn error_message(&self) -> Option<&'static str> {
        match self {
            Self::Qualified => None,
            Self::ValidationFailed => Some("validation failed"),
            Self::PlanFailed => Some("plan failed"),
            Self::ApplyFailed => Some("first apply failed"),
            Self::IdempotencyFailed => Some("idempotency check failed"),
        }
    }
}

/// Determine the verdict from a qualification result.
#[must_use]
pub fn verdict(result: &QualifyResult) -> QualifyVerdict {
    if result.validate.exit_code != 0 {
        return QualifyVerdict::ValidationFailed;
    }
    if result.plan.as_ref().is_some_and(|p| p.exit_code != 0) {
        return QualifyVerdict::PlanFailed;
    }
    if result
        .first_apply
        .as_ref()
        .is_some_and(|a| a.exit_code != 0)
    {
        return QualifyVerdict::ApplyFailed;
    }
    if !result.idempotent {
        return QualifyVerdict::IdempotencyFailed;
    }
    QualifyVerdict::Qualified
}

/// Format a validate-only report line.
#[must_use]
pub fn format_validate_report(file: &Path, exit_code: i32, duration_ms: u64) -> String {
    if exit_code == 0 {
        format!("OK: {} ({}ms)", file.display(), duration_ms)
    } else {
        format!("FAIL: {} (exit {})", file.display(), exit_code)
    }
}

/// Format a full qualification report.
#[must_use]
pub fn format_qualify_report(file: &Path, result: &QualifyResult) -> String {
    let mut lines = Vec::new();
    lines.push(format!("Qualifying: {}", file.display()));
    lines.push(format!(
        "  validate: exit={} ({}ms)",
        result.validate.exit_code, result.validate.duration_ms
    ));

    if let Some(ref plan) = result.plan {
        lines.push(format!(
            "  plan:     exit={} ({}ms)",
            plan.exit_code, plan.duration_ms
        ));
    }
    if let Some(ref apply) = result.first_apply {
        lines.push(format!(
            "  apply:    exit={} ({}ms)",
            apply.exit_code, apply.duration_ms
        ));
    }
    if let Some(ref idem) = result.idempotent_apply {
        lines.push(format!(
            "  idempotent: exit={} ({}ms) zero_changes={}",
            idem.exit_code, idem.duration_ms, result.idempotent
        ));
    }

    let v = verdict(result);
    if v.is_qualified() {
        lines.push(format!("QUALIFIED: {}", file.display()));
    } else if let Some(msg) = v.error_message() {
        lines.push(format!("FAILED: {msg}"));
    }

    lines.join("\n")
}

/// Format a Forjar Score breakdown for inclusion in reports.
#[must_use]
pub fn format_score_report(score: &ForjarScore) -> String {
    let d = &score.dimensions;
    let mut lines = Vec::new();
    lines.push(format!(
        "  score: {} (grade {})",
        score.composite,
        score.grade.as_str()
    ));
    lines.push(format!(
        "    COR={:>3}  IDM={:>3}  PRF={:>3}  SAF={:>3}",
        d.cor, d.idm, d.prf, d.saf
    ));
    lines.push(format!(
        "    OBS={:>3}  DOC={:>3}  RES={:>3}  CMP={:>3}",
        d.obs, d.doc, d.res, d.cmp
    ));
    if !score.penalties.is_empty() {
        lines.push(format!("    penalties: {}", score.penalties.len()));
        for p in &score.penalties {
            lines.push(format!("      -{} {}: {}", p.points, p.dimension, p.reason));
        }
    }
    lines.join("\n")
}

/// Build `RuntimeData` from a `QualifyResult` for scoring integration.
///
/// Maps qualification cycle outcomes to the scoring runtime data structure.
#[must_use]
pub fn runtime_data_from_qualify(result: &QualifyResult) -> cookbook_qualify::RuntimeData {
    let validate_pass = result.validate.exit_code == 0;
    let plan_pass = result.plan.as_ref().is_some_and(|p| p.exit_code == 0);
    let first_apply_pass = result
        .first_apply
        .as_ref()
        .is_some_and(|a| a.exit_code == 0);
    let second_apply_pass = result
        .idempotent_apply
        .as_ref()
        .is_some_and(|a| a.exit_code == 0);

    let first_apply_ms = result.first_apply.as_ref().map_or(0, |a| a.duration_ms);
    let idempotent_apply_ms = result
        .idempotent_apply
        .as_ref()
        .map_or(0, |a| a.duration_ms);

    cookbook_qualify::RuntimeData {
        validate_pass,
        plan_pass,
        first_apply_pass,
        second_apply_pass,
        zero_changes: result.idempotent,
        hash_stable: result.idempotent,
        changed_on_reapply: 0, // TODO: parse from output
        warning_count: 0,      // TODO: parse from output
        first_apply_ms,
        idempotent_apply_ms,
        state_lock_written: first_apply_pass,
        all_resources_converged: first_apply_pass,
    }
}

#[cfg(test)]
mod tests;
