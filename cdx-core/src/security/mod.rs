#![allow(clippy::doc_markdown)] // EdDSA is a proper algorithm name

//! Digital signatures, encryption, and security features.
//!
//! This module provides cryptographic capabilities for Codex documents:
//!
//! - **Signatures**: ECDSA (ES256, ES384), EdDSA (Ed25519), RSA-PSS (PS256), ML-DSA-65 (post-quantum), and WebAuthn/FIDO2 digital signatures
//! - **Encryption**: AES-256-GCM and ChaCha20-Poly1305 authenticated encryption
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
mod annotations;
mod certificate;
#[cfg(feature = "eddsa")]
mod eddsa;
#[cfg(feature = "encryption")]
mod encryption;
#[cfg(feature = "signatures-es384")]
mod es384;
#[cfg(feature = "ml-dsa")]
mod ml_dsa;
#[cfg(feature = "ocsp")]
mod revocation;
#[cfg(feature = "signatures-rsa")]
mod rsa_pss;
mod signature;
mod signer;
#[cfg(test)]
mod test_helpers;
#[cfg(feature = "webauthn")]
mod webauthn;

pub use access_control::{AccessControl, Operation, PermissionGrant, Permissions, Principal};
pub use annotations::{Annotation, AnnotationType, AnnotationsFile};
pub use certificate::{eku, CertificateChain, CertificateInfo, CertificateValidation, KeyUsage};
pub use signature::{
    Signature, SignatureAlgorithm, SignatureFile, SignatureScope, SignatureVerification,
    SignerInfo, TrustedTimestamp, WebAuthnSignature,
};
pub use signer::{EcdsaSigner, EcdsaVerifier, Signer, Verifier};

#[cfg(feature = "eddsa")]
pub use eddsa::{EddsaSigner, EddsaVerifier};

#[cfg(feature = "signatures-es384")]
#[cfg_attr(docsrs, doc(cfg(feature = "signatures-es384")))]
pub use es384::{Es384Signer, Es384Verifier};

#[cfg(feature = "signatures-rsa")]
#[cfg_attr(docsrs, doc(cfg(feature = "signatures-rsa")))]
pub use rsa_pss::{Ps256Signer, Ps256Verifier};

#[cfg(feature = "ml-dsa")]
#[cfg_attr(docsrs, doc(cfg(feature = "ml-dsa")))]
pub use ml_dsa::{MlDsaSigner, MlDsaVerifier};

#[cfg(feature = "encryption")]
pub use encryption::{
    Aes256GcmEncryptor, EncryptedData, EncryptionAlgorithm, EncryptionMetadata, KdfAlgorithm,
    KeyDerivation, KeyManagementAlgorithm, Recipient,
};

#[cfg(feature = "encryption-chacha")]
#[cfg_attr(docsrs, doc(cfg(feature = "encryption-chacha")))]
pub use encryption::ChaCha20Poly1305Encryptor;

#[cfg(feature = "key-wrapping")]
#[cfg_attr(docsrs, doc(cfg(feature = "key-wrapping")))]
pub use encryption::{EcdhEsKeyUnwrapper, EcdhEsKeyWrapper, WrappedKeyData};

#[cfg(feature = "key-wrapping-rsa")]
#[cfg_attr(docsrs, doc(cfg(feature = "key-wrapping-rsa")))]
pub use encryption::{RsaOaepKeyUnwrapper, RsaOaepKeyWrapper, RsaWrappedKeyData};

#[cfg(feature = "key-wrapping-pbes2")]
#[cfg_attr(docsrs, doc(cfg(feature = "key-wrapping-pbes2")))]
pub use encryption::{Pbes2KeyUnwrapper, Pbes2KeyWrapper, Pbes2WrappedKeyData};

#[cfg(feature = "ocsp")]
#[cfg_attr(docsrs, doc(cfg(feature = "ocsp")))]
pub use revocation::{
    RevocationChecker, RevocationConfig, RevocationMethod, RevocationReason, RevocationResult,
    RevocationStatus,
};

#[cfg(feature = "webauthn")]
#[cfg_attr(docsrs, doc(cfg(feature = "webauthn")))]
pub use webauthn::WebAuthnVerifier;
