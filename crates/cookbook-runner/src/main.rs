//! cookbook-runner CLI — qualify and score recipes against forjar.

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

use cookbook_qualify::{ForjarScore, IdempotencyClass, RecipeConfig, RecipeStatus, ScoringInput};
use cookbook_runner::{
    RecipeRunner, format_qualify_report, format_score_report, format_validate_report,
    runtime_data_from_qualify, verdict,
};

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
    let runner = RecipeRunner::new(&cli.forjar);

    match cli.command {
        Commands::Validate { file } => {
            let outcome = runner.validate(&file);
            let report = format_validate_report(&file, outcome.exit_code, outcome.duration_ms);
            eprintln!("{report}");
            if outcome.exit_code != 0 {
                eprintln!("{}", outcome.output);
                return Err(format!("validation failed: {}", file.display()));
            }
        }
        Commands::Qualify { file, state_dir } => {
            let result = runner.qualify(&file, &state_dir);
            let report = format_qualify_report(&file, &result);
            eprintln!("{report}");

            // Compute and display score with runtime data
            if let Ok(raw_yaml) = std::fs::read_to_string(&file) {
                if let Ok(config) = RecipeConfig::from_yaml(&raw_yaml) {
                    let rt = runtime_data_from_qualify(&result);
                    let v = verdict(&result);
                    let status = if v.is_qualified() {
                        RecipeStatus::Qualified
                    } else {
                        RecipeStatus::Pending
                    };
                    let input = ScoringInput {
                        status: &status,
                        idempotency_class: &IdempotencyClass::Strong,
                        config: &config,
                        raw_yaml: &raw_yaml,
                        budget_ms: 0,
                        runtime: Some(&rt),
                    };
                    let score = ForjarScore::compute(&input);
                    let score_report = format_score_report(&score);
                    eprintln!("{score_report}");
                }
            }

            let v = verdict(&result);
            if let Some(msg) = v.error_message() {
                return Err(msg.to_string());
            }
        }
        Commands::Score {
            file,
            status,
            idempotency,
            budget_ms,
        } => {
            let raw_yaml = std::fs::read_to_string(&file)
                .map_err(|e| format!("cannot read {}: {e}", file.display()))?;
            let config = RecipeConfig::from_yaml(&raw_yaml)?;
            let recipe_status = RecipeStatus::from_csv(&status)?;
            let idem_class = IdempotencyClass::from_csv(&idempotency)?;
            let input = ScoringInput {
                status: &recipe_status,
                idempotency_class: &idem_class,
                config: &config,
                raw_yaml: &raw_yaml,
                budget_ms,
                runtime: None, // static-only scoring
            };
            let score = ForjarScore::compute(&input);
            let report = format_score_report(&score);
            println!("{report}");

            // Exit 0 for A-C, exit 1 for D-F
            match score.grade {
                cookbook_qualify::Grade::A
                | cookbook_qualify::Grade::B
                | cookbook_qualify::Grade::C => {}
                _ => return Err(format!("grade {} — below threshold", score.grade.as_str())),
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
