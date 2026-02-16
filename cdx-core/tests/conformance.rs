//! Spec conformance tests.
//!
//! These tests verify that cdx-core's JSON wire format matches the Codex file
//! format specification. Each test compares serialization output against spec
//! examples and verifies backward-compatible deserialization of old formats.

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
