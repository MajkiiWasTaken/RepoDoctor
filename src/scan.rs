/************************************************
* File: scan.rs
* Author: Michal Švrček
*
* Basic development project health checks.
*
* ver. 0.1.0
*************************************************/

use crate::files;
use crate::projects;
use crate::{
    config::Config,
    report::{self, ReportFormat},
};
use crate::{
    git,
    output::{self, Summary},
};
use std::path::Path;

fn has_file(path: &Path, names: &[&str]) -> bool {
    names.iter().any(|name| path.join(name).is_file())
}

pub fn run(path: &Path, export: Option<&Path>, format: ReportFormat) -> Result<(), String> {
    if !path.is_dir() {
        return Err(format!("Directory does not exist: {}", path.display()));
    }

    let path = path
        .canonicalize()
        .map_err(|e| format!("Cannot resolve directory: {e}"))?;

    let config = Config::load(&path)?;

    let display_path = path.to_string_lossy();

    let display_path = display_path.strip_prefix(r"\\?\").unwrap_or(&display_path);

    output::heading(&format!("Project scan: {display_path}"));

    let mut summary = Summary::default();

    if has_file(&path, &["README.md", "README", "README.txt"]) {
        summary.ok("README found");
    } else {
        summary.warn("README missing");
    }

    if has_file(&path, &[".gitignore"]) {
        summary.ok(".gitignore found");
    } else {
        summary.warn(".gitignore missing");
    }

    if has_file(&path, &["LICENSE", "LICENSE.md", "LICENSE.txt", "COPYING"]) {
        summary.ok("License file found");
    } else {
        summary.warn("License file missing");
    }

    output::heading("Git checks");
    git::inspect(&path, &mut summary);

    files::inspect(&path, &mut summary, config.files.large_file_mb);

    // Detect Rust, .NET and Node.js projects in subdirectories.
    projects::inspect(&path, &mut summary);

    summary.print();

    let success = summary.successful(false);

    if let Some(destination) = export {
        report::write(destination, format, "scan", &path, &summary, success)?;
    }

    if !success {
        return Err("Project scan found errors".into());
    }

    Ok(())
}
