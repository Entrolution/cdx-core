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

/// A resolution variant of an image for responsive display.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageVariant {
    /// Path within the archive.
    pub path: String,

    /// Content hash for verification.
    pub hash: DocumentId,

    /// Image width in pixels.
    pub width: u32,

    /// Image height in pixels.
    pub height: u32,

    /// Scale factor (e.g., 1.0 for 1x, 2.0 for 2x, 3.0 for 3x).
    pub scale: f32,

    /// File size in bytes.
    pub size: u64,
}

impl ImageVariant {
    /// Create a new image variant.
    #[must_use]
    pub fn new(path: impl Into<String>, width: u32, height: u32, scale: f32) -> Self {
        Self {
            path: path.into(),
            hash: DocumentId::pending(),
            width,
            height,
            scale,
            size: 0,
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

    /// Create a 1x variant.
    #[must_use]
    pub fn scale_1x(path: impl Into<String>, width: u32, height: u32) -> Self {
        Self::new(path, width, height, 1.0)
    }

    /// Create a 2x (Retina) variant.
    #[must_use]
    pub fn scale_2x(path: impl Into<String>, width: u32, height: u32) -> Self {
        Self::new(path, width, height, 2.0)
    }

    /// Create a 3x variant.
    #[must_use]
    pub fn scale_3x(path: impl Into<String>, width: u32, height: u32) -> Self {
        Self::new(path, width, height, 3.0)
    }
}

/// An image asset embedded in a CDX document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

    /// Resolution variants for responsive images.
    ///
    /// Each variant represents the same image at a different resolution,
    /// typically for different screen densities (1x, 2x, 3x).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<ImageVariant>,
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
            variants: Vec::new(),
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

    /// Add a resolution variant.
    #[must_use]
    pub fn with_variant(mut self, variant: ImageVariant) -> Self {
        self.variants.push(variant);
        self
    }

    /// Add multiple resolution variants.
    #[must_use]
    pub fn with_variants(mut self, variants: Vec<ImageVariant>) -> Self {
        self.variants = variants;
        self
    }

    /// Check if this image has resolution variants.
    #[must_use]
    pub fn has_variants(&self) -> bool {
        !self.variants.is_empty()
    }

    /// Get the variant for a specific scale, if available.
    #[must_use]
    pub fn variant_for_scale(&self, scale: f32) -> Option<&ImageVariant> {
        self.variants
            .iter()
            .find(|v| (v.scale - scale).abs() < 0.01)
    }

    /// Get the best variant for a given target width.
    ///
    /// Returns the smallest variant that is at least as wide as the target,
    /// or the largest available variant if none are wide enough.
    #[must_use]
    pub fn best_variant_for_width(&self, target_width: u32) -> Option<&ImageVariant> {
        if self.variants.is_empty() {
            return None;
        }

        // Find smallest variant >= target width
        let mut candidates: Vec<_> = self
            .variants
            .iter()
            .filter(|v| v.width >= target_width)
            .collect();

        if candidates.is_empty() {
            // No variant is wide enough, return the largest
            self.variants.iter().max_by_key(|v| v.width)
        } else {
            // Return the smallest that fits
            candidates.sort_by_key(|v| v.width);
            candidates.first().copied()
        }
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
            .with_size(524_288);

        assert_eq!(image.width, Some(1920));
        assert_eq!(image.height, Some(1080));
        assert_eq!(image.alt, Some("A beautiful sunset".to_string()));
        assert_eq!(image.size, 524_288);
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

    #[test]
    fn test_image_variant_creation() {
        let variant = ImageVariant::new("assets/images/logo@2x.png", 400, 200, 2.0).with_size(8192);

        assert_eq!(variant.width, 400);
        assert_eq!(variant.height, 200);
        assert!((variant.scale - 2.0).abs() < f32::EPSILON);
        assert_eq!(variant.size, 8192);
    }

    #[test]
    fn test_image_variant_scale_helpers() {
        let v1x = ImageVariant::scale_1x("logo.png", 100, 50);
        let v2x = ImageVariant::scale_2x("logo@2x.png", 200, 100);
        let v3x = ImageVariant::scale_3x("logo@3x.png", 300, 150);

        assert!((v1x.scale - 1.0).abs() < f32::EPSILON);
        assert!((v2x.scale - 2.0).abs() < f32::EPSILON);
        assert!((v3x.scale - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_image_asset_with_variants() {
        let image = ImageAsset::new("logo", ImageFormat::Png)
            .with_dimensions(100, 50)
            .with_variant(ImageVariant::scale_1x("assets/images/logo.png", 100, 50))
            .with_variant(ImageVariant::scale_2x(
                "assets/images/logo@2x.png",
                200,
                100,
            ));

        assert!(image.has_variants());
        assert_eq!(image.variants.len(), 2);
    }

    #[test]
    fn test_image_variant_for_scale() {
        let image = ImageAsset::new("logo", ImageFormat::Png)
            .with_variant(ImageVariant::scale_1x("logo.png", 100, 50))
            .with_variant(ImageVariant::scale_2x("logo@2x.png", 200, 100));

        assert!(image.variant_for_scale(1.0).is_some());
        assert!(image.variant_for_scale(2.0).is_some());
        assert!(image.variant_for_scale(3.0).is_none());
    }

    #[test]
    fn test_image_best_variant_for_width() {
        let image = ImageAsset::new("logo", ImageFormat::Png)
            .with_variant(ImageVariant::scale_1x("logo.png", 100, 50))
            .with_variant(ImageVariant::scale_2x("logo@2x.png", 200, 100))
            .with_variant(ImageVariant::scale_3x("logo@3x.png", 300, 150));

        // Should return 1x (100) - smallest that fits
        let best = image.best_variant_for_width(80);
        assert!(best.is_some());
        assert_eq!(best.unwrap().width, 100);

        // Should return 2x (200) - smallest that fits
        let best = image.best_variant_for_width(150);
        assert!(best.is_some());
        assert_eq!(best.unwrap().width, 200);

        // Should return 3x (300) - only one that fits
        let best = image.best_variant_for_width(250);
        assert!(best.is_some());
        assert_eq!(best.unwrap().width, 300);

        // Should return 3x (300) - largest available
        let best = image.best_variant_for_width(400);
        assert!(best.is_some());
        assert_eq!(best.unwrap().width, 300);
    }

    #[test]
    fn test_image_variant_serialization() {
        let image = ImageAsset::new("responsive", ImageFormat::Png)
            .with_dimensions(100, 50)
            .with_variant(ImageVariant::scale_2x(
                "assets/images/responsive@2x.png",
                200,
                100,
            ));

        let json = serde_json::to_string_pretty(&image).unwrap();
        assert!(json.contains("variants"));
        assert!(json.contains("@2x"));

        let deserialized: ImageAsset = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.variants.len(), 1);
        assert_eq!(deserialized.variants[0].width, 200);
    }
}
