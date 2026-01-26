//! Asset index types for managing collections of assets.

use serde::{Deserialize, Serialize};

use super::{FontAsset, ImageAsset};
use crate::DocumentId;

/// An asset index file structure.
///
/// This represents files like `assets/images/index.json` or `assets/fonts/index.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndex<T> {
    /// Format version.
    pub version: String,

    /// Total count of assets.
    pub count: u32,

    /// Total size of all assets in bytes.
    pub total_size: u64,

    /// Array of asset entries.
    pub assets: Vec<T>,
}

impl<T> AssetIndex<T> {
    /// Create a new empty asset index.
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: crate::SPEC_VERSION.to_string(),
            count: 0,
            total_size: 0,
            assets: Vec::new(),
        }
    }

    /// Add an asset to the index.
    pub fn add(&mut self, asset: T, size: u64) {
        self.assets.push(asset);
        self.count += 1;
        self.total_size += size;
    }

    /// Check if the index is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }

    /// Get the number of assets.
    #[must_use]
    pub fn len(&self) -> usize {
        self.assets.len()
    }
}

impl<T> Default for AssetIndex<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for image asset index.
pub type ImageIndex = AssetIndex<ImageAsset>;

/// Type alias for font asset index.
pub type FontIndex = AssetIndex<FontAsset>;

/// Type alias for embed asset index.
pub type EmbedIndex = AssetIndex<EmbedAsset>;

/// An embedded file asset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbedAsset {
    /// Unique identifier for the embed.
    pub id: String,

    /// Path within the archive (e.g., "assets/embeds/data.csv").
    pub path: String,

    /// Content hash for verification.
    pub hash: DocumentId,

    /// File size in bytes.
    pub size: u64,

    /// MIME type of the embedded file.
    pub mime_type: String,

    /// Original filename (if known).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,

    /// Description of the embedded file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Whether the embed should be displayed inline.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub inline: bool,
}

impl EmbedAsset {
    /// Create a new embed asset.
    #[must_use]
    pub fn new(id: impl Into<String>, mime_type: impl Into<String>) -> Self {
        let id = id.into();
        let path = format!("assets/embeds/{id}");
        Self {
            id,
            path,
            hash: DocumentId::pending(),
            size: 0,
            mime_type: mime_type.into(),
            filename: None,
            description: None,
            inline: false,
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

    /// Set the original filename.
    #[must_use]
    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    /// Set the description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set whether the embed should be displayed inline.
    #[must_use]
    pub const fn with_inline(mut self, inline: bool) -> Self {
        self.inline = inline;
        self
    }

    /// Set a custom path.
    #[must_use]
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }
}

impl super::Asset for EmbedAsset {
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
        &self.mime_type
    }
}

/// Generic asset entry that can represent any asset type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AssetEntry {
    /// Image asset.
    Image(ImageAsset),
    /// Font asset.
    Font(FontAsset),
    /// Embedded file asset.
    Embed(EmbedAsset),
}

impl AssetEntry {
    /// Get the asset ID.
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::Image(a) => &a.id,
            Self::Font(a) => &a.id,
            Self::Embed(a) => &a.id,
        }
    }

    /// Get the asset path.
    #[must_use]
    pub fn path(&self) -> &str {
        match self {
            Self::Image(a) => &a.path,
            Self::Font(a) => &a.path,
            Self::Embed(a) => &a.path,
        }
    }

    /// Get the asset hash.
    #[must_use]
    pub fn hash(&self) -> &DocumentId {
        match self {
            Self::Image(a) => &a.hash,
            Self::Font(a) => &a.hash,
            Self::Embed(a) => &a.hash,
        }
    }

    /// Get the asset size.
    #[must_use]
    pub fn size(&self) -> u64 {
        match self {
            Self::Image(a) => a.size,
            Self::Font(a) => a.size,
            Self::Embed(a) => a.size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::ImageFormat;

    #[test]
    fn test_asset_index_new() {
        let index: ImageIndex = AssetIndex::new();
        assert!(index.is_empty());
        assert_eq!(index.count, 0);
        assert_eq!(index.total_size, 0);
    }

    #[test]
    fn test_asset_index_add() {
        let mut index: ImageIndex = AssetIndex::new();
        let image = ImageAsset::new("test", ImageFormat::Png).with_size(1024);
        index.add(image, 1024);

        assert_eq!(index.len(), 1);
        assert_eq!(index.count, 1);
        assert_eq!(index.total_size, 1024);
    }

    #[test]
    fn test_embed_asset_new() {
        let embed = EmbedAsset::new("data", "text/csv");
        assert_eq!(embed.id, "data");
        assert_eq!(embed.mime_type, "text/csv");
        assert_eq!(embed.path, "assets/embeds/data");
    }

    #[test]
    fn test_embed_asset_builder() {
        let embed = EmbedAsset::new("spreadsheet", "application/vnd.ms-excel")
            .with_filename("sales.xlsx")
            .with_description("Quarterly sales data")
            .with_size(65536)
            .with_inline(false);

        assert_eq!(embed.filename, Some("sales.xlsx".to_string()));
        assert_eq!(embed.description, Some("Quarterly sales data".to_string()));
        assert_eq!(embed.size, 65536);
        assert!(!embed.inline);
    }

    #[test]
    fn test_asset_entry_variants() {
        let image = AssetEntry::Image(ImageAsset::new("img", ImageFormat::Png));
        assert_eq!(image.id(), "img");

        let embed = AssetEntry::Embed(EmbedAsset::new("file", "text/plain"));
        assert_eq!(embed.id(), "file");
    }

    #[test]
    fn test_asset_index_serialization() {
        let mut index: ImageIndex = AssetIndex::new();
        let image = ImageAsset::new("test", ImageFormat::Png).with_size(1024);
        index.add(image, 1024);

        let json = serde_json::to_string_pretty(&index).unwrap();
        assert!(json.contains(r#""count": 1"#));
        assert!(json.contains(r#""totalSize": 1024"#));

        let deserialized: ImageIndex = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.count, 1);
        assert_eq!(deserialized.total_size, 1024);
    }
}
