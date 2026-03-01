//! cookbook-runner CLI — qualify and score recipes against forjar.

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

use cookbook_runner::{grade_passes_threshold, run_qualify, run_validate, score_recipe_file};

/// Forjar cookbook recipe qualification runner.
#[derive(Parser)]
#[command(name = "cookbook-runner", version, about)]
struct Cli {
    /// Path to forjar binary.
    #[arg(long, default_value = "forjar")]
    forjar: String,

    #[command(subcommand)]
    command: Commands,
}

/// Available subcommands.
#[derive(Subcommand)]
enum Commands {
    /// Validate a recipe config.
    Validate {
        /// Path to recipe YAML file.
        #[arg(short, long)]
        file: PathBuf,
    },
    /// Run full qualification cycle for a recipe.
    Qualify {
        /// Path to recipe YAML file.
        #[arg(short, long)]
        file: PathBuf,
        /// State directory for apply.
        #[arg(long, default_value = "/tmp/cookbook-qualify")]
        state_dir: PathBuf,
    },
    /// Score a recipe (static analysis only, no apply).
    Score {
        /// Path to recipe YAML file.
        #[arg(short, long)]
        file: PathBuf,
        /// Recipe status (qualified, blocked, pending).
        #[arg(long, default_value = "pending")]
        status: String,
        /// Idempotency class (strong, weak, eventual).
        #[arg(long, default_value = "strong")]
        idempotency: String,
        /// Performance budget in milliseconds (0 = no budget).
        #[arg(long, default_value_t = 0)]
        budget_ms: u64,
    },
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { file } => {
            let report = run_validate(&cli.forjar, &file)?;
            eprintln!("{report}");
        }
        Commands::Qualify { file, state_dir } => {
            let report = run_qualify(&cli.forjar, &file, &state_dir)?;
            eprintln!("{report}");
        }
        Commands::Score {
            file,
            status,
            idempotency,
            budget_ms,
        } => {
            let (report, score) = score_recipe_file(&file, &status, &idempotency, budget_ms)?;
            println!("{report}");
            if !grade_passes_threshold(&score.grade) {
                return Err(format!("grade {} — below threshold", score.grade.as_str()));
            }
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        process::exit(1);
    }
}
