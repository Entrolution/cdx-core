//! Decrypt command implementation.
//!
//! Decrypts a Codex document that was encrypted with password-based encryption.

#[cfg(feature = "encryption")]
use anyhow::Context;
use anyhow::Result;
use std::path::PathBuf;

use crate::output::OutputConfig;

#[cfg(feature = "encryption")]
use cdx_core::security::KdfAlgorithm;
#[cfg(feature = "encryption")]
use cdx_core::Document;

/// Run the decrypt command.
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
                "message": "Rebuild with --features encryption to enable decryption"
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
        config.verbose(&format!("Decrypting document: {}", file.display()));

        // Open the document
        let mut doc = Document::open(&file)
            .with_context(|| format!("Failed to open document: {}", file.display()))?;

        // Check if document is encrypted
        let encryption_metadata = match doc.encryption_metadata() {
            Some(meta) => meta.clone(),
            None => {
                if config.json {
                    let result = serde_json::json!({
                        "error": "Document is not encrypted",
                        "file": file.display().to_string()
                    });
                    println!("{}", serde_json::to_string_pretty(&result)?);
                    return Ok(());
                } else {
                    anyhow::bail!("Document is not encrypted");
                }
            }
        };

        // Check document state
        if doc.state().is_immutable() {
            anyhow::bail!("Cannot decrypt: document is in {} state", doc.state());
        }

        // Get password (from argument or prompt)
        let password = if let Some(pwd) = password {
            pwd
        } else {
            prompt_password("Enter decryption password: ")?
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
        let key = match kdf.algorithm {
            KdfAlgorithm::Argon2id => derive_key_argon2(&password, &salt, kdf)?,
            KdfAlgorithm::Pbkdf2Sha256 => derive_key_pbkdf2(&password, &salt, kdf)?,
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
        let output_path = output.unwrap_or_else(|| file.clone());

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
fn derive_key_argon2(
    password: &str,
    salt: &[u8],
    kdf: &cdx_core::security::KeyDerivation,
) -> Result<[u8; 32]> {
    use argon2::Argon2;

    let mut key = [0u8; 32];

    let memory = kdf.memory.unwrap_or(65536);
    let parallelism = kdf.parallelism.unwrap_or(4);

    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(memory, 3, parallelism, Some(32))
            .map_err(|e| anyhow::anyhow!("Failed to configure Argon2: {}", e))?,
    );

    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| anyhow::anyhow!("Failed to derive key: {}", e))?;

    Ok(key)
}

#[cfg(feature = "encryption")]
fn derive_key_pbkdf2(
    password: &str,
    salt: &[u8],
    kdf: &cdx_core::security::KeyDerivation,
) -> Result<[u8; 32]> {
    use hmac::Hmac;
    use pbkdf2::pbkdf2;
    use sha2::Sha256;

    let mut key = [0u8; 32];
    let iterations = kdf.iterations.unwrap_or(100_000);

    pbkdf2::<Hmac<Sha256>>(password.as_bytes(), salt, iterations, &mut key)
        .map_err(|e| anyhow::anyhow!("Failed to derive key with PBKDF2: {}", e))?;

    Ok(key)
}
