//! Score all recipes and update the CSV with dimension scores.
//!
//! Usage: `cargo run --example score_all`
//! Reads each recipe YAML, computes `ForjarScore` using existing runtime
//! data from the CSV (if the recipe was previously qualified), and updates
//! docs/certifications/recipes.csv with the results.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use cookbook_qualify::{
    ForjarScore, RecipeQualification, RecipeStatus, RuntimeData, SCORE_VERSION, ScoringInput,
};

fn main() -> ExitCode {
    let root = match cookbook_examples::find_project_root() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    let csv_path = root.join("docs/certifications/recipes.csv");
    let csv_content = match std::fs::read_to_string(&csv_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error reading CSV: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut recipes = match cookbook_qualify::parse_csv(&csv_content) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error parsing CSV: {e}");
            return ExitCode::FAILURE;
        }
    };

    let recipes_dir = root.join("recipes");
    let (scored, errors) = score_recipes(&mut recipes, &recipes_dir);

    // Write updated CSV
    let csv_output = cookbook_qualify::write_csv(&recipes);
    if let Err(e) = std::fs::write(&csv_path, &csv_output) {
        eprintln!("error writing CSV: {e}");
        return ExitCode::FAILURE;
    }

    eprintln!(
        "\nScored {scored} recipes ({errors} errors). CSV updated at {}",
        csv_path.display()
    );

    if errors > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// Score each recipe and update the qualification entries in place.
fn score_recipes(recipes: &mut [RecipeQualification], recipes_dir: &Path) -> (u32, u32) {
    let mut scored = 0u32;
    let mut errors = 0u32;

    for recipe in recipes.iter_mut() {
        let pattern = format!("{:02}-", recipe.recipe_num);
        let Some(file) = find_recipe_file(recipes_dir, &pattern) else {
            eprintln!("  SKIP: recipe {} — file not found", recipe.recipe_num);
            continue;
        };

        let Ok(raw_yaml) = std::fs::read_to_string(&file) else {
            eprintln!("  ERROR: recipe {} — read failed", recipe.recipe_num);
            errors += 1;
            continue;
        };

        // Reconstruct RuntimeData from CSV for previously-qualified recipes.
        let runtime = runtime_from_csv(recipe);
        let budget_ms = parse_budget_from_yaml(&raw_yaml);
        let input = ScoringInput {
            status: &recipe.status,
            idempotency_class: &recipe.idempotency_class,
            raw_yaml: &raw_yaml,
            budget_ms,
            runtime: runtime.as_ref(),
        };

        let score = ForjarScore::compute(&input);
        apply_score(recipe, &score);

        eprintln!(
            "  {:>2} {:<35} score={:>3} grade={} COR={:>3} IDM={:>3} PRF={:>3} SAF={:>3} OBS={:>3} DOC={:>3} RES={:>3} CMP={:>3}",
            recipe.recipe_num,
            recipe.name,
            score.composite,
            score.grade.as_str(),
            score.dimensions.cor,
            score.dimensions.idm,
            score.dimensions.prf,
            score.dimensions.saf,
            score.dimensions.obs,
            score.dimensions.doc,
            score.dimensions.res,
            score.dimensions.cmp,
        );
        scored += 1;
    }

    (scored, errors)
}

/// Apply computed score to a recipe qualification entry.
fn apply_score(recipe: &mut RecipeQualification, score: &ForjarScore) {
    recipe.score = score.composite;
    recipe.grade = score.grade.as_str().to_string();
    recipe.cor = score.dimensions.cor;
    recipe.idm = score.dimensions.idm;
    recipe.prf = score.dimensions.prf;
    recipe.saf = score.dimensions.saf;
    recipe.obs = score.dimensions.obs;
    recipe.doc = score.dimensions.doc;
    recipe.res = score.dimensions.res;
    recipe.cmp = score.dimensions.cmp;
    recipe.score_version = SCORE_VERSION.to_string();
}

/// Reconstruct `RuntimeData` from a previously-qualified recipe's CSV data.
/// Returns `None` for blocked/pending recipes that haven't been qualified.
fn runtime_from_csv(recipe: &RecipeQualification) -> Option<RuntimeData> {
    if recipe.status != RecipeStatus::Qualified || recipe.first_apply_ms == 0 {
        return None;
    }
    Some(RuntimeData {
        validate_pass: true,
        plan_pass: true,
        first_apply_pass: true,
        second_apply_pass: true,
        zero_changes: true,
        hash_stable: true,
        changed_on_reapply: 0,
        warning_count: 0,
        first_apply_ms: recipe.first_apply_ms,
        idempotent_apply_ms: recipe.idempotent_apply_ms,
        state_lock_written: true,
        all_resources_converged: true,
    })
}

/// Parse budget from YAML header comment like `# Budget: first_apply < 60s`.
/// Returns budget in milliseconds, or 0 if not found.
fn parse_budget_from_yaml(yaml: &str) -> u64 {
    for line in yaml.lines().take(10) {
        let trimmed = line.trim();
        if trimmed.starts_with("# Budget:") {
            // Extract "first_apply < 60s" — find the number right after "<"
            if let Some(idx) = trimmed.find("first_apply") {
                let rest = &trimmed[idx..];
                if let Some(lt_idx) = rest.find('<') {
                    // Get text after "<", trim, take first token
                    let after_lt = rest[lt_idx + 1..].trim();
                    let token = after_lt.split([',', ' ']).next().unwrap_or("");
                    if let Some(num_str) = token.strip_suffix('s') {
                        if let Ok(secs) = num_str.parse::<u64>() {
                            return secs * 1000;
                        }
                    }
                }
            }
        }
    }
    0
}

/// Find a recipe file by number prefix in the recipes directory.
fn find_recipe_file(dir: &Path, prefix: &str) -> Option<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return None;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "yaml") {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with(prefix) {
                    return Some(path);
                }
            }
        }
    }
    None
}
