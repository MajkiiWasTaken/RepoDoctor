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
mod output;
mod projects;
mod release;
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

    /// Check whether a project is ready for release
    Release {
        /// Project directory
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Treat warnings as release blockers
        #[arg(long)]
        strict: bool,
    },
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    output::banner();

    match cli.command {
        Commands::Scan { path } => scan::run(&path),
        Commands::Git { path } => git::run(&path),
        Commands::Release { path, strict } => release::run(&path, strict),
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
