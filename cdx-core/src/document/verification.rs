use crate::content::{Block, Text};
use crate::{DocumentId, Hasher, Result};

use super::Document;

impl Document {
    /// Compute the document ID from content.
    ///
    /// The document ID is computed by hashing the canonicalized semantic content layer.
    /// This covers only the content blocks and their structure, not presentation/layout
    /// information. Presentation layers have their own hashes in the manifest.
    ///
    /// # Errors
    ///
    /// Returns an error if canonicalization fails.
    pub fn compute_id(&self) -> Result<DocumentId> {
        // Serialize content to canonical JSON
        let content_json = serde_json::to_vec(&self.content)?;
        let canonical =
            json_canon::to_string(&serde_json::from_slice::<serde_json::Value>(&content_json)?)?;

        Ok(Hasher::hash(
            self.manifest.hash_algorithm,
            canonical.as_bytes(),
        ))
    }

    /// Verify the document integrity.
    ///
    /// This checks:
    /// - Content hash matches manifest
    /// - Document ID is valid (if not pending)
    ///
    /// # Errors
    ///
    /// Returns an error if verification fails.
    pub fn verify(&self) -> Result<VerificationReport> {
        let mut report = VerificationReport {
            content_valid: true,
            id_valid: true,
            errors: Vec::new(),
        };

        // Verify content hash
        // Note: must use to_vec_pretty to match what write_to uses
        if !self.manifest.content.hash.is_pending() {
            let content_json = serde_json::to_vec_pretty(&self.content)?;
            let actual_hash = Hasher::hash(self.manifest.content.hash.algorithm(), &content_json);

            if actual_hash != self.manifest.content.hash {
                report.content_valid = false;
                report.errors.push(format!(
                    "Content hash mismatch: expected {}, got {}",
                    self.manifest.content.hash, actual_hash
                ));
            }
        }

        // Verify document ID
        if !self.manifest.id.is_pending() {
            let computed_id = self.compute_id()?;
            if computed_id != self.manifest.id {
                report.id_valid = false;
                report.errors.push(format!(
                    "Document ID mismatch: expected {}, got {}",
                    self.manifest.id, computed_id
                ));
            }
        }

        Ok(report)
    }

    /// Validate extension declarations.
    ///
    /// This checks that all extension namespaces used in the document's content
    /// (blocks and marks) are declared in the manifest's extensions list.
    ///
    /// # Returns
    ///
    /// An `ExtensionValidationReport` containing:
    /// - List of used extension namespaces
    /// - List of declared extension namespaces
    /// - List of undeclared (used but not declared) namespaces
    /// - Warnings for any issues found
    #[must_use]
    pub fn validate_extensions(&self) -> ExtensionValidationReport {
        // Collect declared namespaces
        let declared_namespaces: Vec<String> = self
            .manifest
            .extensions
            .iter()
            .map(|e| e.namespace().to_string())
            .collect();

        // Collect used namespaces from content
        let mut used = std::collections::HashSet::new();
        Self::collect_extension_namespaces(&self.content.blocks, &mut used);

        let mut used_namespaces: Vec<String> = used.iter().cloned().collect();
        used_namespaces.sort();

        // Find undeclared namespaces
        let mut undeclared = Vec::new();
        let mut warnings = Vec::new();
        for namespace in &used_namespaces {
            if !self.manifest.has_extension(namespace) {
                undeclared.push(namespace.clone());
                warnings.push(format!(
                    "Extension namespace '{namespace}' is used but not declared in manifest"
                ));
            }
        }

        ExtensionValidationReport {
            used_namespaces,
            declared_namespaces,
            undeclared,
            unsupported_required: Vec::new(),
            warnings,
        }
    }

    /// Recursively collect extension namespaces from blocks.
    fn collect_extension_namespaces(
        blocks: &[Block],
        namespaces: &mut std::collections::HashSet<String>,
    ) {
        for block in blocks {
            // Check if this is an extension block
            if let Some(ext) = block.as_extension() {
                namespaces.insert(ext.namespace.clone());
            }

            // Recursively check children and collect marks from text nodes
            match block {
                Block::Paragraph { children, .. }
                | Block::Heading { children, .. }
                | Block::CodeBlock { children, .. }
                | Block::DefinitionTerm { children, .. } => {
                    Self::collect_marks_namespaces(children, namespaces);
                }
                Block::List { children, .. }
                | Block::ListItem { children, .. }
                | Block::Blockquote { children, .. }
                | Block::Table { children, .. }
                | Block::TableRow { children, .. }
                | Block::DefinitionItem { children, .. }
                | Block::DefinitionDescription { children, .. } => {
                    Self::collect_extension_namespaces(children, namespaces);
                }
                Block::DefinitionList(dl) => {
                    Self::collect_extension_namespaces(&dl.children, namespaces);
                }
                Block::TableCell(cell) => {
                    Self::collect_marks_namespaces(&cell.children, namespaces);
                }
                Block::Figure(fig) => {
                    Self::collect_extension_namespaces(&fig.children, namespaces);
                }
                Block::FigCaption(fc) => {
                    Self::collect_marks_namespaces(&fc.children, namespaces);
                }
                Block::Admonition(adm) => {
                    Self::collect_extension_namespaces(&adm.children, namespaces);
                }
                Block::Extension(ext) => {
                    // Already handled above, but also check children
                    Self::collect_extension_namespaces(&ext.children, namespaces);
                }
                // Leaf blocks without children
                Block::HorizontalRule { .. }
                | Block::Image(_)
                | Block::Math(_)
                | Block::Break { .. }
                | Block::Measurement(_)
                | Block::Signature(_)
                | Block::Svg(_)
                | Block::Barcode(_) => {}
            }
        }
    }

    /// Collect extension namespaces from text marks.
    fn collect_marks_namespaces(
        texts: &[Text],
        namespaces: &mut std::collections::HashSet<String>,
    ) {
        for text in texts {
            for mark in &text.marks {
                if let Some(ext) = mark.as_extension() {
                    namespaces.insert(ext.namespace.clone());
                }
            }
        }
    }
}

/// Report from document verification.
#[derive(Debug, Clone)]
pub struct VerificationReport {
    /// Whether content hash is valid.
    pub content_valid: bool,
    /// Whether document ID is valid.
    pub id_valid: bool,
    /// Error messages.
    pub errors: Vec<String>,
}

impl VerificationReport {
    /// Check if verification passed.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.content_valid && self.id_valid && self.errors.is_empty()
    }
}

/// Report from extension validation.
///
/// This report identifies which extension namespaces are used in the document
/// content but not declared in the manifest's extensions list.
#[derive(Debug, Clone, Default)]
pub struct ExtensionValidationReport {
    /// Extension namespaces used in content (from blocks and marks).
    pub used_namespaces: Vec<String>,
    /// Extension namespaces that are declared in the manifest.
    pub declared_namespaces: Vec<String>,
    /// Extension namespaces used but not declared.
    pub undeclared: Vec<String>,
    /// Extension namespaces declared as required but not supported by this reader.
    /// (Currently empty since we support all built-in extensions)
    pub unsupported_required: Vec<String>,
    /// Warning messages.
    pub warnings: Vec<String>,
}

impl ExtensionValidationReport {
    /// Check if extension validation passed without warnings.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.undeclared.is_empty() && self.unsupported_required.is_empty()
    }

    /// Check if there are any warnings.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}
