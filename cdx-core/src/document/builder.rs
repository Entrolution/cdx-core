use crate::archive::{CONTENT_PATH, DUBLIN_CORE_PATH};
use crate::content::{Block, Content, Text};
use crate::metadata::DublinCore;
use crate::{DocumentId, DocumentState, HashAlgorithm, Manifest, Result};

use super::Document;

/// Builder for creating CDX documents.
#[derive(Debug, Clone)]
pub struct DocumentBuilder {
    title: String,
    creator: String,
    description: Option<String>,
    language: Option<String>,
    blocks: Vec<Block>,
    state: DocumentState,
    hash_algorithm: HashAlgorithm,
    content_override: Option<Content>,
    dublin_core_override: Option<DublinCore>,
}

impl Default for DocumentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentBuilder {
    /// Create a new document builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: "Untitled".to_string(),
            creator: "Unknown".to_string(),
            description: None,
            language: None,
            blocks: Vec::new(),
            state: DocumentState::Draft,
            hash_algorithm: HashAlgorithm::default(),
            content_override: None,
            dublin_core_override: None,
        }
    }

    /// Set the document title.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the document creator.
    #[must_use]
    pub fn creator(mut self, creator: impl Into<String>) -> Self {
        self.creator = creator.into();
        self
    }

    /// Set the document description.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the document language.
    #[must_use]
    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Set the document state.
    #[must_use]
    pub fn state(mut self, state: DocumentState) -> Self {
        self.state = state;
        self
    }

    /// Set the hash algorithm.
    #[must_use]
    pub fn hash_algorithm(mut self, algorithm: HashAlgorithm) -> Self {
        self.hash_algorithm = algorithm;
        self
    }

    /// Add a content block.
    #[must_use]
    pub fn add_block(mut self, block: Block) -> Self {
        self.blocks.push(block);
        self
    }

    /// Add a heading block.
    #[must_use]
    pub fn add_heading(self, level: u8, text: impl Into<String>) -> Self {
        self.add_block(Block::heading(level, vec![Text::plain(text)]))
    }

    /// Add a paragraph block.
    #[must_use]
    pub fn add_paragraph(self, text: impl Into<String>) -> Self {
        self.add_block(Block::paragraph(vec![Text::plain(text)]))
    }

    /// Add a code block.
    #[must_use]
    pub fn add_code_block(self, code: impl Into<String>, language: Option<String>) -> Self {
        self.add_block(Block::code_block(code, language))
    }

    /// Set pre-built content, overriding any blocks added via `add_block()`.
    #[must_use]
    pub fn with_content(mut self, content: Content) -> Self {
        self.content_override = Some(content);
        self
    }

    /// Set pre-built Dublin Core metadata, overriding title/creator/description/language.
    #[must_use]
    pub fn with_dublin_core(mut self, dublin_core: DublinCore) -> Self {
        self.dublin_core_override = Some(dublin_core);
        self
    }

    /// Build the document.
    ///
    /// # Errors
    ///
    /// Returns an error if the document cannot be constructed.
    pub fn build(self) -> Result<Document> {
        use crate::manifest::{ContentRef, Metadata};

        let content = self
            .content_override
            .unwrap_or_else(|| Content::new(self.blocks));
        let dublin_core = self.dublin_core_override.unwrap_or_else(|| {
            let mut dc = DublinCore::new(&self.title, &self.creator);
            dc.terms.description = self.description;
            dc.terms.language = self.language;
            dc
        });

        let content_ref = ContentRef {
            path: CONTENT_PATH.to_string(),
            hash: DocumentId::pending(),
            compression: Some("deflate".to_string()),
            merkle_root: None,
            block_count: None,
        };

        let metadata = Metadata {
            dublin_core: DUBLIN_CORE_PATH.to_string(),
            custom: None,
        };

        let mut manifest = Manifest::new(content_ref, metadata);
        manifest.state = self.state;
        manifest.hash_algorithm = self.hash_algorithm;

        Ok(Document {
            manifest,
            content,
            dublin_core,
            #[cfg(feature = "signatures")]
            signature_file: None,
            #[cfg(feature = "encryption")]
            encryption_metadata: None,
            academic_numbering: None,
            comments: None,
            phantom_clusters: None,
            form_data: None,
            bibliography: None,
            jsonld_metadata: None,
        })
    }
}
