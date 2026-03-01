//! Score all recipes and update the CSV with dimension scores.
//!
//! Usage: `cargo run --example score_all`
//! Reads each recipe YAML, computes static-only `ForjarScore`,
//! and updates docs/certifications/recipes.csv with the results.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use cookbook_qualify::{
    ForjarScore, RecipeConfig, RecipeQualification, SCORE_VERSION, ScoringInput,
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

        let Ok(config) = RecipeConfig::from_yaml(&raw_yaml) else {
            eprintln!("  ERROR: recipe {} — YAML parse failed", recipe.recipe_num);
            errors += 1;
            continue;
        };

        let input = ScoringInput {
            status: &recipe.status,
            idempotency_class: &recipe.idempotency_class,
            config: &config,
            raw_yaml: &raw_yaml,
            budget_ms: 0,
            runtime: None,
        };

        let score = ForjarScore::compute(&input);
        apply_score(recipe, &score);

        eprintln!(
            "  {:>2} {:<35} score={:>3} grade={} SAF={:>3} OBS={:>3} DOC={:>3} RES={:>3} CMP={:>3}",
            recipe.recipe_num,
            recipe.name,
            score.composite,
            score.grade.as_str(),
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
