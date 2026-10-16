/************************************************
* File: release.rs
* Author: Michal Švrček
*
* Release readiness and version diagnostics.
*
* ver. 0.4.0
*************************************************/

use crate::output::{self, Summary};
use crate::{
    config::Config,
    report::{self, ReportFormat},
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn git(path: &Path, args: &[&str]) -> Result<String, String> {
    let result = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()
        .map_err(|e| format!("Cannot execute Git: {e}"))?;

    if !result.status.success() {
        return Err(String::from_utf8_lossy(&result.stderr).trim().to_string());
    }

    Ok(String::from_utf8_lossy(&result.stdout).trim().to_owned())
}

fn project_version(path: &Path) -> Result<Option<String>, String> {
    let cargo = path.join("Cargo.toml");

    if cargo.is_file() {
        let content =
            fs::read_to_string(&cargo).map_err(|e| format!("Cannot read Cargo.toml: {e}"))?;

        let document: toml::Value =
            toml::from_str(&content).map_err(|e| format!("Invalid Cargo.toml: {e}"))?;

        if let Some(version) = document
            .get("package")
            .and_then(|p| p.get("version"))
            .and_then(|v| v.as_str())
        {
            return Ok(Some(version.to_owned()));
        }

        return Ok(None);
    }

    let package = path.join("package.json");

    if package.is_file() {
        let content =
            fs::read_to_string(&package).map_err(|e| format!("Cannot read package.json: {e}"))?;

        let document: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| format!("Invalid package.json: {e}"))?;

        return Ok(document
            .get("version")
            .and_then(|v| v.as_str())
            .map(str::to_owned));
    }

    Ok(None)
}

fn has_license(path: &Path) -> bool {
    ["LICENSE", "LICENSE.md", "LICENSE.txt", "COPYING"]
        .iter()
        .any(|name| path.join(name).is_file())
}

fn has_readme(path: &Path) -> bool {
    ["README.md", "README", "README.txt"]
        .iter()
        .any(|name| path.join(name).is_file())
}

fn git_checks(path: &Path, summary: &mut Summary) -> bool {
    if git(path, &["rev-parse", "--show-toplevel"]).is_err() {
        summary.error("Not a Git repository");
        return false;
    }

    summary.ok("Git repository detected");

    match git(path, &["status", "--porcelain=v1"]) {
        Ok(status) if status.is_empty() => {
            summary.ok("Working tree is clean");
        }
        Ok(status) => {
            summary.error(&format!(
                "{} uncommitted or untracked entries",
                status.lines().count()
            ));
        }
        Err(err) => summary.error(&format!("Cannot check Git status: {err}")),
    }

    match git(path, &["symbolic-ref", "--short", "HEAD"]) {
        Ok(branch) => summary.ok(&format!("Branch: {branch}")),
        Err(_) => summary.error("Detached HEAD or unborn branch"),
    }

    match git(path, &["remote"]) {
        Ok(remotes) if !remotes.is_empty() => {
            summary.ok(&format!(
                "Remote configured: {}",
                remotes.replace('\n', ", ")
            ));
        }
        Ok(_) => summary.warn("No Git remote configured"),
        Err(err) => summary.error(&format!("Cannot read remotes: {err}")),
    }

    let upstream = git(
        path,
        &[
            "rev-parse",
            "--abbrev-ref",
            "--symbolic-full-name",
            "@{upstream}",
        ],
    );

    if upstream.is_err() {
        summary.warn("No upstream tracking branch configured");
        return true;
    }

    match git(
        path,
        &["rev-list", "--left-right", "--count", "@{upstream}...HEAD"],
    ) {
        Ok(counts) => {
            let parts: Vec<_> = counts.split_whitespace().collect();

            if parts.len() == 2 {
                match (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                    (Ok(behind), Ok(ahead)) if behind == 0 && ahead == 0 => {
                        summary.ok("Branch matches local upstream tracking ref");
                    }
                    (Ok(behind), Ok(ahead)) => {
                        summary.warn(&format!("Branch differs: {ahead} ahead, {behind} behind"));
                    }
                    _ => summary.error("Invalid upstream comparison values"),
                }
            } else {
                summary.error("Unexpected upstream comparison output");
            }
        }
        Err(err) => summary.error(&format!("Cannot compare upstream: {err}")),
    }

    true
}

fn version_checks(
    path: &Path,
    git_available: bool,
    summary: &mut Summary,
    warn_existing_tag: bool,
) {
    match project_version(path) {
        Ok(Some(version)) => {
            summary.ok(&format!("Project version: {version}"));

            if !git_available {
                return;
            }

            let tag = format!("v{version}");
            let tag_ref = format!("refs/tags/{tag}");

            let result = Command::new("git")
                .arg("-C")
                .arg(path)
                .args(["show-ref", "--verify", "--quiet", &tag_ref])
                .output();

            match result {
                Ok(result) if result.status.success() => {
                    if warn_existing_tag {
                        summary.warn(&format!("Version tag {tag} already exists"));
                    } else {
                        summary.ok(&format!("Version tag {tag} already exists"));
                    }
                }

                Ok(result) if result.status.code() == Some(1) => {
                    summary.ok(&format!("Tag {tag} is available"));
                }

                Ok(result) => {
                    summary.error(&format!(
                        "Cannot verify tag {tag}: {}",
                        String::from_utf8_lossy(&result.stderr).trim()
                    ));
                }

                Err(error) => {
                    summary.error(&format!("Cannot execute Git: {error}"));
                }
            }
        }

        Ok(None) => {
            summary.warn("No project version detected in the selected directory");
        }

        Err(error) => {
            summary.error(&error);
        }
    }
}

pub fn run(
    path: &Path,
    strict: bool,
    ci: bool,
    export: Option<&Path>,
    format: ReportFormat,
) -> Result<(), String> {
    if !path.is_dir() {
        return Err(format!("Directory does not exist: {}", path.display()));
    }

    let path: PathBuf = path
        .canonicalize()
        .map_err(|e| format!("Cannot resolve directory: {e}"))?;

    let config = Config::load(&path)?;

    let display_path = path.to_string_lossy();
    let display_path = display_path.strip_prefix(r"\\?\").unwrap_or(&display_path);

    output::heading(&format!("Release readiness: {display_path}"));

    let mut summary = Summary::default();

    let git_available = git_checks(&path, &mut summary);

    output::heading("Version checks");

    version_checks(
        &path,
        git_available,
        &mut summary,
        config.release.warn_existing_tag,
    );

    output::heading("Documentation checks");

    if has_readme(&path) {
        summary.ok("README found");
    } else {
        summary.warn("README missing");
    }

    if has_license(&path) {
        summary.ok("License file found");
    } else if config.release.require_license {
        summary.error("Required LICENSE file missing");
    } else {
        summary.warn("License file missing");
    }

    summary.print();

    // CI mode automatically enables strict checking.
    let effective_strict = strict || ci;
    let success = summary.successful(effective_strict);

    // Always generate the report, even when validation fails.
    if let Some(destination) = export {
        report::write(destination, format, "release", &path, &summary, success)?;
    }

    if summary.errors > 0 {
        return Err(format!("Release blocked: {} error(s)", summary.errors));
    }

    if effective_strict && summary.warnings > 0 {
        return Err(format!(
            "Strict release check failed: {} warning(s)",
            summary.warnings
        ));
    }

    output::info("Release readiness checks passed");

    Ok(())
}
