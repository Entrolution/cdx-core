//! ML-DSA-65 post-quantum signature implementation (FIPS-204).
//!
//! ML-DSA (Module-Lattice Digital Signature Algorithm) is a post-quantum
//! cryptographic signature scheme standardized in FIPS-204. This module
//! implements the ML-DSA-65 parameter set, providing 128-bit post-quantum
//! security.
//!
//! # Warning
//!
//! Post-quantum cryptography is still maturing. While ML-DSA-65 is
//! standardized by NIST, implementations should be considered experimental.

use crate::{DocumentId, Result};

use super::signature::{Signature, SignatureAlgorithm, SignatureVerification, SignerInfo};
use super::signer::{Signer, Verifier};

/// ML-DSA-65 signer.
///
/// Uses the FIPS-204 ML-DSA-65 parameter set for post-quantum digital signatures.
#[cfg(feature = "ml-dsa")]
pub struct MlDsaSigner {
    secret_key: fips204::ml_dsa_65::PrivateKey,
    public_key: fips204::ml_dsa_65::PublicKey,
    signer_info: SignerInfo,
}

#[cfg(feature = "ml-dsa")]
impl MlDsaSigner {
    /// Create a signer from raw key bytes.
    ///
    /// # Arguments
    ///
    /// * `secret_key_bytes` - The secret key bytes (4032 bytes for ML-DSA-65)
    /// * `signer_info` - Information about the signer
    ///
    /// # Errors
    ///
    /// Returns an error if the key bytes are invalid.
    pub fn from_bytes(secret_key_bytes: &[u8], signer_info: SignerInfo) -> Result<Self> {
        use fips204::traits::SerDes;
        use fips204::traits::Signer as FipsSigner;

        let secret_key =
            fips204::ml_dsa_65::PrivateKey::try_from_bytes(secret_key_bytes.try_into().map_err(
                |_| crate::Error::InvalidManifest {
                    reason: format!(
                        "Invalid ML-DSA-65 secret key length: expected {}, got {}",
                        fips204::ml_dsa_65::SK_LEN,
                        secret_key_bytes.len()
                    ),
                },
            )?)
            .map_err(|e| crate::Error::InvalidManifest {
                reason: format!("Failed to parse ML-DSA-65 secret key: {e:?}"),
            })?;

        // Derive public key from secret key
        let public_key = secret_key.get_public_key();

        Ok(Self {
            secret_key,
            public_key,
            signer_info,
        })
    }

    /// Generate a new random ML-DSA-65 key pair.
    ///
    /// Returns the signer and the public key bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if key generation fails.
    pub fn generate(signer_info: SignerInfo) -> Result<(Self, Vec<u8>)> {
        use fips204::ml_dsa_65;
        use fips204::traits::SerDes;

        let (public_key, secret_key) = ml_dsa_65::try_keygen().map_err(|e| {
            crate::Error::InvalidManifest {
                reason: format!("ML-DSA-65 key generation failed: {e:?}"),
            }
        })?;

        let public_key_bytes = public_key.clone().into_bytes().to_vec();

        Ok((
            Self {
                secret_key,
                public_key,
                signer_info,
            },
            public_key_bytes,
        ))
    }

    /// Get the public key bytes.
    #[must_use]
    pub fn public_key_bytes(&self) -> Vec<u8> {
        use fips204::traits::SerDes;
        self.public_key.clone().into_bytes().to_vec()
    }

    /// Get the secret key bytes.
    ///
    /// # Security Warning
    ///
    /// Handle secret key bytes with care. Do not log or expose them.
    #[must_use]
    pub fn secret_key_bytes(&self) -> Vec<u8> {
        use fips204::traits::SerDes;
        self.secret_key.clone().into_bytes().to_vec()
    }
}

#[cfg(feature = "ml-dsa")]
impl Signer for MlDsaSigner {
    fn algorithm(&self) -> SignatureAlgorithm {
        SignatureAlgorithm::MlDsa65
    }

    fn signer_info(&self) -> SignerInfo {
        self.signer_info.clone()
    }

    fn sign(&self, document_id: &DocumentId) -> Result<Signature> {
        use base64::Engine;
        use fips204::traits::Signer as MlDsaSignerTrait;

        if document_id.is_pending() {
            return Err(crate::Error::InvalidManifest {
                reason: "Cannot sign a pending document ID".to_string(),
            });
        }

        // Sign the document ID bytes (empty context)
        let signature = self
            .secret_key
            .try_sign(document_id.digest(), &[])
            .map_err(|e| crate::Error::InvalidManifest {
                reason: format!("ML-DSA-65 signing failed: {e:?}"),
            })?;

        // Encode as base64
        let value = base64::engine::general_purpose::STANDARD.encode(signature);

        // Generate signature ID
        let sig_id = format!(
            "sig-{}",
            &crate::Hasher::hash(crate::HashAlgorithm::Sha256, value.as_bytes()).hex_digest()[..8]
        );

        Ok(Signature::new(
            sig_id,
            SignatureAlgorithm::MlDsa65,
            self.signer_info.clone(),
            value,
        ))
    }
}

/// ML-DSA-65 verifier.
#[cfg(feature = "ml-dsa")]
pub struct MlDsaVerifier {
    public_key: fips204::ml_dsa_65::PublicKey,
}

#[cfg(feature = "ml-dsa")]
impl MlDsaVerifier {
    /// Create a verifier from raw public key bytes.
    ///
    /// # Arguments
    ///
    /// * `public_key_bytes` - The public key bytes (1952 bytes for ML-DSA-65)
    ///
    /// # Errors
    ///
    /// Returns an error if the key bytes are invalid.
    pub fn from_bytes(public_key_bytes: &[u8]) -> Result<Self> {
        use fips204::traits::SerDes;

        let public_key =
            fips204::ml_dsa_65::PublicKey::try_from_bytes(public_key_bytes.try_into().map_err(
                |_| crate::Error::InvalidManifest {
                    reason: format!(
                        "Invalid ML-DSA-65 public key length: expected {}, got {}",
                        fips204::ml_dsa_65::PK_LEN,
                        public_key_bytes.len()
                    ),
                },
            )?)
            .map_err(|e| crate::Error::InvalidManifest {
                reason: format!("Failed to parse ML-DSA-65 public key: {e:?}"),
            })?;

        Ok(Self { public_key })
    }
}

#[cfg(feature = "ml-dsa")]
impl Verifier for MlDsaVerifier {
    fn verify(
        &self,
        document_id: &DocumentId,
        signature: &Signature,
    ) -> Result<SignatureVerification> {
        use base64::Engine;
        use fips204::traits::Verifier as MlDsaVerifierTrait;

        if signature.algorithm != SignatureAlgorithm::MlDsa65 {
            return Ok(SignatureVerification::invalid(
                &signature.id,
                format!(
                    "Algorithm mismatch: expected ML-DSA-65, got {}",
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

        // Convert to fixed-size array reference
        let sig_array: &[u8; fips204::ml_dsa_65::SIG_LEN] =
            sig_bytes.as_slice().try_into().map_err(|_| crate::Error::InvalidManifest {
                reason: format!(
                    "Invalid ML-DSA-65 signature length: expected {}, got {}",
                    fips204::ml_dsa_65::SIG_LEN,
                    sig_bytes.len()
                ),
            })?;

        // Verify (empty context)
        let is_valid = self.public_key.verify(document_id.digest(), sig_array, &[]);

        if is_valid {
            Ok(SignatureVerification::valid(&signature.id))
        } else {
            Ok(SignatureVerification::invalid(
                &signature.id,
                "ML-DSA-65 signature verification failed",
            ))
        }
    }
}

#[cfg(all(test, feature = "ml-dsa"))]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_sign() {
        let signer_info = SignerInfo::new("Test ML-DSA Signer");
        let (signer, public_key_bytes) = MlDsaSigner::generate(signer_info).unwrap();

        assert!(!public_key_bytes.is_empty());
        assert_eq!(public_key_bytes.len(), fips204::ml_dsa_65::PK_LEN);

        let doc_id = crate::Hasher::hash(crate::HashAlgorithm::Sha256, b"test document");
        let signature = signer.sign(&doc_id).unwrap();

        assert_eq!(signature.algorithm, SignatureAlgorithm::MlDsa65);
        assert!(!signature.value.is_empty());
    }

    #[test]
    fn test_sign_and_verify() {
        let signer_info = SignerInfo::new("Test ML-DSA Signer");
        let (signer, public_key_bytes) = MlDsaSigner::generate(signer_info).unwrap();

        let doc_id = crate::Hasher::hash(crate::HashAlgorithm::Sha256, b"test document");
        let signature = signer.sign(&doc_id).unwrap();

        let verifier = MlDsaVerifier::from_bytes(&public_key_bytes).unwrap();
        let result = verifier.verify(&doc_id, &signature).unwrap();

        assert!(result.is_valid());
    }

    #[test]
    fn test_verify_wrong_document() {
        let signer_info = SignerInfo::new("Test ML-DSA Signer");
        let (signer, public_key_bytes) = MlDsaSigner::generate(signer_info).unwrap();

        let doc_id = crate::Hasher::hash(crate::HashAlgorithm::Sha256, b"original document");
        let signature = signer.sign(&doc_id).unwrap();

        let different_doc_id =
            crate::Hasher::hash(crate::HashAlgorithm::Sha256, b"different document");

        let verifier = MlDsaVerifier::from_bytes(&public_key_bytes).unwrap();
        let result = verifier.verify(&different_doc_id, &signature).unwrap();

        assert!(!result.is_valid());
    }

    #[test]
    fn test_cannot_sign_pending_id() {
        let signer_info = SignerInfo::new("Test ML-DSA Signer");
        let (signer, _) = MlDsaSigner::generate(signer_info).unwrap();

        let pending_id = crate::DocumentId::pending();
        let result = signer.sign(&pending_id);

        assert!(result.is_err());
    }

    #[test]
    fn test_key_round_trip() {
        let signer_info = SignerInfo::new("Test Signer");
        let (original_signer, _) = MlDsaSigner::generate(signer_info.clone()).unwrap();

        // Get key bytes
        let secret_bytes = original_signer.secret_key_bytes();
        let public_bytes = original_signer.public_key_bytes();

        // Recreate signer from bytes
        let restored_signer = MlDsaSigner::from_bytes(&secret_bytes, signer_info).unwrap();

        // Sign with both and verify they produce valid signatures
        let doc_id = crate::Hasher::hash(crate::HashAlgorithm::Sha256, b"test document");
        let sig1 = original_signer.sign(&doc_id).unwrap();
        let sig2 = restored_signer.sign(&doc_id).unwrap();

        let verifier = MlDsaVerifier::from_bytes(&public_bytes).unwrap();
        assert!(verifier.verify(&doc_id, &sig1).unwrap().is_valid());
        assert!(verifier.verify(&doc_id, &sig2).unwrap().is_valid());
    }
}
