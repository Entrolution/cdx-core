#![allow(clippy::doc_markdown)] // EdDSA is a proper algorithm name

//! EdDSA (Ed25519) signature implementation.

use crate::{DocumentId, Result};

use super::signature::{Signature, SignatureAlgorithm, SignatureVerification, SignerInfo};
use super::signer::{Signer, Verifier};

/// EdDSA (Ed25519) signer.
#[cfg(feature = "eddsa")]
pub struct EddsaSigner {
    signing_key: ed25519_dalek::SigningKey,
    signer_info: SignerInfo,
}

#[cfg(feature = "eddsa")]
impl EddsaSigner {
    /// Create a new signer from a PEM-encoded private key.
    ///
    /// # Errors
    ///
    /// Returns an error if the PEM cannot be parsed.
    pub fn from_pem(pem: &str, signer_info: SignerInfo) -> Result<Self> {
        use ed25519_dalek::pkcs8::DecodePrivateKey;

        let signing_key = ed25519_dalek::SigningKey::from_pkcs8_pem(pem).map_err(|e| {
            crate::Error::InvalidManifest {
                reason: format!("Failed to parse EdDSA private key PEM: {e}"),
            }
        })?;

        Ok(Self {
            signing_key,
            signer_info,
        })
    }

    /// Generate a new random signing key.
    ///
    /// Returns the signer and the public key in PEM format.
    ///
    /// # Errors
    ///
    /// Returns an error if key generation fails.
    pub fn generate(signer_info: SignerInfo) -> Result<(Self, String)> {
        use ed25519_dalek::pkcs8::spki::{der::pem::LineEnding, EncodePublicKey};

        let signing_key = ed25519_dalek::SigningKey::generate(&mut rand_core::OsRng);
        let verifying_key = signing_key.verifying_key();
        let public_key_pem = verifying_key
            .to_public_key_pem(LineEnding::LF)
            .map_err(|e| crate::Error::InvalidManifest {
                reason: format!("Failed to encode EdDSA public key: {e}"),
            })?;

        Ok((
            Self {
                signing_key,
                signer_info,
            },
            public_key_pem,
        ))
    }

    /// Get the public key in PEM format.
    ///
    /// # Errors
    ///
    /// Returns an error if encoding fails.
    pub fn public_key_pem(&self) -> Result<String> {
        use ed25519_dalek::pkcs8::spki::{der::pem::LineEnding, EncodePublicKey};

        self.signing_key
            .verifying_key()
            .to_public_key_pem(LineEnding::LF)
            .map_err(|e| crate::Error::InvalidManifest {
                reason: format!("Failed to encode EdDSA public key: {e}"),
            })
    }
}

#[cfg(feature = "eddsa")]
impl Signer for EddsaSigner {
    fn algorithm(&self) -> SignatureAlgorithm {
        SignatureAlgorithm::EdDSA
    }

    fn signer_info(&self) -> SignerInfo {
        self.signer_info.clone()
    }

    fn sign(&self, document_id: &DocumentId) -> Result<Signature> {
        use base64::Engine;
        use ed25519_dalek::Signer as EddsaSignerTrait;

        if document_id.is_pending() {
            return Err(crate::Error::InvalidManifest {
                reason: "Cannot sign a pending document ID".to_string(),
            });
        }

        // Sign the document ID bytes
        let signature = self.signing_key.sign(document_id.digest());

        // Encode as base64
        let value = base64::engine::general_purpose::STANDARD.encode(signature.to_bytes());

        // Generate signature ID
        let sig_id = format!(
            "sig-{}",
            &crate::Hasher::hash(crate::HashAlgorithm::Sha256, value.as_bytes()).hex_digest()[..8]
        );

        Ok(Signature::new(
            sig_id,
            SignatureAlgorithm::EdDSA,
            self.signer_info.clone(),
            value,
        ))
    }
}

/// EdDSA (Ed25519) verifier.
#[cfg(feature = "eddsa")]
pub struct EddsaVerifier {
    verifying_key: ed25519_dalek::VerifyingKey,
}

#[cfg(feature = "eddsa")]
impl EddsaVerifier {
    /// Create a new verifier from a PEM-encoded public key.
    ///
    /// # Errors
    ///
    /// Returns an error if the PEM cannot be parsed.
    pub fn from_pem(pem: &str) -> Result<Self> {
        use ed25519_dalek::pkcs8::DecodePublicKey;

        let verifying_key = ed25519_dalek::VerifyingKey::from_public_key_pem(pem).map_err(|e| {
            crate::Error::InvalidManifest {
                reason: format!("Failed to parse EdDSA public key PEM: {e}"),
            }
        })?;

        Ok(Self { verifying_key })
    }
}

#[cfg(feature = "eddsa")]
impl Verifier for EddsaVerifier {
    fn verify(
        &self,
        document_id: &DocumentId,
        signature: &Signature,
    ) -> Result<SignatureVerification> {
        use base64::Engine;
        use ed25519_dalek::Verifier as EddsaVerifierTrait;

        if signature.algorithm != SignatureAlgorithm::EdDSA {
            return Ok(SignatureVerification::invalid(
                &signature.id,
                format!(
                    "Algorithm mismatch: expected EdDSA, got {}",
                    signature.algorithm
                ),
            ));
        }

        // Decode signature from base64
        let sig_bytes = base64::engine::general_purpose::STANDARD
            .decode(&signature.value)
            .map_err(|e| crate::Error::InvalidManifest {
                reason: format!("Failed to decode signature: {e}"),
            })?;

        // Parse signature
        let sig_array: [u8; 64] =
            sig_bytes
                .try_into()
                .map_err(|_| crate::Error::InvalidManifest {
                    reason: "Invalid EdDSA signature length (expected 64 bytes)".to_string(),
                })?;
        let eddsa_sig = ed25519_dalek::Signature::from_bytes(&sig_array);

        // Verify
        match self.verifying_key.verify(document_id.digest(), &eddsa_sig) {
            Ok(()) => Ok(SignatureVerification::valid(&signature.id)),
            Err(e) => Ok(SignatureVerification::invalid(
                &signature.id,
                format!("EdDSA signature verification failed: {e}"),
            )),
        }
    }
}

#[cfg(all(test, feature = "eddsa"))]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_sign() {
        let signer_info = SignerInfo::new("Test EdDSA Signer");
        let (signer, public_key_pem) = EddsaSigner::generate(signer_info).unwrap();

        assert!(!public_key_pem.is_empty());
        assert!(public_key_pem.contains("BEGIN PUBLIC KEY"));

        let doc_id = crate::Hasher::hash(crate::HashAlgorithm::Sha256, b"test document");
        let signature = signer.sign(&doc_id).unwrap();

        assert_eq!(signature.algorithm, SignatureAlgorithm::EdDSA);
        assert!(!signature.value.is_empty());
    }

    #[test]
    fn test_sign_and_verify() {
        let signer_info = SignerInfo::new("Test EdDSA Signer");
        let (signer, public_key_pem) = EddsaSigner::generate(signer_info).unwrap();

        let doc_id = crate::Hasher::hash(crate::HashAlgorithm::Sha256, b"test document");
        let signature = signer.sign(&doc_id).unwrap();

        let verifier = EddsaVerifier::from_pem(&public_key_pem).unwrap();
        let result = verifier.verify(&doc_id, &signature).unwrap();

        assert!(result.is_valid());
    }

    #[test]
    fn test_verify_wrong_document() {
        let signer_info = SignerInfo::new("Test EdDSA Signer");
        let (signer, public_key_pem) = EddsaSigner::generate(signer_info).unwrap();

        let doc_id = crate::Hasher::hash(crate::HashAlgorithm::Sha256, b"original document");
        let signature = signer.sign(&doc_id).unwrap();

        let different_doc_id =
            crate::Hasher::hash(crate::HashAlgorithm::Sha256, b"different document");

        let verifier = EddsaVerifier::from_pem(&public_key_pem).unwrap();
        let result = verifier.verify(&different_doc_id, &signature).unwrap();

        assert!(!result.is_valid());
    }

    #[test]
    fn test_cannot_sign_pending_id() {
        let signer_info = SignerInfo::new("Test EdDSA Signer");
        let (signer, _) = EddsaSigner::generate(signer_info).unwrap();

        let pending_id = crate::DocumentId::pending();
        let result = signer.sign(&pending_id);

        assert!(result.is_err());
    }
}
