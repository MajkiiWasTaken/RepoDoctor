/************************************************
* File: files.rs
* Author: Michal Švrček
*
* Tracked file inspection and build artifact checks.
*
* ver. 0.2.0
*************************************************/

use crate::output::{self, Summary};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

const LARGE_FILE_LIMIT: u64 = 10 * 1024 * 1024;
const MAX_WARNINGS: usize = 20;

fn git_output(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let result = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|e| format!("Cannot run Git: {e}"))?;

    if !result.status.success() {
        return Err(format!(
            "Git failed: {}",
            String::from_utf8_lossy(&result.stderr).trim()
        ));
    }

    Ok(result.stdout)
}

fn git_root(path: &Path) -> Result<PathBuf, String> {
    let output = git_output(path, &["rev-parse", "--show-toplevel"])?;

    let root = String::from_utf8(output).map_err(|e| format!("Invalid Git path encoding: {e}"))?;

    let root = root.trim();

    if root.is_empty() {
        return Err("Cannot determine Git repository root".into());
    }

    Ok(PathBuf::from(root))
}

fn is_build_artifact(path: &Path) -> bool {
    let components: Vec<String> = path
        .components()
        .map(|part| part.as_os_str().to_string_lossy().to_ascii_lowercase())
        .collect();

    // Consider the directory name, not an incidental substring
    // inside a source filename.
    components.iter().any(|part| {
        matches!(
            part.as_str(),
            "target" | "node_modules" | "__pycache__" | "bin" | "obj" | ".next" | ".gradle"
        )
    })
}

fn human_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    }
}

pub fn inspect(path: &Path, summary: &mut Summary) {
    output::heading("File checks");

    let root = match git_root(path) {
        Ok(root) => root,
        Err(_) => {
            summary.warn("Tracked file analysis requires a Git repository");
            return;
        }
    };

    let output = match git_output(&root, &["ls-files", "--cached", "-z"]) {
        Ok(output) => output,
        Err(error) => {
            summary.error(&format!("Cannot inspect tracked files: {error}"));
            return;
        }
    };

    let mut checked = 0usize;
    let mut large = 0usize;
    let mut artifacts = 0usize;

    for entry in output.split(|byte| *byte == 0) {
        if entry.is_empty() {
            continue;
        }

        #[cfg(unix)]
        let relative = {
            use std::os::unix::ffi::OsStringExt;
            PathBuf::from(std::ffi::OsString::from_vec(entry.to_vec()))
        };

        #[cfg(not(unix))]
        let relative = PathBuf::from(String::from_utf8_lossy(entry).to_string());

        let absolute = root.join(&relative);

        checked += 1;

        if is_build_artifact(&relative) {
            artifacts += 1;

            if artifacts <= MAX_WARNINGS {
                summary.error(&format!("Tracked build artifact: {}", relative.display()));
            }
        }

        if let Ok(metadata) = absolute.metadata() {
            if metadata.is_file() && metadata.len() > LARGE_FILE_LIMIT {
                large += 1;

                if large <= MAX_WARNINGS {
                    summary.warn(&format!(
                        "Large tracked file: {} ({})",
                        relative.display(),
                        human_size(metadata.len())
                    ));
                }
            }
        }
    }

    if artifacts == 0 {
        summary.ok("No tracked build artifacts found");
    }

    if large == 0 {
        summary.ok("No tracked files exceed 10 MB");
    }

    if artifacts > MAX_WARNINGS {
        summary.warn(&format!(
            "{} additional tracked build artifacts omitted",
            artifacts - MAX_WARNINGS
        ));
    }

    if large > MAX_WARNINGS {
        summary.warn(&format!(
            "{} additional large tracked files omitted",
            large - MAX_WARNINGS
        ));
    }

    output::info(&format!("{checked} tracked files inspected"));
}
