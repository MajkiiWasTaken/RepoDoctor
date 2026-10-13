/************************************************
* File: scan.rs
* Author: Michal Švrček
*
* Basic development project health checks.
*
* ver. 0.1.0
*************************************************/

use crate::{files, ignore};
use crate::{
    git,
    output::{self, Summary},
};
use std::path::Path;

fn has_file(path: &Path, names: &[&str]) -> bool {
    names.iter().any(|name| path.join(name).is_file())
}

pub fn run(path: &Path) -> Result<(), String> {
    if !path.is_dir() {
        return Err(format!("Directory does not exist: {}", path.display()));
    }

    let path = path
        .canonicalize()
        .map_err(|e| format!("Cannot resolve directory: {e}"))?;

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

    if has_file(&path, &["Cargo.toml"]) {
        summary.ok("Rust project detected");
    } else if has_file(&path, &["package.json"]) {
        summary.ok("Node.js project detected");
    } else if has_file(&path, &["CMakeLists.txt"]) {
        summary.ok("CMake project detected");
    } else {
        output::info("No recognized project manifest in current directory");
    }

    output::heading("Git checks");
    git::inspect(&path, &mut summary);

    files::inspect(&path, &mut summary);
    ignore::inspect(&path, &mut summary);

    summary.print();

    Ok(())
}
