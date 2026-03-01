//! README sync logic — extracted from main.rs for testability.

use crate::{generate_summary, generate_table, parse_csv, update_readme};
use std::path::Path;

/// Sync the README qualification table from a CSV file.
///
/// Reads the CSV, generates the summary and table, and splices into the README
/// between the HTML comment markers.
///
/// # Errors
///
/// Returns error if CSV parsing fails, README is missing markers, or I/O fails.
pub fn sync_readme(
    csv_path: &Path,
    readme_path: &Path,
    timestamp: &str,
) -> Result<SyncResult, String> {
    let csv_content = std::fs::read_to_string(csv_path)
        .map_err(|e| format!("read {}: {e}", csv_path.display()))?;
    let recipes = parse_csv(&csv_content)?;

    let summary = generate_summary(&recipes, timestamp);
    let table = generate_table(&recipes);
    let full_content = format!("{summary}\n\n{table}");

    let readme = std::fs::read_to_string(readme_path)
        .map_err(|e| format!("read {}: {e}", readme_path.display()))?;
    let updated = update_readme(&readme, &full_content)?;

    std::fs::write(readme_path, &updated)
        .map_err(|e| format!("write {}: {e}", readme_path.display()))?;

    let total = recipes.len();
    let qualified = recipes
        .iter()
        .filter(|r| r.status == crate::RecipeStatus::Qualified)
        .count();

    Ok(SyncResult { total, qualified })
}

/// Result of a README sync operation.
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// Total number of recipes in the CSV.
    pub total: usize,
    /// Number of qualified recipes.
    pub qualified: usize,
}

#[cfg(test)]
mod tests;
