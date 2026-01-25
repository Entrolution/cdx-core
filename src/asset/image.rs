//! Image asset types.

use serde::{Deserialize, Serialize};

use crate::DocumentId;

/// Image format enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    /// AVIF format (required, preferred for raster).
    Avif,
    /// WebP format (required).
    WebP,
    /// PNG format (required).
    Png,
    /// JPEG format (required).
    Jpeg,
    /// SVG format (required for vector).
    Svg,
}

impl ImageFormat {
    /// Get the file extension for this format.
    #[must_use]
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::Avif => "avif",
            Self::WebP => "webp",
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Svg => "svg",
        }
    }

    /// Get the MIME type for this format.
    #[must_use]
    pub const fn mime_type(&self) -> &'static str {
        match self {
            Self::Avif => "image/avif",
            Self::WebP => "image/webp",
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Svg => "image/svg+xml",
        }
    }

    /// Check if this is a vector format.
    #[must_use]
    pub const fn is_vector(&self) -> bool {
        matches!(self, Self::Svg)
    }

    /// Check if this is a raster format.
    #[must_use]
    pub const fn is_raster(&self) -> bool {
        !self.is_vector()
    }
}

impl std::fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.extension())
    }
}

/// An image asset embedded in a Codex document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageAsset {
    /// Unique identifier for the image.
    pub id: String,

    /// Path within the archive (e.g., "assets/images/logo.png").
    pub path: String,

    /// Content hash for verification.
    pub hash: DocumentId,

    /// Image format.
    pub format: ImageFormat,

    /// File size in bytes.
    pub size: u64,

    /// Image width in pixels (for raster images).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,

    /// Image height in pixels (for raster images).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,

    /// Alternative text for accessibility.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,

    /// Optional title/caption.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Source attribution or copyright.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,
}

impl ImageAsset {
    /// Create a new image asset.
    #[must_use]
    pub fn new(id: impl Into<String>, format: ImageFormat) -> Self {
        let id = id.into();
        let path = format!("assets/images/{}.{}", id, format.extension());
        Self {
            id,
            path,
            hash: DocumentId::pending(),
            format,
            size: 0,
            width: None,
            height: None,
            alt: None,
            title: None,
            attribution: None,
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

    /// Set the dimensions.
    #[must_use]
    pub fn with_dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Set the alternative text.
    #[must_use]
    pub fn with_alt(mut self, alt: impl Into<String>) -> Self {
        self.alt = Some(alt.into());
        self
    }

    /// Set the title.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the attribution.
    #[must_use]
    pub fn with_attribution(mut self, attribution: impl Into<String>) -> Self {
        self.attribution = Some(attribution.into());
        self
    }

    /// Set a custom path.
    #[must_use]
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }
}

impl super::Asset for ImageAsset {
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
    fn test_image_format_extension() {
        assert_eq!(ImageFormat::Avif.extension(), "avif");
        assert_eq!(ImageFormat::WebP.extension(), "webp");
        assert_eq!(ImageFormat::Png.extension(), "png");
        assert_eq!(ImageFormat::Jpeg.extension(), "jpg");
        assert_eq!(ImageFormat::Svg.extension(), "svg");
    }

    #[test]
    fn test_image_format_mime_type() {
        assert_eq!(ImageFormat::Avif.mime_type(), "image/avif");
        assert_eq!(ImageFormat::Svg.mime_type(), "image/svg+xml");
    }

    #[test]
    fn test_image_format_vector_raster() {
        assert!(ImageFormat::Svg.is_vector());
        assert!(!ImageFormat::Png.is_vector());
        assert!(ImageFormat::Png.is_raster());
    }

    #[test]
    fn test_image_asset_new() {
        let image = ImageAsset::new("logo", ImageFormat::Png);
        assert_eq!(image.id, "logo");
        assert_eq!(image.path, "assets/images/logo.png");
        assert_eq!(image.format, ImageFormat::Png);
    }

    #[test]
    fn test_image_asset_builder() {
        let image = ImageAsset::new("photo", ImageFormat::Jpeg)
            .with_dimensions(1920, 1080)
            .with_alt("A beautiful sunset")
            .with_size(524288);

        assert_eq!(image.width, Some(1920));
        assert_eq!(image.height, Some(1080));
        assert_eq!(image.alt, Some("A beautiful sunset".to_string()));
        assert_eq!(image.size, 524288);
    }

    #[test]
    fn test_image_asset_serialization() {
        let image = ImageAsset::new("test", ImageFormat::Png)
            .with_dimensions(100, 100)
            .with_alt("Test image");

        let json = serde_json::to_string_pretty(&image).unwrap();
        assert!(json.contains(r#""id": "test""#));
        assert!(json.contains(r#""format": "png""#));
        assert!(json.contains(r#""width": 100"#));

        let deserialized: ImageAsset = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, image.id);
        assert_eq!(deserialized.format, image.format);
    }
}
