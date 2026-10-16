/************************************************
* File: output.rs
* Author: Michal Švrček
*
* Console output and structured diagnostic results.
*
* ver. 0.5.0
*************************************************/

use colored::Colorize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Ok,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct Check {
    pub status: CheckStatus,
    pub message: String,
}

#[derive(Debug, Default, Serialize)]
pub struct Summary {
    pub passed: usize,
    pub warnings: usize,
    pub errors: usize,
    pub checks: Vec<Check>,
}

impl Summary {
    pub fn ok(&mut self, message: &str) {
        self.passed += 1;
        self.checks.push(Check {
            status: CheckStatus::Ok,
            message: message.to_string(),
        });
        println!("{} {message}", "[ OK ]".green().bold());
    }

    pub fn warn(&mut self, message: &str) {
        self.warnings += 1;
        self.checks.push(Check {
            status: CheckStatus::Warning,
            message: message.to_string(),
        });
        println!("{} {message}", "[WARN]".yellow().bold());
    }

    pub fn error(&mut self, message: &str) {
        self.errors += 1;
        self.checks.push(Check {
            status: CheckStatus::Error,
            message: message.to_string(),
        });
        println!("{} {message}", "[ERROR]".red().bold());
    }

    pub fn print(&self) {
        println!();
        println!(
            "{} {} passed · {} warnings · {} errors",
            "[INFO]".blue().bold(),
            self.passed,
            self.warnings,
            self.errors
        );
    }

    pub fn successful(&self, strict: bool) -> bool {
        self.errors == 0 && (!strict || self.warnings == 0)
    }
}

pub fn banner() {
    println!(
        "{}",
        format!("◆ RepoDoctor v{}", env!("CARGO_PKG_VERSION"))
            .bright_blue()
            .bold()
    );
    println!("{}", "────────────────────────────────────────".dimmed());
}

pub fn heading(message: &str) {
    println!("\n{}", message.bright_cyan().bold());
}

pub fn info(message: &str) {
    println!("{} {message}", "[INFO]".blue().bold());
}

pub fn error(message: &str) {
    eprintln!("{} {message}", "[ERROR]".red().bold());
}
