/************************************************
* File: projects.rs
* Author: Michal Švrček
*
* Rust, .NET and Node.js project diagnostics.
*
* ver. 0.3.0
*************************************************/

use crate::output::{self, Summary};
use std::{
    collections::VecDeque,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const MAX_DEPTH: usize = 5;
const MAX_DIRECTORIES: usize = 5000;

const SKIP_DIRECTORIES: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    "bin",
    "obj",
    "build",
    "dist",
    ".next",
    ".gradle",
    "__pycache__",
    ".venv",
];

#[derive(Debug, Clone, Copy)]
enum ProjectKind {
    Rust,
    DotNet,
    Node,
}

impl ProjectKind {
    fn label(self) -> &'static str {
        match self {
            Self::Rust => "Rust",
            Self::DotNet => ".NET",
            Self::Node => "Node.js",
        }
    }
}

struct Project {
    path: PathBuf,
    kind: ProjectKind,
}

fn has_extension(dir: &Path, extension: &str) -> bool {
    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };

    entries.flatten().any(|entry| {
        entry.path().is_file()
            && entry
                .path()
                .extension()
                .is_some_and(|ext| ext.to_string_lossy().eq_ignore_ascii_case(extension))
    })
}

fn detect(dir: &Path) -> Vec<ProjectKind> {
    let mut kinds = Vec::new();

    if dir.join("Cargo.toml").is_file() {
        kinds.push(ProjectKind::Rust);
    }

    if has_extension(dir, "csproj")
        || has_extension(dir, "fsproj")
        || has_extension(dir, "vbproj")
        || has_extension(dir, "sln")
    {
        kinds.push(ProjectKind::DotNet);
    }

    if dir.join("package.json").is_file() {
        kinds.push(ProjectKind::Node);
    }

    kinds
}

fn discover(root: &Path) -> (Vec<Project>, bool) {
    let mut projects = Vec::new();
    let mut queue = VecDeque::from([(root.to_path_buf(), 0usize)]);
    let mut visited = 0usize;

    while let Some((dir, depth)) = queue.pop_front() {
        if visited >= MAX_DIRECTORIES {
            return (projects, true);
        }

        visited += 1;

        for kind in detect(&dir) {
            projects.push(Project {
                path: dir.clone(),
                kind,
            });
        }

        if depth >= MAX_DEPTH {
            continue;
        }

        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };

            // Do not follow directory symlinks or junctions.
            if !file_type.is_dir() || file_type.is_symlink() {
                continue;
            }

            let name = entry.file_name();
            let name = name.to_string_lossy().to_ascii_lowercase();

            if SKIP_DIRECTORIES.contains(&name.as_str()) {
                continue;
            }

            queue.push_back((entry.path(), depth + 1));
        }
    }

    (projects, false)
}

fn git_ignored(root: &Path, relative: &Path) -> Result<bool, String> {
    let result = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["check-ignore", "--quiet", "--no-index", "--"])
        .arg(relative)
        .output()
        .map_err(|e| format!("Cannot execute Git: {e}"))?;

    match result.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(String::from_utf8_lossy(&result.stderr).trim().to_string()),
    }
}

fn check_ignored(root: &Path, project: &Path, directory: &str, summary: &mut Summary) {
    let relative = match project.strip_prefix(root) {
        Ok(relative) => relative.join(directory),
        Err(_) => {
            summary.warn("Cannot resolve project path for Git checks");
            return;
        }
    };

    match git_ignored(root, &relative) {
        Ok(true) => {
            summary.ok(&format!("{} is ignored", relative.display()));
        }
        Ok(false) => {
            summary.warn(&format!("{} is not ignored by Git", relative.display()));
        }
        Err(error) => {
            summary.warn(&format!("Cannot check ignore rules: {error}"));
        }
    }
}

fn inspect_project(root: &Path, project: &Project, git_available: bool, summary: &mut Summary) {
    let relative = project.path.strip_prefix(root).unwrap_or(&project.path);

    let display = if relative.as_os_str().is_empty() {
        ".".to_string()
    } else {
        relative.display().to_string()
    };

    output::heading(&format!("{} project: {display}", project.kind.label()));

    match project.kind {
        ProjectKind::Rust => {
            if project.path.join("Cargo.lock").is_file() {
                summary.ok("Cargo.lock found");
            } else {
                summary.warn("Cargo.lock missing (may be intentional for a library)");
            }

            if project.path.join("src").is_dir() {
                summary.ok("Rust source directory found");
            } else {
                output::info("No src directory (possible Cargo workspace)");
            }

            if git_available {
                check_ignored(root, &project.path, "target/", summary);
            }
        }

        ProjectKind::DotNet => {
            output::info(".NET project or solution detected");

            let contains_project = ["csproj", "fsproj", "vbproj"]
                .iter()
                .any(|ext| has_extension(&project.path, ext));

            if git_available && contains_project {
                check_ignored(root, &project.path, "bin/", summary);
                check_ignored(root, &project.path, "obj/", summary);
            }
        }

        ProjectKind::Node => {
            let lockfile = [
                "package-lock.json",
                "pnpm-lock.yaml",
                "yarn.lock",
                "bun.lock",
                "bun.lockb",
            ]
            .iter()
            .any(|name| project.path.join(name).is_file());

            if lockfile {
                summary.ok("Package manager lockfile found");
            } else {
                summary.warn("No recognized package manager lockfile");
            }

            if git_available {
                check_ignored(root, &project.path, "node_modules/", summary);
            }
        }
    }
}

pub fn inspect(root: &Path, summary: &mut Summary) {
    output::heading("Project discovery");

    let (projects, limited) = discover(root);

    if projects.is_empty() {
        summary.warn("No supported project manifests found");
        return;
    }

    output::info(&format!("{} project manifest(s) detected", projects.len()));

    if limited {
        summary.warn("Directory scan limit reached; results may be incomplete");
    }

    let git_available = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .is_ok_and(|result| result.status.success());

    for project in &projects {
        inspect_project(root, project, git_available, summary);
    }
}
