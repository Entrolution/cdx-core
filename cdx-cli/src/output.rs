//! Output formatting helpers.

use colored::Colorize;
use serde::Serialize;

/// Output configuration from command-line flags.
#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub verbose: bool,
    pub quiet: bool,
    pub json: bool,
}

impl OutputConfig {
    /// Print a success message.
    pub fn success(&self, message: &str) {
        if self.quiet {
            return;
        }
        if self.json {
            let output = serde_json::json!({
                "status": "success",
                "message": message
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        } else {
            println!("{} {}", "✓".green().bold(), message);
        }
    }

    /// Print an info message.
    pub fn info(&self, message: &str) {
        if self.quiet {
            return;
        }
        if !self.json {
            println!("{}", message);
        }
    }

    /// Print a verbose message (only in verbose mode).
    pub fn verbose(&self, message: &str) {
        if self.quiet || !self.verbose {
            return;
        }
        if !self.json {
            println!("{}", message.dimmed());
        }
    }

    /// Print a warning message.
    pub fn warning(&self, message: &str) {
        if self.quiet {
            return;
        }
        if self.json {
            let output = serde_json::json!({
                "status": "warning",
                "message": message
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        } else {
            eprintln!("{} {}", "Warning:".yellow().bold(), message);
        }
    }

    /// Print structured JSON output.
    pub fn json_output<T: Serialize>(&self, data: &T) -> anyhow::Result<()> {
        if self.json {
            println!("{}", serde_json::to_string_pretty(data)?);
        }
        Ok(())
    }

    /// Print a key-value pair.
    pub fn field(&self, key: &str, value: &str) {
        if self.quiet || self.json {
            return;
        }
        println!("{}: {}", key.bold(), value);
    }

    /// Print a labeled section.
    pub fn section(&self, title: &str) {
        if self.quiet || self.json {
            return;
        }
        println!("\n{}", title.blue().bold());
        println!("{}", "-".repeat(title.len()).blue());
    }
}
