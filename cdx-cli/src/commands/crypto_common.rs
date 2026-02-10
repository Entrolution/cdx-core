//! Shared cryptographic helpers for encrypt and decrypt commands.

#[cfg(feature = "encryption")]
use anyhow::{Context, Result};

/// Prompt the user for a password, using hidden input if available.
#[cfg(feature = "encryption")]
pub fn prompt_password(prompt: &str) -> Result<String> {
    use std::io::{self, Write};

    print!("{prompt}");
    io::stdout().flush()?;

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

/// Generate a random 16-byte salt for key derivation.
#[cfg(feature = "encryption")]
pub fn generate_salt() -> [u8; 16] {
    use rand_core::RngCore;
    let mut salt = [0u8; 16];
    rand_core::OsRng.fill_bytes(&mut salt);
    salt
}

/// Derive a 32-byte key from a password and salt using Argon2id.
#[cfg(feature = "encryption")]
pub fn derive_key_argon2(
    password: &str,
    salt: &[u8],
    memory: u32,
    parallelism: u32,
) -> Result<[u8; 32]> {
    use argon2::Argon2;

    let mut key = [0u8; 32];

    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(memory, 3, parallelism, Some(32))
            .map_err(|e| anyhow::anyhow!("Failed to configure Argon2: {e}"))?,
    );

    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| anyhow::anyhow!("Failed to derive key: {e}"))?;

    Ok(key)
}

/// Derive a 32-byte key from a password and salt using PBKDF2-SHA256.
#[cfg(feature = "encryption")]
pub fn derive_key_pbkdf2(password: &str, salt: &[u8], iterations: u32) -> Result<[u8; 32]> {
    use hmac::Hmac;
    use pbkdf2::pbkdf2;
    use sha2::Sha256;

    let mut key = [0u8; 32];

    pbkdf2::<Hmac<Sha256>>(password.as_bytes(), salt, iterations, &mut key)
        .map_err(|e| anyhow::anyhow!("Failed to derive key with PBKDF2: {e}"))?;

    Ok(key)
}
