//! cookbook-runner CLI — qualify recipes against forjar.

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

use cookbook_runner::{RecipeRunner, format_qualify_report, format_validate_report, verdict};

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

            let v = verdict(&result);
            if let Some(msg) = v.error_message() {
                return Err(msg.to_string());
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
