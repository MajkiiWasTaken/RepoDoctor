/************************************************
* File: git.rs
* Author: Michal Švrček
*
* Git repository inspection and diagnostics.
*
* ver. 0.1.0
*************************************************/

use crate::output::{self, Summary};
use std::{path::Path, process::Command};

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

    Ok(String::from_utf8_lossy(&result.stdout).trim().to_string())
}

pub fn inspect(path: &Path, summary: &mut Summary) {
    let root = match git(path, &["rev-parse", "--show-toplevel"]) {
        Ok(root) => {
            summary.ok("Git repository detected");
            root
        }
        Err(_) => {
            summary.warn("Not a Git repository");
            return;
        }
    };

    output::info(&format!("Repository: {root}"));

    match git(path, &["symbolic-ref", "--short", "HEAD"]) {
        Ok(branch) => {
            output::info(&format!("Branch: {branch}"));
        }
        Err(_) => {
            summary.warn("Detached HEAD or no active branch");
        }
    }

    match git(
        path,
        &["status", "--porcelain=v1", "--untracked-files=normal"],
    ) {
        Ok(status) if status.is_empty() => {
            summary.ok("Working tree is clean");
        }
        Ok(status) => {
            let count = status.lines().count();
            summary.warn(&format!("{count} modified or untracked entries"));
        }
        Err(error) => {
            summary.error(&format!("Cannot read Git status: {error}"));
        }
    }

    match git(path, &["remote"]) {
        Ok(remotes) if remotes.is_empty() => {
            summary.warn("No Git remote configured");
        }
        Ok(remotes) => {
            summary.ok(&format!("Git remotes: {}", remotes.replace('\n', ", ")));
        }
        Err(error) => {
            summary.error(&format!("Cannot list Git remotes: {error}"));
        }
    }

    // Only compare against an upstream when one is configured.
    if git(
        path,
        &[
            "rev-parse",
            "--abbrev-ref",
            "--symbolic-full-name",
            "@{upstream}",
        ],
    )
    .is_ok()
    {
        match git(
            path,
            &["rev-list", "--left-right", "--count", "@{upstream}...HEAD"],
        ) {
            Ok(counts) => {
                let parts: Vec<&str> = counts.split_whitespace().collect();

                if parts.len() == 2 {
                    let behind = parts[0].parse::<usize>().unwrap_or(0);
                    let ahead = parts[1].parse::<usize>().unwrap_or(0);

                    if ahead == 0 && behind == 0 {
                        summary.ok("Branch matches local upstream tracking ref");
                    } else {
                        summary.warn(&format!(
                            "Upstream difference: {ahead} ahead, {behind} behind"
                        ));
                    }
                } else {
                    summary.warn("Unexpected upstream comparison result");
                }
            }
            Err(error) => {
                summary.warn(&format!("Cannot compare upstream: {error}"));
            }
        }
    } else {
        summary.warn("No upstream tracking branch configured");
    }
}

pub fn run(path: &Path) -> Result<(), String> {
    if !path.is_dir() {
        return Err(format!("Directory does not exist: {}", path.display()));
    }

    output::heading(&format!("Git diagnostics: {}", path.display()));

    let mut summary = Summary::default();
    inspect(path, &mut summary);
    summary.print();

    Ok(())
}
