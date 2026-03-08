//! Forjar cookbook qualification — CSV parsing, table generation, README sync.
//!
//! Follows the `apr-model-qa-playbook` pattern: a CSV is the source of truth
//! for recipe qualification status, and a sync binary regenerates the README
//! table between HTML comment markers.

mod qualify;
pub mod score;
mod sync;
mod table;

pub use qualify::{IdempotencyClass, RecipeQualification, RecipeStatus};
pub use score::{
    DimensionScores, ForjarScore, Grade, Penalty, RuntimeData, SCORE_VERSION, ScoringInput,
};
pub use sync::{find_project_root, run_readme_sync, sync_readme};
pub use table::{
    END_MARKER, START_MARKER, generate_summary, generate_table, parse_csv, update_readme, write_csv,
};
