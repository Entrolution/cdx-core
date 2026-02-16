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

// NOTE: Extension mark serialization, deserialization, and backward-compat
// tests for citations and glossary have been consolidated into
// tests/spec_compliance.rs (mark_schema_validation + backward_compatibility).

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

// NOTE: spec_example_text_with_citation_mark moved to spec_compliance.rs

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

// ============================================================================
// Manifest conformance tests (Phase 1C)
// Per spec §02-manifest.md
// ============================================================================

/// Per spec §02 §3.2 — Document ID matches `algorithm:hexdigest` or `pending`.
#[test]
fn test_manifest_id_valid_hash_pattern() {
    let doc = cdx_core::Document::builder()
        .title("Test")
        .creator("Author")
        .add_paragraph("Content")
        .build()
        .unwrap();

    // Draft starts as pending
    let id_str = doc.id().to_string();
    assert_eq!(id_str, "pending", "Draft ID should be 'pending'");

    // After submitting for review, ID should match algorithm:hexdigest
    let mut doc = doc;
    doc.submit_for_review().unwrap();
    let id_str = doc.id().to_string();
    assert!(
        id_str.contains(':'),
        "Computed ID must use 'algorithm:hexdigest' format, got: {id_str}"
    );
    let parts: Vec<&str> = id_str.splitn(2, ':').collect();
    assert_eq!(parts.len(), 2);
    let algorithm = parts[0];
    let hexdigest = parts[1];
    assert!(
        ["sha256", "sha384", "sha512", "sha3-256", "sha3-512", "blake3"].contains(&algorithm),
        "Unknown algorithm: {algorithm}"
    );
    assert!(
        hexdigest.chars().all(|c| c.is_ascii_hexdigit()),
        "Digest must be hex, got: {hexdigest}"
    );
}

/// Per spec §02 §3.2 — Timestamps are valid ISO 8601.
#[test]
fn test_manifest_timestamps_iso8601() {
    let doc = cdx_core::Document::builder()
        .title("Test")
        .creator("Author")
        .add_paragraph("Content")
        .build()
        .unwrap();

    let manifest = doc.manifest();
    let json = serde_json::to_value(manifest).unwrap();

    let created = json["created"].as_str().unwrap();
    let modified = json["modified"].as_str().unwrap();

    // Parse back as ISO 8601 timestamps
    assert!(
        chrono::DateTime::parse_from_rfc3339(created).is_ok(),
        "Created timestamp is not valid ISO 8601: {created}"
    );
    assert!(
        chrono::DateTime::parse_from_rfc3339(modified).is_ok(),
        "Modified timestamp is not valid ISO 8601: {modified}"
    );
}

/// Per spec §02 §4.2 — Draft ID can be `pending`.
#[test]
fn test_manifest_id_pending_for_draft() {
    let doc = cdx_core::Document::builder()
        .title("Test")
        .creator("Author")
        .add_paragraph("Content")
        .build()
        .unwrap();

    assert_eq!(doc.state(), cdx_core::DocumentState::Draft);
    assert!(doc.id().is_pending());

    // Verify it serializes as "pending"
    let json = serde_json::to_value(doc.manifest()).unwrap();
    assert_eq!(json["id"], "pending");
}

/// Per spec §02 §4.10 — Extension ID follows `namespace.name` format.
#[test]
fn test_extension_id_format() {
    let ext = cdx_core::Extension::required("codex.semantic", "0.1");
    assert!(
        ext.id.contains('.'),
        "Extension ID should use dot notation: {}",
        ext.id
    );
    assert_eq!(ext.namespace(), "semantic");

    let ext2 = cdx_core::Extension::optional("org.example.custom", "1.0");
    assert_eq!(ext2.namespace(), "custom");
}

/// Per spec §02 §4.10 — Extension has version field.
#[test]
fn test_extension_version_present() {
    let ext = cdx_core::Extension::new("codex.semantic", "0.1", true);
    assert!(!ext.version.is_empty(), "Extension must have a version");

    // Verify it serializes with the version field
    let json = serde_json::to_value(&ext).unwrap();
    assert!(json["version"].is_string());
    assert_eq!(json["version"], "0.1");
}

/// Per spec §02 §5.3 — Frozen/published state requires signatures in manifest.
#[test]
fn test_frozen_requires_signatures_in_manifest() {
    use cdx_core::{ContentRef, DocumentId, DocumentState, Manifest, Metadata, SecurityRef};

    let test_hash: DocumentId =
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
            .parse()
            .unwrap();

    let content = ContentRef {
        path: "content/document.json".to_string(),
        hash: test_hash.clone(),
        compression: None,
        merkle_root: None,
        block_count: None,
    };
    let metadata = Metadata {
        dublin_core: "metadata/dublin-core.json".to_string(),
        custom: None,
    };

    let mut manifest = Manifest::new(content, metadata);
    manifest.id = test_hash.clone();
    manifest.state = DocumentState::Frozen;

    // Without security, validation should fail
    let result = manifest.validate();
    assert!(
        result.is_err(),
        "Frozen manifest without security must fail"
    );

    // With security, add precise layout too
    manifest.security = Some(SecurityRef {
        signatures: Some("security/signatures.json".to_string()),
        encryption: None,
    });
    manifest.presentation.push(cdx_core::PresentationRef {
        presentation_type: "precise".to_string(),
        path: "presentation/layouts/letter.json".to_string(),
        hash: test_hash,
        default: false,
    });

    assert!(
        manifest.validate().is_ok(),
        "Frozen manifest with security and precise layout should pass"
    );
}

// ============================================================================
// State machine enforcement tests (Phase 1D)
// Per spec §07-state-machine.md
// ============================================================================

/// Per spec §07 §3.3 — Review state requires non-pending document ID.
#[test]
fn test_review_state_requires_computed_id() {
    let mut doc = cdx_core::Document::builder()
        .title("Test")
        .creator("Author")
        .add_paragraph("Content")
        .build()
        .unwrap();

    // Before review, ID is pending
    assert!(doc.id().is_pending());

    // submit_for_review should compute the ID
    doc.submit_for_review().unwrap();
    assert_eq!(doc.state(), cdx_core::DocumentState::Review);
    assert!(
        !doc.id().is_pending(),
        "Review state must have a computed ID"
    );

    // Verify manifest validation agrees
    assert!(doc.manifest().validate().is_ok());
}

/// Per spec §07 §3.4 — Frozen requires at least one signature.
#[test]
fn test_frozen_requires_signature() {
    let mut doc = cdx_core::Document::builder()
        .title("Test")
        .creator("Author")
        .add_paragraph("Content")
        .build()
        .unwrap();

    doc.submit_for_review().unwrap();

    // Set lineage (required for freeze)
    doc.set_lineage(None, 1, Some("Initial version".to_string()))
        .unwrap();

    // Attempt to freeze without signatures should fail
    let result = doc.freeze();
    assert!(result.is_err(), "Freezing without signatures must fail");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("signature"),
        "Error should mention signatures: {err}"
    );
}

/// Per spec §07 §3.5 — Published requires signature (inherited from frozen).
#[test]
fn test_published_requires_signature() {
    // A document must be frozen to be published, and frozen requires signatures.
    // Test that publishing from a non-frozen state fails.
    let mut doc = cdx_core::Document::builder()
        .title("Test")
        .creator("Author")
        .add_paragraph("Content")
        .build()
        .unwrap();

    // Draft → Published is not valid
    let result = doc.publish();
    assert!(result.is_err(), "Publishing from draft must fail");

    // Review → Published is not valid either
    doc.submit_for_review().unwrap();
    let result = doc.publish();
    assert!(result.is_err(), "Publishing from review must fail");
}

// ============================================================================
// Provenance & lineage tests (Phase 1F)
// Per spec §09-provenance-and-lineage.md
// ============================================================================

/// Per spec §09 §3.1 — Parent hash uses `algorithm:hexdigest` format.
#[test]
fn test_lineage_parent_hash_format() {
    let parent_id: cdx_core::DocumentId =
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
            .parse()
            .unwrap();
    let lineage = cdx_core::Lineage::from_parent(parent_id.clone(), None);

    let parent = lineage.parent.unwrap();
    let parent_str = parent.to_string();
    assert!(
        parent_str.contains(':'),
        "Parent hash must use algorithm:hexdigest format: {parent_str}"
    );
    assert!(!parent.is_pending());
}

/// Per spec §09 §3.2 — Ancestors ordered nearest-first.
#[test]
fn test_lineage_ancestors_ordered() {
    let root_id: cdx_core::DocumentId =
        "sha256:0000000000000000000000000000000000000000000000000000000000000001"
            .parse()
            .unwrap();
    let root_lineage = cdx_core::Lineage::root();

    let v2_id: cdx_core::DocumentId =
        "sha256:0000000000000000000000000000000000000000000000000000000000000002"
            .parse()
            .unwrap();
    let v2_lineage = cdx_core::Lineage::from_parent(root_id.clone(), Some(&root_lineage));

    let _v3_id: cdx_core::DocumentId =
        "sha256:0000000000000000000000000000000000000000000000000000000000000003"
            .parse()
            .unwrap();
    let v3_lineage = cdx_core::Lineage::from_parent(v2_id.clone(), Some(&v2_lineage));

    // v3's parent is v2
    assert_eq!(v3_lineage.parent, Some(v2_id));
    // v3's ancestors should be [root_id] (nearest first = parent's parent)
    assert_eq!(v3_lineage.ancestors.len(), 1);
    assert_eq!(
        v3_lineage.ancestors[0], root_id,
        "Ancestors must be ordered nearest-first (grandparent first in chain)"
    );
}

/// Per spec §09 §3.1 — Version >= 1.
#[test]
fn test_lineage_version_positive() {
    let root = cdx_core::Lineage::root();
    assert!(root.version.unwrap() >= 1, "Root version must be >= 1");

    let parent_id: cdx_core::DocumentId =
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
            .parse()
            .unwrap();
    let child = cdx_core::Lineage::from_parent(parent_id, Some(&root));
    assert!(child.version.unwrap() >= 1, "Child version must be >= 1");
    assert!(
        child.version.unwrap() > root.version.unwrap(),
        "Child version must be greater than parent version"
    );
}

/// Per spec §09 §3.1 — Depth reflects position in chain.
#[test]
fn test_lineage_depth_matches_ancestors() {
    let root = cdx_core::Lineage::root();
    assert_eq!(root.depth, Some(0), "Root depth must be 0");
    assert!(root.ancestors.is_empty(), "Root must have no ancestors");

    let parent_id: cdx_core::DocumentId =
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
            .parse()
            .unwrap();
    let child = cdx_core::Lineage::from_parent(parent_id.clone(), Some(&root));
    assert_eq!(child.depth, Some(1));

    let grandchild = cdx_core::Lineage::from_parent(
        "sha256:1111111111111111111111111111111111111111111111111111111111111111"
            .parse()
            .unwrap(),
        Some(&child),
    );
    assert_eq!(grandchild.depth, Some(2));
    // Grandchild should have parent_id in ancestors
    assert_eq!(grandchild.ancestors.len(), 1);
}

/// Per spec §09 §4.4 — Merkle root can be stored in manifest content ref.
#[test]
fn test_merkle_root_in_content_ref() {
    let doc = cdx_core::Document::builder()
        .title("Test")
        .creator("Author")
        .add_heading(1, "Intro")
        .add_paragraph("Body text")
        .build()
        .unwrap();

    let merkle_root = doc.merkle_root().unwrap();
    assert!(
        !merkle_root.is_pending(),
        "Merkle root must be a computed hash"
    );

    // Verify it matches the block index root
    let block_index = doc.block_index().unwrap();
    assert_eq!(
        merkle_root,
        *block_index.merkle_root(),
        "Merkle root must match block index root"
    );
}

/// Per spec §09 §4.5 — Block index hashes match individually computed block hashes.
#[test]
fn test_block_index_hash_consistency() {
    use cdx_core::{HashAlgorithm, Hasher};

    let doc = cdx_core::Document::builder()
        .title("Test")
        .creator("Author")
        .add_heading(1, "Title")
        .add_paragraph("First paragraph")
        .add_paragraph("Second paragraph")
        .build()
        .unwrap();

    let block_index = doc.block_index().unwrap();

    // Verify each block hash matches manual computation
    for (i, block) in doc.content().blocks.iter().enumerate() {
        let block_json = serde_json::to_vec(block).unwrap();
        let canonical = json_canon::to_string(
            &serde_json::from_slice::<serde_json::Value>(&block_json).unwrap(),
        )
        .unwrap();
        let expected_hash = Hasher::hash(HashAlgorithm::Sha256, canonical.as_bytes());

        let entry = block_index.get_block(i).unwrap();
        assert_eq!(
            entry.hash, expected_hash,
            "Block {i} hash in index must match manually computed hash"
        );
    }
}

/// Per spec §09 — Fork creates valid lineage chain.
#[test]
fn test_fork_creates_valid_lineage() {
    let doc = cdx_core::Document::builder()
        .title("Original")
        .creator("Author")
        .add_paragraph("Content")
        .build()
        .unwrap();

    let forked = doc.fork().unwrap();
    assert_eq!(forked.state(), cdx_core::DocumentState::Draft);
    assert!(forked.id().is_pending());

    let lineage = forked.manifest().lineage.as_ref().unwrap();
    assert!(lineage.parent.is_some(), "Fork must have parent");
    assert!(lineage.version.unwrap() >= 2, "Forked version must be >= 2");
    assert!(lineage.depth.unwrap() >= 1, "Forked depth must be >= 1");
}

// ============================================================================
// Metadata conformance tests (Phase 1G)
// Per spec §08-metadata.md
// ============================================================================

/// Per spec §08 §2.1 — Dublin Core `title` is required.
#[test]
fn test_dublin_core_title_required() {
    // DublinCore::new requires title - verify it's there
    let dc = cdx_core::metadata::DublinCore::new("Required Title", "Author");
    assert_eq!(dc.title(), "Required Title");
    assert!(
        !dc.title().is_empty(),
        "Dublin Core title must not be empty"
    );

    // Verify it serializes with title
    let json = serde_json::to_value(&dc).unwrap();
    assert!(json["terms"]["title"].is_string());
}

/// Per spec §08 §2.1 — Dublin Core `creator` is required.
#[test]
fn test_dublin_core_creator_required() {
    let dc = cdx_core::metadata::DublinCore::new("Title", "Required Creator");
    assert_eq!(dc.creators(), vec!["Required Creator"]);

    // Verify it serializes with creator
    let json = serde_json::to_value(&dc).unwrap();
    assert!(
        json["terms"]["creator"].is_string() || json["terms"]["creator"].is_array(),
        "Creator must be present in serialized form"
    );
}

// ============================================================================
// Security conformance tests (Phase 1G continued)
// ============================================================================

/// Per spec §security — Signature must have signer name.
#[cfg(feature = "signatures")]
#[test]
fn test_signature_requires_signer_name() {
    use cdx_core::security::{Signature, SignatureAlgorithm, SignerInfo};

    let signer = SignerInfo {
        name: "Alice Smith".to_string(),
        email: None,
        organization: None,
        certificate: None,
        key_id: None,
    };

    let sig = Signature {
        id: "sig-1".to_string(),
        algorithm: SignatureAlgorithm::ES256,
        signed_at: chrono::Utc::now(),
        signer: signer.clone(),
        value: "base64data".to_string(),
        certificate_chain: None,
        scope: None,
        timestamp: None,
        webauthn: None,
    };

    assert!(
        !sig.signer.name.is_empty(),
        "Signature must have a signer name"
    );
    assert_eq!(sig.signer.name, "Alice Smith");
}

/// Per spec §security — Signature documentId must match manifest.
#[cfg(feature = "signatures")]
#[test]
fn test_signature_document_id_matches_manifest() {
    use cdx_core::security::SignatureFile;

    let doc_id: cdx_core::DocumentId =
        "sha256:abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
            .parse()
            .unwrap();

    let sig_file = SignatureFile::new(doc_id.clone());
    assert_eq!(
        sig_file.document_id, doc_id,
        "Signature file documentId must match the document's manifest ID"
    );

    // Verify it round-trips correctly
    let json = sig_file.to_json().unwrap();
    let parsed = SignatureFile::from_json(&json).unwrap();
    assert_eq!(parsed.document_id, doc_id);
}

// ============================================================================
// Extension validation tests (Phase 1H)
// ============================================================================

/// Per spec §extensions — Required unknown extension should be detected.
#[test]
fn test_required_extension_unsupported_detection() {
    let mut doc = cdx_core::Document::builder()
        .title("Test")
        .creator("Author")
        .add_paragraph("Content")
        .build()
        .unwrap();

    // Declare a required extension in manifest
    doc.manifest_mut()
        .extensions
        .push(cdx_core::Extension::required("vendor.unknown", "1.0"));

    // The extension is declared but not used in content, so validate_extensions
    // should report it as declared (it's in the manifest) but it won't be undeclared.
    let report = doc.validate_extensions();
    let declared: Vec<_> = report
        .declared_namespaces
        .iter()
        .map(String::as_str)
        .collect();
    assert!(
        declared.iter().any(|n| *n == "unknown"),
        "Required extension should appear in declared list"
    );
}

/// Per spec §extensions — Optional unknown extension should be tolerated.
#[test]
fn test_optional_extension_unsupported_ok() {
    let mut doc = cdx_core::Document::builder()
        .title("Test")
        .creator("Author")
        .add_paragraph("Content")
        .build()
        .unwrap();

    // Declare an optional extension
    doc.manifest_mut()
        .extensions
        .push(cdx_core::Extension::optional("vendor.optional", "1.0"));

    // Optional extensions should not cause validation warnings
    let report = doc.validate_extensions();
    assert!(
        report.is_valid(),
        "Optional unused extension should not cause validation failure"
    );
}

/// Per spec §extensions — Extension used but not declared produces warning.
#[test]
fn test_undeclared_extension_produces_warning() {
    use cdx_core::content::Block;

    let mut doc = cdx_core::Document::builder()
        .title("Test")
        .creator("Author")
        .build()
        .unwrap();

    // Add content with an extension block but don't declare it
    let ext_block = Block::extension("vendor", "widget");
    let content = doc.content_mut().unwrap();
    content.blocks.push(ext_block);

    let report = doc.validate_extensions();
    assert!(
        !report.is_valid(),
        "Undeclared extension should cause validation failure"
    );
    assert!(
        report.undeclared.contains(&"vendor".to_string()),
        "Undeclared list should contain 'vendor'"
    );
    assert!(
        report.has_warnings(),
        "Should have warnings for undeclared extension"
    );
}

/// Per spec §extensions — Extension ID format and version presence in serialization.
#[test]
fn test_extension_declaration_serialization() {
    let ext = cdx_core::Extension::required("codex.semantic", "0.1");
    let json = serde_json::to_value(&ext).unwrap();

    // Must have id, version, required fields
    assert!(json["id"].is_string(), "Extension must have 'id' field");
    assert!(
        json["version"].is_string(),
        "Extension must have 'version' field"
    );
    assert!(
        json["required"].is_boolean(),
        "Extension must have 'required' field"
    );

    // ID should be dot-notation
    let id = json["id"].as_str().unwrap();
    assert!(
        id.contains('.'),
        "Extension ID should use dot notation: {id}"
    );
}
