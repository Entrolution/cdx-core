//! Roundtrip tests for the Swift bridge type conversion layer.

use super::*;

#[test]
fn test_create_document() {
    let doc = create_document().unwrap();
    let info = doc.get_manifest_info();
    assert_eq!(info.state, CdxDocumentState::Draft);
}

#[test]
fn test_create_document_with_title() {
    let doc = create_document_with_title("Test Document".to_string()).unwrap();
    let meta = doc.get_metadata();
    assert_eq!(meta.title, "Test Document");
    assert_eq!(doc.get_state(), CdxDocumentState::Draft);
}

#[test]
fn test_content_roundtrip() {
    let doc = create_document_with_title("Content Test".to_string()).unwrap();
    let content = doc.get_content();
    doc.set_content(content.clone()).unwrap();
    let content2 = doc.get_content();
    assert_eq!(content.blocks.len(), content2.blocks.len());
    assert_eq!(content.version, content2.version);
}

#[test]
fn test_bytes_roundtrip() {
    let doc = create_document_with_title("Roundtrip".to_string()).unwrap();
    let bytes = doc.to_bytes().unwrap();
    let doc2 = open_document_from_bytes(bytes).unwrap();
    let info1 = doc.get_manifest_info();
    let info2 = doc2.get_manifest_info();
    assert_eq!(info1.cdx_version, info2.cdx_version);
    assert_eq!(info1.state, info2.state);
}

#[test]
fn test_insert_and_remove_block() {
    let doc = create_document_with_title("Blocks".to_string()).unwrap();
    assert_eq!(doc.get_content().blocks.len(), 0);

    let block = CdxBlock::paragraph(
        "test-1".to_string(),
        vec![CdxTextSpan {
            value: "Hello world".to_string(),
            marks: vec![],
        }],
    );
    doc.insert_block(block, 0).unwrap();
    assert_eq!(doc.get_content().blocks.len(), 1);

    doc.remove_block("test-1".to_string()).unwrap();
    assert_eq!(doc.get_content().blocks.len(), 0);
}

#[test]
fn test_update_block() {
    let doc = create_document_with_title("Update".to_string()).unwrap();

    let block = CdxBlock::paragraph(
        "p-1".to_string(),
        vec![CdxTextSpan {
            value: "Original".to_string(),
            marks: vec![],
        }],
    );
    doc.insert_block(block, 0).unwrap();

    let updated = CdxBlock::paragraph(
        "p-1".to_string(),
        vec![CdxTextSpan {
            value: "Updated".to_string(),
            marks: vec![CdxTextMark::Bold],
        }],
    );
    doc.update_block(updated).unwrap();

    let content = doc.get_content();
    assert_eq!(content.blocks.len(), 1);
    assert_eq!(content.blocks[0].text_children[0].value, "Updated");
    assert_eq!(content.blocks[0].text_children[0].marks.len(), 1);
}

#[test]
fn test_insert_heading_block() {
    let doc = create_document_with_title("Heading".to_string()).unwrap();

    let block = CdxBlock::heading(
        "h-1".to_string(),
        2,
        vec![CdxTextSpan {
            value: "Section".to_string(),
            marks: vec![],
        }],
    );
    doc.insert_block(block, 0).unwrap();

    let content = doc.get_content();
    assert_eq!(content.blocks[0].block_type, CdxBlockType::Heading);
    assert_eq!(content.blocks[0].heading_info.as_ref().unwrap().level, 2);
}

#[test]
fn test_state_transitions() {
    let doc = create_document_with_title("States".to_string()).unwrap();
    assert_eq!(doc.get_state(), CdxDocumentState::Draft);

    doc.submit_for_review().unwrap();
    assert_eq!(doc.get_state(), CdxDocumentState::Review);

    // Revert back to draft (no signatures)
    doc.revert_to_draft().unwrap();
    assert_eq!(doc.get_state(), CdxDocumentState::Draft);
}

#[test]
fn test_invalid_state_transition() {
    let doc = create_document_with_title("Invalid".to_string()).unwrap();
    // Cannot freeze from draft (must go through review)
    let result = doc.freeze();
    assert!(result.is_err());
}

#[test]
fn test_mark_roundtrip_with_link() {
    let doc = create_document_with_title("Links".to_string()).unwrap();

    let block = CdxBlock::paragraph(
        "p-link".to_string(),
        vec![CdxTextSpan {
            value: "click here".to_string(),
            marks: vec![CdxTextMark::Link {
                href: "https://example.com".to_string(),
                title: Some("Example".to_string()),
            }],
        }],
    );
    doc.insert_block(block, 0).unwrap();

    let content = doc.get_content();
    let span = &content.blocks[0].text_children[0];
    assert_eq!(span.marks.len(), 1);
    match &span.marks[0] {
        CdxTextMark::Link { href, title } => {
            assert_eq!(href, "https://example.com");
            assert_eq!(title.as_deref(), Some("Example"));
        }
        other => panic!("Expected Link mark, got {other:?}"),
    }
}

#[test]
fn test_mark_roundtrip_with_anchor() {
    let doc = create_document_with_title("Anchors".to_string()).unwrap();

    let block = CdxBlock::paragraph(
        "p-anchor".to_string(),
        vec![CdxTextSpan {
            value: "anchored text".to_string(),
            marks: vec![CdxTextMark::Anchor {
                id: "section-1".to_string(),
            }],
        }],
    );
    doc.insert_block(block, 0).unwrap();

    let content = doc.get_content();
    let span = &content.blocks[0].text_children[0];
    match &span.marks[0] {
        CdxTextMark::Anchor { id } => assert_eq!(id, "section-1"),
        other => panic!("Expected Anchor mark, got {other:?}"),
    }
}

#[test]
fn test_mark_roundtrip_with_footnote() {
    let doc = create_document_with_title("Footnotes".to_string()).unwrap();

    let block = CdxBlock::paragraph(
        "p-fn".to_string(),
        vec![CdxTextSpan {
            value: "noted".to_string(),
            marks: vec![CdxTextMark::Footnote {
                number: 1,
                id: Some("fn-1".to_string()),
            }],
        }],
    );
    doc.insert_block(block, 0).unwrap();

    let content = doc.get_content();
    let span = &content.blocks[0].text_children[0];
    match &span.marks[0] {
        CdxTextMark::Footnote { number, id } => {
            assert_eq!(*number, 1);
            assert_eq!(id.as_deref(), Some("fn-1"));
        }
        other => panic!("Expected Footnote mark, got {other:?}"),
    }
}

#[test]
fn test_mark_roundtrip_all_simple_marks() {
    let doc = create_document_with_title("Marks".to_string()).unwrap();

    let block = CdxBlock::paragraph(
        "p-marks".to_string(),
        vec![CdxTextSpan {
            value: "styled".to_string(),
            marks: vec![
                CdxTextMark::Bold,
                CdxTextMark::Italic,
                CdxTextMark::Code,
                CdxTextMark::Strikethrough,
                CdxTextMark::Underline,
                CdxTextMark::Superscript,
                CdxTextMark::Subscript,
            ],
        }],
    );
    doc.insert_block(block, 0).unwrap();

    let content = doc.get_content();
    let span = &content.blocks[0].text_children[0];
    assert_eq!(span.marks.len(), 7);
}

#[test]
fn test_metadata_roundtrip() {
    let doc = create_document_with_title("Meta".to_string()).unwrap();
    let meta = doc.get_metadata();
    assert_eq!(meta.title, "Meta");

    let updated = CdxMetadata {
        title: "Updated Title".to_string(),
        creator: "Test Author".to_string(),
        description: Some("A description".to_string()),
        language: Some("en".to_string()),
        ..meta
    };
    doc.set_metadata(updated).unwrap();

    let meta2 = doc.get_metadata();
    assert_eq!(meta2.title, "Updated Title");
    assert_eq!(meta2.creator, "Test Author");
    assert_eq!(meta2.description.as_deref(), Some("A description"));
    assert_eq!(meta2.language.as_deref(), Some("en"));
}

#[test]
fn test_is_modified_tracking() {
    let doc = create_document_with_title("Modified".to_string()).unwrap();
    assert!(!doc.is_modified());

    let block = CdxBlock::paragraph(
        "p-mod".to_string(),
        vec![CdxTextSpan {
            value: "hello".to_string(),
            marks: vec![],
        }],
    );
    doc.insert_block(block, 0).unwrap();
    assert!(doc.is_modified());

    doc.mark_saved();
    assert!(!doc.is_modified());
}

#[test]
fn test_remove_nonexistent_block() {
    let doc = create_document_with_title("Remove".to_string()).unwrap();
    let result = doc.remove_block("nonexistent".to_string());
    assert!(result.is_err());
}

#[test]
fn test_error_mapping_from_core() {
    // Verify key error variants map correctly
    let err = cdx_core::Error::InvalidManifest {
        reason: "bad manifest".to_string(),
    };
    let cdx_err: CdxError = err.into();
    assert!(matches!(cdx_err, CdxError::InvalidManifest(_)));

    let err = cdx_core::Error::UnsupportedVersion {
        version: "99.0".to_string(),
    };
    let cdx_err: CdxError = err.into();
    assert!(matches!(cdx_err, CdxError::UnsupportedVersion(_)));

    let err = cdx_core::Error::ImmutableDocument {
        action: "edit".to_string(),
        state: cdx_core::DocumentState::Frozen,
    };
    let cdx_err: CdxError = err.into();
    assert!(matches!(cdx_err, CdxError::ImmutableDocument(_)));

    let err = cdx_core::Error::ValidationFailed {
        reason: "invalid".to_string(),
    };
    let cdx_err: CdxError = err.into();
    assert!(matches!(cdx_err, CdxError::ValidationFailed(_)));

    let err = cdx_core::Error::EncryptionError {
        reason: "bad key".to_string(),
    };
    let cdx_err: CdxError = err.into();
    assert!(matches!(cdx_err, CdxError::EncryptionError(_)));
}

#[test]
fn test_bytes_roundtrip_with_content() {
    let doc = create_document_with_title("Full Roundtrip".to_string()).unwrap();

    // Add some content
    let block = CdxBlock::paragraph(
        "p-rt".to_string(),
        vec![CdxTextSpan {
            value: "Roundtrip content".to_string(),
            marks: vec![CdxTextMark::Bold, CdxTextMark::Italic],
        }],
    );
    doc.insert_block(block, 0).unwrap();

    // Serialize and deserialize
    let bytes = doc.to_bytes().unwrap();
    let doc2 = open_document_from_bytes(bytes).unwrap();

    let content2 = doc2.get_content();
    assert_eq!(content2.blocks.len(), 1);
    assert_eq!(
        content2.blocks[0].text_children[0].value,
        "Roundtrip content"
    );
    assert_eq!(content2.blocks[0].text_children[0].marks.len(), 2);
}

#[cfg(feature = "encryption")]
#[test]
fn test_set_and_clear_encryption() {
    let doc = create_document_with_title("Encrypted".to_string()).unwrap();
    assert!(!doc.is_encrypted());
    assert!(doc.get_encryption_info().is_none());

    doc.set_encryption("test-password-123".to_string()).unwrap();
    assert!(doc.is_encrypted());

    let info = doc.get_encryption_info().unwrap();
    assert_eq!(info.algorithm, "AES-256-GCM");
    assert_eq!(info.kdf_algorithm.as_deref(), Some("Argon2id"));
    assert!(!info.has_recipients);

    doc.clear_encryption().unwrap();
    assert!(!doc.is_encrypted());
    assert!(doc.get_encryption_info().is_none());
}

#[cfg(feature = "encryption")]
#[test]
fn test_encryption_empty_password_rejected() {
    let doc = create_document_with_title("Empty".to_string()).unwrap();
    let result = doc.set_encryption(String::new());
    assert!(result.is_err());
}

#[cfg(feature = "encryption")]
#[test]
fn test_encryption_double_encrypt_rejected() {
    let doc = create_document_with_title("Double".to_string()).unwrap();
    doc.set_encryption("password1".to_string()).unwrap();
    let result = doc.set_encryption("password2".to_string());
    assert!(result.is_err());
}

#[cfg(feature = "encryption")]
#[test]
fn test_encryption_roundtrip_through_bytes() {
    let doc = create_document_with_title("EncRoundtrip".to_string()).unwrap();
    doc.set_encryption("my-password".to_string()).unwrap();

    let bytes = doc.to_bytes().unwrap();
    let doc2 = open_document_from_bytes(bytes).unwrap();

    assert!(doc2.is_encrypted());
    let info = doc2.get_encryption_info().unwrap();
    assert_eq!(info.algorithm, "AES-256-GCM");
}

#[cfg(feature = "encryption")]
#[test]
fn test_encryption_marks_modified() {
    let doc = create_document_with_title("Modified".to_string()).unwrap();
    doc.mark_saved();
    assert!(!doc.is_modified());

    doc.set_encryption("pass".to_string()).unwrap();
    assert!(doc.is_modified());

    doc.mark_saved();
    doc.clear_encryption().unwrap();
    assert!(doc.is_modified());
}
