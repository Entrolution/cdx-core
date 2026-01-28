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

/// State transition tests.
mod state_transition_tests {
    use super::*;

    /// Test draft → review transition.
    #[test]
    fn test_submit_for_review() -> Result<()> {
        let mut doc = Document::builder()
            .title("State Test")
            .creator("Test")
            .add_paragraph("Content.")
            .build()?;

        assert_eq!(doc.state(), DocumentState::Draft);

        doc.submit_for_review()?;
        assert_eq!(doc.state(), DocumentState::Review);

        Ok(())
    }

    /// Test review → draft (revert) transition.
    #[test]
    fn test_revert_to_draft() -> Result<()> {
        let mut doc = Document::builder()
            .title("Revert Test")
            .creator("Test")
            .add_paragraph("Content.")
            .build()?;

        doc.submit_for_review()?;
        assert_eq!(doc.state(), DocumentState::Review);

        doc.revert_to_draft()?;
        assert_eq!(doc.state(), DocumentState::Draft);

        Ok(())
    }

    /// Test that revert fails when document has signatures.
    #[cfg(feature = "signatures")]
    #[test]
    fn test_revert_fails_with_signatures() -> Result<()> {
        use cdx_core::security::{EcdsaSigner, Signer, SignerInfo};

        let mut doc = Document::builder()
            .title("Signed Revert Test")
            .creator("Test")
            .add_paragraph("Content.")
            .build()?;

        let doc_id = doc.compute_id()?;

        // Add a signature
        let signer_info = SignerInfo::new("Test Signer");
        let (signer, _) = EcdsaSigner::generate(signer_info)?;
        let signature = signer.sign(&doc_id)?;
        doc.add_signature(signature)?;

        doc.submit_for_review()?;

        // Revert should fail because document has signatures
        let result = doc.revert_to_draft();
        assert!(result.is_err(), "Revert should fail with signatures");

        Ok(())
    }

    /// Test full lifecycle: draft → review → frozen → published.
    /// Note: freeze requires lineage and precise layout, which are complex to set up.
    /// This test verifies the simpler state transitions.
    #[cfg(feature = "signatures")]
    #[test]
    fn test_lifecycle_draft_review() -> Result<()> {
        use cdx_core::security::{EcdsaSigner, Signer, SignerInfo};

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("lifecycle.cdx");

        let mut doc = Document::builder()
            .title("Lifecycle Test")
            .creator("Test")
            .add_paragraph("Content for lifecycle test.")
            .build()?;

        // Start in draft
        assert_eq!(doc.state(), DocumentState::Draft);

        // Submit for review
        doc.submit_for_review()?;
        assert_eq!(doc.state(), DocumentState::Review);

        // Sign the document
        let doc_id = doc.compute_id()?;
        let signer_info = SignerInfo::new("Reviewer");
        let (signer, _) = EcdsaSigner::generate(signer_info)?;
        let signature = signer.sign(&doc_id)?;
        doc.add_signature(signature)?;

        // Save and verify state persists
        doc.save(&file_path)?;
        let reopened = Document::open(&file_path)?;
        assert_eq!(reopened.state(), DocumentState::Review);
        assert!(reopened.has_signatures());

        Ok(())
    }

    /// Test that freeze fails without proper requirements.
    #[cfg(feature = "signatures")]
    #[test]
    fn test_freeze_requires_lineage() -> Result<()> {
        use cdx_core::security::{EcdsaSigner, Signer, SignerInfo};

        let mut doc = Document::builder()
            .title("Freeze Test")
            .creator("Test")
            .add_paragraph("Content.")
            .build()?;

        let doc_id = doc.compute_id()?;
        let signer_info = SignerInfo::new("Signer");
        let (signer, _) = EcdsaSigner::generate(signer_info)?;
        let signature = signer.sign(&doc_id)?;
        doc.add_signature(signature)?;

        doc.submit_for_review()?;

        // Freeze should fail without lineage
        let result = doc.freeze();
        assert!(result.is_err(), "Freeze should fail without lineage");

        Ok(())
    }

    /// Test fork creates new document with lineage.
    #[test]
    fn test_fork_creates_lineage() -> Result<()> {
        let original = Document::builder()
            .title("Original Document")
            .creator("Author")
            .add_paragraph("Original content.")
            .build()?;

        let original_id = original.compute_id()?;

        let forked = original.fork()?;

        // Forked document should be in draft state
        assert_eq!(forked.state(), DocumentState::Draft);

        // Forked document should have lineage pointing to original
        let lineage = forked.manifest().lineage.as_ref();
        assert!(lineage.is_some());
        let lineage = lineage.unwrap();
        assert_eq!(lineage.parent, Some(original_id));
        assert_eq!(lineage.version, Some(2));

        Ok(())
    }

    /// Test multiple forks build ancestor chain.
    #[test]
    fn test_fork_ancestor_chain() -> Result<()> {
        let v1 = Document::builder()
            .title("Version 1")
            .creator("Author")
            .add_paragraph("First version.")
            .build()?;

        let v1_id = v1.compute_id()?;

        let v2 = v1.fork()?;
        let v2_id = v2.compute_id()?;

        let v3 = v2.fork()?;

        let lineage = v3.manifest().lineage.as_ref().unwrap();
        assert_eq!(lineage.parent, Some(v2_id.clone()));
        assert_eq!(lineage.version, Some(3));

        // Ancestors should contain v1 and v2
        assert!(lineage.ancestors.contains(&v1_id));
        assert!(lineage.ancestors.contains(&v2_id));

        Ok(())
    }
}

/// Proof and Merkle tree tests.
mod proof_tests {
    use super::*;

    /// Test generating and verifying a block proof.
    #[test]
    fn test_block_proof_generation() -> Result<()> {
        let doc = Document::builder()
            .title("Proof Test")
            .creator("Test")
            .add_heading(1, "Chapter 1")
            .add_paragraph("First paragraph.")
            .add_heading(2, "Section 1.1")
            .add_paragraph("Second paragraph.")
            .build()?;

        // Generate proof for first block (index 0)
        let proof = doc.prove_block(0)?;

        // Get block hash from index
        let index = doc.block_index()?;
        let block_hash = &index.get_block(0).unwrap().hash;

        // Verify proof
        let verified = doc.verify_proof(&proof, block_hash);
        assert!(verified, "Proof should verify against document");

        Ok(())
    }

    /// Test proof verification fails for tampered data.
    #[test]
    fn test_proof_fails_for_different_document() -> Result<()> {
        let doc1 = Document::builder()
            .title("Document 1")
            .creator("Test")
            .add_paragraph("Content A")
            .build()?;

        let doc2 = Document::builder()
            .title("Document 2")
            .creator("Test")
            .add_paragraph("Content B")
            .build()?;

        let proof = doc1.prove_block(0)?;
        let index1 = doc1.block_index()?;
        let block_hash = &index1.get_block(0).unwrap().hash;

        // Proof from doc1 should not verify against doc2
        let verified = doc2.verify_proof(&proof, block_hash);
        assert!(
            !verified,
            "Proof should not verify against different document"
        );

        Ok(())
    }

    /// Test provenance record generation.
    #[test]
    fn test_provenance_record() -> Result<()> {
        let doc = Document::builder()
            .title("Provenance Test")
            .creator("Author Name")
            .add_paragraph("Content for provenance.")
            .build()?;

        let record = doc.provenance_record()?;

        assert_eq!(record.document_id, doc.compute_id()?);
        assert!(record.merkle.block_count > 0);
        // Root is a DocumentId, check it's not pending
        assert!(!record.merkle.root.is_pending());

        Ok(())
    }

    /// Test block index generation.
    #[test]
    fn test_block_index() -> Result<()> {
        let doc = Document::builder()
            .title("Block Index Test")
            .creator("Test")
            .add_heading(1, "Title")
            .add_paragraph("First paragraph.")
            .add_paragraph("Second paragraph.")
            .build()?;

        let index = doc.block_index()?;

        // Should have 3 blocks
        assert_eq!(index.block_count(), 3);

        // Each block should have a hash
        for i in 0..3 {
            let entry = index.get_block(i).unwrap();
            assert!(!entry.hash.is_pending());
        }

        Ok(())
    }
}

/// Encryption tests.
#[cfg(feature = "encryption")]
mod encryption_tests {
    use super::*;
    use cdx_core::security::Aes256GcmEncryptor;

    /// Test basic encrypt/decrypt cycle.
    #[test]
    fn test_encrypt_decrypt_content() -> Result<()> {
        let plaintext = b"This is secret document content.";
        let key = Aes256GcmEncryptor::generate_key();
        let encryptor = Aes256GcmEncryptor::new(&key)?;

        let encrypted = encryptor.encrypt(plaintext)?;
        assert_ne!(encrypted.ciphertext.as_slice(), plaintext);

        let decrypted = encryptor.decrypt(&encrypted.ciphertext, &encrypted.nonce)?;
        assert_eq!(decrypted, plaintext);

        Ok(())
    }

    /// Test encryption with wrong key fails.
    #[test]
    fn test_decrypt_wrong_key_fails() -> Result<()> {
        let plaintext = b"Secret content";
        let key1 = Aes256GcmEncryptor::generate_key();
        let key2 = Aes256GcmEncryptor::generate_key();

        let encryptor1 = Aes256GcmEncryptor::new(&key1)?;
        let encryptor2 = Aes256GcmEncryptor::new(&key2)?;

        let encrypted = encryptor1.encrypt(plaintext)?;

        let result = encryptor2.decrypt(&encrypted.ciphertext, &encrypted.nonce);
        assert!(result.is_err(), "Decryption with wrong key should fail");

        Ok(())
    }

    /// Test large content encryption.
    #[test]
    fn test_encrypt_large_content() -> Result<()> {
        let plaintext: Vec<u8> = (0..100_000).map(|i| (i % 256) as u8).collect();
        let key = Aes256GcmEncryptor::generate_key();
        let encryptor = Aes256GcmEncryptor::new(&key)?;

        let encrypted = encryptor.encrypt(&plaintext)?;
        let decrypted = encryptor.decrypt(&encrypted.ciphertext, &encrypted.nonce)?;

        assert_eq!(decrypted, plaintext);

        Ok(())
    }
}

/// Validation tests.
mod validation_tests {
    use cdx_core::validation::{
        validate_block_index, validate_content, validate_dublin_core, validate_manifest,
    };

    #[test]
    fn test_validate_real_manifest() {
        let manifest = r#"{
            "version": "0.1",
            "id": "sha256:abcd1234",
            "state": "draft",
            "created": "2024-01-01T00:00:00Z",
            "modified": "2024-01-01T00:00:00Z",
            "content": {
                "path": "content/content.json"
            }
        }"#;

        let errors = validate_manifest(manifest).unwrap();
        assert!(
            errors.is_empty(),
            "Valid manifest should have no errors: {:?}",
            errors
        );
    }

    #[test]
    fn test_validate_real_content() {
        let content = r#"{
            "version": "0.1",
            "blocks": [
                {
                    "type": "paragraph",
                    "children": [{"type": "text", "value": "Hello"}]
                }
            ]
        }"#;

        let errors = validate_content(content).unwrap();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_real_dublin_core() {
        let dc = r#"{
            "version": "0.1",
            "title": "Test Document",
            "creator": ["Author One", "Author Two"],
            "subject": ["Testing", "Validation"],
            "language": "en"
        }"#;

        let errors = validate_dublin_core(dc).unwrap();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_real_block_index() {
        let index = r#"{
            "version": "0.1",
            "algorithm": "sha256",
            "root": "abc123def456",
            "blocks": [
                {"id": "block-1", "hash": "hash1", "index": 0},
                {"id": "block-2", "hash": "hash2", "index": 1}
            ]
        }"#;

        let errors = validate_block_index(index).unwrap();
        assert!(errors.is_empty());
    }
}

/// Round-trip tests for various document configurations.
mod round_trip_tests {
    use super::*;

    /// Test document with Dublin Core fields.
    #[test]
    fn test_dublin_core_round_trip() -> Result<()> {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("dublin.cdx");

        let doc = Document::builder()
            .title("Dublin Core Test")
            .creator("Author Name")
            .description("A test document with Dublin Core metadata.")
            .language("en-US")
            .add_paragraph("Content.")
            .build()?;

        doc.save(&file_path)?;
        let reopened = Document::open(&file_path)?;

        let dc = reopened.dublin_core();
        assert_eq!(dc.terms.title, "Dublin Core Test");
        // creator is StringOrArray, should contain our author
        assert_eq!(dc.terms.creator.as_slice().len(), 1);
        assert_eq!(dc.terms.creator.as_slice()[0], "Author Name");
        assert_eq!(
            dc.terms.description,
            Some("A test document with Dublin Core metadata.".to_string())
        );
        assert_eq!(dc.terms.language, Some("en-US".to_string()));

        Ok(())
    }

    /// Test document with complex content structure.
    #[test]
    fn test_complex_content_round_trip() -> Result<()> {
        use cdx_core::content::{Block, Text};

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("complex.cdx");

        // Build document with lists and nested content
        let doc = Document::builder()
            .title("Complex Content")
            .creator("Test")
            .add_heading(1, "Introduction")
            .add_paragraph("This is the introduction.")
            .add_heading(2, "List Section")
            .add_block(Block::unordered_list(vec![
                Block::list_item(vec![Block::paragraph(vec![Text::plain("Item 1")])]),
                Block::list_item(vec![Block::paragraph(vec![Text::plain("Item 2")])]),
                Block::list_item(vec![Block::paragraph(vec![Text::plain("Item 3")])]),
            ]))
            .add_heading(2, "Code Section")
            .add_block(Block::code_block(
                "fn main() {\n    println!(\"Hello\");\n}",
                Some("rust".to_string()),
            ))
            .build()?;

        doc.save(&file_path)?;
        let reopened = Document::open(&file_path)?;

        // Verify structure preserved
        assert_eq!(reopened.content().blocks.len(), doc.content().blocks.len());

        // Verify it validates
        let report = reopened.verify()?;
        assert!(report.is_valid());

        Ok(())
    }

    /// Test document with extensions.
    #[test]
    fn test_extension_block_round_trip() -> Result<()> {
        use cdx_core::content::Block;
        use cdx_core::extensions::ExtensionBlock;

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("extension.cdx");

        let ext_block =
            ExtensionBlock::new("custom", "widget").with_attributes(serde_json::json!({
                "color": "blue",
                "size": 42
            }));

        let doc = Document::builder()
            .title("Extension Test")
            .creator("Test")
            .add_paragraph("Before extension.")
            .add_block(Block::Extension(ext_block))
            .add_paragraph("After extension.")
            .build()?;

        doc.save(&file_path)?;
        let reopened = Document::open(&file_path)?;

        // Find the extension block
        let ext = reopened
            .content()
            .blocks
            .iter()
            .find_map(|b| b.as_extension());
        assert!(ext.is_some());

        let ext = ext.unwrap();
        assert_eq!(ext.namespace, "custom");
        assert_eq!(ext.block_type, "widget");
        assert_eq!(
            ext.attributes.get("color"),
            Some(&serde_json::json!("blue"))
        );

        Ok(())
    }
}
