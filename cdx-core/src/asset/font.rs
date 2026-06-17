//! Font asset types.

use serde::{Deserialize, Serialize};

use crate::DocumentId;

/// Font format enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FontFormat {
    /// WOFF2 format (required, preferred).
    Woff2,
    /// WOFF format (required).
    Woff,
    /// TrueType format (optional).
    Ttf,
    /// OpenType format (optional).
    Otf,
}

impl FontFormat {
    /// Get the file extension for this format.
    #[must_use]
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::Woff2 => "woff2",
            Self::Woff => "woff",
            Self::Ttf => "ttf",
            Self::Otf => "otf",
        }
    }

    /// Get the MIME type for this format.
    #[must_use]
    pub const fn mime_type(&self) -> &'static str {
        match self {
            Self::Woff2 => "font/woff2",
            Self::Woff => "font/woff",
            Self::Ttf => "font/ttf",
            Self::Otf => "font/otf",
        }
    }
}

impl std::fmt::Display for FontFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.extension())
    }
}

/// Font weight values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FontWeight {
    /// Thin (100).
    Thin,
    /// Extra Light (200).
    ExtraLight,
    /// Light (300).
    Light,
    /// Normal/Regular (400).
    #[default]
    Normal,
    /// Medium (500).
    Medium,
    /// Semi Bold (600).
    SemiBold,
    /// Bold (700).
    Bold,
    /// Extra Bold (800).
    ExtraBold,
    /// Black (900).
    Black,
    /// Custom numeric weight.
    #[serde(untagged)]
    Custom(u16),
}

impl FontWeight {
    /// Get the numeric weight value.
    #[must_use]
    pub const fn value(&self) -> u16 {
        match self {
            Self::Thin => 100,
            Self::ExtraLight => 200,
            Self::Light => 300,
            Self::Normal => 400,
            Self::Medium => 500,
            Self::SemiBold => 600,
            Self::Bold => 700,
            Self::ExtraBold => 800,
            Self::Black => 900,
            Self::Custom(v) => *v,
        }
    }
}

/// Font style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FontStyle {
    /// Normal (upright) style.
    #[default]
    Normal,
    /// Italic style.
    Italic,
    /// Oblique style.
    Oblique,
}

/// A font asset embedded in a CDX document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontAsset {
    /// Unique identifier for the font.
    pub id: String,

    /// Path within the archive (e.g., "assets/fonts/roboto-regular.woff2").
    pub path: String,

    /// Content hash for verification.
    pub hash: DocumentId,

    /// Font format.
    pub format: FontFormat,

    /// File size in bytes.
    pub size: u64,

    /// Font family name.
    pub family: String,

    /// Font weight.
    #[serde(default)]
    pub weight: FontWeight,

    /// Font style.
    #[serde(default)]
    pub style: FontStyle,

    /// Unicode range covered (CSS unicode-range format).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unicode_range: Option<String>,

    /// Font feature settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feature_settings: Option<String>,

    /// Font variation settings (for variable fonts).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variation_settings: Option<String>,

    /// License information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

impl FontAsset {
    /// Create a new font asset.
    #[must_use]
    pub fn new(id: impl Into<String>, family: impl Into<String>, format: FontFormat) -> Self {
        let id = id.into();
        let family = family.into();
        let path = format!("assets/fonts/{}.{}", id, format.extension());
        Self {
            id,
            path,
            hash: DocumentId::pending(),
            format,
            size: 0,
            family,
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
            unicode_range: None,
            feature_settings: None,
            variation_settings: None,
            license: None,
        }
    }

    /// Set the content hash.
    #[must_use]
    pub fn with_hash(mut self, hash: DocumentId) -> Self {
        self.hash = hash;
        self
    }

    /// Set the file size.
    #[must_use]
    pub const fn with_size(mut self, size: u64) -> Self {
        self.size = size;
        self
    }

    /// Set the font weight.
    #[must_use]
    pub const fn with_weight(mut self, weight: FontWeight) -> Self {
        self.weight = weight;
        self
    }

    /// Set the font style.
    #[must_use]
    pub const fn with_style(mut self, style: FontStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the unicode range.
    #[must_use]
    pub fn with_unicode_range(mut self, range: impl Into<String>) -> Self {
        self.unicode_range = Some(range.into());
        self
    }

    /// Set the license.
    #[must_use]
    pub fn with_license(mut self, license: impl Into<String>) -> Self {
        self.license = Some(license.into());
        self
    }

    /// Set a custom path.
    #[must_use]
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }
}

impl super::Asset for FontAsset {
    fn id(&self) -> &str {
        &self.id
    }

    fn path(&self) -> &str {
        &self.path
    }

    fn hash(&self) -> &DocumentId {
        &self.hash
    }

    fn size(&self) -> u64 {
        self.size
    }

    fn mime_type(&self) -> &str {
        self.format.mime_type()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_format_extension() {
        assert_eq!(FontFormat::Woff2.extension(), "woff2");
        assert_eq!(FontFormat::Woff.extension(), "woff");
        assert_eq!(FontFormat::Ttf.extension(), "ttf");
        assert_eq!(FontFormat::Otf.extension(), "otf");
    }

    #[test]
    fn test_font_format_mime_type() {
        assert_eq!(FontFormat::Woff2.mime_type(), "font/woff2");
        assert_eq!(FontFormat::Ttf.mime_type(), "font/ttf");
    }

    #[test]
    fn test_font_weight_value() {
        assert_eq!(FontWeight::Normal.value(), 400);
        assert_eq!(FontWeight::Bold.value(), 700);
        assert_eq!(FontWeight::Custom(450).value(), 450);
    }

    #[test]
    fn test_font_asset_new() {
        let font = FontAsset::new("roboto-regular", "Roboto", FontFormat::Woff2);
        assert_eq!(font.id, "roboto-regular");
        assert_eq!(font.family, "Roboto");
        assert_eq!(font.path, "assets/fonts/roboto-regular.woff2");
        assert_eq!(font.format, FontFormat::Woff2);
    }

    #[test]
    fn test_font_asset_builder() {
        let font = FontAsset::new("roboto-bold", "Roboto", FontFormat::Woff2)
            .with_weight(FontWeight::Bold)
            .with_style(FontStyle::Normal)
            .with_size(32768)
            .with_license("Apache-2.0");

        assert_eq!(font.weight, FontWeight::Bold);
        assert_eq!(font.style, FontStyle::Normal);
        assert_eq!(font.size, 32768);
        assert_eq!(font.license, Some("Apache-2.0".to_string()));
    }

    #[test]
    fn test_font_asset_serialization() {
        let font = FontAsset::new("test-font", "Test Family", FontFormat::Woff2)
            .with_weight(FontWeight::Bold);

        let json = serde_json::to_string_pretty(&font).unwrap();
        assert!(json.contains(r#""family": "Test Family""#));
        assert!(json.contains(r#""format": "woff2""#));

        let deserialized: FontAsset = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.family, font.family);
        assert_eq!(deserialized.weight, font.weight);
    }
}
