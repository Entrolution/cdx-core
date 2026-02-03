//! WebAuthn/FIDO2 signature verification.
//!
//! WebAuthn signatures are created client-side (in browsers or native apps)
//! using hardware security keys or platform authenticators. This module
//! provides verification of WebAuthn assertion responses.
//!
//! # Verification Process
//!
//! 1. Decode the base64-encoded fields from [`WebAuthnSignature`]
//! 2. Parse the client data JSON and verify the challenge matches the document ID
//! 3. Verify the origin matches the expected relying party
//! 4. Verify the signature over authenticator data + SHA-256(client data JSON)
//!
//! # Example
//!
//! ```ignore
//! use cdx_core::security::{WebAuthnVerifier, WebAuthnSignature, Signature};
//!
//! let verifier = WebAuthnVerifier::new(
//!     "https://example.com",  // expected origin
//!     public_key_bytes,       // credential public key
//! )?;
//!
//! let result = verifier.verify(&document_id, &signature)?;
//! ```

use base64::Engine;
use p256::ecdsa::{signature::Verifier as _, VerifyingKey};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::signature::{Signature, SignatureVerification, WebAuthnSignature};
use super::Verifier;
use crate::{DocumentId, Result};

/// WebAuthn client data structure.
///
/// Parsed from the `clientDataJSON` field of a WebAuthn assertion.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClientData {
    /// The type of operation (should be "webauthn.get" for assertions).
    #[serde(rename = "type")]
    type_: String,

    /// The challenge, base64url-encoded.
    challenge: String,

    /// The origin of the request.
    origin: String,

    /// Cross-origin flag (optional, required by WebAuthn spec for deserialization).
    #[serde(default)]
    #[allow(dead_code)]
    cross_origin: Option<bool>,
}

/// WebAuthn signature verifier.
///
/// Verifies WebAuthn assertion responses using the credential's public key.
/// The document ID is used as the challenge for verification.
pub struct WebAuthnVerifier {
    /// Expected origin (e.g., `https://example.com`).
    expected_origin: String,

    /// The credential's public key for signature verification.
    verifying_key: VerifyingKey,

    /// Expected credential ID (optional, for additional validation).
    expected_credential_id: Option<Vec<u8>>,
}

impl WebAuthnVerifier {
    /// Create a new WebAuthn verifier.
    ///
    /// # Arguments
    ///
    /// * `expected_origin` - The expected origin (e.g., `https://example.com`)
    /// * `public_key` - The credential's public key in uncompressed SEC1 format (65 bytes)
    ///   or compressed format (33 bytes)
    ///
    /// # Errors
    ///
    /// Returns an error if the public key cannot be parsed.
    pub fn new(expected_origin: impl Into<String>, public_key: &[u8]) -> Result<Self> {
        let verifying_key = VerifyingKey::from_sec1_bytes(public_key).map_err(|e| {
            crate::Error::InvalidManifest {
                reason: format!("Invalid WebAuthn public key: {e}"),
            }
        })?;

        Ok(Self {
            expected_origin: expected_origin.into(),
            verifying_key,
            expected_credential_id: None,
        })
    }

    /// Create a verifier from a PEM-encoded public key.
    ///
    /// # Errors
    ///
    /// Returns an error if the PEM cannot be parsed.
    pub fn from_pem(expected_origin: impl Into<String>, pem: &str) -> Result<Self> {
        use p256::pkcs8::DecodePublicKey;

        let verifying_key =
            VerifyingKey::from_public_key_pem(pem).map_err(|e| crate::Error::InvalidManifest {
                reason: format!("Invalid WebAuthn public key PEM: {e}"),
            })?;

        Ok(Self {
            expected_origin: expected_origin.into(),
            verifying_key,
            expected_credential_id: None,
        })
    }

    /// Set the expected credential ID for additional validation.
    #[must_use]
    pub fn with_credential_id(mut self, credential_id: Vec<u8>) -> Self {
        self.expected_credential_id = Some(credential_id);
        self
    }

    /// Verify a WebAuthn signature.
    fn verify_webauthn(
        &self,
        document_id: &DocumentId,
        webauthn: &WebAuthnSignature,
    ) -> Result<SignatureVerification> {
        let engine = base64::engine::general_purpose::STANDARD;

        // Decode credential ID
        let credential_id =
            engine
                .decode(&webauthn.credential_id)
                .map_err(|e| crate::Error::InvalidManifest {
                    reason: format!("Invalid credential ID base64: {e}"),
                })?;

        // Verify credential ID if expected
        if let Some(ref expected) = self.expected_credential_id {
            if &credential_id != expected {
                return Ok(SignatureVerification::invalid("", "Credential ID mismatch"));
            }
        }

        // Decode client data JSON
        let client_data_bytes = engine.decode(&webauthn.client_data_json).map_err(|e| {
            crate::Error::InvalidManifest {
                reason: format!("Invalid clientDataJSON base64: {e}"),
            }
        })?;

        // Parse client data
        let client_data: ClientData = serde_json::from_slice(&client_data_bytes).map_err(|e| {
            crate::Error::InvalidManifest {
                reason: format!("Invalid clientDataJSON: {e}"),
            }
        })?;

        // Verify type
        if client_data.type_ != "webauthn.get" {
            return Ok(SignatureVerification::invalid(
                "",
                format!(
                    "Invalid type: expected 'webauthn.get', got '{}'",
                    client_data.type_
                ),
            ));
        }

        // Verify origin
        if client_data.origin != self.expected_origin {
            return Ok(SignatureVerification::invalid(
                "",
                format!(
                    "Origin mismatch: expected '{}', got '{}'",
                    self.expected_origin, client_data.origin
                ),
            ));
        }

        // Verify challenge matches document ID
        // WebAuthn uses base64url encoding for the challenge
        let challenge_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(&client_data.challenge)
            .map_err(|e| crate::Error::InvalidManifest {
                reason: format!("Invalid challenge base64url: {e}"),
            })?;

        if challenge_bytes != document_id.digest() {
            return Ok(SignatureVerification::invalid(
                "",
                "Challenge does not match document ID",
            ));
        }

        // Decode authenticator data
        let authenticator_data = engine.decode(&webauthn.authenticator_data).map_err(|e| {
            crate::Error::InvalidManifest {
                reason: format!("Invalid authenticatorData base64: {e}"),
            }
        })?;

        // Verify authenticator data is at least 37 bytes (RP ID hash + flags + counter)
        if authenticator_data.len() < 37 {
            return Ok(SignatureVerification::invalid(
                "",
                "Authenticator data too short",
            ));
        }

        // Check user presence flag (bit 0 of flags byte at offset 32)
        let flags = authenticator_data[32];
        if flags & 0x01 == 0 {
            return Ok(SignatureVerification::invalid(
                "",
                "User presence flag not set",
            ));
        }

        // Decode signature
        let signature_bytes =
            engine
                .decode(&webauthn.signature)
                .map_err(|e| crate::Error::InvalidManifest {
                    reason: format!("Invalid signature base64: {e}"),
                })?;

        // Parse the signature (DER-encoded ECDSA signature)
        let signature = p256::ecdsa::DerSignature::from_bytes(&signature_bytes).map_err(|e| {
            crate::Error::InvalidManifest {
                reason: format!("Invalid ECDSA signature: {e}"),
            }
        })?;

        // Compute the signed data: authenticator_data || SHA-256(clientDataJSON)
        let client_data_hash = Sha256::digest(&client_data_bytes);
        let mut signed_data = authenticator_data.clone();
        signed_data.extend_from_slice(&client_data_hash);

        // Verify the signature
        match self.verifying_key.verify(&signed_data, &signature) {
            Ok(()) => Ok(SignatureVerification::valid("")),
            Err(e) => Ok(SignatureVerification::invalid(
                "",
                format!("Signature verification failed: {e}"),
            )),
        }
    }
}

impl Verifier for WebAuthnVerifier {
    fn verify(
        &self,
        document_id: &DocumentId,
        signature: &Signature,
    ) -> Result<SignatureVerification> {
        // Check if this is a WebAuthn signature
        let Some(webauthn) = &signature.webauthn else {
            return Ok(SignatureVerification::invalid(
                &signature.id,
                "Not a WebAuthn signature",
            ));
        };

        // Verify the WebAuthn assertion
        let mut result = self.verify_webauthn(document_id, webauthn)?;

        // Update the signature ID in the result
        result.signature_id.clone_from(&signature.id);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::SignerInfo;
    use crate::{HashAlgorithm, Hasher};

    #[test]
    fn test_webauthn_signature_new() {
        let sig = WebAuthnSignature::new(
            "Y3JlZGVudGlhbC1pZA==",
            "YXV0aGVudGljYXRvci1kYXRh",
            "Y2xpZW50LWRhdGEtanNvbg==",
            "c2lnbmF0dXJl",
        );

        assert_eq!(sig.credential_id, "Y3JlZGVudGlhbC1pZA==");
        assert_eq!(sig.authenticator_data, "YXV0aGVudGljYXRvci1kYXRh");
        assert_eq!(sig.client_data_json, "Y2xpZW50LWRhdGEtanNvbg==");
        assert_eq!(sig.signature, "c2lnbmF0dXJl");
    }

    #[test]
    fn test_signature_new_webauthn() {
        let webauthn = WebAuthnSignature::new("Y3JlZA==", "YXV0aA==", "Y2xpZW50", "c2ln");

        let signer = SignerInfo::new("Test User");
        let sig = Signature::new_webauthn("sig-webauthn-1", signer, webauthn);

        assert!(sig.is_webauthn());
        assert_eq!(sig.algorithm, super::super::SignatureAlgorithm::ES256);
        assert!(sig.value.is_empty());
        assert!(sig.webauthn_data().is_some());
    }

    #[test]
    fn test_webauthn_signature_serialization() {
        let webauthn =
            WebAuthnSignature::new("credential-id", "auth-data", "client-data", "signature");

        let json = serde_json::to_string(&webauthn).unwrap();
        assert!(json.contains("\"credentialId\":\"credential-id\""));
        assert!(json.contains("\"authenticatorData\":\"auth-data\""));
        assert!(json.contains("\"clientDataJson\":\"client-data\""));
        assert!(json.contains("\"signature\":\"signature\""));

        let parsed: WebAuthnSignature = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.credential_id, "credential-id");
    }

    #[test]
    fn test_verifier_rejects_non_webauthn() {
        // Generate a test key pair
        let signing_key = p256::ecdsa::SigningKey::random(&mut rand_core::OsRng);
        let verifying_key = signing_key.verifying_key();
        let public_key = verifying_key.to_sec1_bytes();

        let verifier = WebAuthnVerifier::new("https://example.com", &public_key).unwrap();

        let doc_id = Hasher::hash(HashAlgorithm::Sha256, b"test");
        let signer = SignerInfo::new("Test");
        let sig = Signature::new(
            "sig-1",
            super::super::SignatureAlgorithm::ES256,
            signer,
            "base64sig",
        );

        let result = verifier.verify(&doc_id, &sig).unwrap();
        assert!(!result.is_valid());
        assert!(result.error.as_ref().unwrap().contains("Not a WebAuthn"));
    }
}
