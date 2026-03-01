//! CSV parsing, table generation, and README update logic.

use crate::qualify::{IdempotencyClass, RecipeQualification, RecipeStatus};

/// Start marker for qualification table in README.
pub const START_MARKER: &str = "<!-- QUALIFICATION_TABLE_START -->";

/// End marker for qualification table in README.
pub const END_MARKER: &str = "<!-- QUALIFICATION_TABLE_END -->";

/// Parse recipes from CSV content.
///
/// # Errors
///
/// Returns error if CSV is malformed or contains invalid field values.
pub fn parse_csv(content: &str) -> Result<Vec<RecipeQualification>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(content.as_bytes());

    let mut recipes = Vec::new();
    for (line_num, result) in reader.records().enumerate() {
        let record = result.map_err(|e| format!("CSV line {}: {e}", line_num + 2))?;
        if record.len() < 12 {
            return Err(format!(
                "CSV line {}: expected 12 fields, got {}",
                line_num + 2,
                record.len()
            ));
        }
        let recipe_num: u32 = record[0]
            .parse()
            .map_err(|e| format!("CSV line {}: recipe_num: {e}", line_num + 2))?;
        let status = RecipeStatus::from_csv(&record[3])
            .map_err(|e| format!("CSV line {}: {e}", line_num + 2))?;
        let idempotency_class = IdempotencyClass::from_csv(&record[5])
            .map_err(|e| format!("CSV line {}: {e}", line_num + 2))?;
        let first_apply_ms: u64 = if record[6].is_empty() {
            0
        } else {
            record[6]
                .parse()
                .map_err(|e| format!("CSV line {}: first_apply_ms: {e}", line_num + 2))?
        };
        let idempotent_apply_ms: u64 = if record[7].is_empty() {
            0
        } else {
            record[7]
                .parse()
                .map_err(|e| format!("CSV line {}: idempotent_apply_ms: {e}", line_num + 2))?
        };
        recipes.push(RecipeQualification {
            recipe_num,
            name: record[1].to_string(),
            category: record[2].to_string(),
            status,
            tier: record[4].to_string(),
            idempotency_class,
            first_apply_ms,
            idempotent_apply_ms,
            blocker_ticket: record[8].to_string(),
            blocker_description: record[9].to_string(),
            last_qualified: record[10].to_string(),
            qualified_by: record[11].to_string(),
        });
    }
    Ok(recipes)
}

/// Generate summary counts block.
#[must_use]
pub fn generate_summary(recipes: &[RecipeQualification], timestamp: &str) -> String {
    let qualified = recipes
        .iter()
        .filter(|r| r.status == RecipeStatus::Qualified)
        .count();
    let blocked = recipes
        .iter()
        .filter(|r| r.status == RecipeStatus::Blocked)
        .count();
    let pending = recipes
        .iter()
        .filter(|r| r.status == RecipeStatus::Pending)
        .count();

    format!(
        "**Qualification Summary** (updated: {timestamp})\n\
         \n\
         | Status | Count |\n\
         |--------|-------|\n\
         | Qualified | {qualified} |\n\
         | Blocked   | {blocked} |\n\
         | Pending   | {pending} |"
    )
}

/// Format milliseconds as human-readable duration.
fn format_duration(ms: u64) -> String {
    if ms == 0 {
        return "\u{2014}".to_string(); // em-dash
    }
    if ms < 1000 {
        return format!("{ms}ms");
    }
    #[allow(clippy::cast_precision_loss)] // timing values are well within f64 precision
    let secs = ms as f64 / 1000.0;
    format!("{secs:.1}s")
}

/// Generate the full qualification table (sorted by recipe number).
#[must_use]
pub fn generate_table(recipes: &[RecipeQualification]) -> String {
    let mut sorted: Vec<&RecipeQualification> = recipes.iter().collect();
    sorted.sort_by_key(|r| r.recipe_num);

    let mut lines = Vec::new();
    lines.push(
        "| # | Recipe | Category | Status | Tier | Idempotent | \
         Time (1st) | Time (2nd) | Blocker |"
            .to_string(),
    );
    lines.push(
        "|---|--------|----------|--------|------|------------|\
         ------------|------------|---------|"
            .to_string(),
    );

    for r in &sorted {
        let blocker = if r.blocker_ticket.is_empty() {
            "\u{2014}".to_string()
        } else {
            format!("{}: {}", r.blocker_ticket, r.blocker_description)
        };
        lines.push(format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            r.recipe_num,
            r.name,
            r.category,
            r.status.badge(),
            r.tier,
            r.idempotency_class
                .as_str()
                .chars()
                .next()
                .map_or(String::new(), |c| c.to_uppercase().to_string()
                    + &r.idempotency_class.as_str()[1..]),
            format_duration(r.first_apply_ms),
            format_duration(r.idempotent_apply_ms),
            blocker,
        ));
    }

    lines.join("\n")
}

/// Serialize recipes back to CSV.
#[must_use]
pub fn write_csv(recipes: &[RecipeQualification]) -> String {
    let mut lines = Vec::new();
    lines.push(
        "recipe_num,name,category,status,tier,idempotency_class,\
         first_apply_ms,idempotent_apply_ms,blocker_ticket,\
         blocker_description,last_qualified,qualified_by"
            .to_string(),
    );

    let mut sorted: Vec<&RecipeQualification> = recipes.iter().collect();
    sorted.sort_by_key(|r| r.recipe_num);

    for r in &sorted {
        let first = if r.first_apply_ms == 0 {
            String::new()
        } else {
            r.first_apply_ms.to_string()
        };
        let idem = if r.idempotent_apply_ms == 0 {
            String::new()
        } else {
            r.idempotent_apply_ms.to_string()
        };
        lines.push(format!(
            "{},{},{},{},{},{},{},{},{},{},{},{}",
            r.recipe_num,
            r.name,
            r.category,
            r.status.as_str(),
            r.tier,
            r.idempotency_class.as_str(),
            first,
            idem,
            r.blocker_ticket,
            r.blocker_description,
            r.last_qualified,
            r.qualified_by,
        ));
    }

    lines.join("\n") + "\n"
}

/// Replace content between markers in README.
///
/// # Errors
///
/// Returns error if markers are not found in the readme content.
pub fn update_readme(readme: &str, table_content: &str) -> Result<String, String> {
    let start_idx = readme
        .find(START_MARKER)
        .ok_or_else(|| format!("missing start marker: {START_MARKER}"))?;
    let end_idx = readme
        .find(END_MARKER)
        .ok_or_else(|| format!("missing end marker: {END_MARKER}"))?;

    if end_idx <= start_idx {
        return Err("end marker appears before start marker".to_string());
    }

    let before = &readme[..start_idx + START_MARKER.len()];
    let after = &readme[end_idx..];
    Ok(format!("{before}\n{table_content}\n{after}"))
}

#[cfg(test)]
mod tests;
