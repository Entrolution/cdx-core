//! Encryption support using AES-256-GCM.
//!
//! This module provides encryption and decryption capabilities for Codex documents
//! using the AES-256-GCM authenticated encryption algorithm.

use serde::{Deserialize, Serialize};

use crate::Result;

/// Encryption algorithm enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    /// AES-256-GCM (required).
    #[serde(rename = "AES-256-GCM")]
    Aes256Gcm,
}

impl EncryptionAlgorithm {
    /// Get the algorithm identifier string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Aes256Gcm => "AES-256-GCM",
        }
    }

    /// Get the key size in bytes.
    #[must_use]
    pub const fn key_size(&self) -> usize {
        match self {
            Self::Aes256Gcm => 32, // 256 bits
        }
    }

    /// Get the nonce size in bytes.
    #[must_use]
    pub const fn nonce_size(&self) -> usize {
        match self {
            Self::Aes256Gcm => 12, // 96 bits
        }
    }
}

impl std::fmt::Display for EncryptionAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Encryption metadata stored in the document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptionMetadata {
    /// Encryption algorithm used.
    pub algorithm: EncryptionAlgorithm,

    /// Key derivation function (if password-based).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kdf: Option<KeyDerivation>,

    /// Encrypted content key (if key wrapping is used).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wrapped_key: Option<String>,

    /// Recipients who can decrypt (for multi-recipient encryption).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recipients: Vec<Recipient>,
}

/// Key derivation function parameters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyDerivation {
    /// KDF algorithm.
    pub algorithm: KdfAlgorithm,

    /// Salt (base64 encoded).
    pub salt: String,

    /// Iteration count (for PBKDF2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iterations: Option<u32>,

    /// Memory parameter (for Argon2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<u32>,

    /// Parallelism parameter (for Argon2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parallelism: Option<u32>,
}

/// Key derivation algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KdfAlgorithm {
    /// PBKDF2 with HMAC-SHA256.
    #[serde(rename = "PBKDF2-SHA256")]
    Pbkdf2Sha256,
    /// Argon2id (recommended).
    Argon2id,
}

/// Recipient information for multi-recipient encryption.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Recipient {
    /// Recipient identifier (e.g., key ID, email).
    pub id: String,

    /// Encrypted content key for this recipient (base64 encoded).
    pub encrypted_key: String,

    /// Key encryption algorithm.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<String>,
}

/// Result of encryption operation.
#[derive(Debug, Clone)]
pub struct EncryptedData {
    /// The encrypted ciphertext.
    pub ciphertext: Vec<u8>,

    /// The nonce used for encryption.
    pub nonce: Vec<u8>,

    /// Authentication tag (included in ciphertext for GCM).
    pub tag: Vec<u8>,
}

/// AES-256-GCM encryptor.
#[cfg(feature = "encryption")]
pub struct Aes256GcmEncryptor {
    key: [u8; 32],
}

#[cfg(feature = "encryption")]
impl Aes256GcmEncryptor {
    /// Create a new encryptor with the given key.
    ///
    /// # Errors
    ///
    /// Returns an error if the key is not 32 bytes.
    pub fn new(key: &[u8]) -> Result<Self> {
        let key: [u8; 32] = key.try_into().map_err(|_| crate::Error::InvalidManifest {
            reason: format!("Invalid key length: expected 32 bytes, got {}", key.len()),
        })?;
        Ok(Self { key })
    }

    /// Generate a new random encryption key.
    #[must_use]
    pub fn generate_key() -> [u8; 32] {
        use rand_core::RngCore;
        let mut key = [0u8; 32];
        rand_core::OsRng.fill_bytes(&mut key);
        key
    }

    /// Generate a random nonce.
    #[must_use]
    pub fn generate_nonce() -> [u8; 12] {
        use rand_core::RngCore;
        let mut nonce = [0u8; 12];
        rand_core::OsRng.fill_bytes(&mut nonce);
        nonce
    }

    /// Encrypt data with a random nonce.
    ///
    /// # Errors
    ///
    /// Returns an error if encryption fails.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData> {
        self.encrypt_with_nonce(plaintext, &Self::generate_nonce())
    }

    /// Encrypt data with a specific nonce.
    ///
    /// # Errors
    ///
    /// Returns an error if encryption fails.
    pub fn encrypt_with_nonce(&self, plaintext: &[u8], nonce: &[u8; 12]) -> Result<EncryptedData> {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };

        let cipher =
            Aes256Gcm::new_from_slice(&self.key).map_err(|e| crate::Error::InvalidManifest {
                reason: format!("Failed to create cipher: {e}"),
            })?;

        #[allow(deprecated)] // generic-array 1.x transition
        let nonce_obj = Nonce::from_slice(nonce);
        let ciphertext =
            cipher
                .encrypt(nonce_obj, plaintext)
                .map_err(|e| crate::Error::InvalidManifest {
                    reason: format!("Encryption failed: {e}"),
                })?;

        // GCM appends the tag to the ciphertext
        let tag_start = ciphertext.len().saturating_sub(16);
        let tag = ciphertext[tag_start..].to_vec();

        Ok(EncryptedData {
            ciphertext,
            nonce: nonce.to_vec(),
            tag,
        })
    }

    /// Decrypt data.
    ///
    /// # Errors
    ///
    /// Returns an error if decryption fails (wrong key or tampered data).
    pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>> {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };

        let nonce: [u8; 12] = nonce
            .try_into()
            .map_err(|_| crate::Error::InvalidManifest {
                reason: format!(
                    "Invalid nonce length: expected 12 bytes, got {}",
                    nonce.len()
                ),
            })?;

        let cipher =
            Aes256Gcm::new_from_slice(&self.key).map_err(|e| crate::Error::InvalidManifest {
                reason: format!("Failed to create cipher: {e}"),
            })?;

        #[allow(deprecated)] // generic-array 1.x transition
        let nonce_obj = Nonce::from_slice(&nonce);
        cipher
            .decrypt(nonce_obj, ciphertext)
            .map_err(|e| crate::Error::InvalidManifest {
                reason: format!("Decryption failed: {e}"),
            })
    }
}

#[cfg(all(test, feature = "encryption"))]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = Aes256GcmEncryptor::generate_key();
        let encryptor = Aes256GcmEncryptor::new(&key).unwrap();

        let plaintext = b"Hello, World! This is a test message.";
        let encrypted = encryptor.encrypt(plaintext).unwrap();

        assert_ne!(&encrypted.ciphertext[..plaintext.len()], plaintext);

        let decrypted = encryptor
            .decrypt(&encrypted.ciphertext, &encrypted.nonce)
            .unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = Aes256GcmEncryptor::generate_key();
        let key2 = Aes256GcmEncryptor::generate_key();

        let encryptor1 = Aes256GcmEncryptor::new(&key1).unwrap();
        let encryptor2 = Aes256GcmEncryptor::new(&key2).unwrap();

        let plaintext = b"Secret message";
        let encrypted = encryptor1.encrypt(plaintext).unwrap();

        let result = encryptor2.decrypt(&encrypted.ciphertext, &encrypted.nonce);
        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_data_fails() {
        let key = Aes256GcmEncryptor::generate_key();
        let encryptor = Aes256GcmEncryptor::new(&key).unwrap();

        let plaintext = b"Original message";
        let mut encrypted = encryptor.encrypt(plaintext).unwrap();

        // Tamper with the ciphertext
        if !encrypted.ciphertext.is_empty() {
            encrypted.ciphertext[0] ^= 0xFF;
        }

        let result = encryptor.decrypt(&encrypted.ciphertext, &encrypted.nonce);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_plaintext() {
        let key = Aes256GcmEncryptor::generate_key();
        let encryptor = Aes256GcmEncryptor::new(&key).unwrap();

        let plaintext = b"";
        let encrypted = encryptor.encrypt(plaintext).unwrap();
        let decrypted = encryptor
            .decrypt(&encrypted.ciphertext, &encrypted.nonce)
            .unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn test_large_plaintext() {
        let key = Aes256GcmEncryptor::generate_key();
        let encryptor = Aes256GcmEncryptor::new(&key).unwrap();

        // 1 MB of data
        let plaintext: Vec<u8> = (0..1024 * 1024).map(|i| (i % 256) as u8).collect();
        let encrypted = encryptor.encrypt(&plaintext).unwrap();
        let decrypted = encryptor
            .decrypt(&encrypted.ciphertext, &encrypted.nonce)
            .unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encryption_metadata_serialization() {
        let metadata = EncryptionMetadata {
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            kdf: Some(KeyDerivation {
                algorithm: KdfAlgorithm::Argon2id,
                salt: "base64salt".to_string(),
                iterations: None,
                memory: Some(65536),
                parallelism: Some(4),
            }),
            wrapped_key: None,
            recipients: vec![],
        };

        let json = serde_json::to_string_pretty(&metadata).unwrap();
        assert!(json.contains("AES-256-GCM"));
        assert!(json.contains("Argon2id"));

        let deserialized: EncryptionMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.algorithm, metadata.algorithm);
    }
}
