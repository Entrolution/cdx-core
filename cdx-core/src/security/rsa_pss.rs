//! RSA-PSS (PS256) signature support.
//!
//! This module provides PS256 signing and verification using RSA with PSS padding
//! and SHA-256.

use crate::{DocumentId, Result};

use super::signature::{Signature, SignatureAlgorithm, SignatureVerification, SignerInfo};
use super::signer::{Signer, Verifier};

/// RSA-PSS signer (PS256).
pub struct Ps256Signer {
    signing_key: rsa::RsaPrivateKey,
    signer_info: SignerInfo,
}

impl Ps256Signer {
    /// Create a new signer from a PEM-encoded private key.
    ///
    /// # Errors
    ///
    /// Returns an error if the PEM cannot be parsed.
    pub fn from_pem(pem: &str, signer_info: SignerInfo) -> Result<Self> {
        use rsa::pkcs8::DecodePrivateKey;

        let signing_key =
            rsa::RsaPrivateKey::from_pkcs8_pem(pem).map_err(|e| crate::Error::InvalidManifest {
                reason: format!("Failed to parse RSA private key PEM: {e}"),
            })?;

        Ok(Self {
            signing_key,
            signer_info,
        })
    }

    /// Generate a new random signing key with the specified bit size.
    ///
    /// Common sizes are 2048, 3072, or 4096 bits.
    /// Returns the signer and the public key in PEM format.
    ///
    /// # Errors
    ///
    /// Returns an error if key generation fails.
    pub fn generate(signer_info: SignerInfo, bits: usize) -> Result<(Self, String)> {
        use rsa::pkcs8::EncodePublicKey;

        let signing_key =
            rsa::RsaPrivateKey::new(&mut rand_core::UnwrapErr(getrandom::SysRng), bits).map_err(
                |e| crate::Error::InvalidManifest {
                    reason: format!("Failed to generate RSA key: {e}"),
                },
            )?;

        let public_key = signing_key.to_public_key();
        let public_key_pem = public_key
            .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .map_err(|e| crate::Error::InvalidManifest {
                reason: format!("Failed to encode RSA public key: {e}"),
            })?;

        Ok((
            Self {
                signing_key,
                signer_info,
            },
            public_key_pem,
        ))
    }

    /// Generate a new random signing key with 2048 bits (recommended minimum).
    ///
    /// Returns the signer and the public key in PEM format.
    ///
    /// # Errors
    ///
    /// Returns an error if key generation fails.
    pub fn generate_2048(signer_info: SignerInfo) -> Result<(Self, String)> {
        Self::generate(signer_info, 2048)
    }

    /// Get the public key in PEM format.
    ///
    /// # Errors
    ///
    /// Returns an error if encoding fails.
    pub fn public_key_pem(&self) -> Result<String> {
        use rsa::pkcs8::EncodePublicKey;

        self.signing_key
            .to_public_key()
            .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .map_err(|e| crate::Error::InvalidManifest {
                reason: format!("Failed to encode RSA public key: {e}"),
            })
    }
}

impl Signer for Ps256Signer {
    fn algorithm(&self) -> SignatureAlgorithm {
        SignatureAlgorithm::PS256
    }

    fn signer_info(&self) -> SignerInfo {
        self.signer_info.clone()
    }

    fn sign(&self, document_id: &DocumentId) -> Result<Signature> {
        use base64::Engine;
        use rsa::signature::RandomizedSigner;
        use rsa::signature::SignatureEncoding;

        if document_id.is_pending() {
            return Err(crate::Error::InvalidManifest {
                reason: "Cannot sign a pending document ID".to_string(),
            });
        }

        // Create PSS signing key
        let signing_key = rsa::pss::SigningKey::<rsa::sha2::Sha256>::new(self.signing_key.clone());

        // Sign the document ID bytes (PSS is randomized)
        let signature = signing_key.sign_with_rng(
            &mut rand_core::UnwrapErr(getrandom::SysRng),
            document_id.digest(),
        );

        // Encode as base64
        let value = base64::engine::general_purpose::STANDARD.encode(signature.to_bytes());

        // Generate signature ID
        let sig_id = format!(
            "sig-{}",
            &crate::Hasher::hash(crate::HashAlgorithm::Sha256, value.as_bytes()).hex_digest()[..8]
        );

        Ok(Signature::new(
            sig_id,
            SignatureAlgorithm::PS256,
            self.signer_info.clone(),
            value,
        ))
    }
}

/// RSA-PSS verifier (PS256).
pub struct Ps256Verifier {
    verifying_key: rsa::RsaPublicKey,
}

impl Ps256Verifier {
    /// Create a new verifier from a PEM-encoded public key.
    ///
    /// # Errors
    ///
    /// Returns an error if the PEM cannot be parsed.
    pub fn from_pem(pem: &str) -> Result<Self> {
        use rsa::pkcs8::DecodePublicKey;

        let verifying_key = rsa::RsaPublicKey::from_public_key_pem(pem).map_err(|e| {
            crate::Error::InvalidManifest {
                reason: format!("Failed to parse RSA public key PEM: {e}"),
            }
        })?;

        Ok(Self { verifying_key })
    }
}

impl Verifier for Ps256Verifier {
    fn verify(
        &self,
        document_id: &DocumentId,
        signature: &Signature,
    ) -> Result<SignatureVerification> {
        use base64::Engine;
        use rsa::pss::VerifyingKey;
        use rsa::signature::Verifier as RsaVerifierTrait;

        if signature.algorithm != SignatureAlgorithm::PS256 {
            return Ok(SignatureVerification::invalid(
                &signature.id,
                format!(
                    "Algorithm mismatch: expected PS256, got {}",
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

        // Create PSS verifying key
        let verifying_key = VerifyingKey::<rsa::sha2::Sha256>::new(self.verifying_key.clone());

        // Parse signature
        let rsa_sig = rsa::pss::Signature::try_from(sig_bytes.as_slice()).map_err(|e| {
            crate::Error::InvalidManifest {
                reason: format!("Invalid PS256 signature format: {e}"),
            }
        })?;

        // Verify
        match verifying_key.verify(document_id.digest(), &rsa_sig) {
            Ok(()) => Ok(SignatureVerification::valid(&signature.id)),
            Err(e) => Ok(SignatureVerification::invalid(
                &signature.id,
                format!("PS256 signature verification failed: {e}"),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::test_helpers;

    fn generate_keypair() -> (Ps256Signer, Ps256Verifier) {
        let signer_info = SignerInfo::new("Test PS256 Signer");
        let (signer, public_key_pem) = Ps256Signer::generate_2048(signer_info).unwrap();
        let verifier = Ps256Verifier::from_pem(&public_key_pem).unwrap();
        (signer, verifier)
    }

    #[test]
    fn test_generate_and_sign() {
        let signer_info = SignerInfo::new("Test PS256 Signer");
        let (signer, public_key_pem) = Ps256Signer::generate_2048(signer_info).unwrap();

        assert!(!public_key_pem.is_empty());
        assert!(public_key_pem.contains("BEGIN PUBLIC KEY"));

        test_helpers::assert_sign_produces_valid_signature(&signer, SignatureAlgorithm::PS256);
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
