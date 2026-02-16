//! Spec conformance tests.
//!
//! These tests verify that cdx-core's behavior matches the Codex file format
//! specification. This includes:
//!
//! - **Wire format**: JSON serialization matches spec examples
//! - **Hash boundary**: Document ID includes/excludes the correct data
//! - **Block types**: All block types use correct `type` strings
//! - **State machine**: State transitions enforce spec requirements
//! - **Manifest**: Manifest fields match spec constraints
//! - **Provenance**: Lineage and Merkle structures follow spec
//! - **Metadata**: Dublin Core requirements enforced
//! - **Extensions**: Extension validation follows spec rules

use cdx_core::content::{Block, Mark, MathFormat, Text};
use cdx_core::extensions::ExtensionBlock;

// ============================================================================
// Mark format conformance
// ============================================================================

#[test]
fn simple_marks_serialize_as_strings() {
    let marks = vec![
        (Mark::Bold, "\"bold\""),
        (Mark::Italic, "\"italic\""),
        (Mark::Underline, "\"underline\""),
        (Mark::Strikethrough, "\"strikethrough\""),
        (Mark::Code, "\"code\""),
        (Mark::Superscript, "\"superscript\""),
        (Mark::Subscript, "\"subscript\""),
    ];

    for (mark, expected) in marks {
        let json = serde_json::to_string(&mark).unwrap();
        assert_eq!(
            json, expected,
            "Mark::{mark:?} should serialize as {expected}"
        );
    }
}

#[test]
fn simple_marks_deserialize_from_string() {
    let cases = vec![
        ("\"bold\"", Mark::Bold),
        ("\"italic\"", Mark::Italic),
        ("\"underline\"", Mark::Underline),
        ("\"strikethrough\"", Mark::Strikethrough),
        ("\"code\"", Mark::Code),
        ("\"superscript\"", Mark::Superscript),
        ("\"subscript\"", Mark::Subscript),
    ];

    for (json, expected) in cases {
        let mark: Mark = serde_json::from_str(json).unwrap();
        assert_eq!(
            mark, expected,
            "String {json} should deserialize to {expected:?}"
        );
    }
}

#[test]
fn simple_marks_deserialize_from_object() {
    // Backward compat: old format used objects for simple marks
    let cases = vec![
        (r#"{"type":"bold"}"#, Mark::Bold),
        (r#"{"type":"italic"}"#, Mark::Italic),
        (r#"{"type":"code"}"#, Mark::Code),
    ];

    for (json, expected) in cases {
        let mark: Mark = serde_json::from_str(json).unwrap();
        assert_eq!(
            mark, expected,
            "Object {json} should deserialize to {expected:?}"
        );
    }
}

#[test]
fn mixed_mark_array_deserializes() {
    // Mix of string and object marks in a single array
    let json = r#"["bold", {"type":"link","href":"https://example.com"}, "italic"]"#;
    let marks: Vec<Mark> = serde_json::from_str(json).unwrap();

    assert_eq!(marks.len(), 3);
    assert_eq!(marks[0], Mark::Bold);
    assert!(matches!(&marks[1], Mark::Link { href, .. } if href == "https://example.com"));
    assert_eq!(marks[2], Mark::Italic);
}

#[test]
fn extension_mark_serializes_without_wrapper() {
    use cdx_core::content::ExtensionMark;

    let mark = Mark::Extension(ExtensionMark::citation("smith2023"));
    let json = serde_json::to_string(&mark).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Type should be "semantic:citation", not "extension"
    assert_eq!(val["type"], "semantic:citation");
    assert_eq!(val["ref"], "smith2023");

    // Should NOT have wrapper fields
    assert!(val.get("namespace").is_none());
    assert!(val.get("markType").is_none());
}

#[test]
fn extension_mark_deserializes_new_format() {
    let json = r#"{"type":"semantic:citation","ref":"smith2023"}"#;
    let mark: Mark = serde_json::from_str(json).unwrap();

    if let Mark::Extension(ext) = &mark {
        assert_eq!(ext.namespace, "semantic");
        assert_eq!(ext.mark_type, "citation");
        assert_eq!(ext.get_string_attribute("ref"), Some("smith2023"));
    } else {
        panic!("Expected Extension mark, got {mark:?}");
    }
}

#[test]
fn extension_mark_deserializes_old_format() {
    // Backward compat: old "extension" wrapper format
    let json = r#"{"type":"extension","namespace":"semantic","markType":"citation","attributes":{"ref":"smith2023"}}"#;
    let mark: Mark = serde_json::from_str(json).unwrap();

    if let Mark::Extension(ext) = &mark {
        assert_eq!(ext.namespace, "semantic");
        assert_eq!(ext.mark_type, "citation");
        assert_eq!(ext.get_string_attribute("ref"), Some("smith2023"));
    } else {
        panic!("Expected Extension mark, got {mark:?}");
    }
}

#[test]
fn math_mark_uses_source_field() {
    let mark = Mark::Math {
        format: MathFormat::Latex,
        source: "E=mc^2".to_string(),
    };
    let json = serde_json::to_string(&mark).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(val["type"], "math");
    assert_eq!(val["source"], "E=mc^2");
    // "value" should NOT be present
    assert!(val.get("value").is_none());
}

#[test]
fn math_mark_backward_compat_value_field() {
    // Old format used "value" instead of "source"
    let json = r#"{"type":"math","format":"latex","value":"E=mc^2"}"#;
    let mark: Mark = serde_json::from_str(json).unwrap();

    if let Mark::Math { format, source } = &mark {
        assert_eq!(*format, MathFormat::Latex);
        assert_eq!(source, "E=mc^2");
    } else {
        panic!("Expected Math mark, got {mark:?}");
    }
}

// ============================================================================
// Block format conformance
// ============================================================================

#[test]
fn figcaption_serializes_lowercase() {
    let fc = Block::figcaption(vec![Text::plain("Figure 1")]);
    let json = serde_json::to_string(&fc).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(val["type"], "figcaption");
}

#[test]
fn figcaption_backward_compat_camel_case() {
    // Old format used "figCaption" (camelCase)
    let json = r#"{"type":"figCaption","children":[{"value":"Caption"}]}"#;
    let block: Block = serde_json::from_str(json).unwrap();
    assert_eq!(block.block_type(), "figcaption");
}

#[test]
fn extension_block_serializes_with_colon_type() {
    let ext = ExtensionBlock::new("academic", "theorem")
        .with_id("thm-1")
        .with_attributes(serde_json::json!({"variant": "lemma", "numbered": true}));
    let block = Block::Extension(ext);

    let json = serde_json::to_string(&block).unwrap();
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Type should be "academic:theorem"
    assert_eq!(val["type"], "academic:theorem");
    assert_eq!(val["id"], "thm-1");
    // Attributes should be flattened
    assert_eq!(val["variant"], "lemma");
    assert_eq!(val["numbered"], true);
    // Should NOT have wrapper fields
    assert!(val.get("namespace").is_none());
    assert!(val.get("blockType").is_none());
    assert!(val.get("attributes").is_none());
}

#[test]
fn extension_block_deserializes_new_format() {
    let json = r#"{"type":"academic:theorem","id":"thm-1","variant":"lemma","numbered":true}"#;
    let block: Block = serde_json::from_str(json).unwrap();

    if let Block::Extension(ext) = &block {
        assert_eq!(ext.namespace, "academic");
        assert_eq!(ext.block_type, "theorem");
        assert_eq!(ext.id, Some("thm-1".to_string()));
        assert_eq!(ext.get_string_attribute("variant"), Some("lemma"));
        assert_eq!(ext.get_bool_attribute("numbered"), Some(true));
    } else {
        panic!("Expected Extension block, got paragraph/etc");
    }
}

#[test]
fn extension_block_deserializes_old_format() {
    // Backward compat: old "extension" wrapper format
    let json = r#"{"type":"extension","namespace":"forms","blockType":"textInput","id":"name-field","attributes":{"label":"Name","required":true}}"#;
    let block: Block = serde_json::from_str(json).unwrap();

    if let Block::Extension(ext) = &block {
        assert_eq!(ext.namespace, "forms");
        assert_eq!(ext.block_type, "textInput");
        assert_eq!(ext.id, Some("name-field".to_string()));
        assert_eq!(ext.get_string_attribute("label"), Some("Name"));
        assert_eq!(ext.get_bool_attribute("required"), Some(true));
    } else {
        panic!("Expected Extension block");
    }
}

#[test]
fn extension_block_type_returns_colon_format() {
    let block = Block::extension("forms", "textInput");
    assert_eq!(block.block_type(), "forms:textInput");
}

// ============================================================================
// Spec example round-trips
// ============================================================================

#[test]
fn spec_example_text_with_bold_string_mark() {
    // Spec: bold marks are strings in the marks array
    let spec_json = r#"{"value":"Important","marks":["bold"]}"#;

    // Deserialize
    let text: Text = serde_json::from_str(spec_json).unwrap();
    assert_eq!(text.value, "Important");
    assert_eq!(text.marks, vec![Mark::Bold]);

    // Re-serialize matches spec format
    let output = serde_json::to_string(&text).unwrap();
    let output_val: serde_json::Value = serde_json::from_str(&output).unwrap();
    let spec_val: serde_json::Value = serde_json::from_str(spec_json).unwrap();
    assert_eq!(output_val, spec_val);
}

#[test]
fn spec_example_text_with_citation_mark() {
    // Spec: extension marks use "namespace:markType" as type, attributes flattened
    let spec_json =
        r#"{"value":"important claim","marks":[{"type":"semantic:citation","ref":"smith2023"}]}"#;

    let text: Text = serde_json::from_str(spec_json).unwrap();
    assert_eq!(text.value, "important claim");
    assert_eq!(text.marks.len(), 1);

    if let Mark::Extension(ext) = &text.marks[0] {
        assert_eq!(ext.namespace, "semantic");
        assert_eq!(ext.mark_type, "citation");
        assert_eq!(ext.get_string_attribute("ref"), Some("smith2023"));
    } else {
        panic!("Expected Extension mark");
    }

    // Re-serialize matches spec format
    let output = serde_json::to_string(&text).unwrap();
    let output_val: serde_json::Value = serde_json::from_str(&output).unwrap();
    let spec_val: serde_json::Value = serde_json::from_str(spec_json).unwrap();
    assert_eq!(output_val, spec_val);
}

#[test]
fn spec_example_extension_block_academic_theorem() {
    // Spec: extension blocks use "namespace:blockType" as type
    let spec_json = r#"{
        "type": "academic:theorem",
        "id": "thm-pythagoras",
        "variant": "theorem",
        "children": [
            {"type": "paragraph", "children": [{"value": "In a right triangle..."}]}
        ]
    }"#;

    let block: Block = serde_json::from_str(spec_json).unwrap();
    if let Block::Extension(ext) = &block {
        assert_eq!(ext.namespace, "academic");
        assert_eq!(ext.block_type, "theorem");
        assert_eq!(ext.id, Some("thm-pythagoras".to_string()));
        assert_eq!(ext.get_string_attribute("variant"), Some("theorem"));
        assert_eq!(ext.children.len(), 1);
    } else {
        panic!("Expected Extension block");
    }

    // Re-serialize and verify format
    let output = serde_json::to_string(&block).unwrap();
    let output_val: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(output_val["type"], "academic:theorem");
    assert_eq!(output_val["id"], "thm-pythagoras");
    assert_eq!(output_val["variant"], "theorem");
}

#[test]
fn spec_example_figure_with_figcaption() {
    // Spec: figcaption uses lowercase "figcaption" type
    let spec_json = r#"{
        "type": "figure",
        "children": [
            {"type": "image", "src": "photo.png", "alt": "A photo"},
            {"type": "figcaption", "children": [{"value": "Figure 1: A photo"}]}
        ]
    }"#;

    let block: Block = serde_json::from_str(spec_json).unwrap();
    if let Block::Figure(fig) = &block {
        assert_eq!(fig.children.len(), 2);
        assert_eq!(fig.children[1].block_type(), "figcaption");
    } else {
        panic!("Expected Figure block");
    }

    // Re-serialize and verify format
    let output = serde_json::to_string(&block).unwrap();
    let output_val: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(output_val["children"][1]["type"], "figcaption");
}

#[test]
fn spec_example_math_inline_mark() {
    // Spec: inline math mark uses "source" field
    let spec_json = r#"{"value":"x²","marks":[{"type":"math","format":"latex","source":"x^2"}]}"#;

    let text: Text = serde_json::from_str(spec_json).unwrap();
    if let Mark::Math { format, source } = &text.marks[0] {
        assert_eq!(*format, MathFormat::Latex);
        assert_eq!(source, "x^2");
    } else {
        panic!("Expected Math mark");
    }

    // Re-serialize matches spec
    let output = serde_json::to_string(&text).unwrap();
    let output_val: serde_json::Value = serde_json::from_str(&output).unwrap();
    let spec_val: serde_json::Value = serde_json::from_str(spec_json).unwrap();
    assert_eq!(output_val, spec_val);
}

#[test]
fn extension_block_roundtrip_preserves_format() {
    // Create → serialize → deserialize → serialize again, format should match
    let ext = ExtensionBlock::new("forms", "textInput")
        .with_id("name-field")
        .with_attributes(serde_json::json!({"label": "Full Name", "required": true}));
    let block = Block::Extension(ext);

    let json1 = serde_json::to_string(&block).unwrap();
    let parsed: Block = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&parsed).unwrap();

    let val1: serde_json::Value = serde_json::from_str(&json1).unwrap();
    let val2: serde_json::Value = serde_json::from_str(&json2).unwrap();
    assert_eq!(val1, val2);
}

#[test]
fn extension_mark_roundtrip_preserves_format() {
    use cdx_core::content::ExtensionMark;

    let mark = Mark::Extension(ExtensionMark::theorem_ref_formatted(
        "#thm-1",
        "{variant} {number}",
    ));

    let json1 = serde_json::to_string(&mark).unwrap();
    let parsed: Mark = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&parsed).unwrap();

    let val1: serde_json::Value = serde_json::from_str(&json1).unwrap();
    let val2: serde_json::Value = serde_json::from_str(&json2).unwrap();
    assert_eq!(val1, val2);
}

// ============================================================================
// Document hashing boundary tests (Phase 1A)
// Per spec §06-document-hashing.md §4.1
// ============================================================================

/// Per spec §06 §4.1 — Hash INCLUDES content blocks.
#[test]
fn test_hash_changes_with_content() {
    let doc1 = cdx_core::Document::builder()
        .title("Same Title")
        .creator("Same Creator")
        .add_paragraph("Content version A")
        .build()
        .unwrap();

    let doc2 = cdx_core::Document::builder()
        .title("Same Title")
        .creator("Same Creator")
        .add_paragraph("Content version B")
        .build()
        .unwrap();

    let id1 = doc1.compute_id().unwrap();
    let id2 = doc2.compute_id().unwrap();
    assert_ne!(id1, id2, "Different content must produce different IDs");
}

/// Per spec §06 §4.1 — Hash INCLUDES title metadata.
#[test]
fn test_hash_changes_with_title() {
    let doc1 = cdx_core::Document::builder()
        .title("Title A")
        .creator("Author")
        .add_paragraph("Same content")
        .build()
        .unwrap();

    let doc2 = cdx_core::Document::builder()
        .title("Title B")
        .creator("Author")
        .add_paragraph("Same content")
        .build()
        .unwrap();

    let id1 = doc1.compute_id().unwrap();
    let id2 = doc2.compute_id().unwrap();
    assert_ne!(id1, id2, "Different titles must produce different IDs");
}

/// Per spec §06 §4.1 — Hash INCLUDES creator metadata.
#[test]
fn test_hash_changes_with_creator() {
    let doc1 = cdx_core::Document::builder()
        .title("Title")
        .creator("Author A")
        .add_paragraph("Same content")
        .build()
        .unwrap();

    let doc2 = cdx_core::Document::builder()
        .title("Title")
        .creator("Author B")
        .add_paragraph("Same content")
        .build()
        .unwrap();

    let id1 = doc1.compute_id().unwrap();
    let id2 = doc2.compute_id().unwrap();
    assert_ne!(id1, id2, "Different creators must produce different IDs");
}

/// Per spec §06 §4.1 — Hash INCLUDES subject metadata.
#[test]
fn test_hash_changes_with_subject() {
    use cdx_core::metadata::DublinCore;

    let mut dc_a = DublinCore::new("Title", "Author");
    dc_a.set_subjects(vec!["Science".to_string()]);

    let mut dc_b = DublinCore::new("Title", "Author");
    dc_b.set_subjects(vec!["Mathematics".to_string()]);

    let doc1 = cdx_core::Document::builder()
        .title("Title")
        .creator("Author")
        .add_paragraph("Same content")
        .with_dublin_core(dc_a)
        .build()
        .unwrap();

    let doc2 = cdx_core::Document::builder()
        .title("Title")
        .creator("Author")
        .add_paragraph("Same content")
        .with_dublin_core(dc_b)
        .build()
        .unwrap();

    let id1 = doc1.compute_id().unwrap();
    let id2 = doc2.compute_id().unwrap();
    assert_ne!(id1, id2, "Different subjects must produce different IDs");
}

/// Per spec §06 §4.1 — Hash INCLUDES description metadata.
#[test]
fn test_hash_changes_with_description() {
    let doc1 = cdx_core::Document::builder()
        .title("Title")
        .creator("Author")
        .description("Description A")
        .add_paragraph("Same content")
        .build()
        .unwrap();

    let doc2 = cdx_core::Document::builder()
        .title("Title")
        .creator("Author")
        .description("Description B")
        .add_paragraph("Same content")
        .build()
        .unwrap();

    let id1 = doc1.compute_id().unwrap();
    let id2 = doc2.compute_id().unwrap();
    assert_ne!(
        id1, id2,
        "Different descriptions must produce different IDs"
    );
}

/// Per spec §06 §4.1 — Hash INCLUDES language metadata.
#[test]
fn test_hash_changes_with_language() {
    let doc1 = cdx_core::Document::builder()
        .title("Title")
        .creator("Author")
        .language("en")
        .add_paragraph("Same content")
        .build()
        .unwrap();

    let doc2 = cdx_core::Document::builder()
        .title("Title")
        .creator("Author")
        .language("fr")
        .add_paragraph("Same content")
        .build()
        .unwrap();

    let id1 = doc1.compute_id().unwrap();
    let id2 = doc2.compute_id().unwrap();
    assert_ne!(id1, id2, "Different languages must produce different IDs");
}

/// Per spec §06 §4.1 — Hash EXCLUDES presentation layers.
#[test]
fn test_hash_unchanged_by_presentation() {
    let doc1 = cdx_core::Document::builder()
        .title("Title")
        .creator("Author")
        .add_paragraph("Content")
        .build()
        .unwrap();

    let mut doc2 = cdx_core::Document::builder()
        .title("Title")
        .creator("Author")
        .add_paragraph("Content")
        .build()
        .unwrap();

    // Add a presentation reference to doc2
    let test_hash: cdx_core::DocumentId =
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
            .parse()
            .unwrap();
    doc2.manifest_mut()
        .presentation
        .push(cdx_core::PresentationRef {
            presentation_type: "paginated".to_string(),
            path: "presentation/paginated.json".to_string(),
            hash: test_hash,
            default: true,
        });

    let id1 = doc1.compute_id().unwrap();
    let id2 = doc2.compute_id().unwrap();
    assert_eq!(id1, id2, "Presentation layers must not affect document ID");
}

/// Per spec §06 §4.1 — Hash EXCLUDES security/signatures.
#[test]
fn test_hash_unchanged_by_signatures() {
    let doc1 = cdx_core::Document::builder()
        .title("Title")
        .creator("Author")
        .add_paragraph("Content")
        .build()
        .unwrap();

    let mut doc2 = cdx_core::Document::builder()
        .title("Title")
        .creator("Author")
        .add_paragraph("Content")
        .build()
        .unwrap();

    // Add a security reference to doc2
    doc2.manifest_mut().security = Some(cdx_core::SecurityRef {
        signatures: Some("security/signatures.json".to_string()),
        encryption: None,
    });

    let id1 = doc1.compute_id().unwrap();
    let id2 = doc2.compute_id().unwrap();
    assert_eq!(id1, id2, "Signatures must not affect document ID");
}

/// Per spec §06 §4.1 — Hash EXCLUDES phantom data.
#[test]
fn test_hash_unchanged_by_phantoms() {
    use cdx_core::anchor::ContentAnchor;
    use cdx_core::extensions::{
        Phantom, PhantomCluster, PhantomClusters, PhantomContent, PhantomPosition,
    };

    let doc1 = cdx_core::Document::builder()
        .title("Title")
        .creator("Author")
        .add_paragraph("Content")
        .build()
        .unwrap();

    let mut doc2 = cdx_core::Document::builder()
        .title("Title")
        .creator("Author")
        .add_paragraph("Content")
        .build()
        .unwrap();

    // Add phantom clusters to doc2
    let mut clusters = PhantomClusters::new();
    let position = PhantomPosition::new(100.0, 200.0);
    let content = PhantomContent::paragraph("Ghost text");
    let phantom = Phantom::new("p1", position, content);
    let cluster =
        PhantomCluster::new("c1", ContentAnchor::block("block-1"), "Test").with_phantom(phantom);
    clusters.add_cluster(cluster);
    doc2.set_phantom_clusters(clusters).unwrap();

    let id1 = doc1.compute_id().unwrap();
    let id2 = doc2.compute_id().unwrap();
    assert_eq!(id1, id2, "Phantom data must not affect document ID");
}

/// Per spec §06 §4.1 — Hash EXCLUDES form data.
#[test]
fn test_hash_unchanged_by_forms() {
    use cdx_core::extensions::FormData;

    let doc1 = cdx_core::Document::builder()
        .title("Title")
        .creator("Author")
        .add_paragraph("Content")
        .build()
        .unwrap();

    let mut doc2 = cdx_core::Document::builder()
        .title("Title")
        .creator("Author")
        .add_paragraph("Content")
        .build()
        .unwrap();

    // Add form data to doc2
    let mut form_data = FormData::new();
    form_data.set("name", serde_json::json!("John Doe"));
    doc2.set_form_data(form_data).unwrap();

    let id1 = doc1.compute_id().unwrap();
    let id2 = doc2.compute_id().unwrap();
    assert_eq!(id1, id2, "Form data must not affect document ID");
}

/// Per spec §06 §4.1 — Hash EXCLUDES collaboration data (comments).
#[test]
fn test_hash_unchanged_by_comments() {
    use cdx_core::extensions::{Collaborator, Comment, CommentThread};

    let doc1 = cdx_core::Document::builder()
        .title("Title")
        .creator("Author")
        .add_paragraph("Content")
        .build()
        .unwrap();

    let mut doc2 = cdx_core::Document::builder()
        .title("Title")
        .creator("Author")
        .add_paragraph("Content")
        .build()
        .unwrap();

    // Add comments to doc2
    let mut thread = CommentThread::new();
    let author = Collaborator::new("Alice");
    thread.add(Comment::new("c1", "block-1", author, "A comment"));
    doc2.set_comments(thread).unwrap();

    let id1 = doc1.compute_id().unwrap();
    let id2 = doc2.compute_id().unwrap();
    assert_eq!(id1, id2, "Collaboration data must not affect document ID");
}

/// Per spec §06 §4.3 — Hash determinism: same content always produces same hash.
#[test]
fn test_hash_determinism() {
    let build_doc = || {
        cdx_core::Document::builder()
            .title("Determinism Test")
            .creator("Author")
            .description("A test document")
            .language("en")
            .add_heading(1, "Introduction")
            .add_paragraph("First paragraph.")
            .add_paragraph("Second paragraph.")
            .build()
            .unwrap()
    };

    let id1 = build_doc().compute_id().unwrap();
    let id2 = build_doc().compute_id().unwrap();
    let id3 = build_doc().compute_id().unwrap();

    assert_eq!(id1, id2, "Identical documents must produce identical IDs");
    assert_eq!(id2, id3, "Hash must be deterministic across invocations");
}

/// Per spec §06 §7.1 — Draft documents may have `pending` ID.
#[test]
fn test_draft_pending_id() {
    let doc = cdx_core::Document::builder()
        .title("Draft Document")
        .creator("Author")
        .add_paragraph("Draft content")
        .build()
        .unwrap();

    assert_eq!(doc.state(), cdx_core::DocumentState::Draft);
    assert!(
        doc.id().is_pending(),
        "Draft documents should have a pending ID"
    );
}

// ============================================================================
// Block type wire-format tests (Phase 1B)
// Per spec §03-content-blocks.md
// ============================================================================

/// Verify all core block types serialize with the correct `type` string.
#[test]
fn test_core_block_type_strings() {
    let cases: Vec<(Block, &str)> = vec![
        (Block::paragraph(vec![Text::plain("text")]), "paragraph"),
        (Block::heading(1, vec![Text::plain("title")]), "heading"),
        (
            Block::unordered_list(vec![Block::list_item(vec![Block::paragraph(vec![
                Text::plain("item"),
            ])])]),
            "list",
        ),
        (
            Block::list_item(vec![Block::paragraph(vec![Text::plain("item")])]),
            "listItem",
        ),
        (
            Block::blockquote(vec![Block::paragraph(vec![Text::plain("quote")])]),
            "blockquote",
        ),
        (
            Block::code_block("fn main() {}", Some("rust".to_string())),
            "codeBlock",
        ),
        (Block::horizontal_rule(), "horizontalRule"),
        (Block::image("photo.png", "A photo"), "image"),
        (
            Block::table(vec![Block::table_row(
                vec![Block::table_cell(vec![Text::plain("cell")])],
                false,
            )]),
            "table",
        ),
        (
            Block::table_row(vec![Block::table_cell(vec![Text::plain("cell")])], false),
            "tableRow",
        ),
        (Block::table_cell(vec![Text::plain("cell")]), "tableCell"),
        (Block::math("E=mc^2", MathFormat::Latex, true), "math"),
        (Block::line_break(), "break"),
    ];

    for (block, expected_type) in cases {
        let json = serde_json::to_string(&block).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(
            val["type"],
            expected_type,
            "Block {:?} should serialize with type \"{expected_type}\", got {:?}",
            block.block_type(),
            val["type"]
        );
    }
}

/// Verify definition list block types serialize with the correct `type` string.
#[test]
fn test_definition_block_type_strings() {
    use cdx_core::content::DefinitionListBlock;

    let term = Block::DefinitionTerm {
        id: None,
        children: vec![Text::plain("Term")],
        attributes: Default::default(),
    };
    let description = Block::DefinitionDescription {
        id: None,
        children: vec![Block::paragraph(vec![Text::plain("Description")])],
        attributes: Default::default(),
    };
    let item = Block::DefinitionItem {
        id: None,
        children: vec![term.clone(), description.clone()],
        attributes: Default::default(),
    };
    let list = Block::DefinitionList(DefinitionListBlock {
        id: None,
        children: vec![item.clone()],
        attributes: Default::default(),
    });

    let cases: Vec<(&Block, &str)> = vec![
        (&list, "definitionList"),
        (&item, "definitionItem"),
        (&term, "definitionTerm"),
        (&description, "definitionDescription"),
    ];

    for (block, expected_type) in cases {
        let json = serde_json::to_string(block).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(
            val["type"], expected_type,
            "{expected_type} block has wrong type string: {:?}",
            val["type"]
        );
    }
}

/// Verify special block types serialize with the correct `type` string.
#[test]
fn test_special_block_type_strings() {
    use cdx_core::content::{
        AdmonitionVariant, BarcodeBlock, BarcodeFormat, FigureBlock, MeasurementBlock,
        SignatureBlock, SvgBlock,
    };

    let measurement =
        Block::Measurement(MeasurementBlock::new(9.81, "9.81 m/s²").with_unit("m/s²"));

    let signature = Block::Signature(
        SignatureBlock::new(cdx_core::content::BlockSignatureType::Handwritten)
            .with_signer(cdx_core::content::SignerDetails::new("John Doe"))
            .with_purpose(cdx_core::content::SignaturePurpose::Approval),
    );

    let svg = Block::Svg(SvgBlock::from_content("<svg></svg>"));

    let barcode = Block::Barcode(BarcodeBlock::new(
        BarcodeFormat::Qr,
        "https://example.com",
        "QR code link",
    ));

    let admonition = Block::admonition(
        AdmonitionVariant::Note,
        vec![Block::paragraph(vec![Text::plain("Note text")])],
    );

    let figure = Block::Figure(FigureBlock::new(vec![Block::image("img.png", "An image")]));

    let figcaption = Block::figcaption(vec![Text::plain("Caption")]);

    let cases: Vec<(&Block, &str)> = vec![
        (&measurement, "measurement"),
        (&signature, "signature"),
        (&svg, "svg"),
        (&barcode, "barcode"),
        (&admonition, "admonition"),
        (&figure, "figure"),
        (&figcaption, "figcaption"),
    ];

    for (block, expected_type) in cases {
        let json = serde_json::to_string(block).unwrap();
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(
            val["type"], expected_type,
            "{expected_type} block has wrong type string: {:?}",
            val["type"]
        );
    }
}

/// Verify all block types round-trip through serialize → deserialize with correct type.
#[test]
fn test_block_type_round_trips() {
    use cdx_core::content::{
        AdmonitionVariant, BarcodeBlock, BarcodeFormat, DefinitionListBlock, FigureBlock,
        MeasurementBlock, SignatureBlock, SignerDetails, SvgBlock,
    };

    let blocks: Vec<Block> = vec![
        Block::paragraph(vec![Text::plain("text")]),
        Block::heading(2, vec![Text::plain("heading")]),
        Block::unordered_list(vec![Block::list_item(vec![Block::paragraph(vec![
            Text::plain("item"),
        ])])]),
        Block::list_item(vec![Block::paragraph(vec![Text::plain("item")])]),
        Block::blockquote(vec![Block::paragraph(vec![Text::plain("quote")])]),
        Block::code_block("code", None),
        Block::horizontal_rule(),
        Block::image("img.png", "alt"),
        Block::table(vec![Block::table_row(
            vec![Block::table_cell(vec![Text::plain("cell")])],
            false,
        )]),
        Block::table_row(vec![Block::table_cell(vec![Text::plain("cell")])], false),
        Block::table_cell(vec![Text::plain("cell")]),
        Block::math("x^2", MathFormat::Latex, true),
        Block::line_break(),
        Block::DefinitionList(DefinitionListBlock {
            id: None,
            children: vec![Block::DefinitionItem {
                id: None,
                children: vec![
                    Block::DefinitionTerm {
                        id: None,
                        children: vec![Text::plain("Term")],
                        attributes: Default::default(),
                    },
                    Block::DefinitionDescription {
                        id: None,
                        children: vec![Block::paragraph(vec![Text::plain("Desc")])],
                        attributes: Default::default(),
                    },
                ],
                attributes: Default::default(),
            }],
            attributes: Default::default(),
        }),
        Block::Measurement(MeasurementBlock::new(1.0, "1.0 kg").with_unit("kg")),
        Block::Signature(
            SignatureBlock::new(cdx_core::content::BlockSignatureType::Electronic)
                .with_signer(SignerDetails::new("Signer"))
                .with_purpose(cdx_core::content::SignaturePurpose::Approval),
        ),
        Block::Svg(SvgBlock::from_content("<svg></svg>")),
        Block::Barcode(BarcodeBlock::new(BarcodeFormat::Qr, "data", "alt")),
        Block::Figure(FigureBlock::new(vec![Block::image("img.png", "alt")])),
        Block::figcaption(vec![Text::plain("Caption")]),
        Block::admonition(
            AdmonitionVariant::Warning,
            vec![Block::paragraph(vec![Text::plain("Warn")])],
        ),
        Block::extension("test", "widget"),
    ];

    for block in blocks {
        let original_type = block.block_type().to_string();
        let json = serde_json::to_string(&block).unwrap();
        let deserialized: Block = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.block_type().as_ref(),
            original_type,
            "Round-trip failed for block type \"{original_type}\""
        );
    }
}
