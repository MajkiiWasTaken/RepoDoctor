/************************************************
* File: report.rs
* Author: Michal Švrček
*
* Diagnostic report export in text and JSON.
*
* ver. 0.5.0
*************************************************/

use crate::output::{CheckStatus, Summary};
use serde::Serialize;
use std::{fs, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ReportFormat {
    Text,
    Json,
}

#[derive(Serialize)]
struct Report<'a> {
    tool: &'static str,
    version: &'static str,
    command: &'a str,
    project: String,
    success: bool,
    summary: &'a Summary,
}

pub fn write(
    path: &Path,
    format: ReportFormat,
    command: &str,
    project: &Path,
    summary: &Summary,
    success: bool,
) -> Result<(), String> {
    let report = Report {
        tool: "RepoDoctor",
        version: env!("CARGO_PKG_VERSION"),
        command,
        project: display_path(project),
        success,
        summary,
    };

    let content = match format {
        ReportFormat::Json => serde_json::to_string_pretty(&report)
            .map_err(|e| format!("Cannot serialize report: {e}"))?,

        ReportFormat::Text => {
            let mut text = format!(
                "RepoDoctor v{}\nCommand: {}\nProject: {}\nResult: {}\n\n",
                env!("CARGO_PKG_VERSION"),
                command,
                display_path(project),
                if success { "PASS" } else { "FAIL" }
            );

            for check in &summary.checks {
                let status = match check.status {
                    CheckStatus::Ok => "OK",
                    CheckStatus::Warning => "WARN",
                    CheckStatus::Error => "ERROR",
                };

                text.push_str(&format!("[{status}] {}\n", check.message));
            }

            text.push_str(&format!(
                "\n{} passed, {} warnings, {} errors\n",
                summary.passed, summary.warnings, summary.errors
            ));

            text
        }
    };

    // Refuse to overwrite any existing file.
    // This avoids accidentally replacing source files or old reports.
    use std::io::Write;

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("Cannot create report {}: {e}", path.display()))?;

    file.write_all(content.as_bytes())
        .map_err(|e| format!("Cannot write report: {e}"))?;

    println!("[INFO] Report saved: {}", path.display());
    Ok(())
}

fn display_path(path: &Path) -> String {
    let value = path.to_string_lossy();

    value.strip_prefix(r"\\?\").unwrap_or(&value).to_string()
}
