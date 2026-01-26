//! Manifest structure and types.
//!
//! The manifest (`manifest.json`) is the root metadata structure of a Codex document.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{DocumentId, DocumentState, HashAlgorithm};

/// Document manifest - the root metadata structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Specification version (e.g., "0.1").
    pub codex: String,

    /// Content-addressable document identifier.
    pub id: DocumentId,

    /// Document lifecycle state.
    pub state: DocumentState,

    /// Creation timestamp.
    pub created: DateTime<Utc>,

    /// Last modification timestamp.
    pub modified: DateTime<Utc>,

    /// Content layer reference.
    pub content: ContentRef,

    /// Metadata references.
    pub metadata: Metadata,

    /// Hash algorithm used (defaults to SHA-256).
    #[serde(
        rename = "hashAlgorithm",
        default,
        skip_serializing_if = "is_default_algorithm"
    )]
    pub hash_algorithm: HashAlgorithm,

    /// Presentation layer references.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub presentation: Vec<PresentationRef>,

    /// Asset manifest.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assets: Option<AssetManifest>,

    /// Security layer reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security: Option<SecurityRef>,

    /// Active extensions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extensions: Vec<Extension>,

    /// Version history and parent reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lineage: Option<Lineage>,
}

#[allow(clippy::trivially_copy_pass_by_ref)] // Required by serde skip_serializing_if
fn is_default_algorithm(alg: &HashAlgorithm) -> bool {
    *alg == HashAlgorithm::Sha256
}

impl Manifest {
    /// Create a new manifest with required fields.
    #[must_use]
    pub fn new(content: ContentRef, metadata: Metadata) -> Self {
        let now = Utc::now();
        Self {
            codex: crate::SPEC_VERSION.to_string(),
            id: DocumentId::pending(),
            state: DocumentState::Draft,
            created: now,
            modified: now,
            content,
            metadata,
            hash_algorithm: HashAlgorithm::default(),
            presentation: Vec::new(),
            assets: None,
            security: None,
            extensions: Vec::new(),
            lineage: None,
        }
    }

    /// Check if the manifest is valid.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The Codex version is unsupported
    /// - State requirements are not met (e.g., frozen documents without signatures)
    pub fn validate(&self) -> crate::Result<()> {
        // Check version
        if !self.codex.starts_with("0.") {
            return Err(crate::Error::UnsupportedVersion {
                version: self.codex.clone(),
            });
        }

        // Check state requirements
        if self.state.requires_signature() && self.security.is_none() {
            return Err(crate::Error::StateRequirementNotMet {
                state: self.state,
                requirement: "security signatures".to_string(),
            });
        }

        if self.state.requires_lineage() && self.lineage.is_none() {
            return Err(crate::Error::StateRequirementNotMet {
                state: self.state,
                requirement: "lineage with parent reference".to_string(),
            });
        }

        if self.state.requires_computed_id() && self.id.is_pending() {
            return Err(crate::Error::StateRequirementNotMet {
                state: self.state,
                requirement: "computed document ID".to_string(),
            });
        }

        // Check precise layout requirement for frozen/published states
        if self.state.requires_precise_layout() && !self.has_precise_layout() {
            return Err(crate::Error::StateRequirementNotMet {
                state: self.state,
                requirement: "at least one precise layout".to_string(),
            });
        }

        Ok(())
    }

    /// Check if the manifest contains a precise layout reference.
    #[must_use]
    pub fn has_precise_layout(&self) -> bool {
        self.presentation
            .iter()
            .any(|p| p.presentation_type == "precise")
    }

    /// Get all precise layout references.
    #[must_use]
    pub fn precise_layouts(&self) -> Vec<&PresentationRef> {
        self.presentation
            .iter()
            .filter(|p| p.presentation_type == "precise")
            .collect()
    }
}

/// Reference to a file within the archive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRef {
    /// Relative path within archive.
    pub path: String,

    /// Hash of file contents.
    pub hash: DocumentId,

    /// Compression method used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compression: Option<String>,
}

/// Reference to the content layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRef {
    /// Relative path within archive.
    pub path: String,

    /// Hash of file contents.
    pub hash: DocumentId,

    /// Compression method used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compression: Option<String>,
}

/// Reference to a presentation layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentationRef {
    /// Presentation type identifier.
    #[serde(rename = "type")]
    pub presentation_type: String,

    /// Relative path within archive.
    pub path: String,

    /// Hash of file contents.
    pub hash: DocumentId,

    /// Whether this is the default presentation.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub default: bool,
}

/// Metadata references.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// Path to Dublin Core metadata.
    #[serde(rename = "dublinCore")]
    pub dublin_core: String,

    /// Path to custom metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom: Option<String>,
}

/// Asset manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetManifest {
    /// Image assets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub images: Option<AssetCategory>,

    /// Font assets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fonts: Option<AssetCategory>,

    /// Embedded file assets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embeds: Option<AssetCategory>,
}

/// Asset category summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetCategory {
    /// Number of assets.
    pub count: u32,

    /// Total size in bytes.
    #[serde(rename = "totalSize")]
    pub total_size: u64,

    /// Path to asset index file.
    pub index: String,
}

/// Security layer reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRef {
    /// Path to signatures file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signatures: Option<String>,

    /// Path to encryption metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encryption: Option<String>,
}

/// Extension declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extension {
    /// Extension identifier.
    pub id: String,

    /// Extension version.
    pub version: String,

    /// Whether extension is required for correct rendering.
    pub required: bool,
}

/// Version history and document relationships.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lineage {
    /// Document ID of parent version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<DocumentId>,

    /// Sequential version number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,

    /// Branch identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,

    /// Description of changes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_creation() {
        let content = ContentRef {
            path: "content/document.json".to_string(),
            hash: DocumentId::pending(),
            compression: None,
        };
        let metadata = Metadata {
            dublin_core: "metadata/dublin-core.json".to_string(),
            custom: None,
        };

        let manifest = Manifest::new(content, metadata);
        assert_eq!(manifest.codex, "0.1");
        assert_eq!(manifest.state, DocumentState::Draft);
        assert!(manifest.id.is_pending());
    }

    #[test]
    fn test_manifest_validation() {
        let content = ContentRef {
            path: "content/document.json".to_string(),
            hash: DocumentId::pending(),
            compression: None,
        };
        let metadata = Metadata {
            dublin_core: "metadata/dublin-core.json".to_string(),
            custom: None,
        };

        let manifest = Manifest::new(content, metadata);
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_manifest_serialization() {
        let content = ContentRef {
            path: "content/document.json".to_string(),
            hash: DocumentId::pending(),
            compression: None,
        };
        let metadata = Metadata {
            dublin_core: "metadata/dublin-core.json".to_string(),
            custom: None,
        };

        let manifest = Manifest::new(content, metadata);
        let json = serde_json::to_string_pretty(&manifest).unwrap();
        assert!(json.contains("\"codex\": \"0.1\""));
        assert!(json.contains("\"state\": \"draft\""));
    }

    fn test_hash() -> DocumentId {
        // Valid SHA256 hash (64 hex chars = 32 bytes)
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
            .parse()
            .unwrap()
    }

    #[test]
    fn test_frozen_requires_precise_layout() {
        let content = ContentRef {
            path: "content/document.json".to_string(),
            hash: test_hash(),
            compression: None,
        };
        let metadata = Metadata {
            dublin_core: "metadata/dublin-core.json".to_string(),
            custom: None,
        };

        let mut manifest = Manifest::new(content, metadata);
        manifest.id = test_hash();
        manifest.state = DocumentState::Frozen;
        manifest.security = Some(SecurityRef {
            signatures: Some("security/signatures.json".to_string()),
            encryption: None,
        });
        manifest.lineage = Some(Lineage {
            parent: None,
            version: Some(1),
            branch: None,
            note: None,
        });

        // Without precise layout, validation should fail
        let result = manifest.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            crate::Error::StateRequirementNotMet { .. }
        ));

        // Add precise layout reference
        manifest.presentation.push(PresentationRef {
            presentation_type: "precise".to_string(),
            path: "presentation/layouts/letter.json".to_string(),
            hash: test_hash(),
            default: false,
        });

        // Now validation should pass
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_has_precise_layout() {
        let content = ContentRef {
            path: "content/document.json".to_string(),
            hash: DocumentId::pending(),
            compression: None,
        };
        let metadata = Metadata {
            dublin_core: "metadata/dublin-core.json".to_string(),
            custom: None,
        };

        let mut manifest = Manifest::new(content, metadata);
        assert!(!manifest.has_precise_layout());

        // Add a reactive presentation
        manifest.presentation.push(PresentationRef {
            presentation_type: "paginated".to_string(),
            path: "presentation/paginated.json".to_string(),
            hash: test_hash(),
            default: true,
        });
        assert!(!manifest.has_precise_layout());

        // Add a precise layout
        manifest.presentation.push(PresentationRef {
            presentation_type: "precise".to_string(),
            path: "presentation/layouts/letter.json".to_string(),
            hash: test_hash(),
            default: false,
        });
        assert!(manifest.has_precise_layout());
    }

    #[test]
    fn test_draft_does_not_require_precise_layout() {
        let content = ContentRef {
            path: "content/document.json".to_string(),
            hash: DocumentId::pending(),
            compression: None,
        };
        let metadata = Metadata {
            dublin_core: "metadata/dublin-core.json".to_string(),
            custom: None,
        };

        let manifest = Manifest::new(content, metadata);
        // Draft state should validate without precise layout
        assert!(manifest.validate().is_ok());
    }
}
