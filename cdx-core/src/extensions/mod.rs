//! Extension framework for Codex documents.
//!
//! Extensions allow Codex documents to include specialized content types
//! beyond the core block types. Each extension is namespaced (e.g., "forms",
//! "semantic", "collaboration") and provides custom block types.
//!
//! # Graceful Degradation
//!
//! When a reader encounters an unknown extension, it preserves the raw
//! data as an `ExtensionBlock` and can optionally render fallback content.
//!
//! # Example
//!
//! ```json
//! {
//!   "type": "forms:textInput",
//!   "id": "name-field",
//!   "label": "Full Name",
//!   "required": true,
//!   "fallback": {
//!     "type": "paragraph",
//!     "children": [{"value": "[Form field: Full Name]"}]
//!   }
//! }
//! ```

mod forms;

pub use forms::{
    CheckboxField, DatePickerField, DropdownField, DropdownOption, FormData, FormField,
    FormValidation, RadioGroupField, RadioOption, SignatureField, TextAreaField, TextInputField,
    ValidationRule,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::content::Block;

/// An extension block for unsupported or unknown block types.
///
/// When parsing a document with extension blocks (e.g., "forms:textInput"),
/// this struct preserves the raw data so it can be:
/// - Passed through unchanged when saving
/// - Rendered using fallback content
/// - Processed by extension-aware applications
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionBlock {
    /// The extension namespace (e.g., "forms", "semantic", "collaboration").
    pub namespace: String,

    /// The block type within the namespace (e.g., "textInput", "citation").
    pub block_type: String,

    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Raw attributes from the original block.
    ///
    /// This preserves all extension-specific properties without interpretation.
    #[serde(default, skip_serializing_if = "Value::is_null")]
    pub attributes: Value,

    /// Child blocks (if the extension is a container).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Block>,

    /// Fallback content for renderers that don't support this extension.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback: Option<Box<Block>>,
}

impl ExtensionBlock {
    /// Create a new extension block.
    #[must_use]
    pub fn new(namespace: impl Into<String>, block_type: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            block_type: block_type.into(),
            id: None,
            attributes: Value::Null,
            children: Vec::new(),
            fallback: None,
        }
    }

    /// Parse an extension type string like "forms:textInput" into (namespace, `block_type`).
    ///
    /// Returns `None` if the type doesn't contain a colon.
    #[must_use]
    pub fn parse_type(type_str: &str) -> Option<(&str, &str)> {
        type_str.split_once(':')
    }

    /// Get the full type string (e.g., "forms:textInput").
    #[must_use]
    pub fn full_type(&self) -> String {
        format!("{}:{}", self.namespace, self.block_type)
    }

    /// Check if this extension is from a specific namespace.
    #[must_use]
    pub fn is_namespace(&self, namespace: &str) -> bool {
        self.namespace == namespace
    }

    /// Check if this is a specific extension type.
    #[must_use]
    pub fn is_type(&self, namespace: &str, block_type: &str) -> bool {
        self.namespace == namespace && self.block_type == block_type
    }

    /// Set the block ID.
    #[must_use]
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the attributes.
    #[must_use]
    pub fn with_attributes(mut self, attributes: Value) -> Self {
        self.attributes = attributes;
        self
    }

    /// Set the children.
    #[must_use]
    pub fn with_children(mut self, children: Vec<Block>) -> Self {
        self.children = children;
        self
    }

    /// Set fallback content.
    #[must_use]
    pub fn with_fallback(mut self, fallback: Block) -> Self {
        self.fallback = Some(Box::new(fallback));
        self
    }

    /// Get the fallback content, if any.
    #[must_use]
    pub fn fallback_content(&self) -> Option<&Block> {
        self.fallback.as_deref()
    }

    /// Get an attribute value by key.
    #[must_use]
    pub fn get_attribute(&self, key: &str) -> Option<&Value> {
        self.attributes.get(key)
    }

    /// Get a string attribute.
    #[must_use]
    pub fn get_string_attribute(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).and_then(Value::as_str)
    }

    /// Get a boolean attribute.
    #[must_use]
    pub fn get_bool_attribute(&self, key: &str) -> Option<bool> {
        self.attributes.get(key).and_then(Value::as_bool)
    }

    /// Get an integer attribute.
    #[must_use]
    pub fn get_i64_attribute(&self, key: &str) -> Option<i64> {
        self.attributes.get(key).and_then(Value::as_i64)
    }

    /// Try to convert this extension block to a specific form field type.
    ///
    /// Returns `None` if this is not a forms extension or conversion fails.
    #[must_use]
    pub fn as_form_field(&self) -> Option<FormField> {
        if self.namespace != "forms" {
            return None;
        }
        FormField::from_extension(self)
    }
}

/// Known extension namespaces.
pub mod namespaces {
    /// Forms extension for interactive form fields.
    pub const FORMS: &str = "forms";
    /// Semantic extension for citations, glossary, entity linking.
    pub const SEMANTIC: &str = "semantic";
    /// Collaboration extension for comments, suggestions, change tracking.
    pub const COLLABORATION: &str = "collaboration";
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extension_block_new() {
        let ext = ExtensionBlock::new("forms", "textInput");
        assert_eq!(ext.namespace, "forms");
        assert_eq!(ext.block_type, "textInput");
        assert_eq!(ext.full_type(), "forms:textInput");
    }

    #[test]
    fn test_parse_type() {
        assert_eq!(
            ExtensionBlock::parse_type("forms:textInput"),
            Some(("forms", "textInput"))
        );
        assert_eq!(
            ExtensionBlock::parse_type("semantic:citation"),
            Some(("semantic", "citation"))
        );
        assert_eq!(ExtensionBlock::parse_type("paragraph"), None);
    }

    #[test]
    fn test_extension_block_builder() {
        let ext = ExtensionBlock::new("forms", "checkbox")
            .with_id("accept-terms")
            .with_attributes(json!({
                "label": "I accept the terms",
                "required": true
            }));

        assert_eq!(ext.id, Some("accept-terms".to_string()));
        assert_eq!(
            ext.get_string_attribute("label"),
            Some("I accept the terms")
        );
        assert_eq!(ext.get_bool_attribute("required"), Some(true));
    }

    #[test]
    fn test_extension_namespace_check() {
        let ext = ExtensionBlock::new("forms", "textInput");
        assert!(ext.is_namespace("forms"));
        assert!(!ext.is_namespace("semantic"));
        assert!(ext.is_type("forms", "textInput"));
        assert!(!ext.is_type("forms", "checkbox"));
    }

    #[test]
    fn test_extension_with_fallback() {
        use crate::content::Text;

        let fallback = Block::paragraph(vec![Text::plain("[Form field: Name]")]);
        let ext = ExtensionBlock::new("forms", "textInput").with_fallback(fallback.clone());

        assert!(ext.fallback_content().is_some());
        if let Block::Paragraph { children, .. } = ext.fallback_content().unwrap() {
            assert_eq!(children[0].value, "[Form field: Name]");
        }
    }

    #[test]
    fn test_extension_serialization() {
        let ext = ExtensionBlock::new("forms", "textInput")
            .with_id("name")
            .with_attributes(json!({"label": "Name", "required": true}));

        let json = serde_json::to_string_pretty(&ext).unwrap();
        assert!(json.contains("\"namespace\": \"forms\""));
        assert!(json.contains("\"blockType\": \"textInput\""));
        assert!(json.contains("\"label\": \"Name\""));
    }

    #[test]
    fn test_extension_deserialization() {
        let json = r#"{
            "namespace": "semantic",
            "blockType": "citation",
            "id": "cite-1",
            "attributes": {
                "ref": "smith2023",
                "page": 42
            }
        }"#;

        let ext: ExtensionBlock = serde_json::from_str(json).unwrap();
        assert_eq!(ext.namespace, "semantic");
        assert_eq!(ext.block_type, "citation");
        assert_eq!(ext.id, Some("cite-1".to_string()));
        assert_eq!(ext.get_string_attribute("ref"), Some("smith2023"));
        assert_eq!(ext.get_i64_attribute("page"), Some(42));
    }
}
