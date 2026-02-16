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

use crate::error::invalid_manifest;
use crate::{DocumentId, Result};

use super::signature::{Signature, SignatureAlgorithm, SignatureVerification, SignerInfo};
use super::signer::{Signer, Verifier};

/// ML-DSA-65 signer.
///
/// Uses the FIPS-204 ML-DSA-65 parameter set for post-quantum digital signatures.
///
/// # Key Format
///
/// Keys are represented as 32-byte seeds. A seed deterministically derives
/// both the signing and verifying keys via ML-DSA.KeyGen_internal (FIPS 204).
#[cfg(feature = "ml-dsa")]
pub struct MlDsaSigner {
    signing_key: ml_dsa::SigningKey<ml_dsa::MlDsa65>,
    seed: [u8; 32],
    signer_info: SignerInfo,
}

#[cfg(feature = "ml-dsa")]
impl MlDsaSigner {
    /// Create a signer from a 32-byte seed.
    ///
    /// The seed deterministically derives the full signing and verifying keys.
    ///
    /// # Arguments
    ///
    /// * `seed_bytes` - The 32-byte seed
    /// * `signer_info` - Information about the signer
    ///
    /// # Errors
    ///
    /// Returns an error if the seed is not exactly 32 bytes.
    pub fn from_bytes(seed_bytes: &[u8], signer_info: SignerInfo) -> Result<Self> {
        use ml_dsa::KeyGen;

        let seed: [u8; 32] = seed_bytes.try_into().map_err(|_| {
            invalid_manifest(format!(
                "Invalid ML-DSA-65 seed length: expected 32, got {}",
                seed_bytes.len()
            ))
        })?;

        let kp = ml_dsa::MlDsa65::from_seed(&seed.into());

        Ok(Self {
            signing_key: kp.signing_key().clone(),
            seed,
            signer_info,
        })
    }

    /// Generate a new random ML-DSA-65 key pair.
    ///
    /// Returns the signer and the encoded public (verifying) key bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if key generation fails.
    #[allow(clippy::missing_panics_doc)] // getrandom::SysRng only fails on misconfigured systems
    pub fn generate(signer_info: SignerInfo) -> Result<(Self, Vec<u8>)> {
        use ml_dsa::KeyGen;

        let kp = ml_dsa::MlDsa65::key_gen(&mut rand_core::UnwrapErr(getrandom::SysRng));
        let seed: [u8; 32] = kp.to_seed().into();
        let public_key_bytes = kp.verifying_key().encode().to_vec();

        Ok((
            Self {
                signing_key: kp.signing_key().clone(),
                seed,
                signer_info,
            },
            public_key_bytes,
        ))
    }

    /// Get the public (verifying) key bytes.
    #[must_use]
    pub fn public_key_bytes(&self) -> Vec<u8> {
        self.signing_key.verifying_key().encode().to_vec()
    }

    /// Get the secret key seed (32 bytes).
    ///
    /// # Security Warning
    ///
    /// Handle seed bytes with care. Do not log or expose them.
    #[must_use]
    pub fn secret_key_bytes(&self) -> Vec<u8> {
        self.seed.to_vec()
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
        use ml_dsa::signature::Signer as MlDsaSignerTrait;

        if document_id.is_pending() {
            return Err(crate::Error::InvalidManifest {
                reason: "Cannot sign a pending document ID".to_string(),
            });
        }

        // Sign the document ID bytes
        let signature = self.signing_key.sign(document_id.digest());

        // Encode as base64
        let value = base64::engine::general_purpose::STANDARD.encode(signature.encode());

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
    verifying_key: ml_dsa::VerifyingKey<ml_dsa::MlDsa65>,
}

#[cfg(feature = "ml-dsa")]
impl MlDsaVerifier {
    /// Create a verifier from encoded public key bytes.
    ///
    /// # Arguments
    ///
    /// * `public_key_bytes` - The encoded verifying key bytes
    ///
    /// # Errors
    ///
    /// Returns an error if the key bytes are invalid.
    pub fn from_bytes(public_key_bytes: &[u8]) -> Result<Self> {
        let verifying_key =
            ml_dsa::VerifyingKey::decode(public_key_bytes.try_into().map_err(|_| {
                invalid_manifest(format!(
                    "Invalid ML-DSA-65 public key length: got {}",
                    public_key_bytes.len()
                ))
            })?);

        Ok(Self { verifying_key })
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
        use ml_dsa::signature::Verifier as MlDsaVerifierTrait;

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
            .map_err(|e| invalid_manifest(format!("Failed to decode signature: {e}")))?;

        // Parse ML-DSA signature
        let ml_sig =
            ml_dsa::Signature::<ml_dsa::MlDsa65>::try_from(sig_bytes.as_slice()).map_err(|_| {
                invalid_manifest(format!(
                    "Invalid ML-DSA-65 signature length: got {}",
                    sig_bytes.len()
                ))
            })?;

        // Verify
        match self.verifying_key.verify(document_id.digest(), &ml_sig) {
            Ok(()) => Ok(SignatureVerification::valid(&signature.id)),
            Err(e) => Ok(SignatureVerification::invalid(
                &signature.id,
                format!("ML-DSA-65 signature verification failed: {e}"),
            )),
        }
    }
}

#[cfg(all(test, feature = "ml-dsa"))]
mod tests {
    use super::*;
    use crate::security::test_helpers;

    fn generate_keypair() -> (MlDsaSigner, MlDsaVerifier) {
        let signer_info = SignerInfo::new("Test ML-DSA Signer");
        let (signer, public_key_bytes) = MlDsaSigner::generate(signer_info).unwrap();
        let verifier = MlDsaVerifier::from_bytes(&public_key_bytes).unwrap();
        (signer, verifier)
    }

    #[test]
    fn test_generate_and_sign() {
        let signer_info = SignerInfo::new("Test ML-DSA Signer");
        let (signer, public_key_bytes) = MlDsaSigner::generate(signer_info).unwrap();

        assert!(!public_key_bytes.is_empty());

        test_helpers::assert_sign_produces_valid_signature(&signer, SignatureAlgorithm::MlDsa65);
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

    #[test]
    fn test_key_round_trip() {
        let signer_info = SignerInfo::new("Test Signer");
        let (original_signer, _) = MlDsaSigner::generate(signer_info.clone()).unwrap();

        let secret_bytes = original_signer.secret_key_bytes();
        let public_bytes = original_signer.public_key_bytes();

        let restored_signer = MlDsaSigner::from_bytes(&secret_bytes, signer_info).unwrap();

        let doc_id = crate::Hasher::hash(crate::HashAlgorithm::Sha256, b"test document");
        let sig1 = original_signer.sign(&doc_id).unwrap();
        let sig2 = restored_signer.sign(&doc_id).unwrap();

        let verifier = MlDsaVerifier::from_bytes(&public_bytes).unwrap();
        assert!(verifier.verify(&doc_id, &sig1).unwrap().is_valid());
        assert!(verifier.verify(&doc_id, &sig2).unwrap().is_valid());
    }
}
