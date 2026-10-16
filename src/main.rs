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
mod config;
mod report;

use clap::{Parser, Subcommand};
use std::{path::PathBuf, process::ExitCode};
use report::ReportFormat;

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
        /// Project directory
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Save diagnostic report
        #[arg(long)]
        export: Option<PathBuf>,

        /// Report format: text or json
        #[arg(long, value_enum, default_value_t = ReportFormat::Text)]
        format: ReportFormat,
    },

    /// Inspect Git repository status
    Git {
        /// Repository directory
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

        /// CI mode (enables strict checks)
        #[arg(long)]
        ci: bool,

        /// Save diagnostic report
        #[arg(long)]
        export: Option<PathBuf>,

        /// Report format: text or json
        #[arg(long, value_enum, default_value_t = ReportFormat::Text)]
        format: ReportFormat,
    },
}


fn run() -> Result<(), String> {
    let cli = Cli::parse();
    output::banner();
    
    match cli.command {
        Commands::Scan {
            path,
            export,
            format,
        } => scan::run(
            &path,
            export.as_deref(),
            format,
        ),

        Commands::Git { path } => git::run(&path),

        Commands::Release {
            path,
            strict,
            ci,
            export,
            format,
        } => release::run(
            &path,
            strict,
            ci,
            export.as_deref(),
            format,
        ),
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
