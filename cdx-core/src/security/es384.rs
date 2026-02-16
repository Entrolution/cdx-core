//! ECDSA P-384 (ES384) signature support.
//!
//! This module provides ES384 signing and verification using the NIST P-384 curve.

use crate::{DocumentId, Result};

use super::signature::{Signature, SignatureAlgorithm, SignatureVerification, SignerInfo};
use super::signer::{Signer, Verifier};

/// ECDSA P-384 signer (ES384).
pub struct Es384Signer {
    signing_key: p384::ecdsa::SigningKey,
    signer_info: SignerInfo,
}

impl Es384Signer {
    /// Create a new signer from a PEM-encoded private key.
    ///
    /// # Errors
    ///
    /// Returns an error if the PEM cannot be parsed.
    pub fn from_pem(pem: &str, signer_info: SignerInfo) -> Result<Self> {
        use p384::pkcs8::DecodePrivateKey;

        let signing_key = p384::ecdsa::SigningKey::from_pkcs8_pem(pem).map_err(|e| {
            crate::Error::InvalidManifest {
                reason: format!("Failed to parse P-384 private key PEM: {e}"),
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
        use p384::pkcs8::EncodePublicKey;

        use p384::elliptic_curve::Generate;
        let signing_key = p384::ecdsa::SigningKey::generate();
        let verifying_key = signing_key.verifying_key();
        let public_key_pem = verifying_key
            .to_public_key_pem(p384::pkcs8::LineEnding::LF)
            .map_err(|e| crate::Error::InvalidManifest {
                reason: format!("Failed to encode P-384 public key: {e}"),
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
        use p384::pkcs8::EncodePublicKey;

        self.signing_key
            .verifying_key()
            .to_public_key_pem(p384::pkcs8::LineEnding::LF)
            .map_err(|e| crate::Error::InvalidManifest {
                reason: format!("Failed to encode P-384 public key: {e}"),
            })
    }
}

impl Signer for Es384Signer {
    fn algorithm(&self) -> SignatureAlgorithm {
        SignatureAlgorithm::ES384
    }

    fn signer_info(&self) -> SignerInfo {
        self.signer_info.clone()
    }

    fn sign(&self, document_id: &DocumentId) -> Result<Signature> {
        use base64::Engine;
        use ecdsa::signature::Signer as EcdsaSignerTrait;

        if document_id.is_pending() {
            return Err(crate::Error::InvalidManifest {
                reason: "Cannot sign a pending document ID".to_string(),
            });
        }

        // Sign the document ID bytes
        let signature: p384::ecdsa::Signature = self.signing_key.sign(document_id.digest());

        // Encode as base64
        let value = base64::engine::general_purpose::STANDARD.encode(signature.to_bytes());

        // Generate signature ID
        let sig_id = format!(
            "sig-{}",
            &crate::Hasher::hash(crate::HashAlgorithm::Sha256, value.as_bytes()).hex_digest()[..8]
        );

        Ok(Signature::new(
            sig_id,
            SignatureAlgorithm::ES384,
            self.signer_info.clone(),
            value,
        ))
    }
}

/// ECDSA P-384 verifier (ES384).
pub struct Es384Verifier {
    verifying_key: p384::ecdsa::VerifyingKey,
}

impl Es384Verifier {
    /// Create a new verifier from a PEM-encoded public key.
    ///
    /// # Errors
    ///
    /// Returns an error if the PEM cannot be parsed.
    pub fn from_pem(pem: &str) -> Result<Self> {
        use p384::pkcs8::DecodePublicKey;

        let verifying_key = p384::ecdsa::VerifyingKey::from_public_key_pem(pem).map_err(|e| {
            crate::Error::InvalidManifest {
                reason: format!("Failed to parse P-384 public key PEM: {e}"),
            }
        })?;

        Ok(Self { verifying_key })
    }
}

impl Verifier for Es384Verifier {
    fn verify(
        &self,
        document_id: &DocumentId,
        signature: &Signature,
    ) -> Result<SignatureVerification> {
        use base64::Engine;
        use ecdsa::signature::Verifier as EcdsaVerifierTrait;

        if signature.algorithm != SignatureAlgorithm::ES384 {
            return Ok(SignatureVerification::invalid(
                &signature.id,
                format!(
                    "Algorithm mismatch: expected ES384, got {}",
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
        let ecdsa_sig = p384::ecdsa::Signature::from_slice(&sig_bytes).map_err(|e| {
            crate::Error::InvalidManifest {
                reason: format!("Invalid ES384 signature format: {e}"),
            }
        })?;

        // Verify
        match self.verifying_key.verify(document_id.digest(), &ecdsa_sig) {
            Ok(()) => Ok(SignatureVerification::valid(&signature.id)),
            Err(e) => Ok(SignatureVerification::invalid(
                &signature.id,
                format!("ES384 signature verification failed: {e}"),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::test_helpers;

    fn generate_keypair() -> (Es384Signer, Es384Verifier) {
        let signer_info = SignerInfo::new("Test ES384 Signer");
        let (signer, public_key_pem) = Es384Signer::generate(signer_info).unwrap();
        let verifier = Es384Verifier::from_pem(&public_key_pem).unwrap();
        (signer, verifier)
    }

    #[test]
    fn test_generate_and_sign() {
        let signer_info = SignerInfo::new("Test ES384 Signer");
        let (signer, public_key_pem) = Es384Signer::generate(signer_info).unwrap();

        assert!(!public_key_pem.is_empty());
        assert!(public_key_pem.contains("BEGIN PUBLIC KEY"));

        test_helpers::assert_sign_produces_valid_signature(&signer, SignatureAlgorithm::ES384);
    }

    #[test]
    fn test_sign_and_verify() {
        let (signer, verifier) = generate_keypair();
        test_helpers::assert_sign_verify_roundtrip(&signer, &verifier);
    }

    #[test]
    fn test_verify_wrong_document() {
        let (signer, verifier) = generate_keypair();
        test_helpers::assert_verify_wrong_document_fails(&signer, &verifier);
    }

    #[test]
    fn test_cannot_sign_pending_id() {
        let (signer, _) = generate_keypair();
        test_helpers::assert_cannot_sign_pending_id(&signer);
    }

    #[test]
    fn test_algorithm_mismatch() {
        let (signer, verifier) = generate_keypair();
        test_helpers::assert_algorithm_mismatch_rejected(
            &signer,
            &verifier,
            SignatureAlgorithm::ES256,
        );
    }
}
