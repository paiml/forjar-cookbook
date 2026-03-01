//! cookbook-readme-sync — regenerate README qualification table from CSV.

use cookbook_qualify::sync_readme;
use std::process;

fn find_project_root() -> Result<std::path::PathBuf, String> {
    let mut dir = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
    loop {
        if dir.join("Cargo.toml").exists() && dir.join("docs/certifications").exists() {
            return Ok(dir);
        }
        if !dir.pop() {
            return Err("could not find project root (Cargo.toml + docs/certifications/)".into());
        }
    }
}

fn run() -> Result<(), String> {
    let root = find_project_root()?;
    let csv_path = root.join("docs/certifications/recipes.csv");
    let readme_path = root.join("README.md");

    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();
    let result = sync_readme(&csv_path, &readme_path, &timestamp)?;

    eprintln!(
        "Updated {} ({} recipes, {} qualified)",
        readme_path.display(),
        result.total,
        result.qualified
    );
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        process::exit(1);
    }
}
