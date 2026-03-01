//! Workspace-level integration tests for forjar-cookbook.
//!
//! Tests cross-crate interactions between cookbook-qualify, cookbook-runner,
//! and cookbook-examples.

#![allow(clippy::expect_used)]

use cookbook_qualify::{
    ForjarScore, Grade, IdempotencyClass, RecipeConfig, RecipeStatus, SCORE_VERSION, ScoringInput,
};
use cookbook_runner::{
    QualifyResult, QualifyVerdict, RecipeRunner, RunOutcome, format_qualify_report,
    format_score_report, format_validate_report, runtime_data_from_qualify, verdict,
};

// ---------- CSV round-trip ----------

#[test]
fn csv_round_trip_preserves_data() {
    let csv_input = "\
recipe_num,name,category,status,tier,idempotency_class,\
first_apply_ms,idempotent_apply_ms,blocker_ticket,\
blocker_description,last_qualified,qualified_by,\
score,grade,cor,idm,prf,saf,obs,doc,res,cmp,score_version
1,developer-workstation,infra,qualified,2,strong,45000,1200,,,2026-03-01,cookbook-runner,88,B,90,85,80,95,70,60,75,50,1
2,web-server,infra,pending,2,strong,,,,,,,,,,,,,,,,
7,rocm-gpu,gpu,blocked,3,strong,,,FJ-1126,ROCm not installed,,,,,,,,,,,,
";

    let recipes = cookbook_qualify::parse_csv(csv_input).expect("parse CSV");
    assert_eq!(recipes.len(), 3);
    assert_eq!(recipes[0].recipe_num, 1);
    assert_eq!(recipes[0].name, "developer-workstation");
    assert_eq!(recipes[0].status, RecipeStatus::Qualified);
    assert_eq!(recipes[0].score, 88);
    assert_eq!(recipes[0].grade, "B");
    assert_eq!(recipes[0].cor, 90);
    assert_eq!(recipes[0].score_version, "1");

    // Write back and re-parse
    let csv_output = cookbook_qualify::write_csv(&recipes);
    let reparsed = cookbook_qualify::parse_csv(&csv_output).expect("re-parse CSV");
    assert_eq!(reparsed.len(), 3);
    assert_eq!(reparsed[0].recipe_num, recipes[0].recipe_num);
    assert_eq!(reparsed[0].score, recipes[0].score);
    assert_eq!(reparsed[0].grade, recipes[0].grade);
    assert_eq!(reparsed[2].blocker_ticket, "FJ-1126");
}

// ---------- README update cycle ----------

#[test]
fn readme_update_cycle() {
    let csv_input = "\
recipe_num,name,category,status,tier,idempotency_class,\
first_apply_ms,idempotent_apply_ms,blocker_ticket,\
blocker_description,last_qualified,qualified_by
1,test-recipe,infra,qualified,2,strong,45000,1200,,,2026-03-01,runner
";

    let recipes = cookbook_qualify::parse_csv(csv_input).expect("parse CSV");
    let summary = cookbook_qualify::generate_summary(&recipes, "2026-03-01 12:00 UTC");
    let table = cookbook_qualify::generate_table(&recipes);

    assert!(summary.contains("Qualified | 1"));
    assert!(summary.contains("Pending   | 0"));
    assert!(table.contains("test-recipe"));
    assert!(table.contains("infra"));

    // Insert into a README template
    let readme = "# Forjar Cookbook\n\n\
         <!-- QUALIFICATION_TABLE_START -->\n\
         old content\n\
         <!-- QUALIFICATION_TABLE_END -->\n\n\
         ## Footer"
        .to_string();
    let table_content = format!("{summary}\n\n{table}");
    let updated = cookbook_qualify::update_readme(&readme, &table_content).expect("update README");
    assert!(updated.contains("test-recipe"));
    assert!(updated.contains("Qualified | 1"));
    assert!(updated.contains("## Footer"));
    assert!(!updated.contains("old content"));
}

// ---------- Scoring with runtime data ----------

#[test]
fn scoring_with_runtime_data_from_qualify() {
    // Build a successful QualifyResult
    let result = QualifyResult {
        validate: RunOutcome {
            exit_code: 0,
            output: String::new(),
            duration_ms: 10,
        },
        plan: Some(RunOutcome {
            exit_code: 0,
            output: String::new(),
            duration_ms: 20,
        }),
        first_apply: Some(RunOutcome {
            exit_code: 0,
            output: "3 changed".to_string(),
            duration_ms: 45_000,
        }),
        idempotent_apply: Some(RunOutcome {
            exit_code: 0,
            output: "0 changed".to_string(),
            duration_ms: 1200,
        }),
        idempotent: true,
    };

    let rt = runtime_data_from_qualify(&result);
    assert!(rt.validate_pass);
    assert!(rt.plan_pass);
    assert!(rt.first_apply_pass);
    assert!(rt.second_apply_pass);
    assert!(rt.zero_changes);
    assert_eq!(rt.first_apply_ms, 45_000);
    assert_eq!(rt.idempotent_apply_ms, 1200);

    // Use runtime data to compute a score
    let yaml = "\
version: '1.0'
name: test-recipe
description: integration test recipe
resources:
  install-curl:
    type: package
    name: curl
";
    let config = RecipeConfig::from_yaml(yaml).expect("parse config");
    let status = RecipeStatus::Qualified;
    let idem = IdempotencyClass::Strong;
    let input = ScoringInput {
        status: &status,
        idempotency_class: &idem,
        config: &config,
        raw_yaml: yaml,
        budget_ms: 0,
        runtime: Some(&rt),
    };
    let score = ForjarScore::compute(&input);
    assert!(score.composite > 0);
    assert_ne!(score.grade, Grade::F);
}

// ---------- Verdict ↔ report integration ----------

#[test]
fn verdict_and_report_qualified() {
    let result = QualifyResult {
        validate: RunOutcome {
            exit_code: 0,
            output: String::new(),
            duration_ms: 5,
        },
        plan: Some(RunOutcome {
            exit_code: 0,
            output: String::new(),
            duration_ms: 10,
        }),
        first_apply: Some(RunOutcome {
            exit_code: 0,
            output: "1 changed".to_string(),
            duration_ms: 100,
        }),
        idempotent_apply: Some(RunOutcome {
            exit_code: 0,
            output: "0 changed".to_string(),
            duration_ms: 50,
        }),
        idempotent: true,
    };

    let v = verdict(&result);
    assert_eq!(v, QualifyVerdict::Qualified);
    assert!(v.is_qualified());
    assert!(v.error_message().is_none());

    let report = format_qualify_report(std::path::Path::new("test.yaml"), &result);
    assert!(report.contains("QUALIFIED"));
    assert!(report.contains("test.yaml"));
}

#[test]
fn verdict_and_report_apply_failed() {
    let result = QualifyResult {
        validate: RunOutcome {
            exit_code: 0,
            output: String::new(),
            duration_ms: 5,
        },
        plan: Some(RunOutcome {
            exit_code: 0,
            output: String::new(),
            duration_ms: 10,
        }),
        first_apply: Some(RunOutcome {
            exit_code: 1,
            output: "error".to_string(),
            duration_ms: 100,
        }),
        idempotent_apply: None,
        idempotent: false,
    };

    let v = verdict(&result);
    assert_eq!(v, QualifyVerdict::ApplyFailed);
    assert!(!v.is_qualified());
    assert_eq!(v.error_message(), Some("first apply failed"));
}

// ---------- Score report formatting ----------

#[test]
fn score_report_formatting() {
    let yaml = "\
version: '1.0'
name: format-test
description: format test recipe
resources:
  install-vim:
    type: package
    name: vim
";
    let config = RecipeConfig::from_yaml(yaml).expect("parse config");
    let status = RecipeStatus::Pending;
    let idem = IdempotencyClass::Strong;
    let input = ScoringInput {
        status: &status,
        idempotency_class: &idem,
        config: &config,
        raw_yaml: yaml,
        budget_ms: 0,
        runtime: None,
    };
    let score = ForjarScore::compute(&input);
    let report = format_score_report(&score);
    assert!(report.contains("score:"));
    assert!(report.contains("grade"));
    assert!(report.contains("COR="));
    assert!(report.contains("IDM="));
    assert!(report.contains("PRF="));
    assert!(report.contains("SAF="));
}

// ---------- Validate report formatting ----------

#[test]
fn validate_report_ok_and_fail() {
    let ok = format_validate_report(std::path::Path::new("recipe.yaml"), 0, 42);
    assert!(ok.starts_with("OK:"));
    assert!(ok.contains("42ms"));

    let fail = format_validate_report(std::path::Path::new("bad.yaml"), 1, 10);
    assert!(fail.starts_with("FAIL:"));
    assert!(fail.contains("exit 1"));
}

// ---------- Runner with echo binary ----------

#[test]
fn runner_echo_validates_recipe() {
    let runner = RecipeRunner::new("/bin/echo");
    let outcome = runner.validate(std::path::Path::new("integration-test.yaml"));
    assert_eq!(outcome.exit_code, 0);
    assert!(outcome.output.contains("validate"));
    assert!(outcome.output.contains("integration-test.yaml"));
}

// ---------- CSV parsing edge cases ----------

#[test]
fn csv_parse_original_12_column_format() {
    let csv = "\
recipe_num,name,category,status,tier,idempotency_class,\
first_apply_ms,idempotent_apply_ms,blocker_ticket,\
blocker_description,last_qualified,qualified_by
5,test,infra,pending,1,weak,,,,,,
";
    let recipes = cookbook_qualify::parse_csv(csv).expect("parse");
    assert_eq!(recipes.len(), 1);
    assert_eq!(recipes[0].recipe_num, 5);
    assert_eq!(recipes[0].idempotency_class, IdempotencyClass::Weak);
    // Extended fields default to 0/empty
    assert_eq!(recipes[0].score, 0);
    assert!(recipes[0].grade.is_empty());
}

#[test]
fn csv_parse_invalid_status_returns_error() {
    let csv = "\
recipe_num,name,category,status,tier,idempotency_class,\
first_apply_ms,idempotent_apply_ms,blocker_ticket,\
blocker_description,last_qualified,qualified_by
1,test,infra,invalid_status,1,strong,,,,,,
";
    let result = cookbook_qualify::parse_csv(csv);
    assert!(result.is_err());
}

// ---------- Score version constant ----------

#[test]
fn score_version_is_set() {
    assert!(!SCORE_VERSION.is_empty());
}

// ---------- Grade from composite ----------

#[test]
fn grade_boundaries() {
    // The Grade enum should have from_csv parsing
    assert_eq!(Grade::from_csv("A"), Ok(Grade::A));
    assert_eq!(Grade::from_csv("B"), Ok(Grade::B));
    assert_eq!(Grade::from_csv("C"), Ok(Grade::C));
    assert_eq!(Grade::from_csv("D"), Ok(Grade::D));
    assert_eq!(Grade::from_csv("F"), Ok(Grade::F));
    assert!(Grade::from_csv("X").is_err());
}
