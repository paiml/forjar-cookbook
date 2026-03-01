//! Recipe execution and qualification logic.

use std::path::Path;
use std::process::Command;
use std::time::Instant;

/// Outcome of a single forjar command execution.
#[derive(Debug, Clone)]
pub struct RunOutcome {
    /// Exit code (0 = success).
    pub exit_code: i32,
    /// Combined stdout + stderr.
    pub output: String,
    /// Wall-clock duration in milliseconds.
    pub duration_ms: u64,
}

/// Full qualification result for a recipe.
#[derive(Debug, Clone)]
pub struct QualifyResult {
    /// Validate step outcome.
    pub validate: RunOutcome,
    /// Plan step outcome (None if validate failed).
    pub plan: Option<RunOutcome>,
    /// First apply outcome (None if plan failed).
    pub first_apply: Option<RunOutcome>,
    /// Second apply outcome for idempotency (None if first apply failed).
    pub idempotent_apply: Option<RunOutcome>,
    /// Whether the second apply produced zero changes.
    pub idempotent: bool,
}

/// Recipe execution harness.
pub struct RecipeRunner {
    /// Path to the forjar binary.
    forjar_bin: String,
}

impl RecipeRunner {
    /// Create a new runner with the given forjar binary path.
    #[must_use]
    pub fn new(forjar_bin: &str) -> Self {
        Self {
            forjar_bin: forjar_bin.to_string(),
        }
    }

    /// Find forjar binary on PATH.
    #[must_use]
    pub fn from_path() -> Self {
        Self {
            forjar_bin: "forjar".to_string(),
        }
    }

    /// Run a forjar command and capture the outcome.
    fn run_forjar(&self, args: &[&str]) -> RunOutcome {
        let start = Instant::now();
        let result = Command::new(&self.forjar_bin).args(args).output();

        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                RunOutcome {
                    exit_code: output.status.code().unwrap_or(-1),
                    output: format!("{stdout}{stderr}"),
                    duration_ms,
                }
            }
            Err(e) => RunOutcome {
                exit_code: -1,
                output: format!("failed to execute forjar: {e}"),
                duration_ms,
            },
        }
    }

    /// Run validate on a recipe config.
    #[must_use]
    pub fn validate(&self, config: &Path) -> RunOutcome {
        self.run_forjar(&["validate", "-f", &config.display().to_string()])
    }

    /// Run plan on a recipe config.
    #[must_use]
    pub fn plan(&self, config: &Path, state_dir: &Path) -> RunOutcome {
        self.run_forjar(&[
            "plan",
            "-f",
            &config.display().to_string(),
            "--state-dir",
            &state_dir.display().to_string(),
        ])
    }

    /// Run apply on a recipe config.
    #[must_use]
    pub fn apply(&self, config: &Path, state_dir: &Path) -> RunOutcome {
        self.run_forjar(&[
            "apply",
            "-f",
            &config.display().to_string(),
            "--state-dir",
            &state_dir.display().to_string(),
            "--yes",
        ])
    }

    /// Full qualification cycle: validate → plan → apply → idempotency check.
    #[must_use]
    pub fn qualify(&self, config: &Path, state_dir: &Path) -> QualifyResult {
        let validate = self.validate(config);
        if validate.exit_code != 0 {
            return QualifyResult {
                validate,
                plan: None,
                first_apply: None,
                idempotent_apply: None,
                idempotent: false,
            };
        }

        let plan = self.plan(config, state_dir);
        if plan.exit_code != 0 {
            return QualifyResult {
                validate,
                plan: Some(plan),
                first_apply: None,
                idempotent_apply: None,
                idempotent: false,
            };
        }

        let first_apply = self.apply(config, state_dir);
        if first_apply.exit_code != 0 {
            return QualifyResult {
                validate,
                plan: Some(plan),
                first_apply: Some(first_apply),
                idempotent_apply: None,
                idempotent: false,
            };
        }

        let idempotent_apply = self.apply(config, state_dir);
        let idempotent = idempotent_apply.exit_code == 0
            && (idempotent_apply.output.contains("0 changed")
                || idempotent_apply.output.contains("0 converged"));

        QualifyResult {
            validate,
            plan: Some(plan),
            first_apply: Some(first_apply),
            idempotent_apply: Some(idempotent_apply),
            idempotent,
        }
    }
}

#[cfg(test)]
mod tests;
