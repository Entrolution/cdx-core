//! Spec compliance tests.
//!
//! These tests validate cdx-core against the external Codex file format
//! specification (pinned as a git submodule at `spec/`). Three modules:
//!
//! - **`mark_schema_validation`**: Serialized mark output validated against spec JSON schemas
//! - **`backward_compatibility`**: Legacy format deserialization and migration helpers
//! - **`example_deserialization`**: Spec example documents deserialized and roundtripped

use std::path::{Path, PathBuf};

use cdx_core::content::{ExtensionMark, Mark, Text};
use serde_json::Value;

// ============================================================================
// Helpers
// ============================================================================

/// Path to the spec submodule root. Panics with a helpful message if missing.
fn spec_dir() -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../spec");
    assert!(
        dir.join("schemas").exists(),
        "Spec submodule not found. Run: git submodule update --init"
    );
    dir
}

/// Load a `$defs` entry from a spec schema file and return it as a standalone schema.
fn load_schema_def(schema_file: &str, def_name: &str) -> Value {
    let path = spec_dir().join("schemas").join(schema_file);
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));
    let schema: Value = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {e}", path.display()));
    schema["$defs"][def_name].clone()
}

/// Serialize an `ExtensionMark` to its JSON representation via a `Text` node,
/// then strip the namespace prefix from the `type` field so it matches the
/// spec's mark-level schema (which uses e.g. `"citation"` not `"semantic:citation"`).
fn mark_to_json_for_schema(mark: &ExtensionMark) -> Value {
    let text = Text::with_marks("test", vec![Mark::Extension(mark.clone())]);
    let json = serde_json::to_value(&text).unwrap();
    let mut mark_json = json["marks"][0].clone();

    // Strip namespace prefix: "semantic:citation" → "citation"
    if let Some(type_val) = mark_json.get("type").and_then(Value::as_str) {
        if let Some((_ns, mt)) = type_val.split_once(':') {
            mark_json["type"] = Value::String(mt.to_string());
        }
    }

    mark_json
}

/// Validate a JSON value against a schema, returning a list of error messages.
fn validate_against_schema(instance: &Value, schema: &Value) -> Vec<String> {
    let validator = jsonschema::validator_for(schema).expect("Failed to compile JSON schema");
    validator
        .iter_errors(instance)
        .map(|e| format!("{e} at {}", e.instance_path))
        .collect()
}

// ============================================================================
// Module: mark_schema_validation
// ============================================================================

mod mark_schema_validation {
    use super::*;

    fn assert_valid_against_spec(mark: &ExtensionMark, schema_file: &str, def_name: &str) {
        let schema = load_schema_def(schema_file, def_name);
        assert!(
            schema.is_object(),
            "Schema definition '{def_name}' not found in {schema_file}"
        );
        let mark_json = mark_to_json_for_schema(mark);
        let errors = validate_against_schema(&mark_json, &schema);
        assert!(
            errors.is_empty(),
            "Mark failed schema validation against {schema_file}#/$defs/{def_name}:\n  \
             Mark JSON: {mark_json}\n  Errors:\n  {}",
            errors.join("\n  ")
        );
    }

    // ----- Semantic marks -----

    #[test]
    fn citation_validates_against_spec() {
        let mark = ExtensionMark::citation("smith2023");
        assert_valid_against_spec(&mark, "semantic.schema.json", "citationMark");
    }

    #[test]
    fn citation_with_page_validates_against_spec() {
        // Note: cdx-core emits "locatorType" which is not in the spec schema,
        // but the schema allows additional properties so this still validates.
        let mark = ExtensionMark::citation_with_page("smith2023", "42-45");
        assert_valid_against_spec(&mark, "semantic.schema.json", "citationMark");
    }

    #[test]
    fn multi_citation_validates_against_spec() {
        let refs = vec!["smith2023".to_string(), "jones2024".to_string()];
        let mark = ExtensionMark::multi_citation(&refs);
        assert_valid_against_spec(&mark, "semantic.schema.json", "citationMark");
    }

    #[test]
    fn glossary_validates_against_spec() {
        let mark = ExtensionMark::glossary("ai");
        assert_valid_against_spec(&mark, "semantic.schema.json", "glossaryMark");
    }

    #[test]
    fn entity_validates_against_spec() {
        // Schema entityType enum uses PascalCase: "Person", "Organization", etc.
        let mark = ExtensionMark::entity("https://www.wikidata.org/wiki/Q7251", "Person");
        assert_valid_against_spec(&mark, "semantic.schema.json", "entityMark");
    }

    // ----- Academic marks -----

    #[test]
    fn equation_ref_validates_against_spec() {
        let mark = ExtensionMark::equation_ref("#eq-pythagoras");
        assert_valid_against_spec(&mark, "academic.schema.json", "equationRefMark");
    }

    #[test]
    fn algorithm_ref_validates_against_spec() {
        let mark = ExtensionMark::algorithm_ref("#alg-quicksort");
        assert_valid_against_spec(&mark, "academic.schema.json", "algorithmRefMark");
    }

    #[test]
    fn theorem_ref_validates_against_spec() {
        let mark = ExtensionMark::theorem_ref("#thm-pythagoras");
        assert_valid_against_spec(&mark, "academic.schema.json", "theoremRefMark");
    }

    // ----- Presentation marks -----

    #[test]
    fn index_validates_against_spec() {
        let mark = ExtensionMark::index("machine learning");
        assert_valid_against_spec(&mark, "presentation.schema.json", "indexMark");
    }

    #[test]
    fn index_with_subterm_validates_against_spec() {
        let mark = ExtensionMark::index_with_subterm("algorithms", "quicksort");
        assert_valid_against_spec(&mark, "presentation.schema.json", "indexMark");
    }
}

// ============================================================================
// Module: backward_compatibility
// ============================================================================

mod backward_compatibility {
    use super::*;

    // ----- Citation backward compat -----

    #[test]
    fn citation_deserializes_from_new_refs_format() {
        let json_str =
            r#"{"value":"cited text","marks":[{"type":"semantic:citation","refs":["smith2023"]}]}"#;
        let text: Text = serde_json::from_str(json_str).unwrap();
        let mark = &text.marks[0];
        if let Mark::Extension(ext) = mark {
            assert_eq!(
                ext.get_string_array_attribute("refs"),
                Some(vec!["smith2023"])
            );
        } else {
            panic!("Expected extension mark");
        }
    }

    #[test]
    fn citation_deserializes_from_legacy_ref_format() {
        let json_str =
            r#"{"value":"cited text","marks":[{"type":"semantic:citation","ref":"smith2023"}]}"#;
        let text: Text = serde_json::from_str(json_str).unwrap();
        let mark = &text.marks[0];
        if let Mark::Extension(ext) = mark {
            let refs = ext.get_citation_refs().expect("should have citation refs");
            assert_eq!(refs, vec!["smith2023"]);
        } else {
            panic!("Expected extension mark");
        }
    }

    #[test]
    fn citation_multi_refs_roundtrip() {
        let refs = vec!["smith2023".into(), "jones2024".into()];
        let mark = Mark::Extension(ExtensionMark::multi_citation(&refs));

        let json = serde_json::to_string(&mark).unwrap();
        let val: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(val["refs"], serde_json::json!(["smith2023", "jones2024"]));

        let parsed: Mark = serde_json::from_str(&json).unwrap();
        if let Mark::Extension(ext) = &parsed {
            assert_eq!(
                ext.get_string_array_attribute("refs"),
                Some(vec!["smith2023", "jones2024"])
            );
        } else {
            panic!("Expected Extension mark");
        }
    }

    #[test]
    fn citation_normalize_attrs_rewrites_ref_to_refs() {
        let mut ext = ExtensionMark::new("semantic", "citation")
            .with_attributes(serde_json::json!({"ref": "smith2023"}));
        ext.normalize_citation_attrs();
        assert_eq!(
            ext.get_string_array_attribute("refs"),
            Some(vec!["smith2023"])
        );
        assert!(ext.get_string_attribute("ref").is_none());
    }

    #[test]
    fn citation_struct_emits_refs_not_ref() {
        use cdx_core::extensions::Citation;

        let cite = Citation::new("smith2023");
        let json = serde_json::to_value(&cite).unwrap();
        assert!(json.get("refs").is_some(), "Citation must emit 'refs'");
        assert!(json.get("ref").is_none(), "Citation must not emit 'ref'");
        assert!(json["refs"].is_array());
        assert_eq!(json["refs"][0], "smith2023");
    }

    #[test]
    fn citation_struct_accepts_both_ref_and_refs() {
        use cdx_core::extensions::Citation;

        let new_json = r#"{"refs":["smith2023","jones2024"]}"#;
        let cite: Citation = serde_json::from_str(new_json).unwrap();
        assert_eq!(cite.refs, vec!["smith2023", "jones2024"]);

        let old_json = r#"{"ref":"smith2023"}"#;
        let cite: Citation = serde_json::from_str(old_json).unwrap();
        assert_eq!(cite.refs, vec!["smith2023"]);
    }

    #[test]
    fn citation_struct_multi_ref_roundtrip() {
        use cdx_core::extensions::Citation;

        let cite = Citation::multi(vec!["a".into(), "b".into(), "c".into()]).with_page("10");
        let json = serde_json::to_string(&cite).unwrap();
        let parsed: Citation = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.refs, vec!["a", "b", "c"]);
        assert_eq!(parsed.locator, Some("10".to_string()));
    }

    // ----- Glossary backward compat -----

    #[test]
    fn glossary_deserializes_from_new_ref_format() {
        let json_str = r#"{"value":"term","marks":[{"type":"semantic:glossary","ref":"ai"}]}"#;
        let text: Text = serde_json::from_str(json_str).unwrap();
        if let Mark::Extension(ext) = &text.marks[0] {
            assert_eq!(ext.get_glossary_ref(), Some("ai"));
        } else {
            panic!("Expected extension mark");
        }
    }

    #[test]
    fn glossary_deserializes_from_legacy_term_id_format() {
        let json_str = r#"{"value":"term","marks":[{"type":"semantic:glossary","termId":"ai"}]}"#;
        let text: Text = serde_json::from_str(json_str).unwrap();
        if let Mark::Extension(ext) = &text.marks[0] {
            assert_eq!(ext.get_glossary_ref(), Some("ai"));
        } else {
            panic!("Expected extension mark");
        }
    }

    #[test]
    fn glossary_normalize_attrs_rewrites_term_id_to_ref() {
        let mut ext = ExtensionMark::new("semantic", "glossary")
            .with_attributes(serde_json::json!({"termId": "ai"}));
        ext.normalize_glossary_attrs();
        assert_eq!(ext.get_string_attribute("ref"), Some("ai"));
        assert!(ext.get_string_attribute("termId").is_none());
    }

    #[test]
    fn glossary_ref_struct_emits_ref_not_term_id() {
        use cdx_core::extensions::GlossaryRef;

        let gref = GlossaryRef::new("ai");
        let json = serde_json::to_value(&gref).unwrap();
        assert!(json.get("ref").is_some(), "GlossaryRef must emit 'ref'");
        assert!(
            json.get("termId").is_none(),
            "GlossaryRef must not emit 'termId'"
        );
        assert_eq!(json["ref"], "ai");
    }

    #[test]
    fn glossary_ref_struct_accepts_both_term_id_and_ref() {
        use cdx_core::extensions::GlossaryRef;

        let new_json = r#"{"ref":"ai"}"#;
        let gref: GlossaryRef = serde_json::from_str(new_json).unwrap();
        assert_eq!(gref.term_id, "ai");

        let old_json = r#"{"termId":"ai"}"#;
        let gref: GlossaryRef = serde_json::from_str(old_json).unwrap();
        assert_eq!(gref.term_id, "ai");
    }

    // ----- Extension mark format backward compat -----

    #[test]
    fn extension_mark_deserializes_old_wrapper_format() {
        let json = r#"{"type":"extension","namespace":"semantic","markType":"citation","attributes":{"ref":"smith2023"}}"#;
        let mark: Mark = serde_json::from_str(json).unwrap();
        if let Mark::Extension(ext) = &mark {
            assert_eq!(ext.namespace, "semantic");
            assert_eq!(ext.mark_type, "citation");
            assert_eq!(ext.get_citation_refs(), Some(vec!["smith2023"]));
        } else {
            panic!("Expected Extension mark, got {mark:?}");
        }
    }

    #[test]
    fn extension_mark_deserializes_non_namespaced_type() {
        // Spec examples use non-namespaced types for marks (e.g., "citation" not "semantic:citation")
        let json = r#"{"type":"citation","refs":["smith2023"]}"#;
        let mark: Mark = serde_json::from_str(json).unwrap();
        if let Mark::Extension(ext) = &mark {
            assert_eq!(ext.namespace, "semantic");
            assert_eq!(ext.mark_type, "citation");
            assert_eq!(
                ext.get_string_array_attribute("refs"),
                Some(vec!["smith2023"])
            );
        } else {
            panic!("Expected Extension mark, got {mark:?}");
        }
    }

    #[test]
    fn glossary_mark_deserializes_non_namespaced_type() {
        let json = r#"{"type":"glossary","ref":"turing-machine"}"#;
        let mark: Mark = serde_json::from_str(json).unwrap();
        if let Mark::Extension(ext) = &mark {
            assert_eq!(ext.namespace, "semantic");
            assert_eq!(ext.mark_type, "glossary");
            assert_eq!(ext.get_glossary_ref(), Some("turing-machine"));
        } else {
            panic!("Expected Extension mark, got {mark:?}");
        }
    }
}

// ============================================================================
// Module: example_deserialization
// ============================================================================

mod example_deserialization {
    use super::*;
    use cdx_core::content::Content;

    /// Deserialize a spec example content document and verify it roundtrips.
    fn assert_example_roundtrips(example_name: &str) {
        let path = spec_dir()
            .join("examples")
            .join(example_name)
            .join("content/document.json");
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));

        // Step 1: Deserialize spec JSON into cdx-core Content
        let content: Content = serde_json::from_str(&raw)
            .unwrap_or_else(|e| panic!("{example_name}: failed to deserialize: {e}"));
        assert!(
            !content.is_empty(),
            "{example_name}: deserialized content should not be empty"
        );

        // Step 2: Serialize back to JSON and re-deserialize
        let serialized = serde_json::to_string(&content)
            .unwrap_or_else(|e| panic!("{example_name}: failed to serialize: {e}"));
        let roundtripped: Content = serde_json::from_str(&serialized)
            .unwrap_or_else(|e| panic!("{example_name}: failed to re-deserialize: {e}"));

        // Step 3: Compare the two Content values (structural equality)
        assert_eq!(
            content, roundtripped,
            "{example_name}: roundtrip produced different Content"
        );
    }

    #[test]
    fn simple_document() {
        assert_example_roundtrips("simple-document");
    }

    #[test]
    fn semantic_document() {
        assert_example_roundtrips("semantic-document");
    }

    #[test]
    fn academic_document() {
        assert_example_roundtrips("academic-document");
    }

    #[test]
    fn collaboration_document() {
        assert_example_roundtrips("collaboration-document");
    }

    #[test]
    #[ignore = "spec uses block-level content in tableCell; cdx-core expects inline text"]
    fn presentation_document() {
        assert_example_roundtrips("presentation-document");
    }

    #[test]
    fn forms_document() {
        assert_example_roundtrips("forms-document");
    }

    #[test]
    fn phantoms_document() {
        assert_example_roundtrips("phantoms-document");
    }

    #[test]
    fn legal_document() {
        assert_example_roundtrips("legal-document");
    }

    #[test]
    fn signed_document() {
        assert_example_roundtrips("signed-document");
    }

    #[test]
    #[ignore = "spec uses block-level content in tableCell; cdx-core expects inline text"]
    fn comprehensive_document() {
        assert_example_roundtrips("comprehensive-document");
    }
}
