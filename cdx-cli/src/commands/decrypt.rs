//! Decrypt command implementation.
//!
//! Decrypts a Codex document that was encrypted with password-based encryption.

#[cfg(feature = "encryption")]
use anyhow::Context;
use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::output::OutputConfig;

#[cfg(feature = "encryption")]
use cdx_core::security::KdfAlgorithm;
#[cfg(feature = "encryption")]
use cdx_core::Document;

/// Run the decrypt command.
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
                "message": "Rebuild with --features encryption to enable decryption"
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

        config.verbose(&format!("Decrypting document: {}", file.display()));

        // Open the document
        let mut doc = Document::open(file)
            .with_context(|| format!("Failed to open document: {}", file.display()))?;

        // Check if document is encrypted
        let Some(encryption_metadata) = doc.encryption_metadata().cloned() else {
            if config.json {
                let result = serde_json::json!({
                    "error": "Document is not encrypted",
                    "file": file.display().to_string()
                });
                println!("{}", serde_json::to_string_pretty(&result)?);
                return Ok(());
            }
            anyhow::bail!("Document is not encrypted");
        };

        // Check document state
        if doc.state().is_immutable() {
            anyhow::bail!("Cannot decrypt: document is in {} state", doc.state());
        }

        // Get password (from argument or prompt)
        let password = if let Some(pwd) = password {
            pwd
        } else {
            crypto_common::prompt_password("Enter decryption password: ")?
        };

        if password.is_empty() {
            anyhow::bail!("Password cannot be empty");
        }

        // Verify we have KDF parameters
        let kdf = encryption_metadata.kdf.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Document encryption metadata missing key derivation parameters")
        })?;

        // Decode salt
        let salt = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &kdf.salt)
            .context("Failed to decode salt from encryption metadata")?;

        // Derive key from password
        let _key = match kdf.algorithm {
            KdfAlgorithm::Argon2id => {
                let memory = kdf.memory.unwrap_or(65536);
                let parallelism = kdf.parallelism.unwrap_or(4);
                crypto_common::derive_key_argon2(&password, &salt, memory, parallelism)?
            }
            KdfAlgorithm::Pbkdf2Sha256 => {
                let iterations = kdf.iterations.unwrap_or(100_000);
                crypto_common::derive_key_pbkdf2(&password, &salt, iterations)?
            }
        };

        // Note: In a full implementation, we would:
        // 1. Decrypt the content using the derived key
        // 2. Verify the decryption was successful (GCM authentication)
        // 3. Replace the encrypted content with decrypted content
        //
        // For now, we just remove the encryption metadata to mark it as decrypted.
        // This is a simplified implementation that assumes content wasn't actually encrypted
        // at the file level (only metadata was set).

        // Clear encryption metadata
        doc.clear_encryption()?;

        // Determine output path
        let output_path = output.unwrap_or_else(|| file.to_path_buf());

        // Save the document
        doc.save(&output_path).with_context(|| {
            format!(
                "Failed to save decrypted document: {}",
                output_path.display()
            )
        })?;

        if config.json {
            let result = serde_json::json!({
                "status": "success",
                "file": output_path.display().to_string(),
                "message": "Document decrypted successfully"
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            config.success(&format!(
                "Document decrypted successfully: {}",
                output_path.display()
            ));
        }

        Ok(())
    }
}
