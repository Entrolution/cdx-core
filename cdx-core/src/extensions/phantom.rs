//! Phantom extension for off-page annotation clusters.
//!
//! Phantoms are annotation clusters that exist conceptually outside the page flow,
//! attached to specific content anchors. They enable rich annotation workflows
//! while keeping the main content clean.
//!
//! # Overview
//!
//! - **Phantom clusters** attach to content via anchors
//! - **Phantoms** within clusters have positions, sizes, and content
//! - **Connections** link phantoms to each other within a cluster
//! - **Scopes** control visibility (shared, private, or role-based)
//!
//! # Example
//!
//! ```rust
//! use cdx_core::extensions::phantom::{PhantomClusters, PhantomCluster, Phantom, PhantomScope};
//! use cdx_core::anchor::ContentAnchor;
//!
//! let anchor = ContentAnchor::range("para-1", 10, 25);
//! let cluster = PhantomCluster::new("cluster-1", anchor, "Research Notes")
//!     .with_scope(PhantomScope::Shared);
//! ```

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::anchor::ContentAnchor;
use crate::content::Block;
use crate::extensions::Collaborator;
use crate::DocumentState;

/// Root structure for phantom clusters stored at `phantoms/clusters.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhantomClusters {
    /// Format version.
    pub version: String,

    /// All phantom clusters in the document.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clusters: Vec<PhantomCluster>,
}

impl PhantomClusters {
    /// Create a new empty phantom clusters file.
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: crate::SPEC_VERSION.to_string(),
            clusters: Vec::new(),
        }
    }

    /// Add a cluster.
    pub fn add_cluster(&mut self, cluster: PhantomCluster) {
        self.clusters.push(cluster);
    }

    /// Find a cluster by ID.
    #[must_use]
    pub fn find_cluster(&self, id: &str) -> Option<&PhantomCluster> {
        self.clusters.iter().find(|c| c.id == id)
    }

    /// Find a cluster by ID mutably.
    #[must_use]
    pub fn find_cluster_mut(&mut self, id: &str) -> Option<&mut PhantomCluster> {
        self.clusters.iter_mut().find(|c| c.id == id)
    }

    /// Get the number of clusters.
    #[must_use]
    pub fn len(&self) -> usize {
        self.clusters.len()
    }

    /// Check if there are no clusters.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.clusters.is_empty()
    }

    /// Validate all clusters.
    ///
    /// Returns a list of validation errors, if any.
    #[must_use]
    pub fn validate(&self, state: DocumentState) -> Vec<String> {
        let mut errors = Vec::new();

        for cluster in &self.clusters {
            errors.extend(cluster.validate_connections(state));
        }

        errors
    }
}

impl Default for PhantomClusters {
    fn default() -> Self {
        Self::new()
    }
}

/// A cluster of phantoms attached to a content anchor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhantomCluster {
    /// Unique identifier for this cluster.
    pub id: String,

    /// Content anchor where this cluster attaches.
    pub anchor: ContentAnchor,

    /// Display label for the cluster.
    pub label: String,

    /// Visibility scope for this cluster.
    #[serde(default)]
    pub scope: PhantomScope,

    /// Author who created this cluster.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<Collaborator>,

    /// When the cluster was created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime<Utc>>,

    /// Custom metadata for the cluster.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,

    /// Phantoms within this cluster.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub phantoms: Vec<Phantom>,
}

impl PhantomCluster {
    /// Create a new phantom cluster.
    #[must_use]
    pub fn new(id: impl Into<String>, anchor: ContentAnchor, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            anchor,
            label: label.into(),
            scope: PhantomScope::default(),
            author: None,
            created: Some(Utc::now()),
            metadata: HashMap::new(),
            phantoms: Vec::new(),
        }
    }

    /// Set the visibility scope.
    #[must_use]
    pub fn with_scope(mut self, scope: PhantomScope) -> Self {
        self.scope = scope;
        self
    }

    /// Set the author.
    #[must_use]
    pub fn with_author(mut self, author: Collaborator) -> Self {
        self.author = Some(author);
        self
    }

    /// Add metadata.
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Add a phantom to this cluster.
    pub fn add_phantom(&mut self, phantom: Phantom) {
        self.phantoms.push(phantom);
    }

    /// Add a phantom and return self for chaining.
    #[must_use]
    pub fn with_phantom(mut self, phantom: Phantom) -> Self {
        self.phantoms.push(phantom);
        self
    }

    /// Find a phantom by ID.
    #[must_use]
    pub fn find_phantom(&self, id: &str) -> Option<&Phantom> {
        self.phantoms.iter().find(|p| p.id == id)
    }

    /// Find a phantom by ID mutably.
    #[must_use]
    pub fn find_phantom_mut(&mut self, id: &str) -> Option<&mut Phantom> {
        self.phantoms.iter_mut().find(|p| p.id == id)
    }

    /// Validate connections within this cluster.
    ///
    /// Checks:
    /// - All connection targets exist within this cluster
    /// - No circular references that would cause infinite loops
    ///
    /// Returns errors in Frozen/Published states, warnings in Draft/Review.
    #[must_use]
    pub fn validate_connections(&self, state: DocumentState) -> Vec<String> {
        let mut errors = Vec::new();
        let phantom_ids: HashSet<_> = self.phantoms.iter().map(|p| p.id.as_str()).collect();

        for phantom in &self.phantoms {
            for conn in &phantom.connections {
                // Check target exists in this cluster
                if !phantom_ids.contains(conn.target.as_str()) {
                    let msg = format!(
                        "Phantom '{}' has connection to non-existent target '{}' in cluster '{}'",
                        phantom.id, conn.target, self.id
                    );
                    if state.is_immutable() {
                        errors.push(format!("ERROR: {msg}"));
                    } else {
                        errors.push(format!("WARNING: {msg}"));
                    }
                }

                // Check for self-reference
                if conn.target == phantom.id {
                    let msg = format!(
                        "Phantom '{}' has self-referential connection in cluster '{}'",
                        phantom.id, self.id
                    );
                    if state.is_immutable() {
                        errors.push(format!("ERROR: {msg}"));
                    } else {
                        errors.push(format!("WARNING: {msg}"));
                    }
                }
            }
        }

        // Check for cycles
        if let Some(cycle) = self.detect_cycle() {
            let msg = format!(
                "Circular connection detected in cluster '{}': {}",
                self.id,
                cycle.join(" -> ")
            );
            if state.is_immutable() {
                errors.push(format!("ERROR: {msg}"));
            } else {
                errors.push(format!("WARNING: {msg}"));
            }
        }

        errors
    }

    /// Detect cycles in the connection graph.
    /// Returns the cycle path if one exists.
    fn detect_cycle(&self) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for phantom in &self.phantoms {
            if !visited.contains(&phantom.id) {
                if let Some(cycle) =
                    self.detect_cycle_dfs(&phantom.id, &mut visited, &mut rec_stack, &mut path)
                {
                    return Some(cycle);
                }
            }
        }

        None
    }

    fn detect_cycle_dfs(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(phantom) = self.find_phantom(node) {
            for conn in &phantom.connections {
                if !visited.contains(&conn.target) {
                    if let Some(cycle) =
                        self.detect_cycle_dfs(&conn.target, visited, rec_stack, path)
                    {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(&conn.target) {
                    // Found cycle - return path from target to current
                    let mut cycle = Vec::new();
                    let mut found = false;
                    for p in path.iter() {
                        if p == &conn.target || found {
                            found = true;
                            cycle.push(p.clone());
                        }
                    }
                    cycle.push(conn.target.clone());
                    return Some(cycle);
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
        None
    }
}

/// An individual phantom within a cluster.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Phantom {
    /// Unique identifier for this phantom.
    pub id: String,

    /// Position within the cluster's coordinate space.
    pub position: PhantomPosition,

    /// Optional size of this phantom.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<PhantomSize>,

    /// Content of this phantom.
    pub content: PhantomContent,

    /// Connections to other phantoms in the same cluster.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub connections: Vec<PhantomConnection>,

    /// When this phantom was created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime<Utc>>,

    /// Author who created this phantom.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<Collaborator>,
}

impl Phantom {
    /// Create a new phantom.
    #[must_use]
    pub fn new(id: impl Into<String>, position: PhantomPosition, content: PhantomContent) -> Self {
        Self {
            id: id.into(),
            position,
            size: None,
            content,
            connections: Vec::new(),
            created: Some(Utc::now()),
            author: None,
        }
    }

    /// Set the size.
    #[must_use]
    pub fn with_size(mut self, size: PhantomSize) -> Self {
        self.size = Some(size);
        self
    }

    /// Set the author.
    #[must_use]
    pub fn with_author(mut self, author: Collaborator) -> Self {
        self.author = Some(author);
        self
    }

    /// Add a connection to another phantom.
    #[must_use]
    pub fn with_connection(mut self, connection: PhantomConnection) -> Self {
        self.connections.push(connection);
        self
    }

    /// Add a simple connection to another phantom by ID.
    #[must_use]
    pub fn connect_to(mut self, target_id: impl Into<String>) -> Self {
        self.connections.push(PhantomConnection::new(target_id));
        self
    }
}

/// Position of a phantom in the cluster's coordinate space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PhantomPosition {
    /// X coordinate (horizontal).
    pub x: f64,
    /// Y coordinate (vertical).
    pub y: f64,
}

impl PhantomPosition {
    /// Create a new position.
    #[must_use]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// Size of a phantom.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PhantomSize {
    /// Width.
    pub width: f64,
    /// Height.
    pub height: f64,
}

impl PhantomSize {
    /// Create a new size.
    #[must_use]
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }
}

/// Connection between phantoms in the same cluster.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhantomConnection {
    /// ID of the target phantom (must be in the same cluster).
    pub target: String,

    /// Visual style of the connection line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<ConnectionStyle>,

    /// Optional label for the connection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl PhantomConnection {
    /// Create a new connection to a target phantom.
    #[must_use]
    pub fn new(target: impl Into<String>) -> Self {
        Self {
            target: target.into(),
            style: None,
            label: None,
        }
    }

    /// Set the connection style.
    #[must_use]
    pub fn with_style(mut self, style: ConnectionStyle) -> Self {
        self.style = Some(style);
        self
    }

    /// Set a label for the connection.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// Visual style for phantom connections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionStyle {
    /// Simple line.
    #[default]
    Line,
    /// Line with arrowhead.
    Arrow,
    /// Dashed line.
    Dashed,
}

/// Visibility scope for phantom clusters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum PhantomScope {
    /// Visible to all readers.
    Shared,
    /// Visible only to the author.
    Private,
    /// Visible to users with the specified role.
    Role {
        /// The role that can see this cluster.
        role: String,
    },
}

impl Default for PhantomScope {
    fn default() -> Self {
        Self::Shared
    }
}

impl PhantomScope {
    /// Create a role-based scope.
    #[must_use]
    pub fn role(role: impl Into<String>) -> Self {
        Self::Role { role: role.into() }
    }
}

/// Content of a phantom, wrapping standard content blocks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhantomContent {
    /// Content blocks within this phantom.
    pub blocks: Vec<Block>,
}

impl PhantomContent {
    /// Create new phantom content from blocks.
    #[must_use]
    pub fn new(blocks: Vec<Block>) -> Self {
        Self { blocks }
    }

    /// Create phantom content with a single paragraph.
    #[must_use]
    pub fn paragraph(text: impl Into<String>) -> Self {
        use crate::content::Text;
        Self {
            blocks: vec![Block::paragraph(vec![Text::plain(text)])],
        }
    }

    /// Check if the content is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }
}

impl From<Vec<Block>> for PhantomContent {
    fn from(blocks: Vec<Block>) -> Self {
        Self::new(blocks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phantom_clusters_new() {
        let clusters = PhantomClusters::new();
        assert_eq!(clusters.version, "0.1");
        assert!(clusters.is_empty());
    }

    #[test]
    fn test_phantom_cluster_creation() {
        let anchor = ContentAnchor::block("para-1");
        let cluster = PhantomCluster::new("cluster-1", anchor, "Test Cluster");

        assert_eq!(cluster.id, "cluster-1");
        assert_eq!(cluster.label, "Test Cluster");
        assert!(matches!(cluster.scope, PhantomScope::Shared));
    }

    #[test]
    fn test_phantom_cluster_with_phantoms() {
        let anchor = ContentAnchor::range("para-1", 10, 25);
        let content = PhantomContent::paragraph("Note text");
        let phantom = Phantom::new("phantom-1", PhantomPosition::new(100.0, 50.0), content);

        let cluster =
            PhantomCluster::new("cluster-1", anchor, "Notes").with_phantom(phantom.clone());

        assert_eq!(cluster.phantoms.len(), 1);
        assert!(cluster.find_phantom("phantom-1").is_some());
    }

    #[test]
    fn test_phantom_connections() {
        let anchor = ContentAnchor::block("para-1");

        let phantom1 = Phantom::new(
            "p1",
            PhantomPosition::new(0.0, 0.0),
            PhantomContent::paragraph("First"),
        )
        .connect_to("p2");

        let phantom2 = Phantom::new(
            "p2",
            PhantomPosition::new(100.0, 0.0),
            PhantomContent::paragraph("Second"),
        );

        let cluster = PhantomCluster::new("cluster-1", anchor, "Test")
            .with_phantom(phantom1)
            .with_phantom(phantom2);

        assert_eq!(cluster.phantoms[0].connections.len(), 1);
        assert_eq!(cluster.phantoms[0].connections[0].target, "p2");
    }

    #[test]
    fn test_connection_validation_valid() {
        let anchor = ContentAnchor::block("para-1");

        let phantom1 = Phantom::new(
            "p1",
            PhantomPosition::new(0.0, 0.0),
            PhantomContent::paragraph("First"),
        )
        .connect_to("p2");

        let phantom2 = Phantom::new(
            "p2",
            PhantomPosition::new(100.0, 0.0),
            PhantomContent::paragraph("Second"),
        );

        let cluster = PhantomCluster::new("cluster-1", anchor, "Test")
            .with_phantom(phantom1)
            .with_phantom(phantom2);

        let errors = cluster.validate_connections(DocumentState::Draft);
        assert!(errors.is_empty(), "Valid connections: {:?}", errors);
    }

    #[test]
    fn test_connection_validation_missing_target() {
        let anchor = ContentAnchor::block("para-1");

        let phantom1 = Phantom::new(
            "p1",
            PhantomPosition::new(0.0, 0.0),
            PhantomContent::paragraph("First"),
        )
        .connect_to("nonexistent");

        let cluster = PhantomCluster::new("cluster-1", anchor, "Test").with_phantom(phantom1);

        let errors = cluster.validate_connections(DocumentState::Draft);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].starts_with("WARNING:"));

        let errors = cluster.validate_connections(DocumentState::Frozen);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].starts_with("ERROR:"));
    }

    #[test]
    fn test_connection_validation_self_reference() {
        let anchor = ContentAnchor::block("para-1");

        let phantom1 = Phantom::new(
            "p1",
            PhantomPosition::new(0.0, 0.0),
            PhantomContent::paragraph("First"),
        )
        .connect_to("p1"); // Self-reference

        let cluster = PhantomCluster::new("cluster-1", anchor, "Test").with_phantom(phantom1);

        let errors = cluster.validate_connections(DocumentState::Frozen);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("self-referential")));
    }

    #[test]
    fn test_connection_validation_cycle() {
        let anchor = ContentAnchor::block("para-1");

        let phantom1 = Phantom::new(
            "p1",
            PhantomPosition::new(0.0, 0.0),
            PhantomContent::paragraph("First"),
        )
        .connect_to("p2");

        let phantom2 = Phantom::new(
            "p2",
            PhantomPosition::new(100.0, 0.0),
            PhantomContent::paragraph("Second"),
        )
        .connect_to("p3");

        let phantom3 = Phantom::new(
            "p3",
            PhantomPosition::new(200.0, 0.0),
            PhantomContent::paragraph("Third"),
        )
        .connect_to("p1"); // Creates cycle: p1 -> p2 -> p3 -> p1

        let cluster = PhantomCluster::new("cluster-1", anchor, "Test")
            .with_phantom(phantom1)
            .with_phantom(phantom2)
            .with_phantom(phantom3);

        let errors = cluster.validate_connections(DocumentState::Frozen);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("Circular")));
    }

    #[test]
    fn test_phantom_scope_variants() {
        let shared = PhantomScope::Shared;
        let json = serde_json::to_string(&shared).unwrap();
        assert!(json.contains("\"type\":\"shared\""));

        let private = PhantomScope::Private;
        let json = serde_json::to_string(&private).unwrap();
        assert!(json.contains("\"type\":\"private\""));

        let role = PhantomScope::role("editor");
        let json = serde_json::to_string(&role).unwrap();
        assert!(json.contains("\"type\":\"role\""));
        assert!(json.contains("\"role\":\"editor\""));
    }

    #[test]
    fn test_phantom_clusters_serialization() {
        let anchor = ContentAnchor::block("para-1");
        let content = PhantomContent::paragraph("Note");
        let phantom = Phantom::new("p1", PhantomPosition::new(50.0, 50.0), content)
            .with_size(PhantomSize::new(200.0, 100.0));

        let cluster = PhantomCluster::new("cluster-1", anchor, "Notes")
            .with_scope(PhantomScope::Private)
            .with_phantom(phantom);

        let mut clusters = PhantomClusters::new();
        clusters.add_cluster(cluster);

        let json = serde_json::to_string_pretty(&clusters).unwrap();
        assert!(json.contains("\"version\": \"0.1\""));
        assert!(json.contains("\"clusters\":"));
        assert!(json.contains("\"phantoms\":"));

        // Roundtrip
        let parsed: PhantomClusters = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.clusters.len(), 1);
        assert_eq!(parsed.clusters[0].phantoms.len(), 1);
    }

    #[test]
    fn test_connection_style() {
        let conn = PhantomConnection::new("target")
            .with_style(ConnectionStyle::Arrow)
            .with_label("relates to");

        let json = serde_json::to_string(&conn).unwrap();
        assert!(json.contains("\"target\":\"target\""));
        assert!(json.contains("\"style\":\"arrow\""));
        assert!(json.contains("\"label\":\"relates to\""));
    }
}
