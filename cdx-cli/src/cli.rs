//! CLI argument definitions.

use clap::{Parser, Subcommand};
use clap_complete::Shell;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cdx")]
#[command(author, version, about = "CDX Document Format CLI", long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Increase output verbosity
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress non-error output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Output as JSON (for scripting)
    #[arg(long, global = true)]
    pub json: bool,

    /// Color output control
    #[arg(long, value_enum, default_value = "auto", global = true)]
    pub color: ColorChoice,
}

#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new CDX document
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
        /// CDX document to validate
        file: PathBuf,
    },

    /// Display document information
    Inspect {
        /// CDX document to inspect
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
        /// CDX document to check
        file: PathBuf,
    },

    /// Add a digital signature
    Sign {
        /// CDX document to sign
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
        /// CDX document to verify
        file: PathBuf,

        /// Public key file(s) for signature verification
        #[arg(short, long)]
        key: Vec<PathBuf>,
    },

    /// Extract content or assets
    Extract {
        /// CDX document to extract from
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
        /// CDX document to submit
        file: PathBuf,

        /// Output file (default: overwrite input)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Freeze document (review → frozen)
    Freeze {
        /// CDX document to freeze
        file: PathBuf,

        /// Output file (default: overwrite input)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Publish document (frozen → published)
    Publish {
        /// CDX document to publish
        file: PathBuf,

        /// Output file (default: overwrite input)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Revert document to draft (review → draft)
    Revert {
        /// CDX document to revert
        file: PathBuf,

        /// Output file (default: overwrite input)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Fork document to create new version with lineage
    Fork {
        /// CDX document to fork
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
        /// CDX document
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
        /// CDX document
        file: PathBuf,

        /// Proof JSON file
        proof: PathBuf,
    },

    /// Show document lineage (ancestor chain)
    #[command(name = "show-lineage")]
    ShowLineage {
        /// CDX document
        file: PathBuf,
    },

    /// Display document metadata
    #[command(name = "get-metadata")]
    GetMetadata {
        /// CDX document
        file: PathBuf,
    },

    /// Set document metadata fields
    #[command(name = "set-metadata")]
    SetMetadata {
        /// CDX document
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

    /// Compare two CDX documents
    Diff {
        /// First document
        file1: PathBuf,

        /// Second document
        file2: PathBuf,
    },

    /// Show timestamps in a document
    #[command(name = "show-timestamps")]
    ShowTimestamps {
        /// CDX document
        file: PathBuf,
    },

    /// Verify timestamps in a document
    #[command(name = "verify-timestamps")]
    VerifyTimestamps {
        /// CDX document
        file: PathBuf,
    },

    /// Add a timestamp record to a document
    #[command(name = "add-timestamp")]
    AddTimestamp {
        /// CDX document
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
        /// CDX document to timestamp
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
        /// CDX document to encrypt
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
        /// CDX document to decrypt
        file: PathBuf,

        /// Password (will prompt if not provided)
        #[arg(short, long)]
        password: Option<String>,

        /// Output file (default: overwrite input)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}
