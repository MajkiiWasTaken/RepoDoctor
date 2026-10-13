/************************************************
* File: output.rs
* Author: Michal Švrček
*
* Colored console output and scan statistics.
*
* ver. 0.1.0
*************************************************/

use colored::Colorize;

#[derive(Default)]
pub struct Summary {
    pub passed: usize,
    pub warnings: usize,
    pub errors: usize,
}

impl Summary {
    pub fn ok(&mut self, message: &str) {
        self.passed += 1;
        println!("{} {message}", "[ OK ]".green().bold());
    }

    pub fn warn(&mut self, message: &str) {
        self.warnings += 1;
        println!("{} {message}", "[WARN]".yellow().bold());
    }

    pub fn error(&mut self, message: &str) {
        self.errors += 1;
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
