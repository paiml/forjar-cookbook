//! Forjar Score — multi-dimensional recipe quality scoring.
//!
//! Computes a composite score (0–100) across 8 dimensions and assigns
//! a letter grade (A–F). Designed so A-grade requires >= 80 in every
//! dimension — no gaming by overperforming in easy dimensions.

mod config;
mod dimensions;

use crate::qualify::{IdempotencyClass, RecipeStatus};
pub use config::{PolicyConfig, RecipeConfig};
use dimensions::{
    score_cmp, score_cor, score_doc, score_idm, score_obs, score_prf, score_res, score_saf,
};
use serde::{Deserialize, Serialize};

/// Current scoring algorithm version.
pub const SCORE_VERSION: &str = "1.0";

/// Maximum points per dimension.
const MAX_POINTS: u32 = 100;

/// Safety hard cap when critical violation exists.
pub(crate) const SAFETY_CRITICAL_CAP: u32 = 40;

// ── Dimension weights (must sum to 100) ──────────────────────────

const W_COR: u32 = 20;
const W_IDM: u32 = 20;
const W_PRF: u32 = 15;
const W_SAF: u32 = 15;
const W_OBS: u32 = 10;
const W_DOC: u32 = 8;
const W_RES: u32 = 7;
const W_CMP: u32 = 5;

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
    /// Letter grade.
    pub grade: Grade,
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

// ── Scoring input ────────────────────────────────────────────────

/// Combined input for scoring: recipe status, static config, raw YAML text,
/// performance budget, idempotency class, and optional runtime data.
pub struct ScoringInput<'a> {
    /// Recipe qualification status.
    pub status: &'a RecipeStatus,
    /// Idempotency class from CSV.
    pub idempotency_class: &'a IdempotencyClass,
    /// Parsed recipe config (for static analysis).
    pub config: &'a RecipeConfig,
    /// Raw YAML text (for comment ratio).
    pub raw_yaml: &'a str,
    /// Performance budget for first apply (ms). 0 = no budget.
    pub budget_ms: u64,
    /// Runtime data (None for blocked/pending recipes — static-only scoring).
    pub runtime: Option<&'a RuntimeData>,
}

// ── Scoring logic ────────────────────────────────────────────────

impl ForjarScore {
    /// Compute the Forjar Score for a recipe.
    #[must_use]
    pub fn compute(input: &ScoringInput<'_>) -> Self {
        let mut penalties = Vec::new();

        // Hard-fail: blocked/pending → automatic F
        if *input.status == RecipeStatus::Blocked || *input.status == RecipeStatus::Pending {
            return Self {
                composite: 0,
                grade: Grade::F,
                dimensions: DimensionScores {
                    cor: 0,
                    idm: 0,
                    prf: 0,
                    saf: score_saf(input, &mut penalties),
                    obs: score_obs(input),
                    doc: score_doc(input),
                    res: score_res(input),
                    cmp: score_cmp(input),
                },
                penalties,
                version: SCORE_VERSION.to_string(),
            };
        }

        let cor = score_cor(input, &mut penalties);
        let idm = score_idm(input, &mut penalties);
        let prf = score_prf(input);
        let saf = score_saf(input, &mut penalties);
        let obs = score_obs(input);
        let doc = score_doc(input);
        let res = score_res(input);
        let cmp = score_cmp(input);

        // Hard-fail: runtime validation/plan/apply failure
        if let Some(rt) = input.runtime {
            if !rt.validate_pass || !rt.plan_pass || !rt.first_apply_pass {
                return Self {
                    composite: 0,
                    grade: Grade::F,
                    dimensions: DimensionScores {
                        cor,
                        idm,
                        prf,
                        saf,
                        obs,
                        doc,
                        res,
                        cmp,
                    },
                    penalties,
                    version: SCORE_VERSION.to_string(),
                };
            }
        }

        let dimensions = DimensionScores {
            cor,
            idm,
            prf,
            saf,
            obs,
            doc,
            res,
            cmp,
        };

        let composite = compute_composite(&dimensions);
        let grade = compute_grade(composite, dimensions.min_score());

        Self {
            composite,
            grade,
            dimensions,
            penalties,
            version: SCORE_VERSION.to_string(),
        }
    }
}

/// Weighted composite score.
fn compute_composite(d: &DimensionScores) -> u32 {
    let weighted = u64::from(d.cor) * u64::from(W_COR)
        + u64::from(d.idm) * u64::from(W_IDM)
        + u64::from(d.prf) * u64::from(W_PRF)
        + u64::from(d.saf) * u64::from(W_SAF)
        + u64::from(d.obs) * u64::from(W_OBS)
        + u64::from(d.doc) * u64::from(W_DOC)
        + u64::from(d.res) * u64::from(W_RES)
        + u64::from(d.cmp) * u64::from(W_CMP);
    // Weights sum to 100, so divide by 100 to get 0–100 range.
    #[allow(clippy::cast_possible_truncation)]
    let result = (weighted / 100) as u32;
    result.min(MAX_POINTS)
}

/// Grade from composite + min dimension.
const fn compute_grade(composite: u32, min_dim: u32) -> Grade {
    if composite >= 90 && min_dim >= 80 {
        Grade::A
    } else if composite >= 75 && min_dim >= 60 {
        Grade::B
    } else if composite >= 60 && min_dim >= 40 {
        Grade::C
    } else if composite >= 40 {
        Grade::D
    } else {
        Grade::F
    }
}

#[cfg(test)]
mod tests;
