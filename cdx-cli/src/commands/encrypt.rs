//! Encrypt command implementation.
//!
//! Encrypts a Codex document using AES-256-GCM with password-based key derivation.

#[cfg(feature = "encryption")]
use anyhow::Context;
use anyhow::Result;
#[cfg(feature = "encryption")]
use colored::Colorize;
use std::path::{Path, PathBuf};

use crate::output::OutputConfig;

#[cfg(feature = "encryption")]
use cdx_core::security::{EncryptionAlgorithm, EncryptionMetadata, KdfAlgorithm, KeyDerivation};
#[cfg(feature = "encryption")]
use cdx_core::Document;

/// Run the encrypt command.
#[allow(unused_variables)]
pub fn run(
    file: &Path,
    password: Option<String>,
    output: Option<PathBuf>,
    config: &OutputConfig,
) -> Result<()> {
    #[cfg(not(feature = "encryption"))]
    {
        if config.json {
            let result = serde_json::json!({
                "error": "Encryption feature not enabled",
                "message": "Rebuild with --features encryption to enable encryption"
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
            Ok(())
        } else {
            anyhow::bail!(
                "Encryption feature not enabled. Rebuild with: cargo build --features encryption"
            )
        }
    }

    #[cfg(feature = "encryption")]
    {
        use super::crypto_common;

        config.verbose(&format!("Encrypting document: {}", file.display()));

        // Open the document
        let mut doc = Document::open(file)
            .with_context(|| format!("Failed to open document: {}", file.display()))?;

        // Check if document is already encrypted
        if doc.is_encrypted() {
            if config.json {
                let result = serde_json::json!({
                    "error": "Document is already encrypted",
                    "file": file.display().to_string()
                });
                println!("{}", serde_json::to_string_pretty(&result)?);
                return Ok(());
            }
            anyhow::bail!("Document is already encrypted. Decrypt it first.");
        }

        // Check document state
        if doc.state().is_immutable() {
            anyhow::bail!("Cannot encrypt: document is in {} state", doc.state());
        }

        // Get password (from argument or prompt)
        let password = if let Some(pwd) = password {
            pwd
        } else {
            crypto_common::prompt_password("Enter encryption password: ")?
        };

        if password.is_empty() {
            anyhow::bail!("Password cannot be empty");
        }

        // Confirm password
        if !config.quiet {
            let confirm = crypto_common::prompt_password("Confirm password: ")?;
            if password != confirm {
                anyhow::bail!("Passwords do not match");
            }
        }

        // Generate salt for key derivation
        let salt = crypto_common::generate_salt();
        let salt_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, salt);

        // Derive key from password using Argon2id
        let _key = crypto_common::derive_key_argon2(&password, &salt, 65536, 4)?;

        // Create encryption metadata
        let encryption_metadata = EncryptionMetadata {
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            kdf: Some(KeyDerivation {
                algorithm: KdfAlgorithm::Argon2id,
                salt: salt_b64.clone(),
                iterations: None,
                memory: Some(65536), // 64 MB
                parallelism: Some(4),
            }),
            wrapped_key: None,
            recipients: vec![],
        };

        // Set encryption metadata on document
        doc.set_encryption(encryption_metadata)?;

        // Determine output path
        let output_path = output.unwrap_or_else(|| file.to_path_buf());

        // Save the document
        doc.save(&output_path).with_context(|| {
            format!(
                "Failed to save encrypted document: {}",
                output_path.display()
            )
        })?;

        if config.json {
            let result = serde_json::json!({
                "status": "success",
                "file": output_path.display().to_string(),
                "algorithm": "AES-256-GCM",
                "kdf": "Argon2id",
                "message": "Document encrypted successfully"
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            config.success(&format!(
                "Document encrypted successfully: {}",
                output_path.display()
            ));
            println!();
            config.field("Algorithm", "AES-256-GCM");
            config.field("Key Derivation", "Argon2id");
            println!();
            println!(
                "{} Store your password securely. Lost passwords cannot be recovered.",
                "Warning:".yellow().bold()
            );
        }

        Ok(())
    }
}
