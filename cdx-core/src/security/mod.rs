#![allow(clippy::doc_markdown)] // EdDSA is a proper algorithm name

//! Digital signatures, encryption, and security features.
//!
//! This module provides cryptographic capabilities for Codex documents:
//!
//! - **Signatures**: ECDSA (ES256), EdDSA (Ed25519), and ML-DSA-65 (post-quantum) digital signatures
//! - **Encryption**: AES-256-GCM authenticated encryption
//! - **Certificate Validation**: X.509 certificate chain validation
//! - **Revocation Checking**: OCSP and CRL certificate revocation (feature: `ocsp`)
//! - **Access Control**: Permission management for document operations
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

mod access_control;
mod certificate;
#[cfg(feature = "eddsa")]
mod eddsa;
#[cfg(feature = "encryption")]
mod encryption;
#[cfg(feature = "ml-dsa")]
mod ml_dsa;
#[cfg(feature = "ocsp")]
mod revocation;
mod signature;
mod signer;

pub use access_control::{AccessControl, Operation, PermissionGrant, Permissions, Principal};
pub use certificate::{eku, CertificateChain, CertificateInfo, CertificateValidation, KeyUsage};
pub use signature::{
    Signature, SignatureAlgorithm, SignatureFile, SignatureScope, SignatureVerification, SignerInfo,
};
pub use signer::{EcdsaSigner, EcdsaVerifier, Signer, Verifier};

#[cfg(feature = "eddsa")]
pub use eddsa::{EddsaSigner, EddsaVerifier};

#[cfg(feature = "ml-dsa")]
#[cfg_attr(docsrs, doc(cfg(feature = "ml-dsa")))]
pub use ml_dsa::{MlDsaSigner, MlDsaVerifier};

#[cfg(feature = "encryption")]
pub use encryption::{
    Aes256GcmEncryptor, EncryptedData, EncryptionAlgorithm, EncryptionMetadata, KdfAlgorithm,
    KeyDerivation, Recipient,
};

#[cfg(feature = "ocsp")]
#[cfg_attr(docsrs, doc(cfg(feature = "ocsp")))]
pub use revocation::{
    RevocationChecker, RevocationConfig, RevocationMethod, RevocationReason, RevocationResult,
    RevocationStatus,
};
