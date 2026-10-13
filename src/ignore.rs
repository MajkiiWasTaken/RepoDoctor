/************************************************
* File: ignore.rs
* Author: Michal Švrček
*
* Git ignore rule diagnostics.
*
* ver. 0.2.0
*************************************************/

use crate::output::{self, Summary};
use std::{path::Path, process::Command};

fn ignored(root: &Path, target: &str) -> Result<bool, String> {
    let result = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["check-ignore", "--quiet", "--no-index", "--", target])
        .status()
        .map_err(|e| format!("Cannot execute Git: {e}"))?;

    match result.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err("Cannot check Git ignore rules".into()),
    }
}

pub fn inspect(root: &Path, summary: &mut Summary) {
    output::heading("Git ignore checks");

    let rules = [
        ("Cargo.toml", "target/"),
        ("package.json", "node_modules/"),
        ("package.json", ".next/"),
        ("pyproject.toml", "__pycache__/"),
        ("CMakeLists.txt", "build/"),
    ];

    let mut checked = 0usize;

    for (manifest, directory) in rules {
        if !root.join(manifest).is_file() {
            continue;
        }

        // .next/ is only relevant if a Next.js directory exists.
        if directory == ".next/"
            && !root.join("next.config.js").exists()
            && !root.join("next.config.mjs").exists()
            && !root.join("next.config.ts").exists()
        {
            continue;
        }

        checked += 1;

        match ignored(root, directory) {
            Ok(true) => {
                summary.ok(&format!("{directory} is ignored"));
            }
            Ok(false) => {
                summary.warn(&format!("{directory} is not covered by Git ignore rules"));
            }
            Err(error) => {
                summary.error(&format!("Cannot check {directory}: {error}"));
            }
        }
    }

    if checked == 0 {
        output::info("No applicable build ignore rules to check");
    }
}
