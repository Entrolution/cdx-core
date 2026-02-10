//! Shared test helpers for signature algorithm tests.
//!
//! These helpers factor out the common test logic shared across all signature
//! modules (ES256, ES384, EdDSA, PS256, ML-DSA-65). Each module calls these
//! with its own signer/verifier instances.

use super::signature::{SignatureAlgorithm, SignatureVerification};
use super::signer::{Signer, Verifier};

/// Assert that signing produces a valid signature with the expected algorithm.
pub fn assert_sign_produces_valid_signature(
    signer: &dyn Signer,
    expected_algorithm: SignatureAlgorithm,
) {
    let doc_id = crate::Hasher::hash(crate::HashAlgorithm::Sha256, b"test document");
    let signature = signer.sign(&doc_id).unwrap();

    assert_eq!(signature.algorithm, expected_algorithm);
    assert!(!signature.value.is_empty());
}

/// Assert that sign-then-verify roundtrip succeeds.
pub fn assert_sign_verify_roundtrip(signer: &dyn Signer, verifier: &dyn Verifier) {
    let doc_id = crate::Hasher::hash(crate::HashAlgorithm::Sha256, b"test document");
    let signature = signer.sign(&doc_id).unwrap();

    let result = verifier.verify(&doc_id, &signature).unwrap();
    assert!(result.is_valid());
}

/// Assert that verification fails when the document differs from what was signed.
pub fn assert_verify_wrong_document_fails(signer: &dyn Signer, verifier: &dyn Verifier) {
    let doc_id = crate::Hasher::hash(crate::HashAlgorithm::Sha256, b"original document");
    let signature = signer.sign(&doc_id).unwrap();

    let different_doc_id = crate::Hasher::hash(crate::HashAlgorithm::Sha256, b"different document");
    let result = verifier.verify(&different_doc_id, &signature).unwrap();

    assert!(!result.is_valid());
}

/// Assert that signing a pending document ID returns an error.
pub fn assert_cannot_sign_pending_id(signer: &dyn Signer) {
    let pending_id = crate::DocumentId::pending();
    let result = signer.sign(&pending_id);
    assert!(result.is_err());
}

/// Assert that verification rejects a signature with a mismatched algorithm.
///
/// Creates a valid signature, mutates the algorithm field to `wrong_algorithm`,
/// and verifies that the verifier rejects it with an "Algorithm mismatch" error.
pub fn assert_algorithm_mismatch_rejected(
    signer: &dyn Signer,
    verifier: &dyn Verifier,
    wrong_algorithm: SignatureAlgorithm,
) {
    let doc_id = crate::Hasher::hash(crate::HashAlgorithm::Sha256, b"test document");
    let mut signature = signer.sign(&doc_id).unwrap();

    signature.algorithm = wrong_algorithm;

    let result: SignatureVerification = verifier.verify(&doc_id, &signature).unwrap();
    assert!(!result.is_valid());
    assert!(
        result
            .error
            .as_ref()
            .unwrap()
            .contains("Algorithm mismatch"),
        "Expected 'Algorithm mismatch' error, got: {:?}",
        result.error
    );
}
