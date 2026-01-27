//! Forms extension for interactive form fields.
//!
//! This extension provides form field types for creating interactive documents
//! with data collection capabilities.
//!
//! # Supported Field Types
//!
//! - `forms:textInput` - Single-line text input
//! - `forms:textArea` - Multi-line text input
//! - `forms:checkbox` - Boolean checkbox
//! - `forms:radioGroup` - Single selection from options
//! - `forms:dropdown` - Dropdown selection
//! - `forms:datePicker` - Date/time selection
//! - `forms:signature` - Digital signature capture
//!
//! # Example
//!
//! ```json
//! {
//!   "type": "forms:textInput",
//!   "id": "email",
//!   "label": "Email Address",
//!   "placeholder": "you@example.com",
//!   "required": true,
//!   "validation": {
//!     "rules": [{"type": "email"}]
//!   }
//! }
//! ```

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::ExtensionBlock;

/// A form field that can appear in a Codex document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "fieldType", rename_all = "camelCase")]
pub enum FormField {
    /// Single-line text input.
    TextInput(TextInputField),
    /// Multi-line text input.
    TextArea(TextAreaField),
    /// Boolean checkbox.
    Checkbox(CheckboxField),
    /// Single selection from radio options.
    RadioGroup(RadioGroupField),
    /// Dropdown selection.
    Dropdown(DropdownField),
    /// Date/time picker.
    DatePicker(DatePickerField),
    /// Digital signature capture.
    Signature(SignatureField),
}

impl FormField {
    /// Try to convert an extension block to a form field.
    #[must_use]
    pub fn from_extension(ext: &ExtensionBlock) -> Option<Self> {
        if ext.namespace != "forms" {
            return None;
        }

        match ext.block_type.as_str() {
            "textInput" => serde_json::from_value(ext.attributes.clone())
                .ok()
                .map(FormField::TextInput),
            "textArea" => serde_json::from_value(ext.attributes.clone())
                .ok()
                .map(FormField::TextArea),
            "checkbox" => serde_json::from_value(ext.attributes.clone())
                .ok()
                .map(FormField::Checkbox),
            "radioGroup" => serde_json::from_value(ext.attributes.clone())
                .ok()
                .map(FormField::RadioGroup),
            "dropdown" => serde_json::from_value(ext.attributes.clone())
                .ok()
                .map(FormField::Dropdown),
            "datePicker" => serde_json::from_value(ext.attributes.clone())
                .ok()
                .map(FormField::DatePicker),
            "signature" => serde_json::from_value(ext.attributes.clone())
                .ok()
                .map(FormField::Signature),
            _ => None,
        }
    }

    /// Get the field ID.
    #[must_use]
    pub fn id(&self) -> Option<&str> {
        match self {
            Self::TextInput(f) => f.id.as_deref(),
            Self::TextArea(f) => f.id.as_deref(),
            Self::Checkbox(f) => f.id.as_deref(),
            Self::RadioGroup(f) => f.id.as_deref(),
            Self::Dropdown(f) => f.id.as_deref(),
            Self::DatePicker(f) => f.id.as_deref(),
            Self::Signature(f) => f.id.as_deref(),
        }
    }

    /// Get the field label.
    #[must_use]
    pub fn label(&self) -> &str {
        match self {
            Self::TextInput(f) => &f.label,
            Self::TextArea(f) => &f.label,
            Self::Checkbox(f) => &f.label,
            Self::RadioGroup(f) => &f.label,
            Self::Dropdown(f) => &f.label,
            Self::DatePicker(f) => &f.label,
            Self::Signature(f) => &f.label,
        }
    }

    /// Check if the field is required.
    #[must_use]
    pub fn is_required(&self) -> bool {
        match self {
            Self::TextInput(f) => f.required,
            Self::TextArea(f) => f.required,
            Self::Checkbox(f) => f.required,
            Self::RadioGroup(f) => f.required,
            Self::Dropdown(f) => f.required,
            Self::DatePicker(f) => f.required,
            Self::Signature(f) => f.required,
        }
    }

    /// Get the field's validation rules.
    #[must_use]
    pub fn validation(&self) -> Option<&FormValidation> {
        match self {
            Self::TextInput(f) => f.validation.as_ref(),
            Self::TextArea(f) => f.validation.as_ref(),
            Self::DatePicker(f) => f.validation.as_ref(),
            Self::Checkbox(_) | Self::RadioGroup(_) | Self::Dropdown(_) | Self::Signature(_) => {
                None
            }
        }
    }
}

/// Single-line text input field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextInputField {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Field label displayed to the user.
    pub label: String,

    /// Placeholder text when empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,

    /// Default value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,

    /// Whether the field is required.
    #[serde(default)]
    pub required: bool,

    /// Whether the field is read-only.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub readonly: bool,

    /// Input type hint (text, email, url, tel, password).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_type: Option<String>,

    /// Validation rules.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation: Option<FormValidation>,
}

impl TextInputField {
    /// Create a new text input field.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            id: None,
            label: label.into(),
            placeholder: None,
            default_value: None,
            required: false,
            readonly: false,
            input_type: None,
            validation: None,
        }
    }

    /// Set the field ID.
    #[must_use]
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the placeholder text.
    #[must_use]
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Set the default value.
    #[must_use]
    pub fn with_default(mut self, value: impl Into<String>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    /// Mark as required.
    #[must_use]
    pub const fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Mark as read-only.
    #[must_use]
    pub const fn readonly(mut self) -> Self {
        self.readonly = true;
        self
    }

    /// Set the input type.
    #[must_use]
    pub fn with_input_type(mut self, input_type: impl Into<String>) -> Self {
        self.input_type = Some(input_type.into());
        self
    }

    /// Set validation rules.
    #[must_use]
    pub fn with_validation(mut self, validation: FormValidation) -> Self {
        self.validation = Some(validation);
        self
    }
}

/// Multi-line text area field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextAreaField {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Field label displayed to the user.
    pub label: String,

    /// Placeholder text when empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,

    /// Default value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,

    /// Whether the field is required.
    #[serde(default)]
    pub required: bool,

    /// Whether the field is read-only.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub readonly: bool,

    /// Number of visible rows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows: Option<u32>,

    /// Maximum character length.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,

    /// Validation rules.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation: Option<FormValidation>,
}

impl TextAreaField {
    /// Create a new text area field.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            id: None,
            label: label.into(),
            placeholder: None,
            default_value: None,
            required: false,
            readonly: false,
            rows: None,
            max_length: None,
            validation: None,
        }
    }

    /// Set the field ID.
    #[must_use]
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the number of rows.
    #[must_use]
    pub const fn with_rows(mut self, rows: u32) -> Self {
        self.rows = Some(rows);
        self
    }

    /// Set the maximum length.
    #[must_use]
    pub const fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    /// Mark as required.
    #[must_use]
    pub const fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

/// Boolean checkbox field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckboxField {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Field label displayed to the user.
    pub label: String,

    /// Default checked state.
    #[serde(default)]
    pub default_checked: bool,

    /// Whether the field is required (must be checked).
    #[serde(default)]
    pub required: bool,

    /// Whether the field is read-only.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub readonly: bool,
}

impl CheckboxField {
    /// Create a new checkbox field.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            id: None,
            label: label.into(),
            default_checked: false,
            required: false,
            readonly: false,
        }
    }

    /// Set the field ID.
    #[must_use]
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set default checked state.
    #[must_use]
    pub const fn checked(mut self) -> Self {
        self.default_checked = true;
        self
    }

    /// Mark as required.
    #[must_use]
    pub const fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

/// Radio button option.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadioOption {
    /// Option value (submitted value).
    pub value: String,

    /// Display label.
    pub label: String,

    /// Whether this option is disabled.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub disabled: bool,
}

impl RadioOption {
    /// Create a new radio option.
    #[must_use]
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }
}

/// Radio button group for single selection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadioGroupField {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Field label displayed to the user.
    pub label: String,

    /// Available options.
    pub options: Vec<RadioOption>,

    /// Default selected value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,

    /// Whether a selection is required.
    #[serde(default)]
    pub required: bool,

    /// Whether the field is read-only.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub readonly: bool,
}

impl RadioGroupField {
    /// Create a new radio group field.
    #[must_use]
    pub fn new(label: impl Into<String>, options: Vec<RadioOption>) -> Self {
        Self {
            id: None,
            label: label.into(),
            options,
            default_value: None,
            required: false,
            readonly: false,
        }
    }

    /// Set the field ID.
    #[must_use]
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the default value.
    #[must_use]
    pub fn with_default(mut self, value: impl Into<String>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    /// Mark as required.
    #[must_use]
    pub const fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

/// Dropdown option.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DropdownOption {
    /// Option value (submitted value).
    pub value: String,

    /// Display label.
    pub label: String,

    /// Whether this option is disabled.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub disabled: bool,

    /// Option group (for grouping options).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

impl DropdownOption {
    /// Create a new dropdown option.
    #[must_use]
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
            group: None,
        }
    }

    /// Set the option group.
    #[must_use]
    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }
}

/// Dropdown selection field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DropdownField {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Field label displayed to the user.
    pub label: String,

    /// Available options.
    pub options: Vec<DropdownOption>,

    /// Default selected value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,

    /// Placeholder text when no selection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,

    /// Whether a selection is required.
    #[serde(default)]
    pub required: bool,

    /// Whether the field is read-only.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub readonly: bool,

    /// Allow multiple selections.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub multiple: bool,
}

impl DropdownField {
    /// Create a new dropdown field.
    #[must_use]
    pub fn new(label: impl Into<String>, options: Vec<DropdownOption>) -> Self {
        Self {
            id: None,
            label: label.into(),
            options,
            default_value: None,
            placeholder: None,
            required: false,
            readonly: false,
            multiple: false,
        }
    }

    /// Set the field ID.
    #[must_use]
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the placeholder.
    #[must_use]
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Set the default value.
    #[must_use]
    pub fn with_default(mut self, value: impl Into<String>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    /// Mark as required.
    #[must_use]
    pub const fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Allow multiple selections.
    #[must_use]
    pub const fn multiple(mut self) -> Self {
        self.multiple = true;
        self
    }
}

/// Date/time picker field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatePickerField {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Field label displayed to the user.
    pub label: String,

    /// Picker mode (date, time, datetime).
    #[serde(default)]
    pub mode: DatePickerMode,

    /// Default value (ISO 8601 format).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,

    /// Minimum date/time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<String>,

    /// Maximum date/time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<String>,

    /// Whether the field is required.
    #[serde(default)]
    pub required: bool,

    /// Whether the field is read-only.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub readonly: bool,

    /// Validation rules.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation: Option<FormValidation>,
}

/// Date picker mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatePickerMode {
    /// Date only (YYYY-MM-DD).
    #[default]
    Date,
    /// Time only (HH:MM:SS).
    Time,
    /// Date and time.
    Datetime,
}

impl DatePickerField {
    /// Create a new date picker field.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            id: None,
            label: label.into(),
            mode: DatePickerMode::Date,
            default_value: None,
            min: None,
            max: None,
            required: false,
            readonly: false,
            validation: None,
        }
    }

    /// Set the field ID.
    #[must_use]
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the picker mode.
    #[must_use]
    pub const fn with_mode(mut self, mode: DatePickerMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set the minimum date.
    #[must_use]
    pub fn with_min(mut self, min: impl Into<String>) -> Self {
        self.min = Some(min.into());
        self
    }

    /// Set the maximum date.
    #[must_use]
    pub fn with_max(mut self, max: impl Into<String>) -> Self {
        self.max = Some(max.into());
        self
    }

    /// Mark as required.
    #[must_use]
    pub const fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

/// Digital signature capture field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureField {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Field label displayed to the user.
    pub label: String,

    /// Whether the signature is required.
    #[serde(default)]
    pub required: bool,

    /// Whether the field is read-only.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub readonly: bool,

    /// Legal text that must be agreed to before signing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legal_text: Option<String>,
}

impl SignatureField {
    /// Create a new signature field.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            id: None,
            label: label.into(),
            required: false,
            readonly: false,
            legal_text: None,
        }
    }

    /// Set the field ID.
    #[must_use]
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the legal text.
    #[must_use]
    pub fn with_legal_text(mut self, text: impl Into<String>) -> Self {
        self.legal_text = Some(text.into());
        self
    }

    /// Mark as required.
    #[must_use]
    pub const fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

/// Form validation configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormValidation {
    /// Validation rules to apply.
    pub rules: Vec<ValidationRule>,
}

impl FormValidation {
    /// Create a new validation configuration.
    #[must_use]
    pub fn new(rules: Vec<ValidationRule>) -> Self {
        Self { rules }
    }

    /// Create validation with a single rule.
    #[must_use]
    pub fn with_rule(rule: ValidationRule) -> Self {
        Self { rules: vec![rule] }
    }
}

/// A validation rule for form fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ValidationRule {
    /// Field is required.
    Required {
        /// Custom error message.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    /// Minimum length.
    MinLength {
        /// Minimum character count.
        value: usize,
        /// Custom error message.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    /// Maximum length.
    MaxLength {
        /// Maximum character count.
        value: usize,
        /// Custom error message.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    /// Regular expression pattern.
    Pattern {
        /// Regex pattern.
        pattern: String,
        /// Custom error message.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    /// Email format.
    Email {
        /// Custom error message.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    /// URL format.
    Url {
        /// Custom error message.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    /// Minimum numeric value.
    Min {
        /// Minimum value.
        value: i64,
        /// Custom error message.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    /// Maximum numeric value.
    Max {
        /// Maximum value.
        value: i64,
        /// Custom error message.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    /// Custom validation expression.
    Custom {
        /// Validation expression.
        expression: String,
        /// Error message when validation fails.
        message: String,
    },
}

impl ValidationRule {
    /// Create a required rule.
    #[must_use]
    pub fn required() -> Self {
        Self::Required { message: None }
    }

    /// Create a min length rule.
    #[must_use]
    pub fn min_length(value: usize) -> Self {
        Self::MinLength {
            value,
            message: None,
        }
    }

    /// Create a max length rule.
    #[must_use]
    pub fn max_length(value: usize) -> Self {
        Self::MaxLength {
            value,
            message: None,
        }
    }

    /// Create a pattern rule.
    #[must_use]
    pub fn pattern(pattern: impl Into<String>) -> Self {
        Self::Pattern {
            pattern: pattern.into(),
            message: None,
        }
    }

    /// Create an email rule.
    #[must_use]
    pub fn email() -> Self {
        Self::Email { message: None }
    }

    /// Create a URL rule.
    #[must_use]
    pub fn url() -> Self {
        Self::Url { message: None }
    }
}

/// Form data submitted by users.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormData {
    /// Field values indexed by field ID.
    pub values: HashMap<String, Value>,

    /// Whether the form has been submitted.
    #[serde(default)]
    pub submitted: bool,

    /// When the form was last modified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<DateTime<Utc>>,

    /// Submitter information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub submitted_by: Option<String>,

    /// When the form was submitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub submitted_at: Option<DateTime<Utc>>,
}

impl FormData {
    /// Create new empty form data.
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            submitted: false,
            last_modified: None,
            submitted_by: None,
            submitted_at: None,
        }
    }

    /// Set a field value.
    pub fn set(&mut self, field_id: impl Into<String>, value: Value) {
        self.values.insert(field_id.into(), value);
        self.last_modified = Some(Utc::now());
    }

    /// Get a field value.
    #[must_use]
    pub fn get(&self, field_id: &str) -> Option<&Value> {
        self.values.get(field_id)
    }

    /// Get a string value.
    #[must_use]
    pub fn get_string(&self, field_id: &str) -> Option<&str> {
        self.values.get(field_id).and_then(Value::as_str)
    }

    /// Get a boolean value.
    #[must_use]
    pub fn get_bool(&self, field_id: &str) -> Option<bool> {
        self.values.get(field_id).and_then(Value::as_bool)
    }

    /// Mark the form as submitted.
    pub fn submit(&mut self, by: Option<String>) {
        self.submitted = true;
        self.submitted_by = by;
        self.submitted_at = Some(Utc::now());
    }
}

impl Default for FormData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_text_input_field() {
        let field = TextInputField::new("Full Name")
            .with_id("name")
            .with_placeholder("Enter your name")
            .required();

        assert_eq!(field.label, "Full Name");
        assert_eq!(field.id, Some("name".to_string()));
        assert!(field.required);
    }

    #[test]
    fn test_text_input_serialization() {
        let field = TextInputField::new("Email")
            .with_id("email")
            .with_input_type("email")
            .with_validation(FormValidation::with_rule(ValidationRule::email()));

        let json = serde_json::to_string_pretty(&field).unwrap();
        assert!(json.contains("\"label\": \"Email\""));
        assert!(json.contains("\"inputType\": \"email\""));
        assert!(json.contains("\"type\": \"email\""));
    }

    #[test]
    fn test_checkbox_field() {
        let field = CheckboxField::new("I agree to the terms")
            .with_id("terms")
            .required();

        assert_eq!(field.label, "I agree to the terms");
        assert!(field.required);
        assert!(!field.default_checked);
    }

    #[test]
    fn test_radio_group() {
        let options = vec![
            RadioOption::new("sm", "Small"),
            RadioOption::new("md", "Medium"),
            RadioOption::new("lg", "Large"),
        ];

        let field = RadioGroupField::new("Size", options)
            .with_id("size")
            .with_default("md");

        assert_eq!(field.options.len(), 3);
        assert_eq!(field.default_value, Some("md".to_string()));
    }

    #[test]
    fn test_dropdown_with_groups() {
        let options = vec![
            DropdownOption::new("us", "United States").with_group("North America"),
            DropdownOption::new("ca", "Canada").with_group("North America"),
            DropdownOption::new("uk", "United Kingdom").with_group("Europe"),
        ];

        let field = DropdownField::new("Country", options)
            .with_placeholder("Select a country")
            .required();

        assert_eq!(field.options.len(), 3);
        assert_eq!(field.options[0].group, Some("North America".to_string()));
    }

    #[test]
    fn test_date_picker() {
        let field = DatePickerField::new("Appointment Date")
            .with_id("date")
            .with_mode(DatePickerMode::Datetime)
            .with_min("2024-01-01")
            .required();

        assert_eq!(field.mode, DatePickerMode::Datetime);
        assert_eq!(field.min, Some("2024-01-01".to_string()));
    }

    #[test]
    fn test_signature_field() {
        let field = SignatureField::new("Signature")
            .with_id("sig")
            .with_legal_text("By signing, you agree to...")
            .required();

        assert!(field.required);
        assert!(field.legal_text.is_some());
    }

    #[test]
    fn test_validation_rules() {
        let validation = FormValidation::new(vec![
            ValidationRule::required(),
            ValidationRule::min_length(2),
            ValidationRule::max_length(50),
            ValidationRule::pattern(r"^[A-Za-z\s]+$"),
        ]);

        assert_eq!(validation.rules.len(), 4);
    }

    #[test]
    fn test_form_data() {
        let mut data = FormData::new();
        data.set("name", json!("John Doe"));
        data.set("age", json!(30));
        data.set("active", json!(true));

        assert_eq!(data.get_string("name"), Some("John Doe"));
        assert_eq!(data.get_bool("active"), Some(true));
        assert!(!data.submitted);

        data.submit(Some("user@example.com".to_string()));
        assert!(data.submitted);
        assert_eq!(data.submitted_by, Some("user@example.com".to_string()));
    }

    #[test]
    fn test_form_field_enum() {
        let text_input = FormField::TextInput(TextInputField::new("Name").required());

        assert!(text_input.is_required());
        assert_eq!(text_input.label(), "Name");
    }

    #[test]
    fn test_form_field_serialization() {
        let field = FormField::TextInput(TextInputField::new("Name").with_id("name"));
        let json = serde_json::to_string(&field).unwrap();
        assert!(json.contains("\"fieldType\":\"textInput\""));
    }
}
