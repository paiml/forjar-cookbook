//! Tests for CSV parsing, table generation, and README update.

use super::*;

const SAMPLE_CSV: &str = "\
recipe_num,name,category,status,tier,idempotency_class,first_apply_ms,idempotent_apply_ms,blocker_ticket,blocker_description,last_qualified,qualified_by
1,developer-workstation,infra,qualified,2+3,strong,45000,1200,,,2026-03-01,cookbook-runner
2,web-server,infra,qualified,2+3,strong,62000,1800,,,2026-03-01,cookbook-runner
7,rocm-gpu,gpu,blocked,3,strong,,,FJ-1126,ROCm userspace not installed,,
40,scheduled-tasks,linux,pending,2+3,strong,,,,,,
";

#[test]
fn parse_csv_valid() {
    let recipes = parse_csv(SAMPLE_CSV).unwrap_or_default();
    assert_eq!(recipes.len(), 4);
    assert_eq!(recipes[0].recipe_num, 1);
    assert_eq!(recipes[0].name, "developer-workstation");
    assert_eq!(recipes[0].status, RecipeStatus::Qualified);
    assert_eq!(recipes[0].first_apply_ms, 45000);
}

#[test]
fn parse_csv_blocked_recipe() {
    let recipes = parse_csv(SAMPLE_CSV).unwrap_or_default();
    let rocm = &recipes[2];
    assert_eq!(rocm.recipe_num, 7);
    assert_eq!(rocm.status, RecipeStatus::Blocked);
    assert_eq!(rocm.blocker_ticket, "FJ-1126");
    assert_eq!(rocm.first_apply_ms, 0);
}

#[test]
fn parse_csv_pending_recipe() {
    let recipes = parse_csv(SAMPLE_CSV).unwrap_or_default();
    let cron = &recipes[3];
    assert_eq!(cron.recipe_num, 40);
    assert_eq!(cron.status, RecipeStatus::Pending);
    assert!(cron.last_qualified.is_empty());
}

#[test]
fn parse_csv_empty() {
    let csv = "recipe_num,name,category,status,tier,idempotency_class,\
               first_apply_ms,idempotent_apply_ms,blocker_ticket,\
               blocker_description,last_qualified,qualified_by\n";
    let recipes = parse_csv(csv).unwrap_or_default();
    assert!(recipes.is_empty());
}

#[test]
fn parse_csv_invalid_status() {
    let csv = "recipe_num,name,category,status,tier,idempotency_class,\
               first_apply_ms,idempotent_apply_ms,blocker_ticket,\
               blocker_description,last_qualified,qualified_by\n\
               1,test,infra,invalid,2,strong,,,,,,\n";
    let result = parse_csv(csv);
    assert!(result.is_err());
}

#[test]
fn parse_csv_invalid_recipe_num() {
    let csv = "recipe_num,name,category,status,tier,idempotency_class,\
               first_apply_ms,idempotent_apply_ms,blocker_ticket,\
               blocker_description,last_qualified,qualified_by\n\
               abc,test,infra,pending,2,strong,,,,,,\n";
    let result = parse_csv(csv);
    assert!(result.is_err());
}

#[test]
fn generate_summary_counts() {
    let recipes = parse_csv(SAMPLE_CSV).unwrap_or_default();
    let summary = generate_summary(&recipes, "2026-03-01 12:00 UTC");
    assert!(summary.contains("Qualified | 2"));
    assert!(summary.contains("Blocked   | 1"));
    assert!(summary.contains("Pending   | 1"));
    assert!(summary.contains("2026-03-01 12:00 UTC"));
}

#[test]
fn generate_summary_empty() {
    let summary = generate_summary(&[], "2026-03-01 12:00 UTC");
    assert!(summary.contains("Qualified | 0"));
    assert!(summary.contains("Blocked   | 0"));
    assert!(summary.contains("Pending   | 0"));
}

#[test]
fn generate_table_has_header() {
    let recipes = parse_csv(SAMPLE_CSV).unwrap_or_default();
    let table = generate_table(&recipes);
    assert!(table.contains("| # | Recipe |"));
    assert!(table.contains("|---|"));
}

#[test]
fn generate_table_sorted_by_number() {
    let recipes = parse_csv(SAMPLE_CSV).unwrap_or_default();
    let table = generate_table(&recipes);
    let lines: Vec<&str> = table.lines().collect();
    // Skip header rows (0, 1), data starts at 2
    assert!(lines[2].starts_with("| 1 |"));
    assert!(lines[3].starts_with("| 2 |"));
    assert!(lines[4].starts_with("| 7 |"));
    assert!(lines[5].starts_with("| 40 |"));
}

#[test]
fn generate_table_badges() {
    let recipes = parse_csv(SAMPLE_CSV).unwrap_or_default();
    let table = generate_table(&recipes);
    assert!(table.contains("QUALIFIED-brightgreen"));
    assert!(table.contains("BLOCKED-red"));
    assert!(table.contains("PENDING-lightgray"));
}

#[test]
fn generate_table_timing() {
    let recipes = parse_csv(SAMPLE_CSV).unwrap_or_default();
    let table = generate_table(&recipes);
    assert!(table.contains("45.0s"));
    assert!(table.contains("1.2s"));
    // Blocked/pending have em-dash for timing
    assert!(table.contains('\u{2014}'));
}

#[test]
fn generate_table_blocker() {
    let recipes = parse_csv(SAMPLE_CSV).unwrap_or_default();
    let table = generate_table(&recipes);
    assert!(table.contains("FJ-1126: ROCm userspace not installed"));
}

#[test]
fn format_duration_zero() {
    assert_eq!(format_duration(0), "\u{2014}");
}

#[test]
fn format_duration_millis() {
    assert_eq!(format_duration(500), "500ms");
}

#[test]
fn format_duration_seconds() {
    assert_eq!(format_duration(1200), "1.2s");
    assert_eq!(format_duration(45000), "45.0s");
}

#[test]
fn write_csv_roundtrip() {
    let recipes = parse_csv(SAMPLE_CSV).unwrap_or_default();
    let output = write_csv(&recipes);
    let reparsed = parse_csv(&output).unwrap_or_default();
    assert_eq!(recipes.len(), reparsed.len());
    for (orig, re) in recipes.iter().zip(reparsed.iter()) {
        assert_eq!(orig.recipe_num, re.recipe_num);
        assert_eq!(orig.name, re.name);
        assert_eq!(orig.status, re.status);
    }
}

#[test]
fn write_csv_has_header() {
    let recipes = parse_csv(SAMPLE_CSV).unwrap_or_default();
    let output = write_csv(&recipes);
    assert!(output.starts_with("recipe_num,name,category,status,"));
}

#[test]
fn write_csv_sorted() {
    let recipes = parse_csv(SAMPLE_CSV).unwrap_or_default();
    let output = write_csv(&recipes);
    let lines: Vec<&str> = output.lines().collect();
    assert!(lines[1].starts_with("1,"));
    assert!(lines[2].starts_with("2,"));
    assert!(lines[3].starts_with("7,"));
    assert!(lines[4].starts_with("40,"));
}

#[test]
fn update_readme_success() {
    let readme =
        format!("# Title\n\nSome text.\n\n{START_MARKER}\nold content\n{END_MARKER}\n\nFooter.");
    let result = update_readme(&readme, "new table content");
    assert!(result.is_ok());
    let updated = result.unwrap_or_default();
    assert!(updated.contains("new table content"));
    assert!(!updated.contains("old content"));
    assert!(updated.contains("# Title"));
    assert!(updated.contains("Footer."));
}

#[test]
fn update_readme_missing_start_marker() {
    let readme = format!("# Title\n\n{END_MARKER}\n");
    let result = update_readme(&readme, "content");
    assert!(result.is_err());
    let Err(err) = result else { return };
    assert!(err.contains("start marker"));
}

#[test]
fn update_readme_missing_end_marker() {
    let readme = format!("# Title\n\n{START_MARKER}\n");
    let result = update_readme(&readme, "content");
    assert!(result.is_err());
    let Err(err) = result else { return };
    assert!(err.contains("end marker"));
}

#[test]
fn update_readme_preserves_surrounding_text() {
    let readme = format!("BEFORE\n{START_MARKER}\nstuff\n{END_MARKER}\nAFTER");
    let result = update_readme(&readme, "TABLE").unwrap_or_default();
    assert!(result.starts_with("BEFORE"));
    assert!(result.ends_with("AFTER"));
    assert!(result.contains("TABLE"));
}

#[test]
fn update_readme_reversed_markers() {
    // End marker before start marker
    let readme = format!("{END_MARKER}\nstuff\n{START_MARKER}\n");
    let result = update_readme(&readme, "content");
    assert!(result.is_err());
    let Err(err) = result else { return };
    assert!(err.contains("end marker appears before start marker"));
}

#[test]
fn parse_csv_too_few_fields() {
    // The csv crate with flexible=false (default) will error on mismatched
    // field counts. With flexible=true, our code would catch it. Either way,
    // parse_csv should return an error.
    let csv = "recipe_num,name,category,status,tier,idempotency_class,\
               first_apply_ms,idempotent_apply_ms,blocker_ticket,\
               blocker_description,last_qualified,qualified_by\n\
               1,test,infra\n";
    let result = parse_csv(csv);
    assert!(result.is_err());
}

#[test]
fn generate_table_empty() {
    let table = generate_table(&[]);
    let lines: Vec<&str> = table.lines().collect();
    // Only header and separator
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("# | Recipe"));
}

#[test]
fn write_csv_empty() {
    let output = write_csv(&[]);
    // Just the header line
    assert!(output.starts_with("recipe_num,name,"));
    assert_eq!(output.trim().lines().count(), 1);
}
