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

    /// Show comprehensive document status
    Status {
        /// Codex document to check
        file: PathBuf,
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

    /// Submit document for review (draft → review)
    #[command(name = "submit-review")]
    SubmitReview {
        /// Codex document to submit
        file: PathBuf,

        /// Output file (default: overwrite input)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Freeze document (review → frozen)
    Freeze {
        /// Codex document to freeze
        file: PathBuf,

        /// Output file (default: overwrite input)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Publish document (frozen → published)
    Publish {
        /// Codex document to publish
        file: PathBuf,

        /// Output file (default: overwrite input)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Revert document to draft (review → draft)
    Revert {
        /// Codex document to revert
        file: PathBuf,

        /// Output file (default: overwrite input)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Fork document to create new version with lineage
    Fork {
        /// Codex document to fork
        file: PathBuf,

        /// Output file for the forked document
        #[arg(short, long)]
        output: PathBuf,

        /// Note describing the changes
        #[arg(short, long)]
        note: Option<String>,
    },

    /// Generate a Merkle proof for a block
    Prove {
        /// Codex document
        file: PathBuf,

        /// Block ID to prove
        #[arg(long, conflicts_with = "block_index")]
        block_id: Option<String>,

        /// Block index to prove (0-based)
        #[arg(long, conflicts_with = "block_id")]
        block_index: Option<usize>,

        /// Output file for the proof JSON
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Verify a Merkle proof against a document
    #[command(name = "verify-proof")]
    VerifyProof {
        /// Codex document
        file: PathBuf,

        /// Proof JSON file
        proof: PathBuf,
    },

    /// Show document lineage (ancestor chain)
    #[command(name = "show-lineage")]
    ShowLineage {
        /// Codex document
        file: PathBuf,
    },

    /// Display document metadata
    #[command(name = "get-metadata")]
    GetMetadata {
        /// Codex document
        file: PathBuf,
    },

    /// Set document metadata fields
    #[command(name = "set-metadata")]
    SetMetadata {
        /// Codex document
        file: PathBuf,

        /// Set title
        #[arg(long)]
        title: Option<String>,

        /// Set creator(s)
        #[arg(long)]
        creator: Vec<String>,

        /// Set subject(s)
        #[arg(long)]
        subject: Vec<String>,

        /// Set description
        #[arg(long)]
        description: Option<String>,

        /// Set publisher
        #[arg(long)]
        publisher: Option<String>,

        /// Set language (BCP 47 code)
        #[arg(long)]
        language: Option<String>,

        /// Set rights statement
        #[arg(long)]
        rights: Option<String>,

        /// Output file (default: overwrite input)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Pack a directory or JSON into a .cdx archive
    Pack {
        /// Input directory or JSON file
        input: PathBuf,

        /// Output .cdx file
        #[arg(short, long)]
        output: PathBuf,

        /// Input is combined JSON from Pandoc writer
        #[arg(long)]
        from_json: bool,
    },

    /// Compare two Codex documents
    Diff {
        /// First document
        file1: PathBuf,

        /// Second document
        file2: PathBuf,
    },

    /// Show timestamps in a document
    #[command(name = "show-timestamps")]
    ShowTimestamps {
        /// Codex document
        file: PathBuf,
    },

    /// Verify timestamps in a document
    #[command(name = "verify-timestamps")]
    VerifyTimestamps {
        /// Codex document
        file: PathBuf,
    },

    /// Add a timestamp record to a document
    #[command(name = "add-timestamp")]
    AddTimestamp {
        /// Codex document
        file: PathBuf,

        /// Timestamp method (rfc3161, bitcoin, ethereum, opentimestamps)
        #[arg(long)]
        method: String,

        /// Timestamp authority URL or name
        #[arg(long)]
        authority: String,

        /// Base64-encoded timestamp token
        #[arg(long)]
        token: String,

        /// Timestamp time (RFC 3339 format, defaults to now)
        #[arg(long)]
        time: Option<String>,

        /// Transaction ID (for blockchain timestamps)
        #[arg(long)]
        transaction_id: Option<String>,

        /// Output file (default: overwrite input)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Acquire a timestamp from a timestamp authority
    #[command(name = "timestamp-acquire")]
    TimestampAcquire {
        /// Codex document to timestamp
        file: PathBuf,

        /// Timestamp method (rfc3161, ots, auto)
        #[arg(short, long, default_value = "auto")]
        method: Option<String>,

        /// TSA server URL (for rfc3161, uses defaults if not specified)
        #[arg(short, long)]
        server: Option<String>,

        /// Output file (default: overwrite input)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Encrypt a document with password-based encryption
    Encrypt {
        /// Codex document to encrypt
        file: PathBuf,

        /// Password (will prompt if not provided)
        #[arg(short, long)]
        password: Option<String>,

        /// Output file (default: overwrite input)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Decrypt a password-encrypted document
    Decrypt {
        /// Codex document to decrypt
        file: PathBuf,

        /// Password (will prompt if not provided)
        #[arg(short, long)]
        password: Option<String>,

        /// Output file (default: overwrite input)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() {
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

    let result = run_command(cli.command, &output_config);

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

#[allow(clippy::too_many_lines)]
fn run_command(command: Commands, output_config: &output::OutputConfig) -> Result<()> {
    match command {
        Commands::Create {
            title,
            author,
            state,
            input,
            output: output_path,
        } => commands::create::run(&title, &author, &state, input, &output_path, output_config),

        Commands::Validate { file } => commands::validate::run(&file, output_config),

        Commands::Inspect {
            file,
            blocks,
            signatures,
            provenance,
        } => commands::inspect::run(&file, blocks, signatures, provenance, output_config),

        Commands::Status { file } => commands::status::run(&file, output_config),

        Commands::Sign {
            file,
            key,
            name,
            email,
            algorithm,
            output: output_path,
        } => commands::sign::run(
            &file,
            &key,
            &name,
            email,
            &algorithm,
            output_path,
            output_config,
        ),

        Commands::Verify { file, key } => commands::verify::run(&file, &key, output_config),

        Commands::Extract {
            file,
            output: output_path,
            content,
            text,
            asset,
            all_assets,
        } => commands::extract::run(
            &file,
            &output_path,
            content,
            text,
            asset.as_deref(),
            all_assets,
            output_config,
        ),

        Commands::Completions { shell } => {
            generate(shell, &mut Cli::command(), "cdx", &mut io::stdout());
            Ok(())
        }

        Commands::SubmitReview { file, output } => {
            commands::review::run(&file, output, output_config)
        }

        Commands::Freeze { file, output } => commands::freeze::run(&file, output, output_config),

        Commands::Publish { file, output } => commands::publish::run(&file, output, output_config),

        Commands::Revert { file, output } => commands::revert::run(&file, output, output_config),

        Commands::Fork { file, output, note } => {
            commands::fork::run(&file, &output, note, output_config)
        }

        Commands::Prove {
            file,
            block_id,
            block_index,
            output,
        } => commands::prove::run_prove(&file, block_id, block_index, output, output_config),

        Commands::VerifyProof { file, proof } => {
            commands::prove::run_verify_proof(&file, &proof, output_config)
        }

        Commands::ShowLineage { file } => commands::prove::run_show_lineage(&file, output_config),

        Commands::GetMetadata { file } => {
            commands::metadata::run_get_metadata(&file, output_config)
        }

        Commands::SetMetadata {
            file,
            title,
            creator,
            subject,
            description,
            publisher,
            language,
            rights,
            output,
        } => commands::metadata::run_set_metadata(
            file,
            title,
            &creator,
            &subject,
            description,
            publisher,
            language,
            rights,
            output,
            output_config,
        ),

        Commands::Pack {
            input,
            output: output_path,
            from_json,
        } => commands::pack::run(&input, &output_path, from_json, output_config),

        Commands::Diff { file1, file2 } => commands::diff::run(&file1, &file2, output_config),

        Commands::ShowTimestamps { file } => {
            commands::timestamp::run_show_timestamps(&file, output_config)
        }

        Commands::VerifyTimestamps { file } => {
            commands::timestamp::run_verify_timestamps(&file, output_config)
        }

        Commands::AddTimestamp {
            file,
            method,
            authority,
            token,
            time,
            transaction_id,
            output,
        } => commands::timestamp::run_add_timestamp(
            &file,
            &method,
            authority,
            token,
            time,
            transaction_id,
            output,
            output_config,
        ),

        Commands::TimestampAcquire {
            file,
            method,
            server,
            output,
        } => commands::timestamp::run_acquire_timestamp(
            &file,
            method.as_deref(),
            server.as_deref(),
            output,
            output_config,
        ),

        Commands::Encrypt {
            file,
            password,
            output,
        } => commands::encrypt::run(&file, password, output, output_config),

        Commands::Decrypt {
            file,
            password,
            output,
        } => commands::decrypt::run(&file, password, output, output_config),
    }
}
