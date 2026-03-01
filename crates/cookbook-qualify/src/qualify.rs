//! Recipe qualification types.

use serde::{Deserialize, Serialize};

/// Recipe qualification status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecipeStatus {
    /// Recipe passes all qualification checks.
    Qualified,
    /// Recipe is blocked by a forjar bug or missing feature.
    Blocked,
    /// Recipe has not been qualified yet.
    Pending,
}

impl RecipeStatus {
    /// Parse from CSV string value.
    ///
    /// # Errors
    ///
    /// Returns error if the value is not a valid status.
    pub fn from_csv(s: &str) -> Result<Self, String> {
        match s.trim().to_lowercase().as_str() {
            "qualified" => Ok(Self::Qualified),
            "blocked" => Ok(Self::Blocked),
            "pending" => Ok(Self::Pending),
            other => Err(format!("unknown status: {other}")),
        }
    }

    /// Badge markdown for shields.io.
    #[must_use]
    pub const fn badge(&self) -> &'static str {
        match self {
            Self::Qualified => "![qualified](https://img.shields.io/badge/QUALIFIED-brightgreen)",
            Self::Blocked => "![blocked](https://img.shields.io/badge/BLOCKED-red)",
            Self::Pending => "![pending](https://img.shields.io/badge/PENDING-lightgray)",
        }
    }

    /// Short string for CSV.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Qualified => "qualified",
            Self::Blocked => "blocked",
            Self::Pending => "pending",
        }
    }
}

/// Idempotency classification for a recipe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IdempotencyClass {
    /// Same inputs always produce identical state hash.
    Strong,
    /// Zero changes on re-apply but hash may vary.
    Weak,
    /// May require multiple applies to converge.
    Eventual,
}

impl IdempotencyClass {
    /// Parse from CSV string value.
    ///
    /// # Errors
    ///
    /// Returns error if the value is not a valid class.
    pub fn from_csv(s: &str) -> Result<Self, String> {
        match s.trim().to_lowercase().as_str() {
            "strong" => Ok(Self::Strong),
            "weak" => Ok(Self::Weak),
            "eventual" => Ok(Self::Eventual),
            other => Err(format!("unknown idempotency class: {other}")),
        }
    }

    /// Short string for CSV.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Strong => "strong",
            Self::Weak => "weak",
            Self::Eventual => "eventual",
        }
    }
}

/// A single recipe's qualification record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeQualification {
    /// Recipe number (1-indexed).
    pub recipe_num: u32,
    /// Recipe short name (kebab-case).
    pub name: String,
    /// Category (infra, gpu, nix, rust, package, ops, linux, opentofu).
    pub category: String,
    /// Current status.
    pub status: RecipeStatus,
    /// Testability tier (e.g., "1+2", "2+3", "3").
    pub tier: String,
    /// Idempotency classification.
    pub idempotency_class: IdempotencyClass,
    /// First-apply time in milliseconds (0 = not measured).
    pub first_apply_ms: u64,
    /// Idempotent-apply time in milliseconds (0 = not measured).
    pub idempotent_apply_ms: u64,
    /// Blocker ticket (e.g., "FJ-1126", empty if none).
    pub blocker_ticket: String,
    /// Blocker description (empty if none).
    pub blocker_description: String,
    /// Date last qualified (ISO 8601, empty if never).
    pub last_qualified: String,
    /// Qualified by (agent name, empty if never).
    pub qualified_by: String,
}

#[cfg(test)]
mod tests;
