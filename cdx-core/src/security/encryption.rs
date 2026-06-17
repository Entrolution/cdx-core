//! Encryption support using AES-256-GCM and ChaCha20-Poly1305.
//!
//! This module provides encryption and decryption capabilities for CDX documents
//! using authenticated encryption algorithms (AEAD).

use serde::{Deserialize, Serialize};

use crate::error::encryption_error;
use crate::Result;

/// Encryption algorithm enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display)]
pub enum EncryptionAlgorithm {
    /// AES-256-GCM (required).
    #[serde(rename = "AES-256-GCM")]
    #[strum(serialize = "AES-256-GCM")]
    Aes256Gcm,
    /// ChaCha20-Poly1305 (optional).
    #[serde(rename = "ChaCha20-Poly1305")]
    #[strum(serialize = "ChaCha20-Poly1305")]
    ChaCha20Poly1305,
}

impl EncryptionAlgorithm {
    /// Get the algorithm identifier string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Aes256Gcm => "AES-256-GCM",
            Self::ChaCha20Poly1305 => "ChaCha20-Poly1305",
        }
    }

    /// Get the key size in bytes.
    #[must_use]
    pub const fn key_size(&self) -> usize {
        match self {
            // Both algorithms use 256-bit keys
            Self::Aes256Gcm | Self::ChaCha20Poly1305 => 32,
        }
    }

    /// Get the nonce size in bytes.
    #[must_use]
    pub const fn nonce_size(&self) -> usize {
        match self {
            // Both algorithms use 96-bit nonces
            Self::Aes256Gcm | Self::ChaCha20Poly1305 => 12,
        }
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

    /// Key management algorithm (for key wrapping).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_management: Option<KeyManagementAlgorithm>,

    /// Recipients who can decrypt (for multi-recipient encryption).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recipients: Vec<Recipient>,
}

/// Key management algorithm for key wrapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyManagementAlgorithm {
    /// ECDH-ES with AES-256 Key Wrap.
    #[serde(rename = "ECDH-ES+A256KW")]
    EcdhEsA256kw,
    /// RSA-OAEP with SHA-256.
    #[serde(rename = "RSA-OAEP-256")]
    RsaOaep256,
    /// PBES2-HS256 with AES-256 Key Wrap (password-based).
    #[serde(rename = "PBES2-HS256+A256KW")]
    Pbes2HsA256kw,
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

    /// Ephemeral public key (base64-encoded, for ECDH-ES key agreement).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ephemeral_public_key: Option<String>,
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
#[derive(zeroize::ZeroizeOnDrop)]
pub struct Aes256GcmEncryptor {
    key: [u8; 32],
}

#[cfg(feature = "encryption")]
#[allow(clippy::missing_panics_doc)] // getrandom::fill only fails on misconfigured systems
impl Aes256GcmEncryptor {
    /// Create a new encryptor with the given key.
    ///
    /// # Errors
    ///
    /// Returns an error if the key is not 32 bytes.
    pub fn new(key: &[u8]) -> Result<Self> {
        let key: [u8; 32] = key.try_into().map_err(|_| {
            encryption_error(format!(
                "Invalid key length: expected 32 bytes, got {}",
                key.len()
            ))
        })?;
        Ok(Self { key })
    }

    /// Generate a new random encryption key.
    #[must_use]
    pub fn generate_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        getrandom::fill(&mut key).expect("system RNG failed");
        key
    }

    /// Generate a random nonce.
    #[must_use]
    pub fn generate_nonce() -> [u8; 12] {
        let mut nonce = [0u8; 12];
        getrandom::fill(&mut nonce).expect("system RNG failed");
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

        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| encryption_error(format!("Failed to create cipher: {e}")))?;

        let nonce_obj = Nonce::from(*nonce);
        let ciphertext = cipher
            .encrypt(&nonce_obj, plaintext)
            .map_err(|e| encryption_error(format!("Encryption failed: {e}")))?;

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

        let nonce: [u8; 12] = nonce.try_into().map_err(|_| {
            encryption_error(format!(
                "Invalid nonce length: expected 12 bytes, got {}",
                nonce.len()
            ))
        })?;

        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| encryption_error(format!("Failed to create cipher: {e}")))?;

        let nonce_obj = Nonce::from(nonce);
        cipher
            .decrypt(&nonce_obj, ciphertext)
            .map_err(|e| encryption_error(format!("Decryption failed: {e}")))
    }
}

/// ChaCha20-Poly1305 encryptor.
#[cfg(feature = "encryption-chacha")]
#[derive(zeroize::ZeroizeOnDrop)]
pub struct ChaCha20Poly1305Encryptor {
    key: [u8; 32],
}

#[cfg(feature = "encryption-chacha")]
#[allow(clippy::missing_panics_doc)] // getrandom::fill only fails on misconfigured systems
impl ChaCha20Poly1305Encryptor {
    /// Create a new encryptor with the given key.
    ///
    /// # Errors
    ///
    /// Returns an error if the key is not 32 bytes.
    pub fn new(key: &[u8]) -> Result<Self> {
        let key: [u8; 32] = key.try_into().map_err(|_| {
            encryption_error(format!(
                "Invalid key length: expected 32 bytes, got {}",
                key.len()
            ))
        })?;
        Ok(Self { key })
    }

    /// Generate a new random encryption key.
    #[must_use]
    pub fn generate_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        getrandom::fill(&mut key).expect("system RNG failed");
        key
    }

    /// Generate a random nonce.
    #[must_use]
    pub fn generate_nonce() -> [u8; 12] {
        let mut nonce = [0u8; 12];
        getrandom::fill(&mut nonce).expect("system RNG failed");
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
        use chacha20poly1305::{
            aead::{Aead, KeyInit},
            ChaCha20Poly1305, Nonce,
        };

        let cipher = ChaCha20Poly1305::new_from_slice(&self.key)
            .map_err(|e| encryption_error(format!("Failed to create cipher: {e}")))?;

        let nonce_obj = Nonce::from(*nonce);
        let ciphertext = cipher
            .encrypt(&nonce_obj, plaintext)
            .map_err(|e| encryption_error(format!("Encryption failed: {e}")))?;

        // Poly1305 appends the tag to the ciphertext (16 bytes)
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
        use chacha20poly1305::{
            aead::{Aead, KeyInit},
            ChaCha20Poly1305, Nonce,
        };

        let nonce: [u8; 12] = nonce.try_into().map_err(|_| {
            encryption_error(format!(
                "Invalid nonce length: expected 12 bytes, got {}",
                nonce.len()
            ))
        })?;

        let cipher = ChaCha20Poly1305::new_from_slice(&self.key)
            .map_err(|e| encryption_error(format!("Failed to create cipher: {e}")))?;

        let nonce_obj = Nonce::from(nonce);
        cipher
            .decrypt(&nonce_obj, ciphertext)
            .map_err(|e| encryption_error(format!("Decryption failed: {e}")))
    }
}

// ---------------------------------------------------------------------------
// ECDH-ES+A256KW key wrapping (RFC 3394 / RFC 7518)
// ---------------------------------------------------------------------------

/// Result of wrapping a content encryption key.
#[cfg(feature = "key-wrapping")]
#[derive(Debug, Clone)]
pub struct WrappedKeyData {
    /// The wrapped (encrypted) content encryption key.
    pub wrapped_key: Vec<u8>,
    /// The ephemeral public key (SEC1 uncompressed point), for transmission to the recipient.
    pub ephemeral_public_key: Vec<u8>,
}

/// ECDH-ES+A256KW key wrapper (sender side).
///
/// Generates an ephemeral P-256 keypair, performs ECDH to derive
/// a shared secret, runs HKDF-SHA256 to derive a 256-bit KEK,
/// then wraps the content encryption key with AES Key Wrap (RFC 3394).
#[cfg(feature = "key-wrapping")]
pub struct EcdhEsKeyWrapper {
    /// Recipient's P-256 public key.
    recipient_public_key: p256::PublicKey,
}

#[cfg(feature = "key-wrapping")]
impl EcdhEsKeyWrapper {
    /// Create a key wrapper for the given recipient public key.
    ///
    /// The public key should be the recipient's P-256 (secp256r1) public key.
    #[must_use]
    pub fn new(recipient_public_key: p256::PublicKey) -> Self {
        Self {
            recipient_public_key,
        }
    }

    /// Wrap a content encryption key for the recipient.
    ///
    /// Performs ECDH-ES+A256KW:
    /// 1. Generate ephemeral P-256 keypair
    /// 2. ECDH key agreement with recipient's public key
    /// 3. HKDF-SHA256 to derive a 256-bit KEK
    /// 4. AES Key Wrap (RFC 3394) the content encryption key
    ///
    /// # Errors
    ///
    /// Returns an error if the content key length is not a multiple of 8 bytes
    /// (as required by AES Key Wrap), or if any cryptographic operation fails.
    pub fn wrap(&self, content_key: &[u8]) -> Result<WrappedKeyData> {
        use aes_kw::{cipher::KeyInit, KwAes256};
        use hkdf::Hkdf;
        use p256::ecdh::EphemeralSecret;
        use p256::elliptic_curve::Generate;
        use sha2::Sha256;

        // 1. Generate ephemeral keypair
        let ephemeral_secret = EphemeralSecret::generate();
        let ephemeral_public = p256::PublicKey::from(&ephemeral_secret);

        // 2. ECDH key agreement
        let shared_secret = ephemeral_secret.diffie_hellman(&self.recipient_public_key);

        // 3. HKDF-SHA256 to derive KEK
        //    info string per RFC 7518 §4.6.2 "ECDH-ES+A256KW"
        let hkdf = Hkdf::<Sha256>::new(None, shared_secret.raw_secret_bytes());
        let mut kek_bytes = [0u8; 32];
        hkdf.expand(b"ECDH-ES+A256KW", &mut kek_bytes)
            .map_err(|e| encryption_error(format!("HKDF expansion failed: {e}")))?;

        // 4. AES Key Wrap (RFC 3394)
        let kek = KwAes256::new(&kek_bytes.into());
        let mut wrapped = vec![0u8; content_key.len() + 8]; // AES-KW adds 8-byte IV
        kek.wrap_key(content_key, &mut wrapped)
            .map_err(|e| encryption_error(format!("AES key wrap failed: {e}")))?;

        // Encode ephemeral public key as SEC1 uncompressed point
        let ephemeral_public_bytes = ephemeral_public.to_sec1_bytes().to_vec();

        Ok(WrappedKeyData {
            wrapped_key: wrapped,
            ephemeral_public_key: ephemeral_public_bytes,
        })
    }
}

/// ECDH-ES+A256KW key unwrapper (recipient side).
///
/// Uses the recipient's private key and the sender's ephemeral public key
/// to reverse the ECDH-ES+A256KW key wrapping and recover the content
/// encryption key.
#[cfg(feature = "key-wrapping")]
pub struct EcdhEsKeyUnwrapper {
    /// Recipient's P-256 secret key.
    recipient_secret: p256::SecretKey,
}

#[cfg(feature = "key-wrapping")]
impl EcdhEsKeyUnwrapper {
    /// Create a key unwrapper with the recipient's secret key.
    #[must_use]
    pub fn new(recipient_secret: p256::SecretKey) -> Self {
        Self { recipient_secret }
    }

    /// Unwrap a content encryption key.
    ///
    /// # Errors
    ///
    /// Returns an error if the ephemeral public key is invalid, the wrapped
    /// key is corrupted, or any cryptographic operation fails.
    pub fn unwrap(&self, data: &WrappedKeyData) -> Result<Vec<u8>> {
        use aes_kw::{cipher::KeyInit, KwAes256};
        use hkdf::Hkdf;
        use sha2::Sha256;

        // 1. Decode the ephemeral public key from SEC1 bytes
        let ephemeral_public = p256::PublicKey::from_sec1_bytes(&data.ephemeral_public_key)
            .map_err(|e| encryption_error(format!("Invalid ephemeral public key: {e}")))?;

        // 2. ECDH key agreement using recipient's secret key
        let shared_secret = p256::ecdh::diffie_hellman(
            self.recipient_secret.to_nonzero_scalar(),
            ephemeral_public.as_affine(),
        );

        // 3. HKDF-SHA256 to derive KEK (same parameters as wrap)
        let hkdf = Hkdf::<Sha256>::new(None, shared_secret.raw_secret_bytes());
        let mut kek_bytes = [0u8; 32];
        hkdf.expand(b"ECDH-ES+A256KW", &mut kek_bytes)
            .map_err(|e| encryption_error(format!("HKDF expansion failed: {e}")))?;

        // 4. AES Key Unwrap (wrapped key is original_len + 8 bytes)
        let kek = KwAes256::new(&kek_bytes.into());
        let unwrapped_len = data
            .wrapped_key
            .len()
            .checked_sub(8)
            .ok_or_else(|| encryption_error("Wrapped key too short"))?;
        let mut unwrapped = vec![0u8; unwrapped_len];
        kek.unwrap_key(&data.wrapped_key, &mut unwrapped)
            .map_err(|e| encryption_error(format!("AES key unwrap failed: {e}")))?;
        Ok(unwrapped)
    }
}

// ---------------------------------------------------------------------------
// RSA-OAEP-256 key wrapping
// ---------------------------------------------------------------------------

/// Result of wrapping a content encryption key with RSA-OAEP.
#[cfg(feature = "key-wrapping-rsa")]
#[derive(Debug, Clone)]
pub struct RsaWrappedKeyData {
    /// The wrapped (encrypted) content encryption key.
    pub wrapped_key: Vec<u8>,
}

/// RSA-OAEP-256 key wrapper (sender side).
///
/// Encrypts a content encryption key using the recipient's RSA public key
/// with OAEP padding and SHA-256 as the hash function.
#[cfg(feature = "key-wrapping-rsa")]
pub struct RsaOaepKeyWrapper {
    recipient_public_key: rsa::RsaPublicKey,
}

#[cfg(feature = "key-wrapping-rsa")]
impl RsaOaepKeyWrapper {
    /// Create a key wrapper for the given recipient RSA public key.
    #[must_use]
    pub fn new(recipient_public_key: rsa::RsaPublicKey) -> Self {
        Self {
            recipient_public_key,
        }
    }

    /// Wrap a content encryption key for the recipient.
    ///
    /// # Errors
    ///
    /// Returns an error if RSA-OAEP encryption fails (e.g., key too small for payload).
    pub fn wrap(&self, content_key: &[u8]) -> Result<RsaWrappedKeyData> {
        use rsa::oaep::EncryptingKey;
        use rsa::sha2::Sha256;
        use rsa::traits::RandomizedEncryptor;

        let encrypting_key = EncryptingKey::<Sha256>::new(self.recipient_public_key.clone());
        let wrapped_key = encrypting_key
            .encrypt_with_rng(&mut rand_core::UnwrapErr(getrandom::SysRng), content_key)
            .map_err(|e| encryption_error(format!("RSA-OAEP wrap failed: {e}")))?;

        Ok(RsaWrappedKeyData { wrapped_key })
    }
}

/// RSA-OAEP-256 key unwrapper (recipient side).
///
/// Decrypts a content encryption key using the recipient's RSA private key.
#[cfg(feature = "key-wrapping-rsa")]
pub struct RsaOaepKeyUnwrapper {
    recipient_private_key: rsa::RsaPrivateKey,
}

#[cfg(feature = "key-wrapping-rsa")]
impl RsaOaepKeyUnwrapper {
    /// Create a key unwrapper with the recipient's RSA private key.
    #[must_use]
    pub fn new(recipient_private_key: rsa::RsaPrivateKey) -> Self {
        Self {
            recipient_private_key,
        }
    }

    /// Unwrap a content encryption key.
    ///
    /// # Errors
    ///
    /// Returns an error if RSA-OAEP decryption fails (wrong key or tampered data).
    pub fn unwrap(&self, data: &RsaWrappedKeyData) -> Result<Vec<u8>> {
        use rsa::oaep::DecryptingKey;
        use rsa::sha2::Sha256;
        use rsa::traits::Decryptor;

        let decrypting_key = DecryptingKey::<Sha256>::new(self.recipient_private_key.clone());
        decrypting_key
            .decrypt(&data.wrapped_key)
            .map_err(|e| encryption_error(format!("RSA-OAEP unwrap failed: {e}")))
    }
}

// ---------------------------------------------------------------------------
// PBES2-HS256+A256KW password-based key wrapping
// ---------------------------------------------------------------------------

/// Result of wrapping a content encryption key with PBES2.
#[cfg(feature = "key-wrapping-pbes2")]
#[derive(Debug, Clone)]
pub struct Pbes2WrappedKeyData {
    /// The wrapped (encrypted) content encryption key.
    pub wrapped_key: Vec<u8>,
    /// The salt used for key derivation.
    pub salt: Vec<u8>,
    /// The PBKDF2 iteration count.
    pub iterations: u32,
}

/// PBES2-HS256+A256KW key wrapper (password-based).
///
/// Derives a 256-bit KEK from a password using PBKDF2-HMAC-SHA256,
/// then wraps the content encryption key with AES Key Wrap (RFC 3394).
#[cfg(feature = "key-wrapping-pbes2")]
pub struct Pbes2KeyWrapper {
    password: zeroize::Zeroizing<Vec<u8>>,
    iterations: u32,
}

#[cfg(feature = "key-wrapping-pbes2")]
impl Pbes2KeyWrapper {
    /// Default PBKDF2 iteration count (600,000).
    pub const DEFAULT_ITERATIONS: u32 = 600_000;

    /// Minimum allowed PBKDF2 iteration count.
    pub const MIN_ITERATIONS: u32 = 10_000;

    /// Maximum allowed PBKDF2 iteration count.
    pub const MAX_ITERATIONS: u32 = 10_000_000;

    /// Create a key wrapper with the given password and iteration count.
    ///
    /// # Errors
    ///
    /// Returns an error if `iterations` is outside the allowed range
    /// (`MIN_ITERATIONS..=MAX_ITERATIONS`).
    pub fn new(password: impl AsRef<[u8]>, iterations: u32) -> Result<Self> {
        if !(Self::MIN_ITERATIONS..=Self::MAX_ITERATIONS).contains(&iterations) {
            return Err(encryption_error(format!(
                "PBKDF2 iterations must be between {} and {}, got {iterations}",
                Self::MIN_ITERATIONS,
                Self::MAX_ITERATIONS
            )));
        }
        Ok(Self {
            password: zeroize::Zeroizing::new(password.as_ref().to_vec()),
            iterations,
        })
    }

    /// Wrap a content encryption key.
    ///
    /// Generates a random 16-byte salt, derives a KEK via PBKDF2-HMAC-SHA256,
    /// then wraps the content key with AES-256 Key Wrap.
    ///
    /// # Errors
    ///
    /// Returns an error if AES key wrapping fails.
    pub fn wrap(&self, content_key: &[u8]) -> Result<Pbes2WrappedKeyData> {
        use aes_kw::{cipher::KeyInit, KwAes256};

        // Generate random 16-byte salt
        let mut salt = [0u8; 16];
        getrandom::fill(&mut salt)
            .map_err(|e| encryption_error(format!("System RNG failed: {e}")))?;

        // Derive KEK via PBKDF2-HMAC-SHA256
        let mut kek_bytes = [0u8; 32];
        pbkdf2::pbkdf2_hmac::<sha2::Sha256>(&self.password, &salt, self.iterations, &mut kek_bytes);

        // AES Key Wrap
        let kek = KwAes256::new(&kek_bytes.into());
        let mut wrapped = vec![0u8; content_key.len() + 8];
        kek.wrap_key(content_key, &mut wrapped)
            .map_err(|e| encryption_error(format!("PBES2 AES key wrap failed: {e}")))?;

        Ok(Pbes2WrappedKeyData {
            wrapped_key: wrapped,
            salt: salt.to_vec(),
            iterations: self.iterations,
        })
    }
}

/// PBES2-HS256+A256KW key unwrapper (password-based).
///
/// Derives the same KEK from the password and salt/iterations stored
/// in the wrapped data, then unwraps the content encryption key.
#[cfg(feature = "key-wrapping-pbes2")]
pub struct Pbes2KeyUnwrapper {
    password: zeroize::Zeroizing<Vec<u8>>,
}

#[cfg(feature = "key-wrapping-pbes2")]
impl Pbes2KeyUnwrapper {
    /// Create a key unwrapper with the given password.
    #[must_use]
    pub fn new(password: impl AsRef<[u8]>) -> Self {
        Self {
            password: zeroize::Zeroizing::new(password.as_ref().to_vec()),
        }
    }

    /// Unwrap a content encryption key.
    ///
    /// Uses the salt and iteration count from the wrapped data to derive the KEK,
    /// then unwraps the content key.
    ///
    /// # Errors
    ///
    /// Returns an error if the password is wrong, the wrapped data is tampered,
    /// or the iteration count is outside the allowed range.
    pub fn unwrap(&self, data: &Pbes2WrappedKeyData) -> Result<Vec<u8>> {
        use aes_kw::{cipher::KeyInit, KwAes256};

        // Validate iteration count before doing expensive KDF work
        if !(Pbes2KeyWrapper::MIN_ITERATIONS..=Pbes2KeyWrapper::MAX_ITERATIONS)
            .contains(&data.iterations)
        {
            return Err(encryption_error(format!(
                "PBKDF2 iterations must be between {} and {}, got {}",
                Pbes2KeyWrapper::MIN_ITERATIONS,
                Pbes2KeyWrapper::MAX_ITERATIONS,
                data.iterations
            )));
        }

        // Derive KEK via PBKDF2-HMAC-SHA256 (same params as wrap)
        let mut kek_bytes = [0u8; 32];
        pbkdf2::pbkdf2_hmac::<sha2::Sha256>(
            &self.password,
            &data.salt,
            data.iterations,
            &mut kek_bytes,
        );

        // AES Key Unwrap
        let kek = KwAes256::new(&kek_bytes.into());
        let unwrapped_len = data
            .wrapped_key
            .len()
            .checked_sub(8)
            .ok_or_else(|| encryption_error("Wrapped key too short"))?;
        let mut unwrapped = vec![0u8; unwrapped_len];
        kek.unwrap_key(&data.wrapped_key, &mut unwrapped)
            .map_err(|e| encryption_error(format!("PBES2 AES key unwrap failed: {e}")))?;
        Ok(unwrapped)
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
            key_management: None,
            recipients: vec![],
        };

        let json = serde_json::to_string_pretty(&metadata).unwrap();
        assert!(json.contains("AES-256-GCM"));
        assert!(json.contains("Argon2id"));

        let deserialized: EncryptionMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.algorithm, metadata.algorithm);
    }

    #[test]
    fn test_key_management_algorithm_roundtrip() {
        let variants = [
            (KeyManagementAlgorithm::EcdhEsA256kw, "\"ECDH-ES+A256KW\""),
            (KeyManagementAlgorithm::RsaOaep256, "\"RSA-OAEP-256\""),
            (
                KeyManagementAlgorithm::Pbes2HsA256kw,
                "\"PBES2-HS256+A256KW\"",
            ),
        ];
        for (alg, expected_json) in &variants {
            let json = serde_json::to_string(alg).unwrap();
            assert_eq!(&json, expected_json);
            let parsed: KeyManagementAlgorithm = serde_json::from_str(&json).unwrap();
            assert_eq!(&parsed, alg);
        }
    }

    #[test]
    fn test_encryption_metadata_with_key_management() {
        let metadata = EncryptionMetadata {
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            kdf: None,
            wrapped_key: Some("wrapped-key-base64".to_string()),
            key_management: Some(KeyManagementAlgorithm::EcdhEsA256kw),
            recipients: vec![Recipient {
                id: "recipient-1".to_string(),
                encrypted_key: "enc-key-base64".to_string(),
                algorithm: Some("ECDH-ES+A256KW".to_string()),
                ephemeral_public_key: Some("ephemeral-pk-base64".to_string()),
            }],
        };

        let json = serde_json::to_string_pretty(&metadata).unwrap();
        assert!(json.contains("ECDH-ES+A256KW"));
        assert!(json.contains("ephemeralPublicKey"));

        let parsed: EncryptionMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.key_management, metadata.key_management);
        assert_eq!(
            parsed.recipients[0].ephemeral_public_key,
            Some("ephemeral-pk-base64".to_string())
        );
    }

    #[test]
    fn test_encryption_metadata_backward_compat() {
        // JSON without key_management or ephemeral_public_key should deserialize fine
        let json = r#"{
            "algorithm": "AES-256-GCM",
            "recipients": [{
                "id": "r1",
                "encryptedKey": "key-data"
            }]
        }"#;
        let metadata: EncryptionMetadata = serde_json::from_str(json).unwrap();
        assert!(metadata.key_management.is_none());
        assert!(metadata.recipients[0].ephemeral_public_key.is_none());
    }
}

#[cfg(all(test, feature = "encryption-chacha"))]
mod chacha_tests {
    use super::*;

    #[test]
    fn test_chacha_encrypt_decrypt() {
        let key = ChaCha20Poly1305Encryptor::generate_key();
        let encryptor = ChaCha20Poly1305Encryptor::new(&key).unwrap();

        let plaintext = b"Hello, World! This is a test message.";
        let encrypted = encryptor.encrypt(plaintext).unwrap();

        assert_ne!(&encrypted.ciphertext[..plaintext.len()], plaintext);

        let decrypted = encryptor
            .decrypt(&encrypted.ciphertext, &encrypted.nonce)
            .unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_chacha_wrong_key_fails() {
        let key1 = ChaCha20Poly1305Encryptor::generate_key();
        let key2 = ChaCha20Poly1305Encryptor::generate_key();

        let encryptor1 = ChaCha20Poly1305Encryptor::new(&key1).unwrap();
        let encryptor2 = ChaCha20Poly1305Encryptor::new(&key2).unwrap();

        let plaintext = b"Secret message";
        let encrypted = encryptor1.encrypt(plaintext).unwrap();

        let result = encryptor2.decrypt(&encrypted.ciphertext, &encrypted.nonce);
        assert!(result.is_err());
    }

    #[test]
    fn test_chacha_tampered_data_fails() {
        let key = ChaCha20Poly1305Encryptor::generate_key();
        let encryptor = ChaCha20Poly1305Encryptor::new(&key).unwrap();

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
    fn test_chacha_empty_plaintext() {
        let key = ChaCha20Poly1305Encryptor::generate_key();
        let encryptor = ChaCha20Poly1305Encryptor::new(&key).unwrap();

        let plaintext = b"";
        let encrypted = encryptor.encrypt(plaintext).unwrap();
        let decrypted = encryptor
            .decrypt(&encrypted.ciphertext, &encrypted.nonce)
            .unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_chacha_encryption_algorithm_enum() {
        let algo = EncryptionAlgorithm::ChaCha20Poly1305;
        assert_eq!(algo.as_str(), "ChaCha20-Poly1305");
        assert_eq!(algo.key_size(), 32);
        assert_eq!(algo.nonce_size(), 12);

        let json = serde_json::to_string(&algo).unwrap();
        assert_eq!(json, "\"ChaCha20-Poly1305\"");

        let deserialized: EncryptionAlgorithm = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, algo);
    }
}

#[cfg(all(test, feature = "key-wrapping"))]
// `wrapper`/`wrapped` is the natural wrap-an-object/get-a-wrapped-result idiom in these tests.
#[allow(clippy::similar_names)]
mod key_wrapping_tests {
    use super::*;

    /// Generate a P-256 keypair for testing.
    fn generate_keypair() -> (p256::SecretKey, p256::PublicKey) {
        use p256::elliptic_curve::Generate;
        let secret = p256::SecretKey::generate();
        let public = secret.public_key();
        (secret, public)
    }

    #[test]
    fn test_wrap_unwrap_roundtrip() {
        let (secret, public) = generate_keypair();

        // Generate a 32-byte content encryption key
        let content_key = Aes256GcmEncryptor::generate_key();

        // Wrap
        let wrapper = EcdhEsKeyWrapper::new(public);
        let wrapped = wrapper.wrap(&content_key).unwrap();

        // Wrapped key should be 8 bytes longer than original (AES-KW IV)
        assert_eq!(wrapped.wrapped_key.len(), content_key.len() + 8);
        // Ephemeral public key should be 65 bytes (uncompressed SEC1 point)
        assert_eq!(wrapped.ephemeral_public_key.len(), 65);

        // Unwrap
        let unwrapper = EcdhEsKeyUnwrapper::new(secret);
        let recovered = unwrapper.unwrap(&wrapped).unwrap();

        assert_eq!(recovered, content_key);
    }

    #[test]
    fn test_multi_recipient_wrap() {
        let (secret_a, public_a) = generate_keypair();
        let (secret_b, public_b) = generate_keypair();

        let content_key = Aes256GcmEncryptor::generate_key();

        // Wrap for two recipients
        let wrapper_a = EcdhEsKeyWrapper::new(public_a);
        let wrapped_a = wrapper_a.wrap(&content_key).unwrap();

        let wrapper_b = EcdhEsKeyWrapper::new(public_b);
        let wrapped_b = wrapper_b.wrap(&content_key).unwrap();

        // Each should unwrap independently
        let unwrapper_a = EcdhEsKeyUnwrapper::new(secret_a);
        let recovered_a = unwrapper_a.unwrap(&wrapped_a).unwrap();
        assert_eq!(recovered_a, content_key);

        let unwrapper_b = EcdhEsKeyUnwrapper::new(secret_b);
        let recovered_b = unwrapper_b.unwrap(&wrapped_b).unwrap();
        assert_eq!(recovered_b, content_key);

        // Cross-unwrap should fail (wrong key)
        assert!(unwrapper_a.unwrap(&wrapped_b).is_err());
        assert!(unwrapper_b.unwrap(&wrapped_a).is_err());
    }

    #[test]
    fn test_wrong_private_key_fails() {
        let (_secret, public) = generate_keypair();
        let (wrong_secret, _wrong_public) = generate_keypair();

        let content_key = Aes256GcmEncryptor::generate_key();

        let wrapper = EcdhEsKeyWrapper::new(public);
        let wrapped = wrapper.wrap(&content_key).unwrap();

        // Unwrapping with wrong key should fail
        let unwrapper = EcdhEsKeyUnwrapper::new(wrong_secret);
        let result = unwrapper.unwrap(&wrapped);
        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_wrapped_key_fails() {
        let (secret, public) = generate_keypair();

        let content_key = Aes256GcmEncryptor::generate_key();

        let wrapper = EcdhEsKeyWrapper::new(public);
        let mut wrapped = wrapper.wrap(&content_key).unwrap();

        // Tamper with the wrapped key
        if !wrapped.wrapped_key.is_empty() {
            wrapped.wrapped_key[0] ^= 0xFF;
        }

        let unwrapper = EcdhEsKeyUnwrapper::new(secret);
        let result = unwrapper.unwrap(&wrapped);
        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_ephemeral_key_fails() {
        let (secret, public) = generate_keypair();

        let content_key = Aes256GcmEncryptor::generate_key();

        let wrapper = EcdhEsKeyWrapper::new(public);
        let mut wrapped = wrapper.wrap(&content_key).unwrap();

        // Tamper with the ephemeral public key (change a coordinate byte)
        // Byte 0 is the tag (0x04 for uncompressed), skip it
        if wrapped.ephemeral_public_key.len() > 1 {
            wrapped.ephemeral_public_key[1] ^= 0xFF;
        }

        let unwrapper = EcdhEsKeyUnwrapper::new(secret);
        let result = unwrapper.unwrap(&wrapped);
        // Should either fail to decode the point or produce wrong shared secret
        assert!(result.is_err());
    }

    #[test]
    fn test_integration_encrypt_wrap_unwrap_decrypt() {
        let (secret, public) = generate_keypair();

        // 1. Generate content encryption key and encrypt content
        let content_key = Aes256GcmEncryptor::generate_key();
        let encryptor = Aes256GcmEncryptor::new(&content_key).unwrap();
        let plaintext = b"CDX document content for encryption";
        let encrypted = encryptor.encrypt(plaintext).unwrap();

        // 2. Wrap the content key for the recipient
        let wrapper = EcdhEsKeyWrapper::new(public);
        let wrapped = wrapper.wrap(&content_key).unwrap();

        // 3. Build metadata (as would be serialized in the archive)
        let metadata = EncryptionMetadata {
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            kdf: None,
            wrapped_key: None,
            key_management: Some(KeyManagementAlgorithm::EcdhEsA256kw),
            recipients: vec![Recipient {
                id: "recipient-1".to_string(),
                encrypted_key: base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    &wrapped.wrapped_key,
                ),
                algorithm: Some("ECDH-ES+A256KW".to_string()),
                ephemeral_public_key: Some(base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    &wrapped.ephemeral_public_key,
                )),
            }],
        };

        // 4. Serialize/deserialize metadata (roundtrip)
        let json = serde_json::to_string(&metadata).unwrap();
        let parsed: EncryptionMetadata = serde_json::from_str(&json).unwrap();

        // 5. Recipient unwraps the key
        let recipient = &parsed.recipients[0];
        let wrapped_data = WrappedKeyData {
            wrapped_key: base64::Engine::decode(
                &base64::engine::general_purpose::STANDARD,
                &recipient.encrypted_key,
            )
            .unwrap(),
            ephemeral_public_key: base64::Engine::decode(
                &base64::engine::general_purpose::STANDARD,
                recipient.ephemeral_public_key.as_ref().unwrap(),
            )
            .unwrap(),
        };

        let unwrapper = EcdhEsKeyUnwrapper::new(secret);
        let recovered_key = unwrapper.unwrap(&wrapped_data).unwrap();

        // 6. Decrypt the content
        let decryptor = Aes256GcmEncryptor::new(&recovered_key).unwrap();
        let decrypted = decryptor
            .decrypt(&encrypted.ciphertext, &encrypted.nonce)
            .unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrap_16_byte_key() {
        // AES Key Wrap works with any key that's a multiple of 8 bytes
        let (secret, public) = generate_keypair();
        let content_key = [0x42u8; 16]; // 128-bit key

        let wrapper = EcdhEsKeyWrapper::new(public);
        let wrapped = wrapper.wrap(&content_key).unwrap();
        assert_eq!(wrapped.wrapped_key.len(), 24); // 16 + 8

        let unwrapper = EcdhEsKeyUnwrapper::new(secret);
        let recovered = unwrapper.unwrap(&wrapped).unwrap();
        assert_eq!(recovered, content_key);
    }
}

#[cfg(all(test, feature = "key-wrapping-rsa"))]
// `wrapper`/`wrapped` is the natural wrap-an-object/get-a-wrapped-result idiom in these tests.
#[allow(clippy::similar_names)]
mod rsa_oaep_tests {
    use super::*;

    fn generate_rsa_keypair(bits: usize) -> (rsa::RsaPrivateKey, rsa::RsaPublicKey) {
        let private_key =
            rsa::RsaPrivateKey::new(&mut rand_core::UnwrapErr(getrandom::SysRng), bits).unwrap();
        let public_key = rsa::RsaPublicKey::from(&private_key);
        (private_key, public_key)
    }

    #[test]
    fn test_rsa_oaep_roundtrip_2048() {
        let (private_key, public_key) = generate_rsa_keypair(2048);
        let content_key = Aes256GcmEncryptor::generate_key();

        let wrapper = RsaOaepKeyWrapper::new(public_key);
        let wrapped = wrapper.wrap(&content_key).unwrap();

        let unwrapper = RsaOaepKeyUnwrapper::new(private_key);
        let recovered = unwrapper.unwrap(&wrapped).unwrap();

        assert_eq!(recovered, content_key);
    }

    #[test]
    fn test_rsa_oaep_roundtrip_4096() {
        let (private_key, public_key) = generate_rsa_keypair(4096);
        let content_key = Aes256GcmEncryptor::generate_key();

        let wrapper = RsaOaepKeyWrapper::new(public_key);
        let wrapped = wrapper.wrap(&content_key).unwrap();

        let unwrapper = RsaOaepKeyUnwrapper::new(private_key);
        let recovered = unwrapper.unwrap(&wrapped).unwrap();

        assert_eq!(recovered, content_key);
    }

    #[test]
    fn test_rsa_oaep_wrong_key_fails() {
        let (_private_key, public_key) = generate_rsa_keypair(2048);
        let (wrong_private_key, _wrong_public_key) = generate_rsa_keypair(2048);
        let content_key = Aes256GcmEncryptor::generate_key();

        let wrapper = RsaOaepKeyWrapper::new(public_key);
        let wrapped = wrapper.wrap(&content_key).unwrap();

        let unwrapper = RsaOaepKeyUnwrapper::new(wrong_private_key);
        let result = unwrapper.unwrap(&wrapped);
        assert!(result.is_err());
    }

    #[test]
    fn test_rsa_oaep_tampered_data_fails() {
        let (private_key, public_key) = generate_rsa_keypair(2048);
        let content_key = Aes256GcmEncryptor::generate_key();

        let wrapper = RsaOaepKeyWrapper::new(public_key);
        let mut wrapped = wrapper.wrap(&content_key).unwrap();

        if !wrapped.wrapped_key.is_empty() {
            wrapped.wrapped_key[0] ^= 0xFF;
        }

        let unwrapper = RsaOaepKeyUnwrapper::new(private_key);
        let result = unwrapper.unwrap(&wrapped);
        assert!(result.is_err());
    }

    #[test]
    fn test_rsa_oaep_integration_encrypt_wrap_unwrap_decrypt() {
        let (private_key, public_key) = generate_rsa_keypair(2048);

        // Encrypt content
        let content_key = Aes256GcmEncryptor::generate_key();
        let encryptor = Aes256GcmEncryptor::new(&content_key).unwrap();
        let plaintext = b"CDX document encrypted with RSA-OAEP key wrapping";
        let encrypted = encryptor.encrypt(plaintext).unwrap();

        // Wrap the content key
        let wrapper = RsaOaepKeyWrapper::new(public_key);
        let wrapped = wrapper.wrap(&content_key).unwrap();

        // Unwrap and decrypt
        let unwrapper = RsaOaepKeyUnwrapper::new(private_key);
        let recovered_key = unwrapper.unwrap(&wrapped).unwrap();

        let decryptor = Aes256GcmEncryptor::new(&recovered_key).unwrap();
        let decrypted = decryptor
            .decrypt(&encrypted.ciphertext, &encrypted.nonce)
            .unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_rsa_oaep_metadata_serialization() {
        let metadata = EncryptionMetadata {
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            kdf: None,
            wrapped_key: None,
            key_management: Some(KeyManagementAlgorithm::RsaOaep256),
            recipients: vec![Recipient {
                id: "rsa-recipient".to_string(),
                encrypted_key: "wrapped-key-base64".to_string(),
                algorithm: Some("RSA-OAEP-256".to_string()),
                ephemeral_public_key: None,
            }],
        };

        let json = serde_json::to_string_pretty(&metadata).unwrap();
        assert!(json.contains("RSA-OAEP-256"));
        // RSA-OAEP doesn't use ephemeral keys
        assert!(!json.contains("ephemeralPublicKey"));

        let parsed: EncryptionMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(
            parsed.key_management,
            Some(KeyManagementAlgorithm::RsaOaep256)
        );
    }
}

#[cfg(all(test, feature = "key-wrapping-pbes2"))]
// `wrapper`/`wrapped` is the natural wrap-an-object/get-a-wrapped-result idiom in these tests.
#[allow(clippy::similar_names)]
mod pbes2_tests {
    use super::*;

    #[test]
    fn test_pbes2_roundtrip() {
        let password = b"correct horse battery staple";
        let content_key = Aes256GcmEncryptor::generate_key();

        let wrapper = Pbes2KeyWrapper::new(password, Pbes2KeyWrapper::DEFAULT_ITERATIONS).unwrap();
        let wrapped = wrapper.wrap(&content_key).unwrap();

        assert_eq!(wrapped.wrapped_key.len(), content_key.len() + 8);
        assert_eq!(wrapped.salt.len(), 16);
        assert_eq!(wrapped.iterations, Pbes2KeyWrapper::DEFAULT_ITERATIONS);

        let unwrapper = Pbes2KeyUnwrapper::new(password);
        let recovered = unwrapper.unwrap(&wrapped).unwrap();

        assert_eq!(recovered, content_key);
    }

    #[test]
    fn test_pbes2_wrong_password_fails() {
        let content_key = Aes256GcmEncryptor::generate_key();

        let wrapper =
            Pbes2KeyWrapper::new(b"correct password", Pbes2KeyWrapper::MIN_ITERATIONS).unwrap();
        let wrapped = wrapper.wrap(&content_key).unwrap();

        let unwrapper = Pbes2KeyUnwrapper::new(b"wrong password");
        let result = unwrapper.unwrap(&wrapped);
        assert!(result.is_err());
    }

    #[test]
    fn test_pbes2_tampered_salt_fails() {
        let password = b"my password";
        let content_key = Aes256GcmEncryptor::generate_key();

        let wrapper = Pbes2KeyWrapper::new(password, Pbes2KeyWrapper::MIN_ITERATIONS).unwrap();
        let mut wrapped = wrapper.wrap(&content_key).unwrap();

        // Tamper with the salt
        if !wrapped.salt.is_empty() {
            wrapped.salt[0] ^= 0xFF;
        }

        let unwrapper = Pbes2KeyUnwrapper::new(password);
        let result = unwrapper.unwrap(&wrapped);
        assert!(result.is_err());
    }

    #[test]
    fn test_pbes2_different_iteration_counts() {
        let password = b"shared password";
        let content_key = Aes256GcmEncryptor::generate_key();

        // Wrap with different valid iteration counts
        for &iterations in &[10_000u32, 100_000, 1_000_000] {
            let wrapper = Pbes2KeyWrapper::new(password, iterations).unwrap();
            let wrapped = wrapper.wrap(&content_key).unwrap();
            assert_eq!(wrapped.iterations, iterations);

            let unwrapper = Pbes2KeyUnwrapper::new(password);
            let recovered = unwrapper.unwrap(&wrapped).unwrap();
            assert_eq!(recovered, content_key);
        }
    }

    #[test]
    fn test_pbes2_iteration_bounds() {
        // Below minimum
        assert!(Pbes2KeyWrapper::new(b"password", 0).is_err());
        assert!(Pbes2KeyWrapper::new(b"password", 1).is_err());
        assert!(Pbes2KeyWrapper::new(b"password", 9_999).is_err());

        // At minimum
        assert!(Pbes2KeyWrapper::new(b"password", 10_000).is_ok());

        // At maximum
        assert!(Pbes2KeyWrapper::new(b"password", 10_000_000).is_ok());

        // Above maximum
        assert!(Pbes2KeyWrapper::new(b"password", 10_000_001).is_err());
        assert!(Pbes2KeyWrapper::new(b"password", u32::MAX).is_err());
    }

    #[test]
    fn test_pbes2_unwrap_rejects_bad_iterations() {
        let unwrapper = Pbes2KeyUnwrapper::new(b"password");
        let data = Pbes2WrappedKeyData {
            wrapped_key: vec![0u8; 40],
            salt: vec![0u8; 16],
            iterations: 0,
        };
        assert!(unwrapper.unwrap(&data).is_err());
    }

    #[test]
    fn test_pbes2_integration_encrypt_wrap_unwrap_decrypt() {
        let password = b"document encryption password";

        // Encrypt content
        let content_key = Aes256GcmEncryptor::generate_key();
        let encryptor = Aes256GcmEncryptor::new(&content_key).unwrap();
        let plaintext = b"CDX document with password-based key wrapping";
        let encrypted = encryptor.encrypt(plaintext).unwrap();

        // Wrap the content key with password
        let wrapper = Pbes2KeyWrapper::new(password, Pbes2KeyWrapper::MIN_ITERATIONS).unwrap();
        let wrapped = wrapper.wrap(&content_key).unwrap();

        // Unwrap and decrypt
        let unwrapper = Pbes2KeyUnwrapper::new(password);
        let recovered_key = unwrapper.unwrap(&wrapped).unwrap();

        let decryptor = Aes256GcmEncryptor::new(&recovered_key).unwrap();
        let decrypted = decryptor
            .decrypt(&encrypted.ciphertext, &encrypted.nonce)
            .unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_pbes2_metadata_serialization() {
        let metadata = EncryptionMetadata {
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            kdf: None,
            wrapped_key: None,
            key_management: Some(KeyManagementAlgorithm::Pbes2HsA256kw),
            recipients: vec![],
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("PBES2-HS256+A256KW"));

        let parsed: EncryptionMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(
            parsed.key_management,
            Some(KeyManagementAlgorithm::Pbes2HsA256kw)
        );
    }
}
