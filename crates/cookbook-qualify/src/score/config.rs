//! Lightweight recipe config parser for static analysis scoring.
//!
//! Parses forjar recipe YAML without depending on the forjar crate.
//! Supports both nested `policy:` sections and top-level policy fields.

use serde::Deserialize;

/// Policy section (nested under `policy:` in YAML).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PolicyConfig {
    /// Failure policy.
    #[serde(default)]
    pub failure: Option<String>,
    /// Tripwire policy.
    #[serde(default)]
    pub tripwire: Option<serde_yaml_ng::Value>,
    /// Lock file config.
    #[serde(default)]
    pub lock_file: Option<serde_yaml_ng::Value>,
    /// Pre-apply hook.
    #[serde(default)]
    pub pre_apply: Option<String>,
    /// Post-apply hook.
    #[serde(default)]
    pub post_apply: Option<String>,
    /// Notify hooks.
    #[serde(default)]
    pub notify: Option<serde_yaml_ng::Value>,
    /// SSH retries.
    #[serde(default)]
    pub ssh_retries: Option<u32>,
}

/// Parsed recipe configuration for scoring analysis.
#[derive(Debug, Clone, Deserialize)]
pub struct RecipeConfig {
    /// Recipe name.
    #[serde(default)]
    pub name: String,
    /// Recipe version.
    #[serde(default)]
    pub version: String,
    /// Description field.
    #[serde(default)]
    pub description: Option<String>,
    /// Machines map.
    #[serde(default)]
    pub machines: std::collections::HashMap<String, serde_yaml_ng::Value>,
    /// Resources map.
    #[serde(default)]
    pub resources: std::collections::HashMap<String, serde_yaml_ng::Value>,
    /// Params with defaults.
    #[serde(default)]
    pub params: Option<serde_yaml_ng::Value>,
    /// Outputs section.
    #[serde(default)]
    pub outputs: Option<serde_yaml_ng::Value>,
    /// Includes.
    #[serde(default)]
    pub includes: Option<Vec<String>>,
    /// Policy section (nested).
    #[serde(default)]
    pub policy: Option<PolicyConfig>,
    /// Failure policy (top-level fallback).
    #[serde(default)]
    pub failure: Option<String>,
    /// Tripwire (top-level fallback).
    #[serde(default)]
    pub tripwire: Option<serde_yaml_ng::Value>,
    /// Lock file (top-level fallback).
    #[serde(default)]
    pub lock_file: Option<String>,
    /// Pre-apply hook (top-level fallback).
    #[serde(default)]
    pub pre_apply: Option<String>,
    /// Post-apply hook (top-level fallback).
    #[serde(default)]
    pub post_apply: Option<String>,
    /// Notify hooks (top-level fallback).
    #[serde(default)]
    pub notify: Option<serde_yaml_ng::Value>,
    /// SSH retries (top-level fallback).
    #[serde(default)]
    pub ssh_retries: Option<u32>,
}

impl RecipeConfig {
    /// Parse from YAML string.
    ///
    /// # Errors
    ///
    /// Returns error if YAML is invalid.
    pub fn from_yaml(yaml: &str) -> Result<Self, String> {
        serde_yaml_ng::from_str(yaml).map_err(|e| format!("YAML parse error: {e}"))
    }

    /// Effective failure policy (policy section takes precedence).
    #[must_use]
    pub fn eff_failure(&self) -> Option<&str> {
        self.policy
            .as_ref()
            .and_then(|p| p.failure.as_deref())
            .or(self.failure.as_deref())
    }

    /// Effective tripwire setting.
    #[must_use]
    pub fn eff_tripwire(&self) -> Option<&serde_yaml_ng::Value> {
        self.policy
            .as_ref()
            .and_then(|p| p.tripwire.as_ref())
            .or(self.tripwire.as_ref())
    }

    /// Effective lock file setting.
    #[must_use]
    pub fn eff_lock_file(&self) -> bool {
        self.policy
            .as_ref()
            .and_then(|p| p.lock_file.as_ref())
            .is_some()
            || self.lock_file.is_some()
    }

    /// Effective pre-apply hook.
    #[must_use]
    pub fn eff_pre_apply(&self) -> Option<&str> {
        self.policy
            .as_ref()
            .and_then(|p| p.pre_apply.as_deref())
            .or(self.pre_apply.as_deref())
    }

    /// Effective post-apply hook.
    #[must_use]
    pub fn eff_post_apply(&self) -> Option<&str> {
        self.policy
            .as_ref()
            .and_then(|p| p.post_apply.as_deref())
            .or(self.post_apply.as_deref())
    }

    /// Effective notify hooks.
    #[must_use]
    pub fn eff_notify(&self) -> Option<&serde_yaml_ng::Value> {
        self.policy
            .as_ref()
            .and_then(|p| p.notify.as_ref())
            .or(self.notify.as_ref())
    }

    /// Effective SSH retries.
    #[must_use]
    pub fn eff_ssh_retries(&self) -> Option<u32> {
        self.policy
            .as_ref()
            .and_then(|p| p.ssh_retries)
            .or(self.ssh_retries)
    }
}

// ── Static analysis helpers ──────────────────────────────────────

/// Extract resource field as string from a YAML value.
pub(super) fn resource_str(val: &serde_yaml_ng::Value, key: &str) -> Option<String> {
    val.as_mapping()
        .and_then(|m| m.get(serde_yaml_ng::Value::String(key.to_string())))
        .and_then(serde_yaml_ng::Value::as_str)
        .map(String::from)
}

/// Check if a resource has a `depends_on` field.
pub(super) fn has_depends_on(val: &serde_yaml_ng::Value) -> bool {
    val.as_mapping()
        .and_then(|m| m.get(serde_yaml_ng::Value::String("depends_on".to_string())))
        .is_some()
}

/// Check if a resource has tags.
pub(super) fn has_tags(val: &serde_yaml_ng::Value) -> bool {
    val.as_mapping()
        .and_then(|m| m.get(serde_yaml_ng::Value::String("tags".to_string())))
        .is_some()
}

/// Check if a resource uses templates (either a `template` field or `{{` in values).
pub(super) fn has_template(val: &serde_yaml_ng::Value) -> bool {
    let Some(mapping) = val.as_mapping() else {
        return false;
    };
    // Explicit template field
    if mapping
        .get(serde_yaml_ng::Value::String("template".to_string()))
        .is_some()
    {
        return true;
    }
    // Detect {{...}} interpolation in any string value
    mapping.values().any(|v| {
        v.as_str()
            .is_some_and(|s| s.contains("{{") && s.contains("}}"))
    })
}

/// Check if a resource has a `resource_group` field.
pub(super) fn has_resource_group(val: &serde_yaml_ng::Value) -> bool {
    val.as_mapping()
        .and_then(|m| m.get(serde_yaml_ng::Value::String("resource_group".to_string())))
        .is_some()
}
