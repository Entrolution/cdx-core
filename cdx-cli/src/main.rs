//! Codex Document Format CLI
//!
//! A command-line tool for working with Codex Document Format (.cdx) files.

mod commands;
mod output;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use colored::Colorize;
use std::io;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cdx")]
#[command(author, version, about = "Codex Document Format CLI", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Increase output verbosity
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Suppress non-error output
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Output as JSON (for scripting)
    #[arg(long, global = true)]
    json: bool,

    /// Color output control
    #[arg(long, value_enum, default_value = "auto", global = true)]
    color: ColorChoice,
}

#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum ColorChoice {
    Auto,
    Always,
    Never,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Codex document
    Create {
        /// Document title (required)
        #[arg(short, long)]
        title: String,

        /// Author name(s)
        #[arg(short, long)]
        author: Vec<String>,

        /// Initial state
        #[arg(long, default_value = "draft")]
        state: String,

        /// Input content file (markdown, text)
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Output file
        output: PathBuf,
    },

    /// Validate document structure and hashes
    Validate {
        /// Codex document to validate
        file: PathBuf,
    },

    /// Display document information
    Inspect {
        /// Codex document to inspect
        file: PathBuf,

        /// Show detailed block information
        #[arg(long)]
        blocks: bool,

        /// Show signature details
        #[arg(long)]
        signatures: bool,

        /// Show provenance chain
        #[arg(long)]
        provenance: bool,
    },

    /// Add a digital signature
    Sign {
        /// Codex document to sign
        file: PathBuf,

        /// Private key file (PEM format)
        #[arg(short, long)]
        key: PathBuf,

        /// Signer name
        #[arg(short, long)]
        name: String,

        /// Signer email
        #[arg(short, long)]
        email: Option<String>,

        /// Signature algorithm
        #[arg(short, long, default_value = "ES256")]
        algorithm: String,

        /// Output file (default: overwrite input)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Verify signatures and integrity
    Verify {
        /// Codex document to verify
        file: PathBuf,

        /// Public key file(s) for signature verification
        #[arg(short, long)]
        key: Vec<PathBuf>,
    },

    /// Extract content or assets
    Extract {
        /// Codex document to extract from
        file: PathBuf,

        /// Output directory for extraction
        #[arg(short, long, default_value = ".")]
        output: PathBuf,

        /// Extract content as JSON
        #[arg(long)]
        content: bool,

        /// Extract as plain text
        #[arg(long)]
        text: bool,

        /// Extract specific asset
        #[arg(long)]
        asset: Option<String>,

        /// Extract all assets
        #[arg(long)]
        all_assets: bool,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Configure color output
    match cli.color {
        ColorChoice::Always => colored::control::set_override(true),
        ColorChoice::Never => colored::control::set_override(false),
        ColorChoice::Auto => {}
    }

    let output_config = output::OutputConfig {
        verbose: cli.verbose,
        quiet: cli.quiet,
        json: cli.json,
    };

    let result = match cli.command {
        Commands::Create {
            title,
            author,
            state,
            input,
            output: output_path,
        } => commands::create::run(title, author, state, input, output_path, &output_config),

        Commands::Validate { file } => commands::validate::run(file, &output_config),

        Commands::Inspect {
            file,
            blocks,
            signatures,
            provenance,
        } => commands::inspect::run(file, blocks, signatures, provenance, &output_config),

        Commands::Sign {
            file,
            key,
            name,
            email,
            algorithm,
            output: output_path,
        } => commands::sign::run(file, key, name, email, algorithm, output_path, &output_config),

        Commands::Verify { file, key } => commands::verify::run(file, key, &output_config),

        Commands::Extract {
            file,
            output: output_path,
            content,
            text,
            asset,
            all_assets,
        } => commands::extract::run(
            file,
            output_path,
            content,
            text,
            asset,
            all_assets,
            &output_config,
        ),

        Commands::Completions { shell } => {
            generate(shell, &mut Cli::command(), "cdx", &mut io::stdout());
            Ok(())
        }
    };

    if let Err(e) = result {
        if !cli.quiet {
            eprintln!("{} {}", "Error:".red().bold(), e);
            if cli.verbose {
                // Print error chain
                let mut source = e.source();
                while let Some(cause) = source {
                    eprintln!("  {} {}", "Caused by:".red(), cause);
                    source = cause.source();
                }
            }
        }
        std::process::exit(1);
    }

    Ok(())
}
