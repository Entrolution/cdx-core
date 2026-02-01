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

    /// Check if an extension is declared in the manifest.
    ///
    /// Extension IDs use dot notation like "codex.semantic" or "codex.legal".
    /// This method checks if the given namespace (e.g., "semantic", "legal")
    /// matches any declared extension.
    #[must_use]
    pub fn has_extension(&self, namespace: &str) -> bool {
        // Check for exact match or codex.{namespace} format
        self.extensions.iter().any(|ext| {
            ext.id == namespace
                || ext.id == format!("codex.{namespace}")
                || ext.id.ends_with(&format!(".{namespace}"))
        })
    }

    /// Get a declared extension by namespace.
    ///
    /// Returns the extension declaration if found.
    #[must_use]
    pub fn get_extension(&self, namespace: &str) -> Option<&Extension> {
        self.extensions.iter().find(|ext| {
            ext.id == namespace
                || ext.id == format!("codex.{namespace}")
                || ext.id.ends_with(&format!(".{namespace}"))
        })
    }

    /// Get all declared extension IDs.
    #[must_use]
    pub fn declared_extension_ids(&self) -> Vec<&str> {
        self.extensions.iter().map(|e| e.id.as_str()).collect()
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
                requirement: "lineage information".to_string(),
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

    /// Merkle root hash of the content blocks.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "merkleRoot"
    )]
    pub merkle_root: Option<DocumentId>,

    /// Number of content blocks.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "blockCount"
    )]
    pub block_count: Option<usize>,
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Extension {
    /// Extension identifier (e.g., "codex.semantic", "codex.legal").
    pub id: String,

    /// Extension version.
    pub version: String,

    /// Whether extension is required for correct rendering.
    ///
    /// If `true`, readers that don't support this extension should fail.
    /// If `false`, readers may render fallback content or skip extension blocks.
    pub required: bool,
}

impl Extension {
    /// Create a new extension declaration.
    #[must_use]
    pub fn new(id: impl Into<String>, version: impl Into<String>, required: bool) -> Self {
        Self {
            id: id.into(),
            version: version.into(),
            required,
        }
    }

    /// Create a new required extension declaration.
    #[must_use]
    pub fn required(id: impl Into<String>, version: impl Into<String>) -> Self {
        Self::new(id, version, true)
    }

    /// Create a new optional extension declaration.
    #[must_use]
    pub fn optional(id: impl Into<String>, version: impl Into<String>) -> Self {
        Self::new(id, version, false)
    }

    /// Extract the namespace from the extension ID.
    ///
    /// For "codex.semantic", returns "semantic".
    /// For "semantic", returns "semantic".
    #[must_use]
    pub fn namespace(&self) -> &str {
        self.id.rsplit('.').next().unwrap_or(&self.id)
    }
}

/// Version history and document relationships.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lineage {
    /// Document ID of parent version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<DocumentId>,

    /// Up to 10 levels of ancestors for efficient chain verification.
    /// Ordered from nearest (parent's parent) to furthest ancestor.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ancestors: Vec<DocumentId>,

    /// Sequential version number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,

    /// Distance from the root document (0 for root, 1 for first child, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth: Option<u32>,

    /// Branch identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,

    /// Document IDs of documents merged into this version.
    #[serde(default, skip_serializing_if = "Vec::is_empty", rename = "mergedFrom")]
    pub merged_from: Vec<DocumentId>,

    /// Description of changes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl Lineage {
    /// Create a new root lineage (first version of a document).
    #[must_use]
    pub fn root() -> Self {
        Self {
            parent: None,
            ancestors: Vec::new(),
            version: Some(1),
            depth: Some(0),
            branch: None,
            merged_from: Vec::new(),
            note: None,
        }
    }

    /// Create lineage that derives from a parent document.
    ///
    /// This automatically computes the ancestor chain (up to 10 levels)
    /// and increments the depth.
    #[must_use]
    pub fn from_parent(parent_id: DocumentId, parent_lineage: Option<&Lineage>) -> Self {
        let (ancestors, depth, version) = if let Some(pl) = parent_lineage {
            // Build ancestor chain: parent's ancestors, prepended with parent's parent
            let mut new_ancestors = Vec::with_capacity(10);

            // Add parent's parent (if any) as first ancestor
            if let Some(ref grandparent) = pl.parent {
                new_ancestors.push(grandparent.clone());
            }

            // Add parent's ancestors (up to 9 more to keep total at 10)
            for ancestor in pl.ancestors.iter().take(9) {
                new_ancestors.push(ancestor.clone());
            }

            let new_depth = pl.depth.map_or(1, |d| d + 1);
            let new_version = pl.version.map_or(2, |v| v + 1);

            (new_ancestors, Some(new_depth), Some(new_version))
        } else {
            // Parent has no lineage, this becomes depth 1
            (Vec::new(), Some(1), Some(2))
        };

        Self {
            parent: Some(parent_id),
            ancestors,
            version,
            depth,
            branch: parent_lineage.and_then(|pl| pl.branch.clone()),
            merged_from: Vec::new(),
            note: None,
        }
    }

    /// Add a note describing the changes in this version.
    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Set the branch name.
    #[must_use]
    pub fn with_branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = Some(branch.into());
        self
    }

    /// Record that this version was created by merging another document.
    #[must_use]
    pub fn with_merge(mut self, merged_id: DocumentId) -> Self {
        self.merged_from.push(merged_id);
        self
    }
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
            merkle_root: None,
            block_count: None,
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
            merkle_root: None,
            block_count: None,
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
            merkle_root: None,
            block_count: None,
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
            merkle_root: None,
            block_count: None,
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
        manifest.lineage = Some(Lineage::root());

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
            merkle_root: None,
            block_count: None,
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
            merkle_root: None,
            block_count: None,
        };
        let metadata = Metadata {
            dublin_core: "metadata/dublin-core.json".to_string(),
            custom: None,
        };

        let manifest = Manifest::new(content, metadata);
        // Draft state should validate without precise layout
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_lineage_root() {
        let lineage = Lineage::root();
        assert!(lineage.parent.is_none());
        assert!(lineage.ancestors.is_empty());
        assert_eq!(lineage.version, Some(1));
        assert_eq!(lineage.depth, Some(0));
    }

    #[test]
    fn test_lineage_from_parent() {
        let parent_id = test_hash();
        let parent_lineage = Lineage::root();

        let child = Lineage::from_parent(parent_id.clone(), Some(&parent_lineage));

        assert_eq!(child.parent, Some(parent_id));
        assert!(child.ancestors.is_empty()); // Root has no parent, so child has no ancestors
        assert_eq!(child.version, Some(2));
        assert_eq!(child.depth, Some(1));
    }

    #[test]
    fn test_lineage_ancestor_chain() {
        // Create a chain: root -> v2 -> v3
        let root_id = test_hash();
        let root_lineage = Lineage::root();

        let v2_id: DocumentId =
            "sha256:1111111111111111111111111111111111111111111111111111111111111111"
                .parse()
                .unwrap();
        let v2_lineage = Lineage::from_parent(root_id.clone(), Some(&root_lineage));

        let _v3_id: DocumentId =
            "sha256:2222222222222222222222222222222222222222222222222222222222222222"
                .parse()
                .unwrap();
        let v3_lineage = Lineage::from_parent(v2_id.clone(), Some(&v2_lineage));

        // v3 should have v2 as parent and root_id in ancestors
        assert_eq!(v3_lineage.parent, Some(v2_id));
        assert_eq!(v3_lineage.ancestors.len(), 1);
        assert_eq!(v3_lineage.ancestors[0], root_id);
        assert_eq!(v3_lineage.depth, Some(2));
        assert_eq!(v3_lineage.version, Some(3));
    }

    // Extension tests

    #[test]
    fn test_extension_new() {
        let ext = Extension::new("codex.semantic", "0.1", true);
        assert_eq!(ext.id, "codex.semantic");
        assert_eq!(ext.version, "0.1");
        assert!(ext.required);
    }

    #[test]
    fn test_extension_required() {
        let ext = Extension::required("codex.legal", "0.1");
        assert!(ext.required);
    }

    #[test]
    fn test_extension_optional() {
        let ext = Extension::optional("codex.forms", "0.1");
        assert!(!ext.required);
    }

    #[test]
    fn test_extension_namespace() {
        assert_eq!(
            Extension::new("codex.semantic", "0.1", true).namespace(),
            "semantic"
        );
        assert_eq!(
            Extension::new("semantic", "0.1", true).namespace(),
            "semantic"
        );
        assert_eq!(
            Extension::new("org.example.custom", "0.1", true).namespace(),
            "custom"
        );
    }

    #[test]
    fn test_manifest_has_extension() {
        let content = ContentRef {
            path: "content/document.json".to_string(),
            hash: DocumentId::pending(),
            compression: None,
            merkle_root: None,
            block_count: None,
        };
        let metadata = Metadata {
            dublin_core: "metadata/dublin-core.json".to_string(),
            custom: None,
        };

        let mut manifest = Manifest::new(content, metadata);
        manifest
            .extensions
            .push(Extension::required("codex.semantic", "0.1"));
        manifest
            .extensions
            .push(Extension::optional("codex.legal", "0.1"));

        // Check by namespace
        assert!(manifest.has_extension("semantic"));
        assert!(manifest.has_extension("legal"));
        assert!(!manifest.has_extension("forms"));

        // Check by full ID
        assert!(manifest.has_extension("codex.semantic"));
        assert!(manifest.has_extension("codex.legal"));
    }

    #[test]
    fn test_manifest_get_extension() {
        let content = ContentRef {
            path: "content/document.json".to_string(),
            hash: DocumentId::pending(),
            compression: None,
            merkle_root: None,
            block_count: None,
        };
        let metadata = Metadata {
            dublin_core: "metadata/dublin-core.json".to_string(),
            custom: None,
        };

        let mut manifest = Manifest::new(content, metadata);
        manifest
            .extensions
            .push(Extension::required("codex.semantic", "0.1"));

        let ext = manifest.get_extension("semantic");
        assert!(ext.is_some());
        assert_eq!(ext.unwrap().id, "codex.semantic");
        assert!(ext.unwrap().required);

        assert!(manifest.get_extension("forms").is_none());
    }

    #[test]
    fn test_manifest_declared_extension_ids() {
        let content = ContentRef {
            path: "content/document.json".to_string(),
            hash: DocumentId::pending(),
            compression: None,
            merkle_root: None,
            block_count: None,
        };
        let metadata = Metadata {
            dublin_core: "metadata/dublin-core.json".to_string(),
            custom: None,
        };

        let mut manifest = Manifest::new(content, metadata);
        manifest
            .extensions
            .push(Extension::required("codex.semantic", "0.1"));
        manifest
            .extensions
            .push(Extension::optional("codex.forms", "0.1"));

        let ids = manifest.declared_extension_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"codex.semantic"));
        assert!(ids.contains(&"codex.forms"));
    }

    #[test]
    fn test_extension_serialization() {
        let ext = Extension::required("codex.semantic", "0.1");
        let json = serde_json::to_string(&ext).unwrap();
        assert!(json.contains("\"id\":\"codex.semantic\""));
        assert!(json.contains("\"version\":\"0.1\""));
        assert!(json.contains("\"required\":true"));
    }

    #[test]
    fn test_extension_deserialization() {
        let json = r#"{"id":"codex.legal","version":"0.1","required":false}"#;
        let ext: Extension = serde_json::from_str(json).unwrap();
        assert_eq!(ext.id, "codex.legal");
        assert_eq!(ext.version, "0.1");
        assert!(!ext.required);
    }
}
