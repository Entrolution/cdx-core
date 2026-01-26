//! Integration tests for cdx-core.
//!
//! These tests verify end-to-end functionality of the library.

use cdx_core::{Document, DocumentState, Result};

/// Test creating a document, saving it, and reopening it.
#[test]
fn test_create_save_reopen() -> Result<()> {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("test.cdx");

    // Create a document
    let doc = Document::builder()
        .title("Integration Test Document")
        .creator("Test Runner")
        .add_heading(1, "Test Heading")
        .add_paragraph("This is a test paragraph.")
        .add_heading(2, "Subheading")
        .add_paragraph("More content here.")
        .build()?;

    assert_eq!(doc.state(), DocumentState::Draft);

    // Save the document
    doc.save(&file_path)?;

    // Reopen the document
    let reopened = Document::open(&file_path)?;

    // Verify the content matches
    assert_eq!(
        reopened.dublin_core().terms.title,
        doc.dublin_core().terms.title
    );
    assert_eq!(
        reopened.dublin_core().terms.creator,
        doc.dublin_core().terms.creator
    );
    assert_eq!(reopened.content().blocks.len(), doc.content().blocks.len());

    Ok(())
}

/// Test document verification on a freshly created document.
#[test]
fn test_verification_fresh_document() -> Result<()> {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("verify.cdx");

    let doc = Document::builder()
        .title("Verification Test")
        .creator("Test")
        .add_paragraph("Content to verify.")
        .build()?;

    doc.save(&file_path)?;

    let reopened = Document::open(&file_path)?;
    let report = reopened.verify()?;

    assert!(
        report.is_valid(),
        "Fresh document should verify: {:?}",
        report.errors
    );

    Ok(())
}

/// Test document ID computation is deterministic.
#[test]
fn test_document_id_deterministic() -> Result<()> {
    let doc1 = Document::builder()
        .title("Same Title")
        .creator("Same Creator")
        .add_paragraph("Same content.")
        .build()?;

    let doc2 = Document::builder()
        .title("Same Title")
        .creator("Same Creator")
        .add_paragraph("Same content.")
        .build()?;

    let id1 = doc1.compute_id()?;
    let id2 = doc2.compute_id()?;

    assert_eq!(id1, id2, "Same content should produce same ID");

    Ok(())
}

/// Test that different content produces different IDs.
#[test]
fn test_document_id_changes_with_content() -> Result<()> {
    let doc1 = Document::builder()
        .title("Title")
        .creator("Creator")
        .add_paragraph("Content A")
        .build()?;

    let doc2 = Document::builder()
        .title("Title")
        .creator("Creator")
        .add_paragraph("Content B")
        .build()?;

    let id1 = doc1.compute_id()?;
    let id2 = doc2.compute_id()?;

    assert_ne!(id1, id2, "Different content should produce different IDs");

    Ok(())
}

/// Test opening from bytes (in-memory).
#[test]
fn test_open_from_bytes() -> Result<()> {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("bytes.cdx");

    let doc = Document::builder()
        .title("Bytes Test")
        .creator("Test")
        .add_paragraph("In-memory test.")
        .build()?;

    doc.save(&file_path)?;

    // Read file into bytes
    let bytes = std::fs::read(&file_path)?;

    // Open from bytes
    let from_bytes = Document::from_bytes(bytes)?;

    assert_eq!(
        from_bytes.dublin_core().terms.title,
        doc.dublin_core().terms.title
    );

    Ok(())
}

/// Test multiple blocks of different types.
#[test]
fn test_multiple_block_types() -> Result<()> {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("blocks.cdx");

    let doc = Document::builder()
        .title("Block Types Test")
        .creator("Test")
        .add_heading(1, "Main Title")
        .add_paragraph("Introduction paragraph.")
        .add_heading(2, "Section 1")
        .add_paragraph("First section content.")
        .add_heading(2, "Section 2")
        .add_paragraph("Second section content.")
        .add_heading(3, "Subsection 2.1")
        .add_paragraph("Subsection content.")
        .build()?;

    doc.save(&file_path)?;

    let reopened = Document::open(&file_path)?;
    assert_eq!(reopened.content().blocks.len(), 8);

    Ok(())
}

/// Test empty document (no content blocks).
#[test]
fn test_empty_content() -> Result<()> {
    let doc = Document::builder()
        .title("Empty Document")
        .creator("Test")
        .build()?;

    assert!(doc.content().is_empty());

    Ok(())
}

#[cfg(feature = "signatures")]
mod signature_tests {
    use super::*;
    use cdx_core::security::{EcdsaSigner, EcdsaVerifier, Signer, SignerInfo, Verifier};

    /// Test signing and verifying a document.
    #[test]
    fn test_sign_and_verify_integration() -> Result<()> {
        let doc = Document::builder()
            .title("Signed Document")
            .creator("Signer")
            .add_paragraph("Content to be signed.")
            .build()?;

        let doc_id = doc.compute_id()?;

        // Generate a key pair
        let signer_info = SignerInfo::new("Test Signer").with_email("test@example.com");
        let (signer, public_key_pem) = EcdsaSigner::generate(signer_info)?;

        // Sign
        let signature = signer.sign(&doc_id)?;

        // Verify with matching key
        let verifier = EcdsaVerifier::from_pem(&public_key_pem)?;
        let result = verifier.verify(&doc_id, &signature)?;

        assert!(result.is_valid(), "Signature should verify");

        Ok(())
    }

    /// Test that verification fails with wrong document.
    #[test]
    fn test_signature_fails_for_different_document() -> Result<()> {
        let doc1 = Document::builder()
            .title("Document 1")
            .creator("Test")
            .add_paragraph("Original content.")
            .build()?;

        let doc2 = Document::builder()
            .title("Document 2")
            .creator("Test")
            .add_paragraph("Different content.")
            .build()?;

        let id1 = doc1.compute_id()?;
        let id2 = doc2.compute_id()?;

        let signer_info = SignerInfo::new("Signer");
        let (signer, public_key_pem) = EcdsaSigner::generate(signer_info)?;

        // Sign doc1
        let signature = signer.sign(&id1)?;

        // Try to verify against doc2
        let verifier = EcdsaVerifier::from_pem(&public_key_pem)?;
        let result = verifier.verify(&id2, &signature)?;

        assert!(
            !result.is_valid(),
            "Signature should not verify for different document"
        );

        Ok(())
    }

    /// Test that signatures are persisted when saving and reopening a document.
    #[test]
    fn test_signature_persistence() -> Result<()> {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("signed.cdx");

        // Create a document
        let mut doc = Document::builder()
            .title("Document with Signature")
            .creator("Test")
            .add_paragraph("Content to be signed.")
            .build()?;

        assert!(
            !doc.has_signatures(),
            "New document should have no signatures"
        );

        // Compute the document ID
        let doc_id = doc.compute_id()?;

        // Generate a key pair and sign
        let signer_info = SignerInfo::new("Test Signer").with_email("test@example.com");
        let (signer, public_key_pem) = EcdsaSigner::generate(signer_info)?;
        let signature = signer.sign(&doc_id)?;
        let signature_id = signature.id.clone();

        // Add signature to document
        doc.add_signature(signature)?;

        assert!(
            doc.has_signatures(),
            "Document should have signatures after adding"
        );
        assert_eq!(
            doc.signatures().len(),
            1,
            "Should have exactly one signature"
        );

        // Save the document
        doc.save(&file_path)?;

        // Reopen the document
        let reopened = Document::open(&file_path)?;

        // Verify signatures are persisted
        assert!(
            reopened.has_signatures(),
            "Reopened document should have signatures"
        );
        assert_eq!(
            reopened.signatures().len(),
            1,
            "Reopened document should have exactly one signature"
        );

        let persisted_sig = &reopened.signatures()[0];
        assert_eq!(persisted_sig.id, signature_id, "Signature ID should match");

        // Verify the signature against the reopened document
        let reopened_doc_id = reopened.compute_id()?;
        let verifier = EcdsaVerifier::from_pem(&public_key_pem)?;
        let result = verifier.verify(&reopened_doc_id, persisted_sig)?;

        assert!(result.is_valid(), "Persisted signature should verify");

        Ok(())
    }

    /// Test adding multiple signatures to a document.
    #[test]
    fn test_multiple_signatures() -> Result<()> {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("multi_signed.cdx");

        // Create a document
        let mut doc = Document::builder()
            .title("Multi-Signed Document")
            .creator("Test")
            .add_paragraph("Content to be multiply signed.")
            .build()?;

        let doc_id = doc.compute_id()?;

        // Generate two key pairs and sign with both
        let signer_info1 = SignerInfo::new("Signer One");
        let (signer1, _public_key1) = EcdsaSigner::generate(signer_info1)?;
        let signature1 = signer1.sign(&doc_id)?;

        let signer_info2 = SignerInfo::new("Signer Two");
        let (signer2, _public_key2) = EcdsaSigner::generate(signer_info2)?;
        let signature2 = signer2.sign(&doc_id)?;

        // Add both signatures
        doc.add_signature(signature1)?;
        doc.add_signature(signature2)?;

        assert_eq!(doc.signatures().len(), 2, "Should have two signatures");

        // Save and reopen
        doc.save(&file_path)?;
        let reopened = Document::open(&file_path)?;

        assert_eq!(
            reopened.signatures().len(),
            2,
            "Reopened document should have two signatures"
        );

        // Check signer names are preserved
        let signer_names: Vec<_> = reopened
            .signatures()
            .iter()
            .map(|s| s.signer.name.as_str())
            .collect();
        assert!(signer_names.contains(&"Signer One"));
        assert!(signer_names.contains(&"Signer Two"));

        Ok(())
    }
}
