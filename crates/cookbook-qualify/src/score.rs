//! `ForjarScore` v2 — thin bridge to forjar's scoring engine.
//!
//! Delegates all scoring computation to forjar's v2 two-tier grading system.
//! Keeps cookbook-specific presentation types (`Grade`, `DimensionScores`) for
//! backward compatibility with CSV, table, and report formatting.

use crate::qualify::{IdempotencyClass, RecipeStatus};
use serde::{Deserialize, Serialize};

/// Scoring algorithm version — delegated to forjar.
pub use forjar::core::scoring::SCORE_VERSION;

// ── Types ────────────────────────────────────────────────────────

/// Per-dimension scores (each 0–100).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DimensionScores {
    /// Correctness.
    pub cor: u32,
    /// Idempotency.
    pub idm: u32,
    /// Performance.
    pub prf: u32,
    /// Safety.
    pub saf: u32,
    /// Observability.
    pub obs: u32,
    /// Documentation.
    pub doc: u32,
    /// Resilience.
    pub res: u32,
    /// Composability.
    pub cmp: u32,
}

impl DimensionScores {
    /// Minimum score across all dimensions.
    #[must_use]
    pub fn min_score(&self) -> u32 {
        [
            self.cor, self.idm, self.prf, self.saf, self.obs, self.doc, self.res, self.cmp,
        ]
        .into_iter()
        .min()
        .unwrap_or(0)
    }

    /// Build from forjar's dimension score vector.
    fn from_forjar(dims: &[forjar::core::scoring::DimensionScore]) -> Self {
        let get = |code: &str| dims.iter().find(|d| d.code == code).map_or(0, |d| d.score);
        Self {
            cor: get("COR"),
            idm: get("IDM"),
            prf: get("PRF"),
            saf: get("SAF"),
            obs: get("OBS"),
            doc: get("DOC"),
            res: get("RES"),
            cmp: get("CMP"),
        }
    }
}

/// Letter grade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Grade {
    /// Production-hardened.
    A,
    /// Solid, minor gaps.
    B,
    /// Functional but rough.
    C,
    /// Bare minimum.
    D,
    /// Failed or never qualified.
    F,
}

impl Grade {
    /// Shields.io badge markdown.
    #[must_use]
    pub const fn badge(&self) -> &'static str {
        match self {
            Self::A => "![A](https://img.shields.io/badge/A-brightgreen)",
            Self::B => "![B](https://img.shields.io/badge/B-blue)",
            Self::C => "![C](https://img.shields.io/badge/C-yellow)",
            Self::D => "![D](https://img.shields.io/badge/D-orange)",
            Self::F => "![F](https://img.shields.io/badge/F-red)",
        }
    }

    /// Short string for CSV.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::F => "F",
        }
    }

    /// Parse from CSV string.
    ///
    /// # Errors
    ///
    /// Returns error if the value is not a valid grade.
    pub fn from_csv(s: &str) -> Result<Self, String> {
        match s.trim().to_uppercase().as_str() {
            "A" => Ok(Self::A),
            "B" => Ok(Self::B),
            "C" => Ok(Self::C),
            "D" => Ok(Self::D),
            "F" | "" => Ok(Self::F),
            other => Err(format!("unknown grade: {other}")),
        }
    }

    /// Convert from forjar's `char` grade.
    const fn from_char(c: char) -> Self {
        match c {
            'A' => Self::A,
            'B' => Self::B,
            'C' => Self::C,
            'D' => Self::D,
            _ => Self::F,
        }
    }
}

/// A scoring penalty applied to a dimension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Penalty {
    /// Which dimension this penalty affects.
    pub dimension: String,
    /// Points deducted (positive = deduction).
    pub points: i32,
    /// Human-readable reason.
    pub reason: String,
}

/// Complete Forjar Score for a recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForjarScore {
    /// Composite score (0–100).
    pub composite: u32,
    /// Overall letter grade — `min(static, runtime)`.
    pub grade: Grade,
    /// Static grade (design quality).
    pub static_grade: Grade,
    /// Runtime grade (operational quality), or `None` if not yet qualified.
    pub runtime_grade: Option<Grade>,
    /// Per-dimension breakdown.
    pub dimensions: DimensionScores,
    /// Penalties applied during scoring.
    pub penalties: Vec<Penalty>,
    /// Scoring algorithm version.
    pub version: String,
}

// ── Runtime data from qualification ──────────────────────────────

/// Runtime data collected during qualification (from `cookbook-runner`).
#[derive(Debug, Clone, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct RuntimeData {
    /// Whether `forjar validate` passed.
    pub validate_pass: bool,
    /// Whether `forjar plan` passed.
    pub plan_pass: bool,
    /// Whether first `forjar apply` passed.
    pub first_apply_pass: bool,
    /// Whether second apply passed.
    pub second_apply_pass: bool,
    /// Whether second apply had zero changes.
    pub zero_changes: bool,
    /// Whether state hashes are stable across applies.
    pub hash_stable: bool,
    /// Number of resources that changed on second apply.
    pub changed_on_reapply: u32,
    /// Number of warnings during apply.
    pub warning_count: u32,
    /// First apply time in milliseconds.
    pub first_apply_ms: u64,
    /// Idempotent apply time in milliseconds.
    pub idempotent_apply_ms: u64,
    /// Whether state lock file was written.
    pub state_lock_written: bool,
    /// Whether all resources converged.
    pub all_resources_converged: bool,
}

impl RuntimeData {
    /// Convert to forjar's `RuntimeData` type.
    const fn to_forjar(&self) -> forjar::core::scoring::RuntimeData {
        forjar::core::scoring::RuntimeData {
            validate_pass: self.validate_pass,
            plan_pass: self.plan_pass,
            first_apply_pass: self.first_apply_pass,
            second_apply_pass: self.second_apply_pass,
            zero_changes_on_reapply: self.zero_changes,
            hash_stable: self.hash_stable,
            all_resources_converged: self.all_resources_converged,
            state_lock_written: self.state_lock_written,
            warning_count: self.warning_count,
            changed_on_reapply: self.changed_on_reapply,
            first_apply_ms: self.first_apply_ms,
            second_apply_ms: self.idempotent_apply_ms,
        }
    }
}

// ── Scoring input ────────────────────────────────────────────────

/// Combined input for scoring: recipe status, raw YAML text,
/// performance budget, idempotency class, and optional runtime data.
pub struct ScoringInput<'a> {
    /// Recipe qualification status.
    pub status: &'a RecipeStatus,
    /// Idempotency class from CSV.
    pub idempotency_class: &'a IdempotencyClass,
    /// Raw YAML text (parsed internally by forjar).
    pub raw_yaml: &'a str,
    /// Performance budget for first apply (ms). 0 = no budget.
    pub budget_ms: u64,
    /// Runtime data (None for blocked/pending recipes — static-only scoring).
    pub runtime: Option<&'a RuntimeData>,
}

// ── Scoring logic — delegates to forjar v2 ───────────────────────

impl ForjarScore {
    /// Compute the Forjar Score by delegating to forjar's v2 scoring engine.
    #[must_use]
    pub fn compute(input: &ScoringInput<'_>) -> Self {
        // Parse raw YAML into ForjarConfig
        let config: forjar::core::types::ForjarConfig =
            match serde_yaml_ng::from_str(input.raw_yaml) {
                Ok(c) => c,
                Err(_) => {
                    return Self {
                        composite: 0,
                        grade: Grade::F,
                        static_grade: Grade::F,
                        runtime_grade: None,
                        dimensions: DimensionScores {
                            cor: 0,
                            idm: 0,
                            prf: 0,
                            saf: 0,
                            obs: 0,
                            doc: 0,
                            res: 0,
                            cmp: 0,
                        },
                        penalties: vec![Penalty {
                            dimension: "ALL".into(),
                            points: 100,
                            reason: "YAML parse failure".into(),
                        }],
                        version: SCORE_VERSION.to_string(),
                    };
                }
            };

        // Map RuntimeData to forjar's type
        let fj_runtime = input.runtime.map(RuntimeData::to_forjar);

        // Build forjar's ScoringInput
        let fj_input = forjar::core::scoring::ScoringInput {
            status: input.status.as_str().to_string(),
            idempotency: input.idempotency_class.as_str().to_string(),
            budget_ms: input.budget_ms,
            runtime: fj_runtime,
            raw_yaml: Some(input.raw_yaml.to_string()),
        };

        // Compute via forjar v2
        let result = forjar::core::scoring::compute(&config, &fj_input);

        // Determine overall grade: min(static, runtime) or static-only
        let grade_char = match result.runtime_grade {
            Some(rg) => grade_min_char(result.static_grade, rg),
            None => result.static_grade,
        };

        Self {
            composite: result.composite,
            grade: Grade::from_char(grade_char),
            static_grade: Grade::from_char(result.static_grade),
            runtime_grade: result.runtime_grade.map(Grade::from_char),
            dimensions: DimensionScores::from_forjar(&result.dimensions),
            penalties: Vec::new(),
            version: SCORE_VERSION.to_string(),
        }
    }
}

/// Minimum of two grade chars.
fn grade_min_char(a: char, b: char) -> char {
    let ord = |g: char| match g {
        'A' => 4,
        'B' => 3,
        'C' => 2,
        'D' => 1,
        _ => 0,
    };
    match ord(a).min(ord(b)) {
        4 => 'A',
        3 => 'B',
        2 => 'C',
        1 => 'D',
        _ => 'F',
    }
}

#[cfg(test)]
mod tests;
