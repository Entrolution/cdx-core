//! Encrypt command implementation.
//!
//! Encrypts a Codex document using AES-256-GCM with password-based key derivation.

#[cfg(feature = "encryption")]
use anyhow::Context;
use anyhow::Result;
#[cfg(feature = "encryption")]
use colored::Colorize;
use std::path::PathBuf;

use crate::output::OutputConfig;

#[cfg(feature = "encryption")]
use cdx_core::security::{EncryptionAlgorithm, EncryptionMetadata, KdfAlgorithm, KeyDerivation};
#[cfg(feature = "encryption")]
use cdx_core::Document;

/// Run the encrypt command.
#[allow(unused_variables)]
pub fn run(
    file: PathBuf,
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
            return Ok(());
        } else {
            anyhow::bail!(
                "Encryption feature not enabled. Rebuild with: cargo build --features encryption"
            );
        }
    }

    #[cfg(feature = "encryption")]
    {
        config.verbose(&format!("Encrypting document: {}", file.display()));

        // Open the document
        let mut doc = Document::open(&file)
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
            } else {
                anyhow::bail!("Document is already encrypted. Decrypt it first.");
            }
        }

        // Check document state
        if doc.state().is_immutable() {
            anyhow::bail!("Cannot encrypt: document is in {} state", doc.state());
        }

        // Get password (from argument or prompt)
        let password = if let Some(pwd) = password {
            pwd
        } else {
            prompt_password("Enter encryption password: ")?
        };

        if password.is_empty() {
            anyhow::bail!("Password cannot be empty");
        }

        // Confirm password
        if !config.quiet {
            let confirm = prompt_password("Confirm password: ")?;
            if password != confirm {
                anyhow::bail!("Passwords do not match");
            }
        }

        // Generate salt for key derivation
        let salt = generate_salt();
        let salt_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &salt);

        // Derive key from password using Argon2id
        let key = derive_key(&password, &salt)?;

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
        let output_path = output.unwrap_or_else(|| file.clone());

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

#[cfg(feature = "encryption")]
fn prompt_password(prompt: &str) -> Result<String> {
    use std::io::{self, Write};

    print!("{}", prompt);
    io::stdout().flush()?;

    // Try to use rpassword for hidden input, fall back to regular input
    #[cfg(feature = "rpassword")]
    {
        rpassword::read_password().context("Failed to read password")
    }

    #[cfg(not(feature = "rpassword"))]
    {
        let mut password = String::new();
        io::stdin().read_line(&mut password)?;
        Ok(password.trim().to_string())
    }
}

#[cfg(feature = "encryption")]
fn generate_salt() -> [u8; 16] {
    use rand_core::RngCore;
    let mut salt = [0u8; 16];
    rand_core::OsRng.fill_bytes(&mut salt);
    salt
}

#[cfg(feature = "encryption")]
fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32]> {
    use argon2::Argon2;

    let mut key = [0u8; 32];

    // Use Argon2id with recommended parameters
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(65536, 3, 4, Some(32))
            .map_err(|e| anyhow::anyhow!("Failed to configure Argon2: {}", e))?,
    );

    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| anyhow::anyhow!("Failed to derive key: {}", e))?;

    Ok(key)
}
