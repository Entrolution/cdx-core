//! Content types for the Swift bridge.
//!
//! These types mirror cdx-core's content model but are structured
//! for easy consumption via UniFFI.

/// Document state enum exposed to Swift.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum CdxDocumentState {
    Draft,
    Review,
    Frozen,
    Published,
}

impl From<cdx_core::DocumentState> for CdxDocumentState {
    fn from(state: cdx_core::DocumentState) -> Self {
        match state {
            cdx_core::DocumentState::Draft => CdxDocumentState::Draft,
            cdx_core::DocumentState::Review => CdxDocumentState::Review,
            cdx_core::DocumentState::Frozen => CdxDocumentState::Frozen,
            cdx_core::DocumentState::Published => CdxDocumentState::Published,
        }
    }
}

impl From<CdxDocumentState> for cdx_core::DocumentState {
    fn from(state: CdxDocumentState) -> Self {
        match state {
            CdxDocumentState::Draft => cdx_core::DocumentState::Draft,
            CdxDocumentState::Review => cdx_core::DocumentState::Review,
            CdxDocumentState::Frozen => cdx_core::DocumentState::Frozen,
            CdxDocumentState::Published => cdx_core::DocumentState::Published,
        }
    }
}

/// Verification state enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum CdxVerificationState {
    Unchecked,
    Verified,
    Unsigned,
    Invalid,
    Warning,
}

/// Text mark types with associated data.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum CdxTextMark {
    Bold,
    Italic,
    Code,
    Strikethrough,
    Underline,
    Superscript,
    Subscript,
    Link { href: String, title: Option<String> },
    Anchor { id: String },
    Footnote { number: u32, id: Option<String> },
    Math { value: String, display: bool },
}

/// Block types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum CdxBlockType {
    Paragraph,
    Heading,
    List,
    ListItem,
    Blockquote,
    CodeBlock,
    HorizontalRule,
    Image,
    Table,
    TableRow,
    TableCell,
    Math,
    Break,
    DefinitionList,
    DefinitionItem,
    DefinitionTerm,
    DefinitionDescription,
    Figure,
    FigCaption,
    Admonition,
    Extension,
}

/// Text span with formatting.
#[derive(Debug, Clone, uniffi::Record)]
pub struct CdxTextSpan {
    pub value: String,
    pub marks: Vec<CdxTextMark>,
}

/// Block attributes.
#[derive(Debug, Clone, Default, uniffi::Record)]
pub struct CdxBlockAttributes {
    pub direction: Option<String>,
    pub language: Option<String>,
}

/// Heading info.
#[derive(Debug, Clone, uniffi::Record)]
pub struct CdxHeadingInfo {
    pub level: u8,
}

/// List info.
#[derive(Debug, Clone, uniffi::Record)]
pub struct CdxListInfo {
    pub ordered: bool,
    pub start: Option<u32>,
}

/// Code block info.
#[derive(Debug, Clone, uniffi::Record)]
pub struct CdxCodeBlockInfo {
    pub language: Option<String>,
}

/// Image info.
#[derive(Debug, Clone, uniffi::Record)]
pub struct CdxImageInfo {
    pub src: String,
    pub alt: Option<String>,
    pub title: Option<String>,
}

/// Table cell info.
#[derive(Debug, Clone, uniffi::Record)]
pub struct CdxTableCellInfo {
    pub colspan: u32,
    pub rowspan: u32,
}

/// Math info.
#[derive(Debug, Clone, uniffi::Record)]
pub struct CdxMathInfo {
    pub content: String,
    pub display_mode: bool,
}

/// Admonition info.
#[derive(Debug, Clone, uniffi::Record)]
pub struct CdxAdmonitionInfo {
    pub variant: String,
    pub title: Option<String>,
}

/// Content block.
#[derive(Debug, Clone, uniffi::Record)]
pub struct CdxBlock {
    pub id: String,
    pub block_type: CdxBlockType,
    pub text_children: Vec<CdxTextSpan>,
    pub block_children: Vec<CdxBlock>,
    pub attributes: Option<CdxBlockAttributes>,
    pub heading_info: Option<CdxHeadingInfo>,
    pub list_info: Option<CdxListInfo>,
    pub code_block_info: Option<CdxCodeBlockInfo>,
    pub image_info: Option<CdxImageInfo>,
    pub table_cell_info: Option<CdxTableCellInfo>,
    pub math_info: Option<CdxMathInfo>,
    pub admonition_info: Option<CdxAdmonitionInfo>,
    pub list_item_checked: Option<bool>,
    pub table_row_header: Option<bool>,
}

impl CdxBlock {
    /// Create a new paragraph block.
    pub fn paragraph(id: String, text: Vec<CdxTextSpan>) -> Self {
        Self {
            id,
            block_type: CdxBlockType::Paragraph,
            text_children: text,
            block_children: vec![],
            attributes: None,
            heading_info: None,
            list_info: None,
            code_block_info: None,
            image_info: None,
            table_cell_info: None,
            math_info: None,
            admonition_info: None,
            list_item_checked: None,
            table_row_header: None,
        }
    }

    /// Create a new heading block.
    pub fn heading(id: String, level: u8, text: Vec<CdxTextSpan>) -> Self {
        Self {
            id,
            block_type: CdxBlockType::Heading,
            text_children: text,
            block_children: vec![],
            attributes: None,
            heading_info: Some(CdxHeadingInfo { level }),
            list_info: None,
            code_block_info: None,
            image_info: None,
            table_cell_info: None,
            math_info: None,
            admonition_info: None,
            list_item_checked: None,
            table_row_header: None,
        }
    }

    fn empty(id: Option<String>, block_type: CdxBlockType) -> Self {
        Self {
            id: id.unwrap_or_default(),
            block_type,
            text_children: vec![],
            block_children: vec![],
            attributes: None,
            heading_info: None,
            list_info: None,
            code_block_info: None,
            image_info: None,
            table_cell_info: None,
            math_info: None,
            admonition_info: None,
            list_item_checked: None,
            table_row_header: None,
        }
    }
}

/// Document content.
#[derive(Debug, Clone, uniffi::Record)]
pub struct CdxContent {
    pub version: String,
    pub blocks: Vec<CdxBlock>,
}

impl Default for CdxContent {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            blocks: vec![],
        }
    }
}

/// Dublin Core metadata.
#[derive(Debug, Clone, Default, uniffi::Record)]
pub struct CdxMetadata {
    pub title: String,
    pub creator: String,
    pub subject: Option<String>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub contributor: Option<String>,
    pub date: Option<String>,
    pub type_field: Option<String>,
    pub format: Option<String>,
    pub identifier: Option<String>,
    pub source: Option<String>,
    pub language: Option<String>,
    pub relation: Option<String>,
    pub coverage: Option<String>,
    pub rights: Option<String>,
}

/// Signer information.
#[derive(Debug, Clone, uniffi::Record)]
pub struct CdxSignerInfo {
    pub id: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub organization: Option<String>,
    pub signed_at: String,
}

/// Signature information.
#[derive(Debug, Clone, uniffi::Record)]
pub struct CdxSignature {
    pub id: String,
    pub algorithm: String,
    pub signed_at: String,
    pub signer: CdxSignerInfo,
    pub scope_description: Option<String>,
}

/// Verification result.
#[derive(Debug, Clone, uniffi::Record)]
pub struct CdxVerificationResult {
    pub state: CdxVerificationState,
    pub signatures: Vec<CdxSignature>,
    pub error_message: Option<String>,
}

impl Default for CdxVerificationResult {
    fn default() -> Self {
        Self {
            state: CdxVerificationState::Unchecked,
            signatures: vec![],
            error_message: None,
        }
    }
}

/// Document manifest info.
#[derive(Debug, Clone, uniffi::Record)]
pub struct CdxManifestInfo {
    pub document_id: String,
    pub state: CdxDocumentState,
    pub created: String,
    pub modified: String,
    pub codex_version: String,
    pub hash_algorithm: String,
}

/// Request to sign a document.
#[derive(Debug, Clone, uniffi::Record)]
pub struct CdxSigningRequest {
    pub name: String,
    pub email: Option<String>,
    pub organization: Option<String>,
}

/// Result of signing a document.
#[derive(Debug, Clone, uniffi::Record)]
pub struct CdxSigningResult {
    pub signature_id: String,
    pub public_key_pem: String,
    pub signed_at: String,
}

/// Encryption information.
#[derive(Debug, Clone, uniffi::Record)]
pub struct CdxEncryptionInfo {
    pub algorithm: String,
    pub kdf_algorithm: Option<String>,
    pub has_recipients: bool,
}

// --- Conversion functions from cdx-core types ---

impl From<&cdx_core::content::Mark> for CdxTextMark {
    fn from(mark: &cdx_core::content::Mark) -> Self {
        match mark {
            cdx_core::content::Mark::Bold => CdxTextMark::Bold,
            cdx_core::content::Mark::Italic => CdxTextMark::Italic,
            cdx_core::content::Mark::Code => CdxTextMark::Code,
            cdx_core::content::Mark::Strikethrough => CdxTextMark::Strikethrough,
            cdx_core::content::Mark::Underline => CdxTextMark::Underline,
            cdx_core::content::Mark::Superscript => CdxTextMark::Superscript,
            cdx_core::content::Mark::Subscript => CdxTextMark::Subscript,
            cdx_core::content::Mark::Link { href, title } => CdxTextMark::Link {
                href: href.clone(),
                title: title.clone(),
            },
            cdx_core::content::Mark::Anchor { id } => CdxTextMark::Anchor { id: id.clone() },
            cdx_core::content::Mark::Footnote { number, id } => CdxTextMark::Footnote {
                number: *number,
                id: id.clone(),
            },
            cdx_core::content::Mark::Math {
                format: _, source, ..
            } => CdxTextMark::Math {
                value: source.clone(),
                display: false,
            },
            cdx_core::content::Mark::Extension(_) => {
                // Extension marks are not directly representable; skip
                CdxTextMark::Code // fallback — callers use filter_map
            }
        }
    }
}

impl From<&cdx_core::content::Text> for CdxTextSpan {
    fn from(text: &cdx_core::content::Text) -> Self {
        let marks = text
            .marks
            .iter()
            .filter_map(|m| {
                if matches!(m, cdx_core::content::Mark::Extension(_)) {
                    None
                } else {
                    Some(CdxTextMark::from(m))
                }
            })
            .collect();

        CdxTextSpan {
            value: text.value.clone(),
            marks,
        }
    }
}

fn convert_block_attributes(attrs: &cdx_core::content::BlockAttributes) -> CdxBlockAttributes {
    CdxBlockAttributes {
        direction: attrs.dir.clone(),
        language: attrs.lang.clone(),
    }
}

impl From<&cdx_core::content::Block> for CdxBlock {
    fn from(block: &cdx_core::content::Block) -> Self {
        match block {
            cdx_core::content::Block::Paragraph {
                id,
                children,
                attributes,
            } => CdxBlock {
                id: id.clone().unwrap_or_default(),
                block_type: CdxBlockType::Paragraph,
                text_children: children.iter().map(CdxTextSpan::from).collect(),
                block_children: vec![],
                attributes: Some(convert_block_attributes(attributes)),
                heading_info: None,
                list_info: None,
                code_block_info: None,
                image_info: None,
                table_cell_info: None,
                math_info: None,
                admonition_info: None,
                list_item_checked: None,
                table_row_header: None,
            },
            cdx_core::content::Block::Heading {
                id,
                level,
                children,
                attributes,
            } => CdxBlock {
                id: id.clone().unwrap_or_default(),
                block_type: CdxBlockType::Heading,
                text_children: children.iter().map(CdxTextSpan::from).collect(),
                block_children: vec![],
                attributes: Some(convert_block_attributes(attributes)),
                heading_info: Some(CdxHeadingInfo { level: *level }),
                list_info: None,
                code_block_info: None,
                image_info: None,
                table_cell_info: None,
                math_info: None,
                admonition_info: None,
                list_item_checked: None,
                table_row_header: None,
            },
            cdx_core::content::Block::List {
                id,
                ordered,
                start,
                children,
                attributes,
            } => CdxBlock {
                id: id.clone().unwrap_or_default(),
                block_type: CdxBlockType::List,
                text_children: vec![],
                block_children: children.iter().map(CdxBlock::from).collect(),
                attributes: Some(convert_block_attributes(attributes)),
                heading_info: None,
                list_info: Some(CdxListInfo {
                    ordered: *ordered,
                    start: *start,
                }),
                code_block_info: None,
                image_info: None,
                table_cell_info: None,
                math_info: None,
                admonition_info: None,
                list_item_checked: None,
                table_row_header: None,
            },
            cdx_core::content::Block::ListItem {
                id,
                checked,
                children,
                attributes,
            } => CdxBlock {
                id: id.clone().unwrap_or_default(),
                block_type: CdxBlockType::ListItem,
                text_children: vec![],
                block_children: children.iter().map(CdxBlock::from).collect(),
                attributes: Some(convert_block_attributes(attributes)),
                heading_info: None,
                list_info: None,
                code_block_info: None,
                image_info: None,
                table_cell_info: None,
                math_info: None,
                admonition_info: None,
                list_item_checked: *checked,
                table_row_header: None,
            },
            cdx_core::content::Block::Blockquote {
                id,
                children,
                attributes,
            } => CdxBlock {
                id: id.clone().unwrap_or_default(),
                block_type: CdxBlockType::Blockquote,
                text_children: vec![],
                block_children: children.iter().map(CdxBlock::from).collect(),
                attributes: Some(convert_block_attributes(attributes)),
                heading_info: None,
                list_info: None,
                code_block_info: None,
                image_info: None,
                table_cell_info: None,
                math_info: None,
                admonition_info: None,
                list_item_checked: None,
                table_row_header: None,
            },
            cdx_core::content::Block::CodeBlock {
                id,
                language,
                children,
                attributes,
                ..
            } => CdxBlock {
                id: id.clone().unwrap_or_default(),
                block_type: CdxBlockType::CodeBlock,
                text_children: children.iter().map(CdxTextSpan::from).collect(),
                block_children: vec![],
                attributes: Some(convert_block_attributes(attributes)),
                heading_info: None,
                list_info: None,
                code_block_info: Some(CdxCodeBlockInfo {
                    language: language.clone(),
                }),
                image_info: None,
                table_cell_info: None,
                math_info: None,
                admonition_info: None,
                list_item_checked: None,
                table_row_header: None,
            },
            cdx_core::content::Block::HorizontalRule { id } => {
                CdxBlock::empty(id.clone(), CdxBlockType::HorizontalRule)
            }
            cdx_core::content::Block::Image(img) => CdxBlock {
                id: img.id.clone().unwrap_or_default(),
                block_type: CdxBlockType::Image,
                text_children: vec![],
                block_children: vec![],
                attributes: None,
                heading_info: None,
                list_info: None,
                code_block_info: None,
                image_info: Some(CdxImageInfo {
                    src: img.src.clone(),
                    alt: Some(img.alt.clone()),
                    title: img.title.clone(),
                }),
                table_cell_info: None,
                math_info: None,
                admonition_info: None,
                list_item_checked: None,
                table_row_header: None,
            },
            cdx_core::content::Block::Table {
                id,
                children,
                attributes,
            } => CdxBlock {
                id: id.clone().unwrap_or_default(),
                block_type: CdxBlockType::Table,
                text_children: vec![],
                block_children: children.iter().map(CdxBlock::from).collect(),
                attributes: Some(convert_block_attributes(attributes)),
                heading_info: None,
                list_info: None,
                code_block_info: None,
                image_info: None,
                table_cell_info: None,
                math_info: None,
                admonition_info: None,
                list_item_checked: None,
                table_row_header: None,
            },
            cdx_core::content::Block::TableRow {
                id,
                header,
                children,
                attributes,
            } => CdxBlock {
                id: id.clone().unwrap_or_default(),
                block_type: CdxBlockType::TableRow,
                text_children: vec![],
                block_children: children.iter().map(CdxBlock::from).collect(),
                attributes: Some(convert_block_attributes(attributes)),
                heading_info: None,
                list_info: None,
                code_block_info: None,
                image_info: None,
                table_cell_info: None,
                math_info: None,
                admonition_info: None,
                list_item_checked: None,
                table_row_header: Some(*header),
            },
            cdx_core::content::Block::TableCell(cell) => CdxBlock {
                id: cell.id.clone().unwrap_or_default(),
                block_type: CdxBlockType::TableCell,
                text_children: cell.children.iter().map(CdxTextSpan::from).collect(),
                block_children: vec![],
                attributes: Some(convert_block_attributes(&cell.attributes)),
                heading_info: None,
                list_info: None,
                code_block_info: None,
                image_info: None,
                table_cell_info: Some(CdxTableCellInfo {
                    colspan: cell.colspan,
                    rowspan: cell.rowspan,
                }),
                math_info: None,
                admonition_info: None,
                list_item_checked: None,
                table_row_header: None,
            },
            cdx_core::content::Block::Math(math) => CdxBlock {
                id: math.id.clone().unwrap_or_default(),
                block_type: CdxBlockType::Math,
                text_children: vec![],
                block_children: vec![],
                attributes: None,
                heading_info: None,
                list_info: None,
                code_block_info: None,
                image_info: None,
                table_cell_info: None,
                math_info: Some(CdxMathInfo {
                    content: math.value.clone(),
                    display_mode: math.display,
                }),
                admonition_info: None,
                list_item_checked: None,
                table_row_header: None,
            },
            cdx_core::content::Block::Break { id } => {
                CdxBlock::empty(id.clone(), CdxBlockType::Break)
            }
            cdx_core::content::Block::DefinitionList(dl) => CdxBlock {
                id: dl.id.clone().unwrap_or_default(),
                block_type: CdxBlockType::DefinitionList,
                text_children: vec![],
                block_children: dl.children.iter().map(CdxBlock::from).collect(),
                attributes: Some(convert_block_attributes(&dl.attributes)),
                heading_info: None,
                list_info: None,
                code_block_info: None,
                image_info: None,
                table_cell_info: None,
                math_info: None,
                admonition_info: None,
                list_item_checked: None,
                table_row_header: None,
            },
            cdx_core::content::Block::DefinitionItem {
                id,
                children,
                attributes,
            } => CdxBlock {
                id: id.clone().unwrap_or_default(),
                block_type: CdxBlockType::DefinitionItem,
                text_children: vec![],
                block_children: children.iter().map(CdxBlock::from).collect(),
                attributes: Some(convert_block_attributes(attributes)),
                heading_info: None,
                list_info: None,
                code_block_info: None,
                image_info: None,
                table_cell_info: None,
                math_info: None,
                admonition_info: None,
                list_item_checked: None,
                table_row_header: None,
            },
            cdx_core::content::Block::DefinitionTerm {
                id,
                children,
                attributes,
            } => CdxBlock {
                id: id.clone().unwrap_or_default(),
                block_type: CdxBlockType::DefinitionTerm,
                text_children: children.iter().map(CdxTextSpan::from).collect(),
                block_children: vec![],
                attributes: Some(convert_block_attributes(attributes)),
                heading_info: None,
                list_info: None,
                code_block_info: None,
                image_info: None,
                table_cell_info: None,
                math_info: None,
                admonition_info: None,
                list_item_checked: None,
                table_row_header: None,
            },
            cdx_core::content::Block::DefinitionDescription {
                id,
                children,
                attributes,
            } => CdxBlock {
                id: id.clone().unwrap_or_default(),
                block_type: CdxBlockType::DefinitionDescription,
                text_children: vec![],
                block_children: children.iter().map(CdxBlock::from).collect(),
                attributes: Some(convert_block_attributes(attributes)),
                heading_info: None,
                list_info: None,
                code_block_info: None,
                image_info: None,
                table_cell_info: None,
                math_info: None,
                admonition_info: None,
                list_item_checked: None,
                table_row_header: None,
            },
            cdx_core::content::Block::Figure(fig) => CdxBlock {
                id: fig.id.clone().unwrap_or_default(),
                block_type: CdxBlockType::Figure,
                text_children: vec![],
                block_children: fig.children.iter().map(CdxBlock::from).collect(),
                attributes: Some(convert_block_attributes(&fig.attributes)),
                heading_info: None,
                list_info: None,
                code_block_info: None,
                image_info: None,
                table_cell_info: None,
                math_info: None,
                admonition_info: None,
                list_item_checked: None,
                table_row_header: None,
            },
            cdx_core::content::Block::FigCaption(cap) => CdxBlock {
                id: cap.id.clone().unwrap_or_default(),
                block_type: CdxBlockType::FigCaption,
                text_children: cap.children.iter().map(CdxTextSpan::from).collect(),
                block_children: vec![],
                attributes: Some(convert_block_attributes(&cap.attributes)),
                heading_info: None,
                list_info: None,
                code_block_info: None,
                image_info: None,
                table_cell_info: None,
                math_info: None,
                admonition_info: None,
                list_item_checked: None,
                table_row_header: None,
            },
            cdx_core::content::Block::Admonition(adm) => CdxBlock {
                id: adm.id.clone().unwrap_or_default(),
                block_type: CdxBlockType::Admonition,
                text_children: vec![],
                block_children: adm.children.iter().map(CdxBlock::from).collect(),
                attributes: Some(convert_block_attributes(&adm.attributes)),
                heading_info: None,
                list_info: None,
                code_block_info: None,
                image_info: None,
                table_cell_info: None,
                math_info: None,
                admonition_info: Some(CdxAdmonitionInfo {
                    variant: adm.variant.to_string(),
                    title: adm.title.clone(),
                }),
                list_item_checked: None,
                table_row_header: None,
            },
            // Measurement, Signature, Svg, Barcode — map to Extension for now
            cdx_core::content::Block::Measurement(m) => {
                CdxBlock::empty(m.id.clone(), CdxBlockType::Extension)
            }
            cdx_core::content::Block::Signature(s) => {
                CdxBlock::empty(s.id.clone(), CdxBlockType::Extension)
            }
            cdx_core::content::Block::Svg(s) => {
                CdxBlock::empty(s.id.clone(), CdxBlockType::Extension)
            }
            cdx_core::content::Block::Barcode(b) => {
                CdxBlock::empty(b.id.clone(), CdxBlockType::Extension)
            }
            cdx_core::content::Block::Extension(ext) => CdxBlock {
                id: ext.id.clone().unwrap_or_default(),
                block_type: CdxBlockType::Extension,
                text_children: vec![],
                block_children: ext.children.iter().map(CdxBlock::from).collect(),
                attributes: None,
                heading_info: None,
                list_info: None,
                code_block_info: None,
                image_info: None,
                table_cell_info: None,
                math_info: None,
                admonition_info: None,
                list_item_checked: None,
                table_row_header: None,
            },
        }
    }
}

impl From<&cdx_core::content::Content> for CdxContent {
    fn from(content: &cdx_core::content::Content) -> Self {
        CdxContent {
            version: content.version.clone(),
            blocks: content.blocks.iter().map(CdxBlock::from).collect(),
        }
    }
}

impl From<&cdx_core::metadata::DublinCore> for CdxMetadata {
    fn from(dc: &cdx_core::metadata::DublinCore) -> Self {
        CdxMetadata {
            title: dc.title().to_string(),
            creator: dc
                .creators()
                .first()
                .map(|s| s.to_string())
                .unwrap_or_default(),
            subject: dc.subjects().first().map(|s| s.to_string()),
            description: dc.description().map(|s| s.to_string()),
            publisher: dc.publisher().map(|s| s.to_string()),
            contributor: dc.contributors().first().map(|s| s.to_string()),
            date: dc.date().map(|s| s.to_string()),
            type_field: dc.dc_type().map(|s| s.to_string()),
            format: dc.format().map(|s| s.to_string()),
            identifier: dc.identifier().map(|s| s.to_string()),
            source: dc.source().map(|s| s.to_string()),
            language: dc.language().map(|s| s.to_string()),
            relation: dc.relation().map(|s| s.to_string()),
            coverage: dc.coverage().map(|s| s.to_string()),
            rights: dc.rights().map(|s| s.to_string()),
        }
    }
}
