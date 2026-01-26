//! Asset management for embedded images, fonts, and files.
//!
//! This module provides types and utilities for managing assets embedded
//! within Codex documents.
//!
//! # Asset Types
//!
//! - **Images**: AVIF, WebP, PNG, JPEG, SVG formats with dimensions and alt text
//! - **Fonts**: WOFF2, WOFF, TTF, OTF formats with font family metadata
//! - **Embeds**: Arbitrary files with MIME type and description
//!
//! # Asset Index
//!
//! Each asset category has an index file (e.g., `assets/images/index.json`)
//! that lists all assets with their metadata and hashes for verification.
//!
//! # Example
//!
//! ```rust,ignore
//! use cdx_core::asset::{ImageAsset, ImageFormat};
//!
//! let image = ImageAsset::new("logo", ImageFormat::Png)
//!     .with_dimensions(200, 100)
//!     .with_alt("Company logo");
//! ```

mod font;
mod image;
mod index;

pub use font::{FontAsset, FontFormat, FontStyle, FontWeight};
pub use image::{ImageAsset, ImageFormat, ImageVariant};
pub use index::{
    AssetAlias, AssetEntry, AssetIndex, EmbedAsset, EmbedIndex, FontIndex, ImageIndex,
};

use crate::{DocumentId, Result};

/// Common trait for all asset types.
pub trait Asset {
    /// Get the asset's unique identifier.
    fn id(&self) -> &str;

    /// Get the path to the asset file within the archive.
    fn path(&self) -> &str;

    /// Get the content hash of the asset.
    fn hash(&self) -> &DocumentId;

    /// Get the size in bytes.
    fn size(&self) -> u64;

    /// Get the MIME type.
    fn mime_type(&self) -> &str;
}

/// Verify an asset's integrity by checking its hash.
///
/// # Errors
///
/// Returns an error if the hash does not match the expected value.
pub fn verify_asset_hash(
    path: &str,
    data: &[u8],
    expected: &DocumentId,
    algorithm: crate::HashAlgorithm,
) -> Result<()> {
    let actual = crate::Hasher::hash(algorithm, data);
    if actual.hex_digest() != expected.hex_digest() {
        return Err(crate::Error::HashMismatch {
            path: path.to_string(),
            expected: expected.hex_digest(),
            actual: actual.hex_digest(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HashAlgorithm;

    #[test]
    fn test_verify_asset_hash_valid() {
        let data = b"test asset data";
        let hash = crate::Hasher::hash(HashAlgorithm::Sha256, data);
        assert!(verify_asset_hash("test.png", data, &hash, HashAlgorithm::Sha256).is_ok());
    }

    #[test]
    fn test_verify_asset_hash_invalid() {
        let data = b"test asset data";
        let wrong_hash = crate::Hasher::hash(HashAlgorithm::Sha256, b"different data");
        assert!(verify_asset_hash("test.png", data, &wrong_hash, HashAlgorithm::Sha256).is_err());
    }
}
