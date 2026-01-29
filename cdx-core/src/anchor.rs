//! Unified anchor system for content references.
//!
//! Anchors provide a way to reference specific locations within document content,
//! supporting block-level, point, and range anchors.
//!
//! # Anchor Types
//!
//! - **Block-level**: References an entire block (`#blockId`)
//! - **Point**: References a specific character offset (`#blockId/15`)
//! - **Range**: References a character range (`#blockId/10-25`)
//!
//! # Example
//!
//! ```rust
//! use cdx_core::anchor::{ContentAnchor, ContentAnchorUri};
//!
//! // Block-level anchor
//! let anchor = ContentAnchor::block("para-1");
//!
//! // Point anchor at offset 15
//! let point_anchor = ContentAnchor::point("para-1", 15);
//!
//! // Range anchor from offset 10 to 25
//! let range_anchor = ContentAnchor::range("para-1", 10, 25);
//!
//! // Parse from URI format
//! let uri: ContentAnchorUri = "#para-1/10-25".parse().unwrap();
//! let anchor = ContentAnchor::from(uri);
//! ```

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::DocumentId;

/// A content anchor referencing a location within the document.
///
/// Anchors can reference:
/// - An entire block (block-level anchor)
/// - A specific character position within a block (point anchor)
/// - A character range within a block (range anchor)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentAnchor {
    /// The ID of the referenced block.
    pub block_id: String,

    /// Character offset for point anchors.
    /// When present without `end`, this is a point anchor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,

    /// Start of range for range anchors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<usize>,

    /// End of range for range anchors (exclusive).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<usize>,

    /// Optional content hash for stale detection.
    /// When set, can be used to detect if the referenced content has changed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<DocumentId>,
}

impl ContentAnchor {
    /// Create a block-level anchor referencing an entire block.
    #[must_use]
    pub fn block(block_id: impl Into<String>) -> Self {
        Self {
            block_id: block_id.into(),
            offset: None,
            start: None,
            end: None,
            content_hash: None,
        }
    }

    /// Create a point anchor at a specific character offset.
    #[must_use]
    pub fn point(block_id: impl Into<String>, offset: usize) -> Self {
        Self {
            block_id: block_id.into(),
            offset: Some(offset),
            start: None,
            end: None,
            content_hash: None,
        }
    }

    /// Create a range anchor spanning a character range.
    #[must_use]
    pub fn range(block_id: impl Into<String>, start: usize, end: usize) -> Self {
        Self {
            block_id: block_id.into(),
            offset: None,
            start: Some(start),
            end: Some(end),
            content_hash: None,
        }
    }

    /// Set the content hash for stale detection.
    #[must_use]
    pub fn with_content_hash(mut self, hash: DocumentId) -> Self {
        self.content_hash = Some(hash);
        self
    }

    /// Check if this is a block-level anchor (no point or range).
    #[must_use]
    pub fn is_block_anchor(&self) -> bool {
        self.offset.is_none() && self.start.is_none() && self.end.is_none()
    }

    /// Check if this is a point anchor.
    #[must_use]
    pub fn is_point_anchor(&self) -> bool {
        self.offset.is_some()
    }

    /// Check if this is a range anchor.
    #[must_use]
    pub fn is_range_anchor(&self) -> bool {
        self.start.is_some() && self.end.is_some()
    }

    /// Convert to URI format.
    #[must_use]
    pub fn to_uri(&self) -> ContentAnchorUri {
        ContentAnchorUri::from(self.clone())
    }
}

impl From<ContentAnchorUri> for ContentAnchor {
    fn from(uri: ContentAnchorUri) -> Self {
        Self {
            block_id: uri.block_id,
            offset: uri.offset,
            start: uri.start,
            end: uri.end,
            content_hash: None,
        }
    }
}

/// URI representation of a content anchor.
///
/// Format: `#blockId` | `#blockId/offset` | `#blockId/start-end`
///
/// # Examples
///
/// - `#para-1` - Block-level anchor
/// - `#para-1/15` - Point anchor at offset 15
/// - `#para-1/10-25` - Range anchor from 10 to 25
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentAnchorUri {
    /// The ID of the referenced block.
    pub block_id: String,

    /// Character offset for point anchors.
    pub offset: Option<usize>,

    /// Start of range for range anchors.
    pub start: Option<usize>,

    /// End of range for range anchors.
    pub end: Option<usize>,
}

impl ContentAnchorUri {
    /// Create a new block-level anchor URI.
    #[must_use]
    pub fn block(block_id: impl Into<String>) -> Self {
        Self {
            block_id: block_id.into(),
            offset: None,
            start: None,
            end: None,
        }
    }

    /// Create a new point anchor URI.
    #[must_use]
    pub fn point(block_id: impl Into<String>, offset: usize) -> Self {
        Self {
            block_id: block_id.into(),
            offset: Some(offset),
            start: None,
            end: None,
        }
    }

    /// Create a new range anchor URI.
    #[must_use]
    pub fn range(block_id: impl Into<String>, start: usize, end: usize) -> Self {
        Self {
            block_id: block_id.into(),
            offset: None,
            start: Some(start),
            end: Some(end),
        }
    }
}

impl FromStr for ContentAnchorUri {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Must start with #
        let s = s
            .strip_prefix('#')
            .ok_or_else(|| crate::Error::InvalidManifest {
                reason: format!("Anchor URI must start with '#': {s}"),
            })?;

        // Check if there's a position specifier
        if let Some((block_id, position)) = s.split_once('/') {
            if block_id.is_empty() {
                return Err(crate::Error::InvalidManifest {
                    reason: "Anchor URI block ID cannot be empty".to_string(),
                });
            }

            // Check if it's a range (contains '-')
            if let Some((start_str, end_str)) = position.split_once('-') {
                let start =
                    start_str
                        .parse::<usize>()
                        .map_err(|_| crate::Error::InvalidManifest {
                            reason: format!("Invalid range start in anchor URI: {start_str}"),
                        })?;
                let end = end_str
                    .parse::<usize>()
                    .map_err(|_| crate::Error::InvalidManifest {
                        reason: format!("Invalid range end in anchor URI: {end_str}"),
                    })?;

                Ok(Self::range(block_id, start, end))
            } else {
                // Point anchor
                let offset =
                    position
                        .parse::<usize>()
                        .map_err(|_| crate::Error::InvalidManifest {
                            reason: format!("Invalid offset in anchor URI: {position}"),
                        })?;

                Ok(Self::point(block_id, offset))
            }
        } else {
            // Block-level anchor
            if s.is_empty() {
                return Err(crate::Error::InvalidManifest {
                    reason: "Anchor URI block ID cannot be empty".to_string(),
                });
            }
            Ok(Self::block(s))
        }
    }
}

impl fmt::Display for ContentAnchorUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.block_id)?;

        if let Some(offset) = self.offset {
            write!(f, "/{offset}")?;
        } else if let (Some(start), Some(end)) = (self.start, self.end) {
            write!(f, "/{start}-{end}")?;
        }

        Ok(())
    }
}

impl From<ContentAnchor> for ContentAnchorUri {
    fn from(anchor: ContentAnchor) -> Self {
        Self {
            block_id: anchor.block_id,
            offset: anchor.offset,
            start: anchor.start,
            end: anchor.end,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_anchor() {
        let anchor = ContentAnchor::block("para-1");
        assert_eq!(anchor.block_id, "para-1");
        assert!(anchor.is_block_anchor());
        assert!(!anchor.is_point_anchor());
        assert!(!anchor.is_range_anchor());
    }

    #[test]
    fn test_point_anchor() {
        let anchor = ContentAnchor::point("para-1", 15);
        assert_eq!(anchor.block_id, "para-1");
        assert_eq!(anchor.offset, Some(15));
        assert!(!anchor.is_block_anchor());
        assert!(anchor.is_point_anchor());
        assert!(!anchor.is_range_anchor());
    }

    #[test]
    fn test_range_anchor() {
        let anchor = ContentAnchor::range("para-1", 10, 25);
        assert_eq!(anchor.block_id, "para-1");
        assert_eq!(anchor.start, Some(10));
        assert_eq!(anchor.end, Some(25));
        assert!(!anchor.is_block_anchor());
        assert!(!anchor.is_point_anchor());
        assert!(anchor.is_range_anchor());
    }

    #[test]
    fn test_anchor_uri_parse_block() {
        let uri: ContentAnchorUri = "#blockId".parse().unwrap();
        assert_eq!(uri.block_id, "blockId");
        assert!(uri.offset.is_none());
        assert!(uri.start.is_none());
        assert!(uri.end.is_none());
    }

    #[test]
    fn test_anchor_uri_parse_point() {
        let uri: ContentAnchorUri = "#blockId/15".parse().unwrap();
        assert_eq!(uri.block_id, "blockId");
        assert_eq!(uri.offset, Some(15));
        assert!(uri.start.is_none());
        assert!(uri.end.is_none());
    }

    #[test]
    fn test_anchor_uri_parse_range() {
        let uri: ContentAnchorUri = "#blockId/10-25".parse().unwrap();
        assert_eq!(uri.block_id, "blockId");
        assert!(uri.offset.is_none());
        assert_eq!(uri.start, Some(10));
        assert_eq!(uri.end, Some(25));
    }

    #[test]
    fn test_anchor_uri_display() {
        let block_uri = ContentAnchorUri::block("para-1");
        assert_eq!(block_uri.to_string(), "#para-1");

        let point_uri = ContentAnchorUri::point("para-1", 15);
        assert_eq!(point_uri.to_string(), "#para-1/15");

        let range_uri = ContentAnchorUri::range("para-1", 10, 25);
        assert_eq!(range_uri.to_string(), "#para-1/10-25");
    }

    #[test]
    fn test_anchor_uri_roundtrip() {
        let cases = vec![
            "#block-1",
            "#para-1/15",
            "#heading-2/10-25",
            "#complex-id-123/0-100",
        ];

        for case in cases {
            let uri: ContentAnchorUri = case.parse().unwrap();
            assert_eq!(uri.to_string(), case);
        }
    }

    #[test]
    fn test_anchor_to_uri_conversion() {
        let anchor = ContentAnchor::range("para-1", 10, 25);
        let uri = anchor.to_uri();
        assert_eq!(uri.to_string(), "#para-1/10-25");

        // And back
        let anchor2 = ContentAnchor::from(uri);
        assert_eq!(anchor2.block_id, "para-1");
        assert_eq!(anchor2.start, Some(10));
        assert_eq!(anchor2.end, Some(25));
    }

    #[test]
    fn test_anchor_uri_parse_errors() {
        // Missing #
        assert!("blockId".parse::<ContentAnchorUri>().is_err());

        // Empty block ID
        assert!("#".parse::<ContentAnchorUri>().is_err());
        assert!("#/15".parse::<ContentAnchorUri>().is_err());

        // Invalid offset
        assert!("#blockId/abc".parse::<ContentAnchorUri>().is_err());

        // Invalid range
        assert!("#blockId/abc-25".parse::<ContentAnchorUri>().is_err());
        assert!("#blockId/10-def".parse::<ContentAnchorUri>().is_err());
    }

    #[test]
    fn test_anchor_serialization() {
        let anchor = ContentAnchor::range("para-1", 10, 25);
        let json = serde_json::to_string(&anchor).unwrap();
        assert!(json.contains("\"blockId\":\"para-1\""));
        assert!(json.contains("\"start\":10"));
        assert!(json.contains("\"end\":25"));

        let parsed: ContentAnchor = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, anchor);
    }

    #[test]
    fn test_anchor_with_content_hash() {
        let hash = crate::Hasher::hash(crate::HashAlgorithm::Sha256, b"test content");
        let anchor = ContentAnchor::block("para-1").with_content_hash(hash.clone());
        assert_eq!(anchor.content_hash, Some(hash));
    }
}
