#![allow(clippy::doc_markdown)] // EdDSA is a proper algorithm name

//! Digital signatures, encryption, and security features.
//!
//! This module provides cryptographic capabilities for Codex documents:
//!
//! - **Signatures**: ECDSA (ES256) and EdDSA (Ed25519) digital signatures
//! - **Encryption**: AES-256-GCM authenticated encryption
//!
//! # Signing Documents (ECDSA)
//!
//! ```rust,ignore
//! use cdx_core::security::{EcdsaSigner, SignerInfo, Signer};
//!
//! let signer_info = SignerInfo::new("Alice");
//! let (signer, public_key_pem) = EcdsaSigner::generate(signer_info)?;
//! let signature = signer.sign(&document_id)?;
//! ```
//!
//! # Signing Documents (EdDSA)
//!
//! ```rust,ignore
//! use cdx_core::security::{EddsaSigner, SignerInfo, Signer};
//!
//! let signer_info = SignerInfo::new("Alice");
//! let (signer, public_key_pem) = EddsaSigner::generate(signer_info)?;
//! let signature = signer.sign(&document_id)?;
//! ```
//!
//! # Encrypting Data
//!
//! ```rust,ignore
//! use cdx_core::security::Aes256GcmEncryptor;
//!
//! let key = Aes256GcmEncryptor::generate_key();
//! let encryptor = Aes256GcmEncryptor::new(&key)?;
//! let encrypted = encryptor.encrypt(b"secret data")?;
//! let decrypted = encryptor.decrypt(&encrypted.ciphertext, &encrypted.nonce)?;
//! ```

#[cfg(feature = "eddsa")]
mod eddsa;
#[cfg(feature = "encryption")]
mod encryption;
mod signature;
mod signer;

pub use signature::{
    Signature, SignatureAlgorithm, SignatureFile, SignatureVerification, SignerInfo,
};
pub use signer::{EcdsaSigner, EcdsaVerifier, Signer, Verifier};

#[cfg(feature = "eddsa")]
pub use eddsa::{EddsaSigner, EddsaVerifier};

#[cfg(feature = "encryption")]
pub use encryption::{
    Aes256GcmEncryptor, EncryptedData, EncryptionAlgorithm, EncryptionMetadata, KdfAlgorithm,
    KeyDerivation, Recipient,
};
