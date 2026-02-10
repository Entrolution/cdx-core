//! JSON Schema validation for Codex Document Format files.
//!
//! This module provides validation functions for the core CDX file types:
//! - Manifest (`manifest.json`)
//! - Content (`content/content.json`)
//! - Dublin Core metadata (`metadata/dublin-core.json`)
//!
//! # Feature Flag
//!
//! This module requires the `validation` feature:
//!
//! ```toml
//! [dependencies]
//! cdx-core = { version = "0.1", features = ["validation"] }
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use cdx_core::validation::{validate_manifest, validate_content};
//!
//! let manifest_json = r#"{"version": "0.1", "id": "sha256:abc..."}"#;
//! let errors = validate_manifest(manifest_json)?;
//! if errors.is_empty() {
//!     println!("Manifest is valid");
//! } else {
//!     for error in errors {
//!         println!("Validation error: {}", error);
//!     }
//! }
//! ```

use std::fmt;

/// JSON schema validation error for manifest and metadata files.
///
/// Reports type mismatches, missing required properties, and invalid
/// enum values when validating manifest, content, Dublin Core metadata,
/// block index, and signature JSON files against their schemas.
///
/// See also [`crate::content::ValidationError`] for content structure
/// validation (block hierarchy, unique IDs, etc.).
#[derive(Debug, Clone)]
pub struct SchemaValidationError {
    /// JSON path to the invalid element (empty for root-level errors).
    pub path: String,
    /// Description of the validation failure.
    pub message: String,
}

impl fmt::Display for SchemaValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.path.is_empty() {
            write!(f, "{}", self.message)
        } else {
            write!(f, "{}: {}", self.path, self.message)
        }
    }
}

impl std::error::Error for SchemaValidationError {}

/// Result type for validation operations.
pub type ValidationResult = Result<Vec<SchemaValidationError>, crate::Error>;

/// Validate a manifest JSON string against the CDX manifest schema.
///
/// # Arguments
///
/// * `json` - The JSON string to validate
///
/// # Returns
///
/// A vector of validation errors. An empty vector means the JSON is valid.
///
/// # Errors
///
/// Returns an error if the JSON cannot be parsed.
pub fn validate_manifest(json: &str) -> ValidationResult {
    validate_json(json, SchemaType::Manifest)
}

/// Validate a content JSON string against the CDX content schema.
///
/// # Arguments
///
/// * `json` - The JSON string to validate
///
/// # Returns
///
/// A vector of validation errors. An empty vector means the JSON is valid.
///
/// # Errors
///
/// Returns an error if the JSON cannot be parsed.
pub fn validate_content(json: &str) -> ValidationResult {
    validate_json(json, SchemaType::Content)
}

/// Validate a Dublin Core metadata JSON string against the schema.
///
/// # Arguments
///
/// * `json` - The JSON string to validate
///
/// # Returns
///
/// A vector of validation errors. An empty vector means the JSON is valid.
///
/// # Errors
///
/// Returns an error if the JSON cannot be parsed.
pub fn validate_dublin_core(json: &str) -> ValidationResult {
    validate_json(json, SchemaType::DublinCore)
}

/// Validate a block index JSON string against the schema.
///
/// # Arguments
///
/// * `json` - The JSON string to validate
///
/// # Returns
///
/// A vector of validation errors. An empty vector means the JSON is valid.
///
/// # Errors
///
/// Returns an error if the JSON cannot be parsed.
pub fn validate_block_index(json: &str) -> ValidationResult {
    validate_json(json, SchemaType::BlockIndex)
}

/// Validate a signatures JSON string against the schema.
///
/// # Arguments
///
/// * `json` - The JSON string to validate
///
/// # Returns
///
/// A vector of validation errors. An empty vector means the JSON is valid.
///
/// # Errors
///
/// Returns an error if the JSON cannot be parsed.
pub fn validate_signatures(json: &str) -> ValidationResult {
    validate_json(json, SchemaType::Signatures)
}

/// Schema types for validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SchemaType {
    Manifest,
    Content,
    DublinCore,
    BlockIndex,
    Signatures,
}

/// Internal validation function.
fn validate_json(json: &str, schema_type: SchemaType) -> ValidationResult {
    // Parse the JSON
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| crate::Error::InvalidManifest {
            reason: format!("Invalid JSON: {e}"),
        })?;

    // Get the schema for this type
    let schema = get_schema(schema_type);

    // Validate against schema
    let mut errors = Vec::new();
    validate_value(&value, &schema, "", &mut errors);

    Ok(errors)
}

/// Get the schema definition for a given type.
fn get_schema(schema_type: SchemaType) -> Schema {
    match schema_type {
        SchemaType::Manifest => manifest_schema(),
        SchemaType::Content => content_schema(),
        SchemaType::DublinCore => dublin_core_schema(),
        SchemaType::BlockIndex => block_index_schema(),
        SchemaType::Signatures => signatures_schema(),
    }
}

/// Simple schema representation for validation.
#[derive(Debug, Clone)]
struct Schema {
    /// Required properties.
    required: Vec<&'static str>,
    /// Property schemas.
    properties: Vec<(&'static str, PropertySchema)>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Some variants are for future use
enum PropertySchema {
    String,
    Integer,
    Number,
    Boolean,
    Object,
    Array,
    Any,
    StringEnum(Vec<&'static str>),
}

/// Validate a JSON value against a schema.
fn validate_value(
    value: &serde_json::Value,
    schema: &Schema,
    path: &str,
    errors: &mut Vec<SchemaValidationError>,
) {
    let Some(obj) = value.as_object() else {
        errors.push(SchemaValidationError {
            path: path.to_string(),
            message: "expected object".to_string(),
        });
        return;
    };

    // Check required properties
    for required in &schema.required {
        if !obj.contains_key(*required) {
            errors.push(SchemaValidationError {
                path: if path.is_empty() {
                    (*required).to_string()
                } else {
                    format!("{path}.{required}")
                },
                message: format!("missing required property '{required}'"),
            });
        }
    }

    // Validate property types
    for (prop_name, prop_schema) in &schema.properties {
        if let Some(prop_value) = obj.get(*prop_name) {
            let prop_path = if path.is_empty() {
                (*prop_name).to_string()
            } else {
                format!("{path}.{prop_name}")
            };
            validate_property(prop_value, prop_schema, &prop_path, errors);
        }
    }
}

/// Validate a property value against its schema.
fn validate_property(
    value: &serde_json::Value,
    schema: &PropertySchema,
    path: &str,
    errors: &mut Vec<SchemaValidationError>,
) {
    match schema {
        PropertySchema::String => {
            if !value.is_string() {
                errors.push(SchemaValidationError {
                    path: path.to_string(),
                    message: format!("expected string, got {}", value_type_name(value)),
                });
            }
        }
        PropertySchema::Integer => {
            if !value.is_i64() && !value.is_u64() {
                errors.push(SchemaValidationError {
                    path: path.to_string(),
                    message: format!("expected integer, got {}", value_type_name(value)),
                });
            }
        }
        PropertySchema::Number => {
            if !value.is_number() {
                errors.push(SchemaValidationError {
                    path: path.to_string(),
                    message: format!("expected number, got {}", value_type_name(value)),
                });
            }
        }
        PropertySchema::Boolean => {
            if !value.is_boolean() {
                errors.push(SchemaValidationError {
                    path: path.to_string(),
                    message: format!("expected boolean, got {}", value_type_name(value)),
                });
            }
        }
        PropertySchema::Object => {
            if !value.is_object() {
                errors.push(SchemaValidationError {
                    path: path.to_string(),
                    message: format!("expected object, got {}", value_type_name(value)),
                });
            }
        }
        PropertySchema::Array => {
            if !value.is_array() {
                errors.push(SchemaValidationError {
                    path: path.to_string(),
                    message: format!("expected array, got {}", value_type_name(value)),
                });
            }
        }
        PropertySchema::Any => {
            // Any type is valid
        }
        PropertySchema::StringEnum(variants) => {
            if let Some(s) = value.as_str() {
                if !variants.contains(&s) {
                    errors.push(SchemaValidationError {
                        path: path.to_string(),
                        message: format!(
                            "invalid value '{}', expected one of: {}",
                            s,
                            variants.join(", ")
                        ),
                    });
                }
            } else {
                errors.push(SchemaValidationError {
                    path: path.to_string(),
                    message: format!("expected string, got {}", value_type_name(value)),
                });
            }
        }
    }
}

/// Get a human-readable type name for a JSON value.
fn value_type_name(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

// Schema definitions

fn manifest_schema() -> Schema {
    Schema {
        required: vec!["version"],
        properties: vec![
            ("version", PropertySchema::String),
            ("id", PropertySchema::String),
            (
                "state",
                PropertySchema::StringEnum(vec!["draft", "review", "frozen", "published"]),
            ),
            ("created", PropertySchema::String),
            ("modified", PropertySchema::String),
            ("content", PropertySchema::Object),
            ("metadata", PropertySchema::Object),
            ("security", PropertySchema::Object),
            ("presentation", PropertySchema::Object),
            ("assets", PropertySchema::Object),
            ("lineage", PropertySchema::Object),
        ],
    }
}

fn content_schema() -> Schema {
    Schema {
        required: vec!["version", "blocks"],
        properties: vec![
            ("version", PropertySchema::String),
            ("blocks", PropertySchema::Array),
        ],
    }
}

fn dublin_core_schema() -> Schema {
    Schema {
        required: vec!["version"],
        properties: vec![
            ("version", PropertySchema::String),
            ("title", PropertySchema::String),
            ("creator", PropertySchema::Any), // Can be string or array
            ("subject", PropertySchema::Any),
            ("description", PropertySchema::String),
            ("publisher", PropertySchema::String),
            ("contributor", PropertySchema::Any),
            ("date", PropertySchema::String),
            ("type", PropertySchema::String),
            ("format", PropertySchema::String),
            ("identifier", PropertySchema::String),
            ("source", PropertySchema::String),
            ("language", PropertySchema::String),
            ("relation", PropertySchema::String),
            ("coverage", PropertySchema::String),
            ("rights", PropertySchema::String),
        ],
    }
}

fn block_index_schema() -> Schema {
    Schema {
        required: vec!["version", "algorithm", "root", "blocks"],
        properties: vec![
            ("version", PropertySchema::String),
            (
                "algorithm",
                PropertySchema::StringEnum(vec!["sha256", "sha384", "sha512", "blake3"]),
            ),
            ("root", PropertySchema::String),
            ("blocks", PropertySchema::Array),
        ],
    }
}

fn signatures_schema() -> Schema {
    Schema {
        required: vec!["version", "signatures"],
        properties: vec![
            ("version", PropertySchema::String),
            ("signatures", PropertySchema::Array),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_manifest_valid() {
        let json = r#"{
            "version": "0.1",
            "state": "draft",
            "created": "2024-01-01T00:00:00Z"
        }"#;

        let errors = validate_manifest(json).unwrap();
        assert!(errors.is_empty(), "Expected no errors: {errors:?}");
    }

    #[test]
    fn test_validate_manifest_missing_version() {
        let json = r#"{
            "state": "draft"
        }"#;

        let errors = validate_manifest(json).unwrap();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("version"));
    }

    #[test]
    fn test_validate_manifest_invalid_state() {
        let json = r#"{
            "version": "0.1",
            "state": "invalid"
        }"#;

        let errors = validate_manifest(json).unwrap();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("invalid"));
    }

    #[test]
    fn test_validate_manifest_wrong_type() {
        let json = r#"{
            "version": 123
        }"#;

        let errors = validate_manifest(json).unwrap();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("string"));
    }

    #[test]
    fn test_validate_content_valid() {
        let json = r#"{
            "version": "0.1",
            "blocks": []
        }"#;

        let errors = validate_content(json).unwrap();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_content_missing_blocks() {
        let json = r#"{
            "version": "0.1"
        }"#;

        let errors = validate_content(json).unwrap();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("blocks"));
    }

    #[test]
    fn test_validate_dublin_core_valid() {
        let json = r#"{
            "version": "0.1",
            "title": "Test Document",
            "creator": "Test Author"
        }"#;

        let errors = validate_dublin_core(json).unwrap();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_dublin_core_array_creator() {
        let json = r#"{
            "version": "0.1",
            "title": "Test Document",
            "creator": ["Author 1", "Author 2"]
        }"#;

        let errors = validate_dublin_core(json).unwrap();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_block_index_valid() {
        let json = r#"{
            "version": "0.1",
            "algorithm": "sha256",
            "root": "abc123",
            "blocks": []
        }"#;

        let errors = validate_block_index(json).unwrap();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_block_index_invalid_algorithm() {
        let json = r#"{
            "version": "0.1",
            "algorithm": "md5",
            "root": "abc123",
            "blocks": []
        }"#;

        let errors = validate_block_index(json).unwrap();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("md5"));
    }

    #[test]
    fn test_validate_signatures_valid() {
        let json = r#"{
            "version": "0.1",
            "signatures": []
        }"#;

        let errors = validate_signatures(json).unwrap();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_invalid_json() {
        let json = "not valid json";

        let result = validate_manifest(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_display() {
        let error = SchemaValidationError {
            path: "manifest.version".to_string(),
            message: "expected string".to_string(),
        };
        assert_eq!(error.to_string(), "manifest.version: expected string");
    }

    #[test]
    fn test_error_display_empty_path() {
        let error = SchemaValidationError {
            path: String::new(),
            message: "expected object".to_string(),
        };
        assert_eq!(error.to_string(), "expected object");
    }
}
