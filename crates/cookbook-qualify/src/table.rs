//! CSV parsing, table generation, and README update logic.

use crate::qualify::{IdempotencyClass, RecipeQualification, RecipeStatus};
use crate::score::Grade;

/// Start marker for qualification table in README.
pub const START_MARKER: &str = "<!-- QUALIFICATION_TABLE_START -->";

/// End marker for qualification table in README.
pub const END_MARKER: &str = "<!-- QUALIFICATION_TABLE_END -->";

/// Minimum number of CSV fields (original 12-column schema).
const MIN_FIELDS: usize = 12;

/// Parse an optional u32 field from CSV (empty → 0).
fn parse_optional_u32(
    record: &csv::StringRecord,
    idx: usize,
    line_num: usize,
) -> Result<u32, String> {
    if idx >= record.len() || record[idx].is_empty() {
        return Ok(0);
    }
    record[idx]
        .parse()
        .map_err(|e| format!("CSV line {}: field {idx}: {e}", line_num + 2))
}

/// Parse an optional u64 field from CSV (empty → 0).
fn parse_optional_u64(
    record: &csv::StringRecord,
    idx: usize,
    line_num: usize,
) -> Result<u64, String> {
    if idx >= record.len() || record[idx].is_empty() {
        return Ok(0);
    }
    record[idx]
        .parse()
        .map_err(|e| format!("CSV line {}: field {idx}: {e}", line_num + 2))
}

/// Parse an optional string field from CSV (missing → empty).
fn parse_optional_str(record: &csv::StringRecord, idx: usize) -> String {
    if idx >= record.len() {
        return String::new();
    }
    record[idx].to_string()
}

/// Parse recipes from CSV content.
///
/// Backward compatible: accepts both 12-column (original) and 23-column
/// (extended with Forjar Score fields) CSV formats. Missing fields default
/// to 0 or empty string.
///
/// # Errors
///
/// Returns error if CSV is malformed or contains invalid field values.
pub fn parse_csv(content: &str) -> Result<Vec<RecipeQualification>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(content.as_bytes());

    let mut recipes = Vec::new();
    for (line_num, result) in reader.records().enumerate() {
        let record = result.map_err(|e| format!("CSV line {}: {e}", line_num + 2))?;
        if record.len() < MIN_FIELDS {
            return Err(format!(
                "CSV line {}: expected at least {MIN_FIELDS} fields, got {}",
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
        let first_apply_ms = parse_optional_u64(&record, 6, line_num)?;
        let idempotent_apply_ms = parse_optional_u64(&record, 7, line_num)?;

        // Extended fields (columns 12–22), default to 0/empty if missing
        let score = parse_optional_u32(&record, 12, line_num)?;
        let grade = parse_optional_str(&record, 13);
        let cor = parse_optional_u32(&record, 14, line_num)?;
        let idm = parse_optional_u32(&record, 15, line_num)?;
        let prf = parse_optional_u32(&record, 16, line_num)?;
        let saf = parse_optional_u32(&record, 17, line_num)?;
        let obs = parse_optional_u32(&record, 18, line_num)?;
        let doc = parse_optional_u32(&record, 19, line_num)?;
        let res = parse_optional_u32(&record, 20, line_num)?;
        let cmp = parse_optional_u32(&record, 21, line_num)?;
        let score_version = parse_optional_str(&record, 22);

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
            score,
            grade,
            cor,
            idm,
            prf,
            saf,
            obs,
            doc,
            res,
            cmp,
            score_version,
        });
    }
    Ok(recipes)
}

/// Grade badge for a grade string (A/B/C/D/F).
fn grade_badge(grade_str: &str) -> &'static str {
    Grade::from_csv(grade_str).map_or("", |g| g.badge())
}

/// Generate summary counts block with grade distribution.
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

    // Grade distribution
    let grade_a = recipes.iter().filter(|r| r.grade == "A").count();
    let grade_b = recipes.iter().filter(|r| r.grade == "B").count();
    let grade_c = recipes.iter().filter(|r| r.grade == "C").count();
    let grade_d = recipes.iter().filter(|r| r.grade == "D").count();
    let grade_f = recipes
        .iter()
        .filter(|r| r.grade == "F" || r.grade.is_empty())
        .count();

    format!(
        "**Qualification Summary** (updated: {timestamp})\n\
         \n\
         | Status | Count |\n\
         |--------|-------|\n\
         | Qualified | {qualified} |\n\
         | Blocked   | {blocked} |\n\
         | Pending   | {pending} |\n\
         \n\
         **Grade Distribution**\n\
         \n\
         | Grade | Count |\n\
         |-------|-------|\n\
         | A | {grade_a} |\n\
         | B | {grade_b} |\n\
         | C | {grade_c} |\n\
         | D | {grade_d} |\n\
         | F | {grade_f} |"
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
        "| # | Recipe | Category | Status | Grade | Tier | Idempotent | \
         Time (1st) | Time (2nd) | Score | Blocker |"
            .to_string(),
    );
    lines.push(
        "|---|--------|----------|--------|-------|------|------------|\
         ------------|------------|-------|---------|"
            .to_string(),
    );

    for r in &sorted {
        let blocker = if r.blocker_ticket.is_empty() {
            "\u{2014}".to_string()
        } else {
            format!("{}: {}", r.blocker_ticket, r.blocker_description)
        };
        let grade_col = if r.grade.is_empty() {
            "\u{2014}".to_string()
        } else {
            grade_badge(&r.grade).to_string()
        };
        let score_col = if r.score == 0 && r.grade.is_empty() {
            "\u{2014}".to_string()
        } else {
            r.score.to_string()
        };
        lines.push(format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            r.recipe_num,
            r.name,
            r.category,
            r.status.badge(),
            grade_col,
            r.tier,
            r.idempotency_class
                .as_str()
                .chars()
                .next()
                .map_or(String::new(), |c| c.to_uppercase().to_string()
                    + &r.idempotency_class.as_str()[1..]),
            format_duration(r.first_apply_ms),
            format_duration(r.idempotent_apply_ms),
            score_col,
            blocker,
        ));
    }

    lines.join("\n")
}

/// Format a u32 as empty string if zero, otherwise its decimal representation.
fn fmt_optional_u32(val: u32) -> String {
    if val == 0 {
        String::new()
    } else {
        val.to_string()
    }
}

/// CSV header for the 23-column format.
const CSV_HEADER: &str = "recipe_num,name,category,status,tier,idempotency_class,\
first_apply_ms,idempotent_apply_ms,blocker_ticket,\
blocker_description,last_qualified,qualified_by,\
score,grade,cor,idm,prf,saf,obs,doc,res,cmp,score_version";

/// Serialize recipes back to CSV (23-column format).
#[must_use]
pub fn write_csv(recipes: &[RecipeQualification]) -> String {
    let mut lines = Vec::new();
    lines.push(CSV_HEADER.to_string());

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
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
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
            fmt_optional_u32(r.score),
            r.grade,
            fmt_optional_u32(r.cor),
            fmt_optional_u32(r.idm),
            fmt_optional_u32(r.prf),
            fmt_optional_u32(r.saf),
            fmt_optional_u32(r.obs),
            fmt_optional_u32(r.doc),
            fmt_optional_u32(r.res),
            fmt_optional_u32(r.cmp),
            r.score_version,
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
