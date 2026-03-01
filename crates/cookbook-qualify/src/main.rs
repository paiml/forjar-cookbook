//! cookbook-readme-sync — regenerate README qualification table from CSV.

use cookbook_qualify::run_readme_sync;
use std::process;

fn main() {
    match run_readme_sync() {
        Ok((readme_path, result)) => {
            eprintln!(
                "Updated {} ({} recipes, {} qualified)",
                readme_path.display(),
                result.total,
                result.qualified
            );
        }
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    }
}
