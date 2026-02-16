//! Extension attribute schema tests.
//!
//! Table-driven tests verifying field names and types for every `ExtensionMark`
//! convenience constructor. These tests prevent field-level spec divergences by
//! asserting the exact attribute schema each constructor produces.

use cdx_core::content::{ExtensionMark, Mark, Text};
use serde_json::Value;

// ===== Helper functions =====

/// Serialize an `ExtensionMark` via a `Text` node and return the mark's JSON value.
fn mark_to_json(mark: &ExtensionMark) -> Value {
    let text = Text::with_marks("test", vec![Mark::Extension(mark.clone())]);
    let json = serde_json::to_value(&text).unwrap();
    // marks is an array; grab the first mark
    json["marks"][0].clone()
}

/// Assert that a JSON object contains a key with a string value.
fn assert_string_field(val: &Value, key: &str, context: &str) {
    let field = &val[key];
    assert!(
        field.is_string(),
        "{context}: expected '{key}' to be a string, got {field}"
    );
}

/// Assert that a JSON object contains a key with a string array value.
fn assert_string_array_field(val: &Value, key: &str, min_len: usize, context: &str) {
    let field = &val[key];
    assert!(
        field.is_array(),
        "{context}: expected '{key}' to be an array, got {field}"
    );
    let arr = field.as_array().unwrap();
    assert!(
        arr.len() >= min_len,
        "{context}: expected '{key}' array to have >= {min_len} items, got {}",
        arr.len()
    );
    for (i, item) in arr.iter().enumerate() {
        assert!(
            item.is_string(),
            "{context}: expected '{key}[{i}]' to be a string, got {item}"
        );
    }
}

/// Assert that a JSON object does NOT contain a key.
fn assert_field_absent(val: &Value, key: &str, context: &str) {
    assert!(
        val.get(key).is_none(),
        "{context}: expected '{key}' to be absent, but found {}",
        val[key]
    );
}

// ===== Citation constructors =====

#[test]
fn schema_citation_emits_refs_array() {
    let mark = ExtensionMark::citation("smith2023");
    let json = mark_to_json(&mark);

    assert_eq!(json["type"], "semantic:citation", "type field");
    assert_string_array_field(&json, "refs", 1, "citation");
    assert_eq!(json["refs"][0], "smith2023");
    assert_field_absent(&json, "ref", "citation must not emit legacy 'ref'");
}

#[test]
fn schema_citation_with_page_emits_refs_array_and_locator() {
    let mark = ExtensionMark::citation_with_page("smith2023", "42-45");
    let json = mark_to_json(&mark);

    assert_eq!(json["type"], "semantic:citation");
    assert_string_array_field(&json, "refs", 1, "citation_with_page");
    assert_string_field(&json, "locator", "citation_with_page");
    assert_string_field(&json, "locatorType", "citation_with_page");
    assert_eq!(json["locator"], "42-45");
    assert_eq!(json["locatorType"], "page");
    assert_field_absent(
        &json,
        "ref",
        "citation_with_page must not emit legacy 'ref'",
    );
}

#[test]
fn schema_multi_citation_emits_refs_array_with_multiple_items() {
    let refs = vec!["smith2023".to_string(), "jones2024".to_string()];
    let mark = ExtensionMark::multi_citation(&refs);
    let json = mark_to_json(&mark);

    assert_eq!(json["type"], "semantic:citation");
    assert_string_array_field(&json, "refs", 2, "multi_citation");
    assert_eq!(json["refs"][0], "smith2023");
    assert_eq!(json["refs"][1], "jones2024");
    assert_field_absent(&json, "ref", "multi_citation must not emit legacy 'ref'");
}

// ===== Entity link =====

#[test]
fn schema_entity_emits_uri_and_entity_type() {
    let mark = ExtensionMark::entity("https://example.org/entity/1", "person");
    let json = mark_to_json(&mark);

    assert_eq!(json["type"], "semantic:entity");
    assert_string_field(&json, "uri", "entity");
    assert_string_field(&json, "entityType", "entity");
    assert_eq!(json["uri"], "https://example.org/entity/1");
    assert_eq!(json["entityType"], "person");
}

// ===== Glossary =====

#[test]
fn schema_glossary_emits_ref() {
    let mark = ExtensionMark::glossary("ai");
    let json = mark_to_json(&mark);

    assert_eq!(json["type"], "semantic:glossary");
    assert_string_field(&json, "ref", "glossary");
    assert_eq!(json["ref"], "ai");
    assert_field_absent(&json, "termId", "glossary must not emit legacy 'termId'");
}

// ===== Index =====

#[test]
fn schema_index_emits_term() {
    let mark = ExtensionMark::index("machine learning");
    let json = mark_to_json(&mark);

    assert_eq!(json["type"], "presentation:index");
    assert_string_field(&json, "term", "index");
    assert_eq!(json["term"], "machine learning");
}

#[test]
fn schema_index_with_subterm_emits_term_and_subterm() {
    let mark = ExtensionMark::index_with_subterm("algorithms", "quicksort");
    let json = mark_to_json(&mark);

    assert_eq!(json["type"], "presentation:index");
    assert_string_field(&json, "term", "index_with_subterm");
    assert_string_field(&json, "subterm", "index_with_subterm");
    assert_eq!(json["term"], "algorithms");
    assert_eq!(json["subterm"], "quicksort");
}

// ===== Academic: equation references =====

#[test]
fn schema_equation_ref_emits_target() {
    let mark = ExtensionMark::equation_ref("#eq-pythagoras");
    let json = mark_to_json(&mark);

    assert_eq!(json["type"], "academic:equation-ref");
    assert_string_field(&json, "target", "equation_ref");
    assert_eq!(json["target"], "#eq-pythagoras");
}

#[test]
fn schema_equation_ref_formatted_emits_target_and_format() {
    let mark = ExtensionMark::equation_ref_formatted("#eq-1", "Eq. ({number})");
    let json = mark_to_json(&mark);

    assert_eq!(json["type"], "academic:equation-ref");
    assert_string_field(&json, "target", "equation_ref_formatted");
    assert_string_field(&json, "format", "equation_ref_formatted");
    assert_eq!(json["target"], "#eq-1");
    assert_eq!(json["format"], "Eq. ({number})");
}

// ===== Academic: algorithm references =====

#[test]
fn schema_algorithm_ref_emits_target() {
    let mark = ExtensionMark::algorithm_ref("#alg-quicksort");
    let json = mark_to_json(&mark);

    assert_eq!(json["type"], "academic:algorithm-ref");
    assert_string_field(&json, "target", "algorithm_ref");
    assert_eq!(json["target"], "#alg-quicksort");
}

#[test]
fn schema_algorithm_ref_line_emits_target_and_line() {
    let mark = ExtensionMark::algorithm_ref_line("#alg-quicksort", "5");
    let json = mark_to_json(&mark);

    assert_eq!(json["type"], "academic:algorithm-ref");
    assert_string_field(&json, "target", "algorithm_ref_line");
    assert_string_field(&json, "line", "algorithm_ref_line");
    assert_eq!(json["target"], "#alg-quicksort");
    assert_eq!(json["line"], "5");
}

#[test]
fn schema_algorithm_ref_formatted_emits_target_and_format() {
    let mark = ExtensionMark::algorithm_ref_formatted("#alg-1", "Algorithm {number}");
    let json = mark_to_json(&mark);

    assert_eq!(json["type"], "academic:algorithm-ref");
    assert_string_field(&json, "target", "algorithm_ref_formatted");
    assert_string_field(&json, "format", "algorithm_ref_formatted");
}

#[test]
fn schema_algorithm_ref_line_formatted_emits_all_fields() {
    let mark = ExtensionMark::algorithm_ref_line_formatted("#alg-1", "5", "Alg. {number}, L{line}");
    let json = mark_to_json(&mark);

    assert_eq!(json["type"], "academic:algorithm-ref");
    assert_string_field(&json, "target", "algorithm_ref_line_formatted");
    assert_string_field(&json, "line", "algorithm_ref_line_formatted");
    assert_string_field(&json, "format", "algorithm_ref_line_formatted");
    assert_eq!(json["target"], "#alg-1");
    assert_eq!(json["line"], "5");
    assert_eq!(json["format"], "Alg. {number}, L{line}");
}

// ===== Academic: theorem references =====

#[test]
fn schema_theorem_ref_emits_target() {
    let mark = ExtensionMark::theorem_ref("#thm-pythagoras");
    let json = mark_to_json(&mark);

    assert_eq!(json["type"], "academic:theorem-ref");
    assert_string_field(&json, "target", "theorem_ref");
    assert_eq!(json["target"], "#thm-pythagoras");
}

#[test]
fn schema_theorem_ref_formatted_emits_target_and_format() {
    let mark = ExtensionMark::theorem_ref_formatted("#thm-1", "Theorem {number}");
    let json = mark_to_json(&mark);

    assert_eq!(json["type"], "academic:theorem-ref");
    assert_string_field(&json, "target", "theorem_ref_formatted");
    assert_string_field(&json, "format", "theorem_ref_formatted");
    assert_eq!(json["target"], "#thm-1");
    assert_eq!(json["format"], "Theorem {number}");
}

// ===== Collaboration: highlight =====

#[test]
fn schema_highlight_emits_color() {
    let mark = ExtensionMark::highlight("yellow");
    let json = mark_to_json(&mark);

    assert_eq!(json["type"], "collaboration:highlight");
    assert_string_field(&json, "color", "highlight");
    assert_eq!(json["color"], "yellow");
}

// ===== Citation backward-compatibility deserialization =====

#[test]
fn schema_citation_deserializes_from_new_refs_format() {
    let json_str =
        r#"{"value":"cited text","marks":[{"type":"semantic:citation","refs":["smith2023"]}]}"#;
    let text: Text = serde_json::from_str(json_str).unwrap();
    let mark = &text.marks[0];
    if let cdx_core::content::Mark::Extension(ext) = mark {
        assert_eq!(
            ext.get_string_array_attribute("refs"),
            Some(vec!["smith2023"])
        );
    } else {
        panic!("Expected extension mark");
    }
}

#[test]
fn schema_citation_deserializes_from_legacy_ref_format() {
    let json_str =
        r#"{"value":"cited text","marks":[{"type":"semantic:citation","ref":"smith2023"}]}"#;
    let text: Text = serde_json::from_str(json_str).unwrap();
    let mark = &text.marks[0];
    if let cdx_core::content::Mark::Extension(ext) = mark {
        // Legacy "ref" should be accessible via get_citation_refs()
        let refs = ext.get_citation_refs().expect("should have citation refs");
        assert_eq!(refs, vec!["smith2023"]);
    } else {
        panic!("Expected extension mark");
    }
}

// ===== Citation struct schema tests =====

#[test]
fn schema_citation_struct_emits_refs_not_ref() {
    use cdx_core::extensions::Citation;

    let cite = Citation::new("smith2023");
    let json = serde_json::to_value(&cite).unwrap();

    assert!(json.get("refs").is_some(), "Citation must emit 'refs'");
    assert!(
        json.get("ref").is_none(),
        "Citation must not emit legacy 'ref'"
    );
    assert!(json["refs"].is_array());
    assert_eq!(json["refs"][0], "smith2023");
}

#[test]
fn schema_citation_struct_accepts_both_ref_and_refs() {
    use cdx_core::extensions::Citation;

    // New format: "refs" array
    let new_json = r#"{"refs":["smith2023","jones2024"]}"#;
    let cite: Citation = serde_json::from_str(new_json).unwrap();
    assert_eq!(cite.refs, vec!["smith2023", "jones2024"]);

    // Legacy format: "ref" string
    let old_json = r#"{"ref":"smith2023"}"#;
    let cite: Citation = serde_json::from_str(old_json).unwrap();
    assert_eq!(cite.refs, vec!["smith2023"]);
}

#[test]
fn schema_citation_struct_multi_ref_roundtrip() {
    use cdx_core::extensions::Citation;

    let cite = Citation::multi(vec!["a".into(), "b".into(), "c".into()]).with_page("10");
    let json = serde_json::to_string(&cite).unwrap();
    let parsed: Citation = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.refs, vec!["a", "b", "c"]);
    assert_eq!(parsed.locator, Some("10".to_string()));
}
