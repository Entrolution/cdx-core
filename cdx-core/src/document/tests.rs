#[cfg(test)]
mod tests {
    use crate::content::{Block, Content, Mark, Text};
    use crate::document::verification::ExtensionValidationReport;
    use crate::{Document, DocumentState};

    #[test]
    fn test_builder_basic() {
        let doc = Document::builder()
            .title("Test Document")
            .creator("Test Author")
            .add_heading(1, "Introduction")
            .add_paragraph("This is the first paragraph.")
            .build()
            .unwrap();

        assert_eq!(doc.title(), "Test Document");
        assert_eq!(doc.creators(), vec!["Test Author"]);
        assert_eq!(doc.content().len(), 2);
        assert_eq!(doc.state(), DocumentState::Draft);
    }

    #[test]
    fn test_builder_with_description() {
        let doc = Document::builder()
            .title("Report")
            .creator("Author")
            .description("A detailed report")
            .language("en")
            .build()
            .unwrap();

        assert_eq!(doc.dublin_core().description(), Some("A detailed report"));
        assert_eq!(doc.dublin_core().language(), Some("en"));
    }

    #[test]
    fn test_document_round_trip() {
        let original = Document::builder()
            .title("Round Trip Test")
            .creator("Tester")
            .add_heading(1, "Title")
            .add_paragraph("Content")
            .build()
            .unwrap();

        // Write to bytes
        let bytes = original.to_bytes().unwrap();

        // Read back
        let loaded = Document::from_bytes(bytes).unwrap();

        assert_eq!(loaded.title(), "Round Trip Test");
        assert_eq!(loaded.creators(), vec!["Tester"]);
        assert_eq!(loaded.content().len(), 2);
    }

    #[test]
    fn test_compute_id() {
        let doc = Document::builder()
            .title("ID Test")
            .creator("Author")
            .add_paragraph("Test content")
            .build()
            .unwrap();

        let id = doc.compute_id().unwrap();
        assert!(!id.is_pending());
        assert_eq!(id.algorithm(), crate::HashAlgorithm::Sha256);
    }

    #[test]
    fn test_verification() {
        let doc = Document::builder()
            .title("Verify Test")
            .creator("Author")
            .add_paragraph("Content")
            .build()
            .unwrap();

        // Fresh documents with pending hashes should verify
        let report = doc.verify().unwrap();
        assert!(report.is_valid());
    }

    // Extension validation tests

    #[test]
    fn test_extension_validation_no_extensions() {
        let doc = Document::builder()
            .title("Simple Doc")
            .creator("Author")
            .add_paragraph("Just plain text")
            .build()
            .unwrap();

        let report = doc.validate_extensions();
        assert!(report.is_valid());
        assert!(report.used_namespaces.is_empty());
        assert!(report.undeclared.is_empty());
    }

    #[test]
    fn test_extension_validation_with_extension_block() {
        use crate::extensions::ExtensionBlock;

        let ext_block = Block::Extension(
            ExtensionBlock::new("forms", "textInput")
                .with_id("name-field")
                .with_attributes(serde_json::json!({"label": "Name"})),
        );

        let content = Content::new(vec![
            Block::paragraph(vec![Text::plain("Fill out this form:")]),
            ext_block,
        ]);

        let doc = Document::builder()
            .title("Form Doc")
            .creator("Author")
            .with_content(content)
            .build()
            .unwrap();

        let report = doc.validate_extensions();
        assert!(!report.is_valid()); // undeclared extension
        assert_eq!(report.used_namespaces, vec!["forms"]);
        assert_eq!(report.undeclared, vec!["forms"]);
        assert!(report.warnings[0].contains("forms"));
    }

    #[test]
    fn test_extension_validation_with_declared_extension() {
        use crate::extensions::ExtensionBlock;
        use crate::manifest::Extension;

        let ext_block = Block::Extension(
            ExtensionBlock::new("semantic", "citation")
                .with_attributes(serde_json::json!({"refs": ["smith2023"]})),
        );

        let content = Content::new(vec![
            Block::paragraph(vec![Text::plain("According to research")]),
            ext_block,
        ]);

        let mut doc = Document::builder()
            .title("Academic Doc")
            .creator("Author")
            .with_content(content)
            .build()
            .unwrap();

        // Declare the extension
        doc.manifest_mut()
            .extensions
            .push(Extension::required("codex.semantic", "0.1"));

        let report = doc.validate_extensions();
        assert!(report.is_valid());
        assert_eq!(report.used_namespaces, vec!["semantic"]);
        assert!(report.undeclared.is_empty());
    }

    #[test]
    fn test_extension_validation_with_extension_marks() {
        use crate::content::ExtensionMark;

        let citation_mark = Mark::Extension(ExtensionMark::citation("smith2023"));
        let text_with_citation = Text::with_marks("important finding", vec![citation_mark]);

        let content = Content::new(vec![Block::paragraph(vec![text_with_citation])]);

        let doc = Document::builder()
            .title("Cited Doc")
            .creator("Author")
            .with_content(content)
            .build()
            .unwrap();

        let report = doc.validate_extensions();
        assert!(!report.is_valid()); // undeclared
        assert_eq!(report.used_namespaces, vec!["semantic"]);
        assert_eq!(report.undeclared, vec!["semantic"]);
    }

    #[test]
    fn test_extension_validation_mixed() {
        use crate::content::ExtensionMark;
        use crate::extensions::ExtensionBlock;
        use crate::manifest::Extension;

        // Create content with multiple extensions
        let citation_mark = Mark::Extension(ExtensionMark::citation("smith2023"));
        let entity_mark =
            Mark::Extension(ExtensionMark::entity("https://wikidata.org/Q937", "person"));

        let form_block = Block::Extension(
            ExtensionBlock::new("forms", "textInput")
                .with_id("email")
                .with_attributes(serde_json::json!({"label": "Email"})),
        );

        let content = Content::new(vec![
            Block::paragraph(vec![
                Text::with_marks("Einstein", vec![entity_mark]),
                Text::plain(" published his theory "),
                Text::with_marks("(ref)", vec![citation_mark]),
            ]),
            form_block,
        ]);

        let mut doc = Document::builder()
            .title("Mixed Extensions")
            .creator("Author")
            .with_content(content)
            .build()
            .unwrap();

        // Only declare semantic, not forms
        doc.manifest_mut()
            .extensions
            .push(Extension::required("codex.semantic", "0.1"));

        let report = doc.validate_extensions();
        assert!(!report.is_valid()); // forms not declared
        assert!(report.used_namespaces.contains(&"semantic".to_string()));
        assert!(report.used_namespaces.contains(&"forms".to_string()));
        assert_eq!(report.undeclared, vec!["forms"]);
        assert!(report.warnings.len() == 1);
    }

    #[test]
    fn test_extension_validation_report_methods() {
        let mut report = ExtensionValidationReport::default();
        assert!(report.is_valid());
        assert!(!report.has_warnings());

        report.undeclared.push("test".to_string());
        report.warnings.push("Test warning".to_string());
        assert!(!report.is_valid());
        assert!(report.has_warnings());
    }

    // ===== Extension File I/O Tests =====

    #[test]
    fn test_comments_round_trip() {
        use crate::extensions::{Collaborator, Comment, CommentThread};

        let mut doc = Document::builder()
            .title("Comments Test")
            .creator("Author")
            .add_paragraph("Content to comment on")
            .build()
            .unwrap();

        // Create a comment thread
        let mut thread = CommentThread::new();
        let author = Collaborator::new("Alice");
        let comment = Comment::new("c1", "block-1", author, "This is a test comment");
        thread.add(comment);

        // Set comments
        doc.set_comments(thread).unwrap();
        assert!(doc.has_comments());

        // Round-trip through bytes
        let bytes = doc.to_bytes().unwrap();
        let loaded = Document::from_bytes(bytes).unwrap();

        assert!(loaded.has_comments());
        let loaded_thread = loaded.comments().unwrap();
        assert_eq!(loaded_thread.comments.len(), 1);
        assert_eq!(loaded_thread.comments[0].id, "c1");
        assert_eq!(loaded_thread.comments[0].content, "This is a test comment");
    }

    #[test]
    fn test_phantom_clusters_round_trip() {
        use crate::anchor::ContentAnchor;
        use crate::extensions::{
            Phantom, PhantomCluster, PhantomClusters, PhantomContent, PhantomPosition, PhantomScope,
        };

        let mut doc = Document::builder()
            .title("Phantoms Test")
            .creator("Author")
            .add_paragraph("Content with phantoms")
            .build()
            .unwrap();

        // Create phantom clusters
        let mut clusters = PhantomClusters::new();
        let position = PhantomPosition::new(100.0, 200.0);
        let content = PhantomContent::paragraph("Alternative text");
        let phantom = Phantom::new("phantom-1", position, content);
        let cluster =
            PhantomCluster::new("cluster-1", ContentAnchor::block("block-1"), "Test cluster")
                .with_phantom(phantom)
                .with_scope(PhantomScope::Shared);
        clusters.add_cluster(cluster);

        // Set phantom clusters
        doc.set_phantom_clusters(clusters).unwrap();
        assert!(doc.has_phantom_clusters());

        // Round-trip through bytes
        let bytes = doc.to_bytes().unwrap();
        let loaded = Document::from_bytes(bytes).unwrap();

        assert!(loaded.has_phantom_clusters());
        let loaded_clusters = loaded.phantom_clusters().unwrap();
        assert_eq!(loaded_clusters.len(), 1);
        assert_eq!(loaded_clusters.clusters[0].id, "cluster-1");
    }

    #[test]
    fn test_form_data_round_trip() {
        use crate::extensions::FormData;

        let mut doc = Document::builder()
            .title("Form Data Test")
            .creator("Author")
            .add_paragraph("Form content")
            .build()
            .unwrap();

        // Create form data
        let mut form_data = FormData::new();
        form_data.set("name", serde_json::json!("John Doe"));
        form_data.set("email", serde_json::json!("john@example.com"));
        form_data.set("age", serde_json::json!(30));

        // Set form data
        doc.set_form_data(form_data).unwrap();
        assert!(doc.has_form_data());

        // Round-trip through bytes
        let bytes = doc.to_bytes().unwrap();
        let loaded = Document::from_bytes(bytes).unwrap();

        assert!(loaded.has_form_data());
        let loaded_form = loaded.form_data().unwrap();
        assert_eq!(
            loaded_form.get("name"),
            Some(&serde_json::json!("John Doe"))
        );
        assert_eq!(
            loaded_form.get("email"),
            Some(&serde_json::json!("john@example.com"))
        );
        assert_eq!(loaded_form.get("age"), Some(&serde_json::json!(30)));
    }

    #[test]
    fn test_bibliography_round_trip() {
        use crate::extensions::{Bibliography, BibliographyEntry, CitationStyle, EntryType};

        let mut doc = Document::builder()
            .title("Bibliography Test")
            .creator("Author")
            .add_paragraph("Content with citations")
            .build()
            .unwrap();

        // Create bibliography
        let mut bibliography = Bibliography::new(CitationStyle::Apa);
        let entry = BibliographyEntry::new("smith2023", EntryType::Article, "Test Article");
        bibliography.add_entry(entry);

        // Set bibliography
        doc.set_bibliography(bibliography).unwrap();
        assert!(doc.has_bibliography());

        // Round-trip through bytes
        let bytes = doc.to_bytes().unwrap();
        let loaded = Document::from_bytes(bytes).unwrap();

        assert!(loaded.has_bibliography());
        let loaded_bib = loaded.bibliography().unwrap();
        assert_eq!(loaded_bib.len(), 1);
        assert_eq!(loaded_bib.style, CitationStyle::Apa);
        assert!(loaded_bib.contains("smith2023"));
    }

    #[test]
    fn test_jsonld_round_trip() {
        use crate::extensions::JsonLdMetadata;
        use serde_json::json;

        let mut doc = Document::builder()
            .title("JSON-LD Test")
            .creator("Author")
            .add_paragraph("Content with structured data")
            .build()
            .unwrap();

        // Create JSON-LD metadata
        let mut jsonld = JsonLdMetadata::new();
        jsonld.add_node(json!({
            "@type": "Article",
            "name": "Test Article",
            "author": {
                "@type": "Person",
                "name": "Test Author"
            }
        }));

        // Set JSON-LD metadata
        doc.set_jsonld_metadata(jsonld).unwrap();
        assert!(doc.has_jsonld_metadata());

        // Round-trip through bytes
        let bytes = doc.to_bytes().unwrap();
        let loaded = Document::from_bytes(bytes).unwrap();

        assert!(loaded.has_jsonld_metadata());
        let loaded_jsonld = loaded.jsonld_metadata().unwrap();
        assert_eq!(loaded_jsonld.graph.len(), 1);
        assert!(loaded_jsonld
            .context
            .contains(&"https://schema.org".to_string()));
    }

    #[test]
    fn test_clear_extension_data() {
        use crate::extensions::{Bibliography, CitationStyle, CommentThread, FormData};

        let mut doc = Document::builder()
            .title("Clear Test")
            .creator("Author")
            .add_paragraph("Content")
            .build()
            .unwrap();

        // Set all extension data
        doc.set_comments(CommentThread::new()).unwrap();
        doc.set_form_data(FormData::new()).unwrap();
        doc.set_bibliography(Bibliography::new(CitationStyle::Chicago))
            .unwrap();

        assert!(doc.has_comments());
        assert!(doc.has_form_data());
        assert!(doc.has_bibliography());

        // Clear each
        doc.clear_comments().unwrap();
        doc.clear_form_data().unwrap();
        doc.clear_bibliography().unwrap();

        assert!(!doc.has_comments());
        assert!(!doc.has_form_data());
        assert!(!doc.has_bibliography());
    }

    #[test]
    fn test_extension_data_mutable_access() {
        use crate::extensions::{Collaborator, Comment, CommentThread};

        let mut doc = Document::builder()
            .title("Mutable Test")
            .creator("Author")
            .add_paragraph("Content")
            .build()
            .unwrap();

        // Set initial comments
        let mut thread = CommentThread::new();
        let author = Collaborator::new("Alice");
        thread.add(Comment::new("c1", "block-1", author.clone(), "First"));
        doc.set_comments(thread).unwrap();

        // Modify through mutable reference
        if let Some(comments) = doc.comments_mut().unwrap() {
            comments.add(Comment::new("c2", "block-2", author, "Second"));
        }

        assert_eq!(doc.comments().unwrap().comments.len(), 2);
    }

    #[cfg(any(feature = "signatures", feature = "encryption"))]
    #[test]
    fn test_write_to_clears_stale_security_ref() {
        use crate::manifest::SecurityRef;

        // Create a document and add a signature
        let mut doc = Document::builder()
            .title("Security Ref Test")
            .creator("Author")
            .add_paragraph("Content")
            .build()
            .unwrap();

        // Manually set a security reference as if it had signatures before
        doc.manifest_mut().security = Some(SecurityRef {
            signatures: Some("security/signatures.json".to_string()),
            encryption: None,
        });

        // Save and reload — signature_file is None so the security ref should be cleared
        let bytes = doc.to_bytes().unwrap();
        let loaded = Document::from_bytes(bytes).unwrap();
        assert!(
            loaded.manifest().security.is_none(),
            "security ref should be None when no signatures or encryption exist"
        );
    }

    #[test]
    fn test_root_document_can_freeze_after_set_lineage() {
        let mut doc = Document::builder()
            .title("Root Doc")
            .creator("Author")
            .add_paragraph("Content")
            .build()
            .unwrap();

        // Set root lineage
        doc.set_lineage(None, 1, None).unwrap();
        assert!(doc.manifest().lineage.is_some());
    }

    #[test]
    fn test_freeze_lineage_error_message_mentions_set_lineage() {
        let mut doc = Document::builder()
            .title("Lineage Error Test")
            .creator("Author")
            .add_paragraph("Content")
            .build()
            .unwrap();

        doc.submit_for_review().unwrap();

        // Add a dummy signature so the signature check passes
        #[cfg(feature = "signatures")]
        {
            use crate::security::{Signature, SignatureAlgorithm, SignerInfo};
            let sig = Signature::new(
                "sig-1",
                SignatureAlgorithm::ES256,
                SignerInfo::new("test@example.com"),
                "dummysig",
            );
            doc.add_signature(sig).unwrap();
        }

        let err = doc.freeze().unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("set_lineage"),
            "error message should mention set_lineage, got: {msg}"
        );
    }

    // Phase 2: Document mutation consistency tests

    #[test]
    fn test_set_extension_updates_modified() {
        use crate::extensions::Bibliography;

        let mut doc = Document::builder()
            .title("Extension Modified Test")
            .creator("Author")
            .add_paragraph("Content")
            .build()
            .unwrap();

        let before = doc.manifest().modified;
        std::thread::sleep(std::time::Duration::from_millis(10));

        doc.set_bibliography(Bibliography::default()).unwrap();
        assert!(
            doc.manifest().modified > before,
            "modified timestamp should update after set_bibliography"
        );
    }

    #[test]
    fn test_clear_extension_updates_modified() {
        use crate::extensions::Bibliography;

        let mut doc = Document::builder()
            .title("Clear Extension Modified Test")
            .creator("Author")
            .add_paragraph("Content")
            .build()
            .unwrap();

        doc.set_bibliography(Bibliography::default()).unwrap();
        let before = doc.manifest().modified;
        std::thread::sleep(std::time::Duration::from_millis(10));

        doc.clear_bibliography().unwrap();
        assert!(
            doc.manifest().modified > before,
            "modified timestamp should update after clear_bibliography"
        );
    }

    #[cfg(feature = "encryption")]
    #[test]
    fn test_set_encryption_updates_modified() {
        use crate::security::{EncryptionAlgorithm, EncryptionMetadata};

        let mut doc = Document::builder()
            .title("Encryption Modified Test")
            .creator("Author")
            .add_paragraph("Content")
            .build()
            .unwrap();

        let before = doc.manifest().modified;
        std::thread::sleep(std::time::Duration::from_millis(10));

        let meta = EncryptionMetadata {
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            kdf: None,
            wrapped_key: None,
            key_management: None,
            recipients: Vec::new(),
        };
        doc.set_encryption(meta).unwrap();
        assert!(
            doc.manifest().modified > before,
            "modified timestamp should update after set_encryption"
        );
    }
}
