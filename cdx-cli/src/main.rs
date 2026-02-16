//! Codex Document Format CLI
//!
//! A command-line tool for working with Codex Document Format (.cdx) files.

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod cli;
mod commands;
mod dispatcher;
mod output;

use clap::Parser;
use colored::Colorize;

fn main() {
    let cli = cli::Cli::parse();

    // Configure color output
    match cli.color {
        cli::ColorChoice::Always => colored::control::set_override(true),
        cli::ColorChoice::Never => colored::control::set_override(false),
        cli::ColorChoice::Auto => {}
    }

    let output_config = output::OutputConfig {
        verbose: cli.verbose,
        quiet: cli.quiet,
        json: cli.json,
    };

    let result = dispatcher::run_command(cli.command, &output_config);

    if let Err(e) = result {
        if !cli.quiet {
            eprintln!("{} {}", "Error:".red().bold(), e);
            if cli.verbose {
                let mut source = e.source();
                while let Some(cause) = source {
                    eprintln!("  {} {}", "Caused by:".red(), cause);
                    source = cause.source();
                }
            }
        }
        std::process::exit(1);
    }
}
