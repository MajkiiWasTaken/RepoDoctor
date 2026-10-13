/************************************************
* File: main.rs
* Author: Michal Švrček
*
* RepoDoctor command line interface.
*
* ver. 0.1.0
*************************************************/

mod files;
mod git;
mod ignore;
mod output;
mod scan;

use clap::{Parser, Subcommand};
use std::{path::PathBuf, process::ExitCode};

#[derive(Parser)]
#[command(
    name = "rdoctor",
    version,
    about = "Cross-platform Git and project diagnostics",
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inspect a development project
    Scan {
        /// Project directory (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Inspect Git repository status
    Git {
        /// Repository directory (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    output::banner();

    match cli.command {
        Commands::Scan { path } => scan::run(&path),
        Commands::Git { path } => git::run(&path),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            output::error(&error);
            ExitCode::FAILURE
        }
    }
}
