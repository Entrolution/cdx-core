//! Content block types.

use std::borrow::Cow;

use serde::de;
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};

use super::Text;
use crate::extensions::ExtensionBlock;

/// Root content structure for a Codex document.
///
/// The content file contains a version identifier and an array of blocks
/// that make up the document content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Content {
    /// Content model version (e.g., "0.1").
    pub version: String,

    /// Array of content blocks.
    pub blocks: Vec<Block>,
}

impl Content {
    /// Create new content with the default version.
    #[must_use]
    pub fn new(blocks: Vec<Block>) -> Self {
        Self {
            version: crate::SPEC_VERSION.to_string(),
            blocks,
        }
    }

    /// Create empty content.
    #[must_use]
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    /// Check if the content has any blocks.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// Get the number of blocks.
    #[must_use]
    pub fn len(&self) -> usize {
        self.blocks.len()
    }
}

impl Default for Content {
    fn default() -> Self {
        Self::empty()
    }
}

/// Common attributes that can appear on any block.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockAttributes {
    /// Text direction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dir: Option<String>,

    /// BCP 47 language tag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,

    /// Writing mode for text direction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub writing_mode: Option<WritingMode>,
}

impl BlockAttributes {
    /// Check if attributes are empty (all None).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.dir.is_none() && self.lang.is_none() && self.writing_mode.is_none()
    }
}

/// A content block in the document tree.
///
/// Blocks are the structural elements of a document, containing
/// either other blocks (containers) or text content (leaves).
///
/// # Serialization
///
/// Blocks serialize as JSON objects with a `"type"` field. Core block types
/// use camelCase names (e.g., `"paragraph"`, `"codeBlock"`). Extension blocks
/// use colon-delimited types (e.g., `"forms:textInput"`, `"academic:theorem"`).
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    /// Standard paragraph block.
    Paragraph {
        /// Optional unique identifier.
        id: Option<String>,

        /// Text content.
        children: Vec<Text>,

        /// Block attributes.
        attributes: BlockAttributes,
    },

    /// Section heading (levels 1-6).
    Heading {
        /// Optional unique identifier.
        id: Option<String>,

        /// Heading level (1-6).
        level: u8,

        /// Text content.
        children: Vec<Text>,

        /// Block attributes.
        attributes: BlockAttributes,
    },

    /// Ordered or unordered list.
    List {
        /// Optional unique identifier.
        id: Option<String>,

        /// Whether the list is ordered (numbered).
        ordered: bool,

        /// Starting number for ordered lists.
        start: Option<u32>,

        /// List items (must be `ListItem` blocks).
        children: Vec<Block>,

        /// Block attributes.
        attributes: BlockAttributes,
    },

    /// Item within a list.
    ListItem {
        /// Optional unique identifier.
        id: Option<String>,

        /// Checkbox state (None = not a checkbox).
        checked: Option<bool>,

        /// Block content.
        children: Vec<Block>,

        /// Block attributes.
        attributes: BlockAttributes,
    },

    /// Quoted content block.
    Blockquote {
        /// Optional unique identifier.
        id: Option<String>,

        /// Block content.
        children: Vec<Block>,

        /// Block attributes.
        attributes: BlockAttributes,
    },

    /// Source code or preformatted text.
    CodeBlock {
        /// Optional unique identifier.
        id: Option<String>,

        /// Programming language identifier.
        language: Option<String>,

        /// Syntax highlighting theme.
        highlighting: Option<String>,

        /// Pre-tokenized syntax highlighting.
        tokens: Option<Vec<CodeToken>>,

        /// Code content (single text node, no marks).
        children: Vec<Text>,

        /// Block attributes.
        attributes: BlockAttributes,
    },

    /// Thematic break between sections.
    HorizontalRule {
        /// Optional unique identifier.
        id: Option<String>,
    },

    /// Embedded or referenced image.
    Image(ImageBlock),

    /// Tabular data.
    Table {
        /// Optional unique identifier.
        id: Option<String>,

        /// Table rows.
        children: Vec<Block>,

        /// Block attributes.
        attributes: BlockAttributes,
    },

    /// Row within a table.
    TableRow {
        /// Optional unique identifier.
        id: Option<String>,

        /// Whether this is a header row.
        header: bool,

        /// Table cells.
        children: Vec<Block>,

        /// Block attributes.
        attributes: BlockAttributes,
    },

    /// Cell within a table row.
    TableCell(TableCellBlock),

    /// Mathematical content.
    Math(MathBlock),

    /// Line break within a block.
    Break {
        /// Optional unique identifier.
        id: Option<String>,
    },

    /// Definition list.
    DefinitionList(DefinitionListBlock),

    /// Definition item (term + description pair).
    DefinitionItem {
        /// Optional unique identifier.
        id: Option<String>,

        /// Children (typically `DefinitionTerm` and `DefinitionDescription`).
        children: Vec<Block>,

        /// Block attributes.
        attributes: BlockAttributes,
    },

    /// Definition term.
    DefinitionTerm {
        /// Optional unique identifier.
        id: Option<String>,

        /// Term text content.
        children: Vec<Text>,

        /// Block attributes.
        attributes: BlockAttributes,
    },

    /// Definition description.
    DefinitionDescription {
        /// Optional unique identifier.
        id: Option<String>,

        /// Description content (blocks).
        children: Vec<Block>,

        /// Block attributes.
        attributes: BlockAttributes,
    },

    /// Scientific/technical measurement.
    Measurement(MeasurementBlock),

    /// Block-level signature.
    Signature(SignatureBlock),

    /// SVG image.
    Svg(SvgBlock),

    /// Barcode (QR, Data Matrix, etc.).
    Barcode(BarcodeBlock),

    /// Figure container.
    Figure(FigureBlock),

    /// Figure caption.
    FigCaption(FigCaptionBlock),

    /// Admonition block (note, warning, tip, etc.).
    Admonition(AdmonitionBlock),

    /// Extension block for custom/unknown block types.
    ///
    /// Extension blocks use namespaced types like "forms:textInput" or
    /// "semantic:citation". When parsing, unknown types are preserved
    /// as extension blocks with their raw attributes intact.
    Extension(ExtensionBlock),
}

/// Image block content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageBlock {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Image source (path or URL).
    pub src: String,

    /// Alternative text for accessibility.
    pub alt: String,

    /// Image title/caption.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Intrinsic width in pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,

    /// Intrinsic height in pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
}

/// Table cell block content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableCellBlock {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Number of columns to span.
    #[serde(default = "default_span", skip_serializing_if = "is_default_span")]
    pub colspan: u32,

    /// Number of rows to span.
    #[serde(default = "default_span", skip_serializing_if = "is_default_span")]
    pub rowspan: u32,

    /// Text alignment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub align: Option<CellAlign>,

    /// Cell content.
    pub children: Vec<Text>,

    /// Block attributes.
    #[serde(default, skip_serializing_if = "BlockAttributes::is_empty")]
    pub attributes: BlockAttributes,
}

fn default_span() -> u32 {
    1
}

#[allow(clippy::trivially_copy_pass_by_ref)] // Required by serde skip_serializing_if
fn is_default_span(span: &u32) -> bool {
    *span == 1
}

/// Cell text alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CellAlign {
    /// Left alignment.
    Left,
    /// Center alignment.
    Center,
    /// Right alignment.
    Right,
}

/// Mathematical content block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MathBlock {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Display mode (true) vs inline (false).
    pub display: bool,

    /// Math format.
    pub format: MathFormat,

    /// Math content in the specified format.
    pub value: String,
}

/// Mathematical content format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MathFormat {
    /// LaTeX format.
    Latex,
    /// `MathML` format.
    Mathml,
}

/// Writing mode for text direction.
///
/// Controls the direction in which text flows within a block.
/// This is particularly important for CJK (Chinese, Japanese, Korean)
/// languages which can be written vertically.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WritingMode {
    /// Horizontal text, top-to-bottom block flow (default).
    /// Used for Latin, Cyrillic, Arabic, Hebrew scripts.
    #[default]
    HorizontalTb,

    /// Vertical text, right-to-left block flow.
    /// Traditional Chinese, Japanese, Korean.
    VerticalRl,

    /// Vertical text, left-to-right block flow.
    /// Used for Mongolian script.
    VerticalLr,

    /// Sideways text, right-to-left (90° clockwise rotation).
    SidewaysRl,

    /// Sideways text, left-to-right (90° counter-clockwise rotation).
    SidewaysLr,
}

/// Measurement block for scientific/technical values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeasurementBlock {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// The numeric value.
    pub value: f64,

    /// Uncertainty value (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uncertainty: Option<f64>,

    /// How to display uncertainty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uncertainty_notation: Option<UncertaintyNotation>,

    /// Scientific notation exponent (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exponent: Option<i32>,

    /// Display format (required).
    pub display: String,

    /// Unit of measurement (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
}

impl MeasurementBlock {
    /// Create a new measurement block.
    #[must_use]
    pub fn new(value: f64, display: impl Into<String>) -> Self {
        Self {
            id: None,
            value,
            uncertainty: None,
            uncertainty_notation: None,
            exponent: None,
            display: display.into(),
            unit: None,
        }
    }

    /// Set the unit.
    #[must_use]
    pub fn with_unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// Set the uncertainty.
    #[must_use]
    pub fn with_uncertainty(mut self, uncertainty: f64, notation: UncertaintyNotation) -> Self {
        self.uncertainty = Some(uncertainty);
        self.uncertainty_notation = Some(notation);
        self
    }

    /// Set the exponent.
    #[must_use]
    pub fn with_exponent(mut self, exponent: i32) -> Self {
        self.exponent = Some(exponent);
        self
    }
}

/// Uncertainty notation format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UncertaintyNotation {
    /// Parenthetical notation, e.g., 1.234(5).
    Parenthetical,
    /// Plus-minus notation, e.g., 1.234 ± 0.005.
    Plusminus,
    /// Range notation, e.g., 1.229–1.239.
    Range,
    /// Percentage notation, e.g., 1.234 ± 0.4%.
    Percent,
}

/// Block signature for attestation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureBlock {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Type of signature.
    pub signature_type: BlockSignatureType,

    /// Signer information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signer: Option<SignerDetails>,

    /// Timestamp of signature.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,

    /// Purpose of signature.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purpose: Option<SignaturePurpose>,

    /// Reference to digital signature if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digital_signature_ref: Option<String>,
}

impl SignatureBlock {
    /// Create a new signature block.
    #[must_use]
    pub fn new(signature_type: BlockSignatureType) -> Self {
        Self {
            id: None,
            signature_type,
            signer: None,
            timestamp: None,
            purpose: None,
            digital_signature_ref: None,
        }
    }

    /// Set the signer details.
    #[must_use]
    pub fn with_signer(mut self, signer: SignerDetails) -> Self {
        self.signer = Some(signer);
        self
    }

    /// Set the purpose.
    #[must_use]
    pub fn with_purpose(mut self, purpose: SignaturePurpose) -> Self {
        self.purpose = Some(purpose);
        self
    }
}

/// Type of block-level signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlockSignatureType {
    /// Handwritten signature.
    Handwritten,
    /// Digital cryptographic signature.
    Digital,
    /// Electronic signature (e.g., typed name).
    Electronic,
    /// Stamp or seal.
    Stamp,
}

/// Purpose of a signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SignaturePurpose {
    /// Certification of accuracy.
    Certification,
    /// Approval of content.
    Approval,
    /// Witnessing.
    Witness,
    /// Acknowledgment of receipt/understanding.
    Acknowledgment,
    /// Authorship attribution.
    Authorship,
}

/// Signer details for signatures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignerDetails {
    /// Signer's name.
    pub name: String,

    /// Signer's title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Signer's organization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,

    /// Signer's email.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Signer's unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

impl SignerDetails {
    /// Create new signer details.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            title: None,
            organization: None,
            email: None,
            id: None,
        }
    }

    /// Set the title.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the organization.
    #[must_use]
    pub fn with_organization(mut self, organization: impl Into<String>) -> Self {
        self.organization = Some(organization.into());
        self
    }
}

/// SVG image block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SvgBlock {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Source reference (mutually exclusive with content).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,

    /// Inline SVG content (mutually exclusive with src).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Width.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,

    /// Height.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,

    /// Alternative text for accessibility.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,
}

impl SvgBlock {
    /// Create an SVG block from a source reference.
    #[must_use]
    pub fn from_src(src: impl Into<String>) -> Self {
        Self {
            id: None,
            src: Some(src.into()),
            content: None,
            width: None,
            height: None,
            alt: None,
        }
    }

    /// Create an SVG block from inline content.
    #[must_use]
    pub fn from_content(content: impl Into<String>) -> Self {
        Self {
            id: None,
            src: None,
            content: Some(content.into()),
            width: None,
            height: None,
            alt: None,
        }
    }

    /// Set the alt text.
    #[must_use]
    pub fn with_alt(mut self, alt: impl Into<String>) -> Self {
        self.alt = Some(alt.into());
        self
    }

    /// Set dimensions.
    #[must_use]
    pub fn with_dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }
}

/// Barcode block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BarcodeBlock {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Barcode format.
    pub format: BarcodeFormat,

    /// Encoded data.
    pub data: String,

    /// Error correction level (for QR codes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_correction: Option<ErrorCorrectionLevel>,

    /// Size specification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<BarcodeSize>,

    /// Quiet zone around barcode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quiet_zone: Option<String>,

    /// Alternative text (required for accessibility).
    pub alt: String,
}

impl BarcodeBlock {
    /// Create a new barcode block.
    #[must_use]
    pub fn new(format: BarcodeFormat, data: impl Into<String>, alt: impl Into<String>) -> Self {
        Self {
            id: None,
            format,
            data: data.into(),
            error_correction: None,
            size: None,
            quiet_zone: None,
            alt: alt.into(),
        }
    }

    /// Set error correction level.
    #[must_use]
    pub fn with_error_correction(mut self, level: ErrorCorrectionLevel) -> Self {
        self.error_correction = Some(level);
        self
    }

    /// Set the size.
    #[must_use]
    pub fn with_size(mut self, width: String, height: String) -> Self {
        self.size = Some(BarcodeSize { width, height });
        self
    }
}

/// Barcode format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BarcodeFormat {
    /// QR Code.
    Qr,
    /// Data Matrix.
    DataMatrix,
    /// Code 128.
    Code128,
    /// Code 39.
    Code39,
    /// EAN-13.
    Ean13,
    /// EAN-8.
    Ean8,
    /// UPC-A.
    UpcA,
    /// PDF417.
    Pdf417,
}

/// Error correction level for barcodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCorrectionLevel {
    /// Low (~7% recovery).
    L,
    /// Medium (~15% recovery).
    M,
    /// Quartile (~25% recovery).
    Q,
    /// High (~30% recovery).
    H,
}

/// Barcode size specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BarcodeSize {
    /// Width with units.
    pub width: String,
    /// Height with units.
    pub height: String,
}

/// Figure block (container for images/diagrams with captions).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FigureBlock {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Figure numbering mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numbering: Option<FigureNumbering>,

    /// Subfigures within this figure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subfigures: Option<Vec<Subfigure>>,

    /// Figure contents (usually image, svg, or other visual block).
    pub children: Vec<Block>,

    /// Block attributes.
    #[serde(default, skip_serializing_if = "BlockAttributes::is_empty")]
    pub attributes: BlockAttributes,
}

impl FigureBlock {
    /// Create a new figure block.
    #[must_use]
    pub fn new(children: Vec<Block>) -> Self {
        Self {
            id: None,
            numbering: None,
            subfigures: None,
            children,
            attributes: BlockAttributes::default(),
        }
    }
}

/// Figure numbering mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FigureNumbering {
    /// Automatic numbering.
    Auto,
    /// No numbering.
    #[serde(rename = "none")]
    Unnumbered,
    /// Explicit number.
    #[serde(untagged)]
    Number(u32),
}

/// A subfigure within a figure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Subfigure {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Optional label (e.g., "(a)", "(b)").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// Subfigure content blocks.
    pub children: Vec<Block>,
}

/// A pre-tokenized syntax highlighting token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeToken {
    /// Token type (e.g., "keyword", "string", "comment").
    pub token_type: String,

    /// Token value text.
    pub value: String,

    /// Optional scope name for the token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// Figure caption block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FigCaptionBlock {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Caption text content.
    pub children: Vec<Text>,

    /// Block attributes.
    #[serde(default, skip_serializing_if = "BlockAttributes::is_empty")]
    pub attributes: BlockAttributes,
}

impl FigCaptionBlock {
    /// Create a new figure caption.
    #[must_use]
    pub fn new(children: Vec<Text>) -> Self {
        Self {
            id: None,
            children,
            attributes: BlockAttributes::default(),
        }
    }
}

/// Admonition block for callout boxes (note, warning, tip, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdmonitionBlock {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Admonition variant (note, tip, warning, etc.).
    pub variant: AdmonitionVariant,

    /// Optional title (if not provided, variant name is used).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Content blocks within the admonition.
    pub children: Vec<Block>,

    /// Block attributes.
    #[serde(default, skip_serializing_if = "BlockAttributes::is_empty")]
    pub attributes: BlockAttributes,
}

impl AdmonitionBlock {
    /// Create a new admonition block.
    #[must_use]
    pub fn new(variant: AdmonitionVariant, children: Vec<Block>) -> Self {
        Self {
            id: None,
            variant,
            title: None,
            children,
            attributes: BlockAttributes::default(),
        }
    }

    /// Set the admonition title.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the block ID.
    #[must_use]
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

/// Admonition variant/type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
pub enum AdmonitionVariant {
    /// General note or information.
    Note,
    /// Helpful tip.
    Tip,
    /// Informational notice.
    Info,
    /// Warning about potential issues.
    Warning,
    /// Caution about important considerations.
    Caution,
    /// Danger alert for critical issues.
    Danger,
    /// Important notice requiring attention.
    Important,
    /// Example content.
    Example,
}

/// Definition list block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefinitionListBlock {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Definition items (must be `DefinitionItem` blocks).
    pub children: Vec<Block>,

    /// Block attributes.
    #[serde(default, skip_serializing_if = "BlockAttributes::is_empty")]
    pub attributes: BlockAttributes,
}

impl DefinitionListBlock {
    /// Create a new definition list.
    #[must_use]
    pub fn new(children: Vec<Block>) -> Self {
        Self {
            id: None,
            children,
            attributes: BlockAttributes::default(),
        }
    }
}

/// Helper: serialize a Block variant by converting to Value, injecting "type", then serializing.
fn serialize_block_as_map<S: serde::Serializer>(
    type_str: &str,
    value: &serde_json::Value,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use serde::ser::Error;
    let obj = value
        .as_object()
        .ok_or_else(|| S::Error::custom("expected object"))?;
    let mut map = serializer.serialize_map(Some(1 + obj.len()))?;
    map.serialize_entry("type", type_str)?;
    for (k, v) in obj {
        map.serialize_entry(k, v)?;
    }
    map.end()
}

/// Helper struct for serializing inline struct variants (Paragraph, Heading, etc.).
/// These don't have their own named struct, so we serialize field-by-field.
#[derive(Serialize)]
struct InlineParagraph<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: &'a Option<String>,
    children: &'a Vec<Text>,
    #[serde(skip_serializing_if = "BlockAttributes::is_empty")]
    attributes: &'a BlockAttributes,
}

#[derive(Serialize)]
struct InlineHeading<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: &'a Option<String>,
    level: u8,
    children: &'a Vec<Text>,
    #[serde(skip_serializing_if = "BlockAttributes::is_empty")]
    attributes: &'a BlockAttributes,
}

#[derive(Serialize)]
struct InlineList<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: &'a Option<String>,
    ordered: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    start: &'a Option<u32>,
    children: &'a Vec<Block>,
    #[serde(skip_serializing_if = "BlockAttributes::is_empty")]
    attributes: &'a BlockAttributes,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InlineListItem<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: &'a Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    checked: &'a Option<bool>,
    children: &'a Vec<Block>,
    #[serde(skip_serializing_if = "BlockAttributes::is_empty")]
    attributes: &'a BlockAttributes,
}

#[derive(Serialize)]
struct InlineContainer<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: &'a Option<String>,
    children: &'a Vec<Block>,
    #[serde(skip_serializing_if = "BlockAttributes::is_empty")]
    attributes: &'a BlockAttributes,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InlineCodeBlock<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: &'a Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: &'a Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    highlighting: &'a Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tokens: &'a Option<Vec<CodeToken>>,
    children: &'a Vec<Text>,
    #[serde(skip_serializing_if = "BlockAttributes::is_empty")]
    attributes: &'a BlockAttributes,
}

#[derive(Serialize)]
struct InlineTableRow<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: &'a Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    header: bool,
    children: &'a Vec<Block>,
    #[serde(skip_serializing_if = "BlockAttributes::is_empty")]
    attributes: &'a BlockAttributes,
}

#[derive(Serialize)]
struct InlineTextContainer<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: &'a Option<String>,
    children: &'a Vec<Text>,
    #[serde(skip_serializing_if = "BlockAttributes::is_empty")]
    attributes: &'a BlockAttributes,
}

#[derive(Serialize)]
struct InlineIdOnly<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: &'a Option<String>,
}

impl Serialize for Block {
    #[allow(clippy::too_many_lines)] // mechanical match over 20+ block variants — splitting would obscure the dispatch
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::Error;

        match self {
            Self::Paragraph {
                id,
                children,
                attributes,
            } => {
                let inner = InlineParagraph {
                    id,
                    children,
                    attributes,
                };
                let val = serde_json::to_value(&inner).map_err(S::Error::custom)?;
                serialize_block_as_map("paragraph", &val, serializer)
            }
            Self::Heading {
                id,
                level,
                children,
                attributes,
            } => {
                let inner = InlineHeading {
                    id,
                    level: *level,
                    children,
                    attributes,
                };
                let val = serde_json::to_value(&inner).map_err(S::Error::custom)?;
                serialize_block_as_map("heading", &val, serializer)
            }
            Self::List {
                id,
                ordered,
                start,
                children,
                attributes,
            } => {
                let inner = InlineList {
                    id,
                    ordered: *ordered,
                    start,
                    children,
                    attributes,
                };
                let val = serde_json::to_value(&inner).map_err(S::Error::custom)?;
                serialize_block_as_map("list", &val, serializer)
            }
            Self::ListItem {
                id,
                checked,
                children,
                attributes,
            } => {
                let inner = InlineListItem {
                    id,
                    checked,
                    children,
                    attributes,
                };
                let val = serde_json::to_value(&inner).map_err(S::Error::custom)?;
                serialize_block_as_map("listItem", &val, serializer)
            }
            Self::Blockquote {
                id,
                children,
                attributes,
            } => {
                let inner = InlineContainer {
                    id,
                    children,
                    attributes,
                };
                let val = serde_json::to_value(&inner).map_err(S::Error::custom)?;
                serialize_block_as_map("blockquote", &val, serializer)
            }
            Self::CodeBlock {
                id,
                language,
                highlighting,
                tokens,
                children,
                attributes,
            } => {
                let inner = InlineCodeBlock {
                    id,
                    language,
                    highlighting,
                    tokens,
                    children,
                    attributes,
                };
                let val = serde_json::to_value(&inner).map_err(S::Error::custom)?;
                serialize_block_as_map("codeBlock", &val, serializer)
            }
            Self::HorizontalRule { id } => {
                let inner = InlineIdOnly { id };
                let val = serde_json::to_value(&inner).map_err(S::Error::custom)?;
                serialize_block_as_map("horizontalRule", &val, serializer)
            }
            Self::Image(img) => {
                let val = serde_json::to_value(img).map_err(S::Error::custom)?;
                serialize_block_as_map("image", &val, serializer)
            }
            Self::Table {
                id,
                children,
                attributes,
            } => {
                let inner = InlineContainer {
                    id,
                    children,
                    attributes,
                };
                let val = serde_json::to_value(&inner).map_err(S::Error::custom)?;
                serialize_block_as_map("table", &val, serializer)
            }
            Self::TableRow {
                id,
                header,
                children,
                attributes,
            } => {
                let inner = InlineTableRow {
                    id,
                    header: *header,
                    children,
                    attributes,
                };
                let val = serde_json::to_value(&inner).map_err(S::Error::custom)?;
                serialize_block_as_map("tableRow", &val, serializer)
            }
            Self::TableCell(cell) => {
                let val = serde_json::to_value(cell).map_err(S::Error::custom)?;
                serialize_block_as_map("tableCell", &val, serializer)
            }
            Self::Math(math) => {
                let val = serde_json::to_value(math).map_err(S::Error::custom)?;
                serialize_block_as_map("math", &val, serializer)
            }
            Self::Break { id } => {
                let inner = InlineIdOnly { id };
                let val = serde_json::to_value(&inner).map_err(S::Error::custom)?;
                serialize_block_as_map("break", &val, serializer)
            }
            Self::DefinitionList(dl) => {
                let val = serde_json::to_value(dl).map_err(S::Error::custom)?;
                serialize_block_as_map("definitionList", &val, serializer)
            }
            Self::DefinitionItem {
                id,
                children,
                attributes,
            } => {
                let inner = InlineContainer {
                    id,
                    children,
                    attributes,
                };
                let val = serde_json::to_value(&inner).map_err(S::Error::custom)?;
                serialize_block_as_map("definitionItem", &val, serializer)
            }
            Self::DefinitionTerm {
                id,
                children,
                attributes,
            } => {
                let inner = InlineTextContainer {
                    id,
                    children,
                    attributes,
                };
                let val = serde_json::to_value(&inner).map_err(S::Error::custom)?;
                serialize_block_as_map("definitionTerm", &val, serializer)
            }
            Self::DefinitionDescription {
                id,
                children,
                attributes,
            } => {
                let inner = InlineContainer {
                    id,
                    children,
                    attributes,
                };
                let val = serde_json::to_value(&inner).map_err(S::Error::custom)?;
                serialize_block_as_map("definitionDescription", &val, serializer)
            }
            Self::Measurement(m) => {
                let val = serde_json::to_value(m).map_err(S::Error::custom)?;
                serialize_block_as_map("measurement", &val, serializer)
            }
            Self::Signature(sig) => {
                let val = serde_json::to_value(sig).map_err(S::Error::custom)?;
                serialize_block_as_map("signature", &val, serializer)
            }
            Self::Svg(svg) => {
                let val = serde_json::to_value(svg).map_err(S::Error::custom)?;
                serialize_block_as_map("svg", &val, serializer)
            }
            Self::Barcode(bc) => {
                let val = serde_json::to_value(bc).map_err(S::Error::custom)?;
                serialize_block_as_map("barcode", &val, serializer)
            }
            Self::Figure(fig) => {
                let val = serde_json::to_value(fig).map_err(S::Error::custom)?;
                serialize_block_as_map("figure", &val, serializer)
            }
            Self::FigCaption(fc) => {
                let val = serde_json::to_value(fc).map_err(S::Error::custom)?;
                serialize_block_as_map("figcaption", &val, serializer)
            }
            Self::Admonition(adm) => {
                let val = serde_json::to_value(adm).map_err(S::Error::custom)?;
                serialize_block_as_map("admonition", &val, serializer)
            }
            Self::Extension(ext) => {
                // Extension blocks: type is "namespace:blockType", flatten id/children/fallback/attributes
                let type_str = ext.full_type();
                let attr_count = ext.attributes.as_object().map_or(0, serde_json::Map::len);
                let mut count = 1; // type
                if ext.id.is_some() {
                    count += 1;
                }
                if !ext.children.is_empty() {
                    count += 1;
                }
                if ext.fallback.is_some() {
                    count += 1;
                }
                count += attr_count;

                let mut map = serializer.serialize_map(Some(count))?;
                map.serialize_entry("type", &type_str)?;
                if let Some(id) = &ext.id {
                    map.serialize_entry("id", id)?;
                }
                if !ext.children.is_empty() {
                    map.serialize_entry("children", &ext.children)?;
                }
                if let Some(fallback) = &ext.fallback {
                    map.serialize_entry("fallback", fallback)?;
                }
                if let Some(obj) = ext.attributes.as_object() {
                    for (k, v) in obj {
                        map.serialize_entry(k, v)?;
                    }
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Block {
    #[allow(clippy::too_many_lines)] // mechanical match over 20+ block type strings — splitting would obscure the dispatch
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Deserialize into a generic Value first, then dispatch based on "type"
        let mut value = serde_json::Value::deserialize(deserializer)?;

        let obj = value
            .as_object_mut()
            .ok_or_else(|| de::Error::custom("block must be an object"))?;

        let type_str = obj
            .get("type")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| de::Error::missing_field("type"))?
            .to_string();

        // Remove "type" before deserializing into inner structs (they don't expect it)
        obj.remove("type");

        match type_str.as_str() {
            "paragraph" => {
                #[derive(Deserialize)]
                struct Inner {
                    #[serde(default)]
                    id: Option<String>,
                    #[serde(default)]
                    children: Vec<Text>,
                    #[serde(default)]
                    attributes: BlockAttributes,
                }
                let inner: Inner = serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Block::Paragraph {
                    id: inner.id,
                    children: inner.children,
                    attributes: inner.attributes,
                })
            }
            "heading" => {
                #[derive(Deserialize)]
                struct Inner {
                    #[serde(default)]
                    id: Option<String>,
                    level: u8,
                    #[serde(default)]
                    children: Vec<Text>,
                    #[serde(default)]
                    attributes: BlockAttributes,
                }
                let inner: Inner = serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Block::Heading {
                    id: inner.id,
                    level: inner.level.clamp(1, 6),
                    children: inner.children,
                    attributes: inner.attributes,
                })
            }
            "list" => {
                #[derive(Deserialize)]
                struct Inner {
                    #[serde(default)]
                    id: Option<String>,
                    #[serde(default)]
                    ordered: bool,
                    #[serde(default)]
                    start: Option<u32>,
                    #[serde(default)]
                    children: Vec<Block>,
                    #[serde(default)]
                    attributes: BlockAttributes,
                }
                let inner: Inner = serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Block::List {
                    id: inner.id,
                    ordered: inner.ordered,
                    start: inner.start,
                    children: inner.children,
                    attributes: inner.attributes,
                })
            }
            "listItem" => {
                #[derive(Deserialize)]
                struct Inner {
                    #[serde(default)]
                    id: Option<String>,
                    #[serde(default)]
                    checked: Option<bool>,
                    #[serde(default)]
                    children: Vec<Block>,
                    #[serde(default)]
                    attributes: BlockAttributes,
                }
                let inner: Inner = serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Block::ListItem {
                    id: inner.id,
                    checked: inner.checked,
                    children: inner.children,
                    attributes: inner.attributes,
                })
            }
            "blockquote" => {
                #[derive(Deserialize)]
                struct Inner {
                    #[serde(default)]
                    id: Option<String>,
                    #[serde(default)]
                    children: Vec<Block>,
                    #[serde(default)]
                    attributes: BlockAttributes,
                }
                let inner: Inner = serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Block::Blockquote {
                    id: inner.id,
                    children: inner.children,
                    attributes: inner.attributes,
                })
            }
            "codeBlock" => {
                #[derive(Deserialize)]
                struct Inner {
                    #[serde(default)]
                    id: Option<String>,
                    #[serde(default)]
                    language: Option<String>,
                    #[serde(default)]
                    highlighting: Option<String>,
                    #[serde(default)]
                    tokens: Option<Vec<CodeToken>>,
                    #[serde(default)]
                    children: Vec<Text>,
                    #[serde(default)]
                    attributes: BlockAttributes,
                }
                let inner: Inner = serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Block::CodeBlock {
                    id: inner.id,
                    language: inner.language,
                    highlighting: inner.highlighting,
                    tokens: inner.tokens,
                    children: inner.children,
                    attributes: inner.attributes,
                })
            }
            "horizontalRule" => {
                #[derive(Deserialize)]
                struct Inner {
                    #[serde(default)]
                    id: Option<String>,
                }
                let inner: Inner = serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Block::HorizontalRule { id: inner.id })
            }
            "image" => {
                let img: ImageBlock = serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Block::Image(img))
            }
            "table" => {
                #[derive(Deserialize)]
                struct Inner {
                    #[serde(default)]
                    id: Option<String>,
                    #[serde(default)]
                    children: Vec<Block>,
                    #[serde(default)]
                    attributes: BlockAttributes,
                }
                let inner: Inner = serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Block::Table {
                    id: inner.id,
                    children: inner.children,
                    attributes: inner.attributes,
                })
            }
            "tableRow" => {
                #[derive(Deserialize)]
                struct Inner {
                    #[serde(default)]
                    id: Option<String>,
                    #[serde(default)]
                    header: bool,
                    #[serde(default)]
                    children: Vec<Block>,
                    #[serde(default)]
                    attributes: BlockAttributes,
                }
                let inner: Inner = serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Block::TableRow {
                    id: inner.id,
                    header: inner.header,
                    children: inner.children,
                    attributes: inner.attributes,
                })
            }
            "tableCell" => {
                let cell: TableCellBlock =
                    serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Block::TableCell(cell))
            }
            "math" => {
                let math: MathBlock = serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Block::Math(math))
            }
            "break" => {
                #[derive(Deserialize)]
                struct Inner {
                    #[serde(default)]
                    id: Option<String>,
                }
                let inner: Inner = serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Block::Break { id: inner.id })
            }
            "definitionList" => {
                let dl: DefinitionListBlock =
                    serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Block::DefinitionList(dl))
            }
            "definitionItem" => {
                #[derive(Deserialize)]
                struct Inner {
                    #[serde(default)]
                    id: Option<String>,
                    #[serde(default)]
                    children: Vec<Block>,
                    #[serde(default)]
                    attributes: BlockAttributes,
                }
                let inner: Inner = serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Block::DefinitionItem {
                    id: inner.id,
                    children: inner.children,
                    attributes: inner.attributes,
                })
            }
            "definitionTerm" => {
                #[derive(Deserialize)]
                struct Inner {
                    #[serde(default)]
                    id: Option<String>,
                    #[serde(default)]
                    children: Vec<Text>,
                    #[serde(default)]
                    attributes: BlockAttributes,
                }
                let inner: Inner = serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Block::DefinitionTerm {
                    id: inner.id,
                    children: inner.children,
                    attributes: inner.attributes,
                })
            }
            "definitionDescription" => {
                #[derive(Deserialize)]
                struct Inner {
                    #[serde(default)]
                    id: Option<String>,
                    #[serde(default)]
                    children: Vec<Block>,
                    #[serde(default)]
                    attributes: BlockAttributes,
                }
                let inner: Inner = serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Block::DefinitionDescription {
                    id: inner.id,
                    children: inner.children,
                    attributes: inner.attributes,
                })
            }
            "measurement" => {
                let m: MeasurementBlock =
                    serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Block::Measurement(m))
            }
            "signature" => {
                let sig: SignatureBlock =
                    serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Block::Signature(sig))
            }
            "svg" => {
                let svg: SvgBlock = serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Block::Svg(svg))
            }
            "barcode" => {
                let bc: BarcodeBlock = serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Block::Barcode(bc))
            }
            "figure" => {
                let fig: FigureBlock = serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Block::Figure(fig))
            }
            "figcaption" | "figCaption" => {
                // Accept both "figcaption" (spec) and "figCaption" (old format)
                let fc: FigCaptionBlock =
                    serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Block::FigCaption(fc))
            }
            "admonition" => {
                let adm: AdmonitionBlock =
                    serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Block::Admonition(adm))
            }

            // Old format backward compat: {"type": "extension", "namespace": "...", "blockType": "..."}
            "extension" => {
                let ext: ExtensionBlock =
                    serde_json::from_value(value).map_err(de::Error::custom)?;
                Ok(Block::Extension(ext))
            }

            // Colon-delimited extension type (new format)
            other if other.contains(':') => {
                let (namespace, block_type) = other.split_once(':').unwrap();
                let obj = value
                    .as_object()
                    .ok_or_else(|| de::Error::custom("expected object"))?;

                let id = obj
                    .get("id")
                    .and_then(serde_json::Value::as_str)
                    .map(ToString::to_string);
                let children: Vec<Block> = obj
                    .get("children")
                    .map(|v| serde_json::from_value(v.clone()))
                    .transpose()
                    .map_err(de::Error::custom)?
                    .unwrap_or_default();
                let fallback: Option<Box<Block>> = obj
                    .get("fallback")
                    .map(|v| serde_json::from_value(v.clone()))
                    .transpose()
                    .map_err(de::Error::custom)?;

                // Collect remaining keys as attributes
                let reserved = ["id", "children", "fallback"];
                let mut attrs = serde_json::Map::new();
                for (k, v) in obj {
                    if !reserved.contains(&k.as_str()) {
                        attrs.insert(k.clone(), v.clone());
                    }
                }
                let attributes = if attrs.is_empty() {
                    serde_json::Value::Null
                } else {
                    serde_json::Value::Object(attrs)
                };

                Ok(Block::Extension(ExtensionBlock {
                    namespace: namespace.to_string(),
                    block_type: block_type.to_string(),
                    id,
                    attributes,
                    children,
                    fallback,
                }))
            }

            unknown => Err(de::Error::custom(format!("unknown block type: {unknown}"))),
        }
    }
}

// Convenience constructors
impl Block {
    /// Create a paragraph block.
    #[must_use]
    pub fn paragraph(children: Vec<Text>) -> Self {
        Self::Paragraph {
            id: None,
            children,
            attributes: BlockAttributes::default(),
        }
    }

    /// Create a heading block.
    #[must_use]
    pub fn heading(level: u8, children: Vec<Text>) -> Self {
        Self::Heading {
            id: None,
            level: level.clamp(1, 6),
            children,
            attributes: BlockAttributes::default(),
        }
    }

    /// Create an unordered list.
    #[must_use]
    pub fn unordered_list(items: Vec<Block>) -> Self {
        Self::List {
            id: None,
            ordered: false,
            start: None,
            children: items,
            attributes: BlockAttributes::default(),
        }
    }

    /// Create an ordered list.
    #[must_use]
    pub fn ordered_list(items: Vec<Block>) -> Self {
        Self::List {
            id: None,
            ordered: true,
            start: None,
            children: items,
            attributes: BlockAttributes::default(),
        }
    }

    /// Create a list item.
    #[must_use]
    pub fn list_item(children: Vec<Block>) -> Self {
        Self::ListItem {
            id: None,
            checked: None,
            children,
            attributes: BlockAttributes::default(),
        }
    }

    /// Create a checkbox list item.
    #[must_use]
    pub fn checkbox(checked: bool, children: Vec<Block>) -> Self {
        Self::ListItem {
            id: None,
            checked: Some(checked),
            children,
            attributes: BlockAttributes::default(),
        }
    }

    /// Create a blockquote.
    #[must_use]
    pub fn blockquote(children: Vec<Block>) -> Self {
        Self::Blockquote {
            id: None,
            children,
            attributes: BlockAttributes::default(),
        }
    }

    /// Create a code block.
    #[must_use]
    pub fn code_block(code: impl Into<String>, language: Option<String>) -> Self {
        Self::CodeBlock {
            id: None,
            language,
            highlighting: None,
            tokens: None,
            children: vec![Text::plain(code)],
            attributes: BlockAttributes::default(),
        }
    }

    /// Create a horizontal rule.
    #[must_use]
    pub fn horizontal_rule() -> Self {
        Self::HorizontalRule { id: None }
    }

    /// Create an image block.
    #[must_use]
    pub fn image(src: impl Into<String>, alt: impl Into<String>) -> Self {
        Self::Image(ImageBlock {
            id: None,
            src: src.into(),
            alt: alt.into(),
            title: None,
            width: None,
            height: None,
        })
    }

    /// Create a table.
    #[must_use]
    pub fn table(rows: Vec<Block>) -> Self {
        Self::Table {
            id: None,
            children: rows,
            attributes: BlockAttributes::default(),
        }
    }

    /// Create a table row.
    #[must_use]
    pub fn table_row(cells: Vec<Block>, header: bool) -> Self {
        Self::TableRow {
            id: None,
            header,
            children: cells,
            attributes: BlockAttributes::default(),
        }
    }

    /// Create a table cell.
    #[must_use]
    pub fn table_cell(children: Vec<Text>) -> Self {
        Self::TableCell(TableCellBlock {
            id: None,
            colspan: 1,
            rowspan: 1,
            align: None,
            children,
            attributes: BlockAttributes::default(),
        })
    }

    /// Create a math block.
    #[must_use]
    pub fn math(value: impl Into<String>, format: MathFormat, display: bool) -> Self {
        Self::Math(MathBlock {
            id: None,
            display,
            format,
            value: value.into(),
        })
    }

    /// Create a line break.
    #[must_use]
    pub fn line_break() -> Self {
        Self::Break { id: None }
    }

    /// Create a definition list.
    #[must_use]
    pub fn definition_list(items: Vec<Block>) -> Self {
        Self::DefinitionList(DefinitionListBlock::new(items))
    }

    /// Create a definition item.
    #[must_use]
    pub fn definition_item(children: Vec<Block>) -> Self {
        Self::DefinitionItem {
            id: None,
            children,
            attributes: BlockAttributes::default(),
        }
    }

    /// Create a definition term.
    #[must_use]
    pub fn definition_term(children: Vec<Text>) -> Self {
        Self::DefinitionTerm {
            id: None,
            children,
            attributes: BlockAttributes::default(),
        }
    }

    /// Create a definition description.
    #[must_use]
    pub fn definition_description(children: Vec<Block>) -> Self {
        Self::DefinitionDescription {
            id: None,
            children,
            attributes: BlockAttributes::default(),
        }
    }

    /// Create a measurement block.
    #[must_use]
    pub fn measurement(value: f64, display: impl Into<String>) -> Self {
        Self::Measurement(MeasurementBlock::new(value, display))
    }

    /// Create a signature block.
    #[must_use]
    pub fn signature(signature_type: BlockSignatureType) -> Self {
        Self::Signature(SignatureBlock::new(signature_type))
    }

    /// Create an SVG block from a source reference.
    #[must_use]
    pub fn svg_from_src(src: impl Into<String>) -> Self {
        Self::Svg(SvgBlock::from_src(src))
    }

    /// Create an SVG block from inline content.
    #[must_use]
    pub fn svg_from_content(content: impl Into<String>) -> Self {
        Self::Svg(SvgBlock::from_content(content))
    }

    /// Create a barcode block.
    #[must_use]
    pub fn barcode(format: BarcodeFormat, data: impl Into<String>, alt: impl Into<String>) -> Self {
        Self::Barcode(BarcodeBlock::new(format, data, alt))
    }

    /// Create a figure block.
    #[must_use]
    pub fn figure(children: Vec<Block>) -> Self {
        Self::Figure(FigureBlock::new(children))
    }

    /// Create a figure caption.
    #[must_use]
    pub fn figcaption(children: Vec<Text>) -> Self {
        Self::FigCaption(FigCaptionBlock::new(children))
    }

    /// Create an admonition block.
    #[must_use]
    pub fn admonition(variant: AdmonitionVariant, children: Vec<Block>) -> Self {
        Self::Admonition(AdmonitionBlock::new(variant, children))
    }

    /// Get the block type as a string.
    ///
    /// For core blocks, returns the camelCase type name.
    /// For extension blocks, returns the colon-delimited type (e.g., `"forms:textInput"`).
    #[must_use]
    pub fn block_type(&self) -> Cow<'_, str> {
        match self {
            Self::Paragraph { .. } => Cow::Borrowed("paragraph"),
            Self::Heading { .. } => Cow::Borrowed("heading"),
            Self::List { .. } => Cow::Borrowed("list"),
            Self::ListItem { .. } => Cow::Borrowed("listItem"),
            Self::Blockquote { .. } => Cow::Borrowed("blockquote"),
            Self::CodeBlock { .. } => Cow::Borrowed("codeBlock"),
            Self::HorizontalRule { .. } => Cow::Borrowed("horizontalRule"),
            Self::Image(_) => Cow::Borrowed("image"),
            Self::Table { .. } => Cow::Borrowed("table"),
            Self::TableRow { .. } => Cow::Borrowed("tableRow"),
            Self::TableCell(_) => Cow::Borrowed("tableCell"),
            Self::Math(_) => Cow::Borrowed("math"),
            Self::Break { .. } => Cow::Borrowed("break"),
            Self::DefinitionList(_) => Cow::Borrowed("definitionList"),
            Self::DefinitionItem { .. } => Cow::Borrowed("definitionItem"),
            Self::DefinitionTerm { .. } => Cow::Borrowed("definitionTerm"),
            Self::DefinitionDescription { .. } => Cow::Borrowed("definitionDescription"),
            Self::Measurement(_) => Cow::Borrowed("measurement"),
            Self::Signature(_) => Cow::Borrowed("signature"),
            Self::Svg(_) => Cow::Borrowed("svg"),
            Self::Barcode(_) => Cow::Borrowed("barcode"),
            Self::Figure(_) => Cow::Borrowed("figure"),
            Self::FigCaption(_) => Cow::Borrowed("figcaption"),
            Self::Admonition(_) => Cow::Borrowed("admonition"),
            Self::Extension(ext) => Cow::Owned(ext.full_type()),
        }
    }

    /// Get the block's ID if it has one.
    #[must_use]
    pub fn id(&self) -> Option<&str> {
        match self {
            Self::Paragraph { id, .. }
            | Self::Heading { id, .. }
            | Self::List { id, .. }
            | Self::ListItem { id, .. }
            | Self::Blockquote { id, .. }
            | Self::CodeBlock { id, .. }
            | Self::HorizontalRule { id }
            | Self::Table { id, .. }
            | Self::TableRow { id, .. }
            | Self::Break { id }
            | Self::DefinitionItem { id, .. }
            | Self::DefinitionTerm { id, .. }
            | Self::DefinitionDescription { id, .. } => id.as_deref(),
            Self::Image(img) => img.id.as_deref(),
            Self::TableCell(cell) => cell.id.as_deref(),
            Self::Math(math) => math.id.as_deref(),
            Self::DefinitionList(dl) => dl.id.as_deref(),
            Self::Measurement(m) => m.id.as_deref(),
            Self::Signature(sig) => sig.id.as_deref(),
            Self::Svg(svg) => svg.id.as_deref(),
            Self::Barcode(bc) => bc.id.as_deref(),
            Self::Figure(fig) => fig.id.as_deref(),
            Self::FigCaption(fc) => fc.id.as_deref(),
            Self::Admonition(adm) => adm.id.as_deref(),
            Self::Extension(ext) => ext.id.as_deref(),
        }
    }

    /// Create an extension block.
    #[must_use]
    pub fn extension(namespace: impl Into<String>, block_type: impl Into<String>) -> Self {
        Self::Extension(ExtensionBlock::new(namespace, block_type))
    }

    /// Check if this is an extension block.
    #[must_use]
    pub fn is_extension(&self) -> bool {
        matches!(self, Self::Extension(_))
    }

    /// Get the extension block if this is one.
    #[must_use]
    pub fn as_extension(&self) -> Option<&ExtensionBlock> {
        match self {
            Self::Extension(ext) => Some(ext),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_new() {
        let content = Content::new(vec![Block::paragraph(vec![Text::plain("Hello")])]);
        assert_eq!(content.version, "0.1");
        assert_eq!(content.len(), 1);
        assert!(!content.is_empty());
    }

    #[test]
    fn test_content_empty() {
        let content = Content::empty();
        assert!(content.is_empty());
        assert_eq!(content.len(), 0);
    }

    #[test]
    fn test_paragraph() {
        let block = Block::paragraph(vec![Text::plain("Hello")]);
        assert_eq!(block.block_type(), "paragraph");
        assert!(block.id().is_none());
    }

    #[test]
    fn test_heading() {
        let block = Block::heading(1, vec![Text::plain("Title")]);
        if let Block::Heading { level, .. } = &block {
            assert_eq!(*level, 1);
        } else {
            panic!("Expected Heading");
        }

        // Test clamping
        let block = Block::heading(10, vec![Text::plain("Title")]);
        if let Block::Heading { level, .. } = &block {
            assert_eq!(*level, 6);
        }
    }

    #[test]
    fn test_list() {
        let items = vec![
            Block::list_item(vec![Block::paragraph(vec![Text::plain("Item 1")])]),
            Block::list_item(vec![Block::paragraph(vec![Text::plain("Item 2")])]),
        ];
        let list = Block::unordered_list(items);
        assert_eq!(list.block_type(), "list");
    }

    #[test]
    fn test_code_block() {
        let block = Block::code_block("fn main() {}", Some("rust".to_string()));
        if let Block::CodeBlock {
            language, children, ..
        } = &block
        {
            assert_eq!(language.as_deref(), Some("rust"));
            assert_eq!(children[0].value, "fn main() {}");
        } else {
            panic!("Expected CodeBlock");
        }
    }

    #[test]
    fn test_image() {
        let block = Block::image("assets/photo.png", "A photo");
        if let Block::Image(img) = &block {
            assert_eq!(img.src, "assets/photo.png");
            assert_eq!(img.alt, "A photo");
        } else {
            panic!("Expected Image");
        }
    }

    #[test]
    fn test_math() {
        let block = Block::math("E = mc^2", MathFormat::Latex, true);
        if let Block::Math(math) = &block {
            assert_eq!(math.value, "E = mc^2");
            assert_eq!(math.format, MathFormat::Latex);
            assert!(math.display);
        } else {
            panic!("Expected Math");
        }
    }

    #[test]
    fn test_block_serialization() {
        let block = Block::paragraph(vec![Text::plain("Test")]);
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("\"type\":\"paragraph\""));
    }

    #[test]
    fn test_content_serialization() {
        let content = Content::new(vec![
            Block::heading(1, vec![Text::plain("Title")]),
            Block::paragraph(vec![Text::plain("Body")]),
        ]);
        let json = serde_json::to_string_pretty(&content).unwrap();
        assert!(json.contains("\"version\": \"0.1\""));
        assert!(json.contains("\"type\": \"heading\""));
        assert!(json.contains("\"type\": \"paragraph\""));
    }

    #[test]
    fn test_block_deserialization() {
        let json = r#"{
            "type": "heading",
            "level": 2,
            "children": [{"value": "Section"}]
        }"#;
        let block: Block = serde_json::from_str(json).unwrap();
        if let Block::Heading {
            level, children, ..
        } = block
        {
            assert_eq!(level, 2);
            assert_eq!(children[0].value, "Section");
        } else {
            panic!("Expected Heading");
        }
    }

    #[test]
    fn test_table_serialization() {
        let table = Block::table(vec![Block::table_row(
            vec![Block::table_cell(vec![Text::plain("Header")])],
            true,
        )]);
        let json = serde_json::to_string(&table).unwrap();
        assert!(json.contains("\"type\":\"table\""));
        assert!(json.contains("\"type\":\"tableRow\""));
        assert!(json.contains("\"header\":true"));
    }

    #[test]
    fn test_extension_block() {
        let ext = Block::extension("forms", "textInput");
        assert!(ext.is_extension());
        assert_eq!(ext.block_type(), "forms:textInput");

        if let Block::Extension(inner) = &ext {
            assert_eq!(inner.namespace, "forms");
            assert_eq!(inner.block_type, "textInput");
            assert_eq!(inner.full_type(), "forms:textInput");
        } else {
            panic!("Expected Extension");
        }
    }

    #[test]
    fn test_extension_as_extension() {
        let ext = Block::extension("semantic", "citation");
        let inner = ext.as_extension().expect("should be extension");
        assert_eq!(inner.namespace, "semantic");

        let para = Block::paragraph(vec![Text::plain("Not extension")]);
        assert!(para.as_extension().is_none());
    }

    #[test]
    fn test_extension_with_fallback() {
        let fallback = Block::paragraph(vec![Text::plain("[Form field]")]);
        let ext = ExtensionBlock::new("forms", "textInput")
            .with_id("name-field")
            .with_fallback(fallback);

        assert_eq!(ext.id, Some("name-field".to_string()));
        assert!(ext.fallback_content().is_some());
    }

    // Tests for new block types

    #[test]
    fn test_definition_list() {
        let dl = Block::definition_list(vec![Block::definition_item(vec![
            Block::definition_term(vec![Text::plain("Term")]),
            Block::definition_description(vec![Block::paragraph(vec![Text::plain("Description")])]),
        ])]);
        assert_eq!(dl.block_type(), "definitionList");
    }

    #[test]
    fn test_measurement() {
        let m = Block::measurement(9.81, "9.81 m/s²");
        assert_eq!(m.block_type(), "measurement");
        if let Block::Measurement(meas) = &m {
            assert!((meas.value - 9.81).abs() < 0.001);
            assert_eq!(meas.display, "9.81 m/s²");
        } else {
            panic!("Expected Measurement");
        }
    }

    #[test]
    fn test_measurement_with_uncertainty() {
        let m = MeasurementBlock::new(1.234, "1.234(5) m")
            .with_unit("m")
            .with_uncertainty(0.005, UncertaintyNotation::Parenthetical);
        assert_eq!(m.unit, Some("m".to_string()));
        assert!(m.uncertainty.is_some());
        assert_eq!(
            m.uncertainty_notation,
            Some(UncertaintyNotation::Parenthetical)
        );
    }

    #[test]
    fn test_signature() {
        let sig = Block::signature(BlockSignatureType::Digital);
        assert_eq!(sig.block_type(), "signature");
        if let Block::Signature(s) = &sig {
            assert_eq!(s.signature_type, BlockSignatureType::Digital);
        } else {
            panic!("Expected Signature");
        }
    }

    #[test]
    fn test_signature_with_signer() {
        let signer = SignerDetails::new("John Doe")
            .with_title("CEO")
            .with_organization("Acme Corp");
        let sig = SignatureBlock::new(BlockSignatureType::Handwritten)
            .with_signer(signer)
            .with_purpose(SignaturePurpose::Approval);
        assert!(sig.signer.is_some());
        assert_eq!(sig.purpose, Some(SignaturePurpose::Approval));
    }

    #[test]
    fn test_svg_from_src() {
        let svg = Block::svg_from_src("diagram.svg");
        assert_eq!(svg.block_type(), "svg");
        if let Block::Svg(s) = &svg {
            assert_eq!(s.src, Some("diagram.svg".to_string()));
            assert!(s.content.is_none());
        } else {
            panic!("Expected Svg");
        }
    }

    #[test]
    fn test_svg_from_content() {
        let svg = Block::svg_from_content("<svg>...</svg>");
        if let Block::Svg(s) = &svg {
            assert!(s.src.is_none());
            assert_eq!(s.content, Some("<svg>...</svg>".to_string()));
        } else {
            panic!("Expected Svg");
        }
    }

    #[test]
    fn test_barcode() {
        let bc = Block::barcode(
            BarcodeFormat::Qr,
            "https://example.com",
            "Link to example.com",
        );
        assert_eq!(bc.block_type(), "barcode");
        if let Block::Barcode(b) = &bc {
            assert_eq!(b.format, BarcodeFormat::Qr);
            assert_eq!(b.data, "https://example.com");
            assert_eq!(b.alt, "Link to example.com");
        } else {
            panic!("Expected Barcode");
        }
    }

    #[test]
    fn test_barcode_with_options() {
        let bc = BarcodeBlock::new(BarcodeFormat::Qr, "data", "Meaningful alt text")
            .with_error_correction(ErrorCorrectionLevel::H)
            .with_size("2in".to_string(), "2in".to_string());
        assert_eq!(bc.error_correction, Some(ErrorCorrectionLevel::H));
        assert!(bc.size.is_some());
    }

    #[test]
    fn test_figure() {
        let fig = Block::figure(vec![
            Block::image("photo.png", "A photo"),
            Block::figcaption(vec![Text::plain("Figure 1: A photo")]),
        ]);
        assert_eq!(fig.block_type(), "figure");
    }

    #[test]
    fn test_figcaption() {
        let fc = Block::figcaption(vec![Text::plain("Caption text")]);
        assert_eq!(fc.block_type(), "figcaption");
    }

    #[test]
    fn test_writing_mode_serialization() {
        let attrs = BlockAttributes {
            dir: None,
            lang: None,
            writing_mode: Some(WritingMode::VerticalRl),
        };
        let json = serde_json::to_string(&attrs).unwrap();
        assert!(json.contains("\"writingMode\":\"vertical-rl\""));
    }

    #[test]
    fn test_writing_mode_deserialization() {
        let json = r#"{"writingMode":"vertical-lr"}"#;
        let attrs: BlockAttributes = serde_json::from_str(json).unwrap();
        assert_eq!(attrs.writing_mode, Some(WritingMode::VerticalLr));
    }

    #[test]
    fn test_measurement_serialization() {
        let m = MeasurementBlock::new(299_792_458.0, "299,792,458 m/s")
            .with_unit("m/s")
            .with_exponent(8);
        let json = serde_json::to_string(&m).unwrap();
        assert!(json.contains("\"value\":299792458")); // JSON doesn't use underscores
        assert!(json.contains("\"unit\":\"m/s\""));
        assert!(json.contains("\"exponent\":8"));
    }

    #[test]
    fn test_barcode_format_serialization() {
        let bc = BarcodeBlock::new(BarcodeFormat::DataMatrix, "ABC123", "Product code ABC123");
        let json = serde_json::to_string(&bc).unwrap();
        // DataMatrix becomes datamatrix with lowercase rename_all
        assert!(json.contains("\"format\":\"datamatrix\""));
    }

    #[test]
    fn test_signature_type_serialization() {
        let sig = SignatureBlock::new(BlockSignatureType::Electronic);
        let json = serde_json::to_string(&sig).unwrap();
        assert!(json.contains("\"signatureType\":\"electronic\""));
    }

    #[test]
    fn test_new_block_types_deserialization() {
        // Definition list
        let json = r#"{"type":"definitionList","children":[]}"#;
        let block: Block = serde_json::from_str(json).unwrap();
        assert_eq!(block.block_type(), "definitionList");

        // Measurement
        let json = r#"{"type":"measurement","value":3.14159,"display":"π"}"#;
        let block: Block = serde_json::from_str(json).unwrap();
        assert_eq!(block.block_type(), "measurement");

        // SVG
        let json = r#"{"type":"svg","src":"diagram.svg"}"#;
        let block: Block = serde_json::from_str(json).unwrap();
        assert_eq!(block.block_type(), "svg");

        // Barcode
        let json = r#"{"type":"barcode","format":"qr","data":"test","alt":"Test QR code"}"#;
        let block: Block = serde_json::from_str(json).unwrap();
        assert_eq!(block.block_type(), "barcode");

        // Figure
        let json = r#"{"type":"figure","children":[]}"#;
        let block: Block = serde_json::from_str(json).unwrap();
        assert_eq!(block.block_type(), "figure");
    }

    // Admonition tests

    #[test]
    fn test_admonition() {
        let adm = Block::admonition(
            AdmonitionVariant::Warning,
            vec![Block::paragraph(vec![Text::plain("Be careful!")])],
        );
        assert_eq!(adm.block_type(), "admonition");
        if let Block::Admonition(a) = &adm {
            assert_eq!(a.variant, AdmonitionVariant::Warning);
            assert_eq!(a.children.len(), 1);
        } else {
            panic!("Expected Admonition");
        }
    }

    #[test]
    fn test_admonition_with_title() {
        let adm = AdmonitionBlock::new(
            AdmonitionVariant::Note,
            vec![Block::paragraph(vec![Text::plain("Important info")])],
        )
        .with_title("Please Note")
        .with_id("note-1");

        assert_eq!(adm.title, Some("Please Note".to_string()));
        assert_eq!(adm.id, Some("note-1".to_string()));
    }

    #[test]
    fn test_admonition_serialization() {
        let adm = Block::admonition(
            AdmonitionVariant::Tip,
            vec![Block::paragraph(vec![Text::plain("Pro tip!")])],
        );
        let json = serde_json::to_string(&adm).unwrap();
        assert!(json.contains("\"type\":\"admonition\""));
        assert!(json.contains("\"variant\":\"tip\""));
    }

    #[test]
    fn test_admonition_deserialization() {
        let json = r#"{
            "type": "admonition",
            "variant": "danger",
            "title": "Warning!",
            "children": [
                {"type": "paragraph", "children": [{"value": "Do not proceed!"}]}
            ]
        }"#;
        let block: Block = serde_json::from_str(json).unwrap();
        assert_eq!(block.block_type(), "admonition");
        if let Block::Admonition(adm) = block {
            assert_eq!(adm.variant, AdmonitionVariant::Danger);
            assert_eq!(adm.title, Some("Warning!".to_string()));
            assert_eq!(adm.children.len(), 1);
        } else {
            panic!("Expected Admonition");
        }
    }

    #[test]
    fn test_admonition_variant_display() {
        assert_eq!(AdmonitionVariant::Note.to_string(), "Note");
        assert_eq!(AdmonitionVariant::Warning.to_string(), "Warning");
        assert_eq!(AdmonitionVariant::Important.to_string(), "Important");
    }

    #[test]
    fn test_all_admonition_variants() {
        let variants = [
            (AdmonitionVariant::Note, "note"),
            (AdmonitionVariant::Tip, "tip"),
            (AdmonitionVariant::Info, "info"),
            (AdmonitionVariant::Warning, "warning"),
            (AdmonitionVariant::Caution, "caution"),
            (AdmonitionVariant::Danger, "danger"),
            (AdmonitionVariant::Important, "important"),
            (AdmonitionVariant::Example, "example"),
        ];

        for (variant, expected_name) in variants {
            let adm = AdmonitionBlock::new(variant, vec![]);
            let json = serde_json::to_string(&adm).unwrap();
            assert!(
                json.contains(&format!("\"variant\":\"{expected_name}\"")),
                "Variant {variant:?} should serialize as {expected_name}",
            );
        }
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    /// Generate arbitrary text content for testing.
    fn arb_text_content() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9 .,!?]{0,100}".prop_map(|s| s)
    }

    /// Generate arbitrary optional string.
    fn arb_optional_string() -> impl Strategy<Value = Option<String>> {
        prop_oneof![Just(None), "[a-zA-Z0-9_-]{1,20}".prop_map(Some),]
    }

    /// Generate arbitrary heading level.
    fn arb_heading_level() -> impl Strategy<Value = u8> {
        1u8..=6u8
    }

    /// Generate arbitrary paragraph block.
    fn arb_paragraph() -> impl Strategy<Value = Block> {
        (arb_optional_string(), arb_text_content()).prop_map(|(id, text)| {
            let mut block = Block::paragraph(vec![Text::plain(text)]);
            if let Block::Paragraph {
                id: ref mut block_id,
                ..
            } = block
            {
                *block_id = id;
            }
            block
        })
    }

    /// Generate arbitrary heading block.
    fn arb_heading() -> impl Strategy<Value = Block> {
        (
            arb_optional_string(),
            arb_heading_level(),
            arb_text_content(),
        )
            .prop_map(|(id, level, text)| {
                let mut block = Block::heading(level, vec![Text::plain(text)]);
                if let Block::Heading {
                    id: ref mut block_id,
                    ..
                } = block
                {
                    *block_id = id;
                }
                block
            })
    }

    /// Generate arbitrary code block.
    fn arb_code_block() -> impl Strategy<Value = Block> {
        (
            arb_optional_string(),
            arb_text_content(),
            arb_optional_string(),
        )
            .prop_map(|(id, code, language)| {
                let mut block = Block::code_block(code, language);
                if let Block::CodeBlock {
                    id: ref mut block_id,
                    ..
                } = block
                {
                    *block_id = id;
                }
                block
            })
    }

    /// Generate arbitrary math block.
    fn arb_math_block() -> impl Strategy<Value = Block> {
        (
            arb_optional_string(),
            arb_text_content(),
            prop_oneof![Just(MathFormat::Latex), Just(MathFormat::Mathml)],
            any::<bool>(),
        )
            .prop_map(|(id, value, format, display)| {
                let mut block = Block::math(value, format, display);
                if let Block::Math(ref mut math) = block {
                    math.id = id;
                }
                block
            })
    }

    proptest! {
        /// Paragraph JSON roundtrip - serialize and deserialize should preserve data.
        #[test]
        fn paragraph_json_roundtrip(para in arb_paragraph()) {
            let json = serde_json::to_string(&para).unwrap();
            let parsed: Block = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(para, parsed);
        }

        /// Heading JSON roundtrip - serialize and deserialize should preserve data.
        #[test]
        fn heading_json_roundtrip(heading in arb_heading()) {
            let json = serde_json::to_string(&heading).unwrap();
            let parsed: Block = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(heading, parsed);
        }

        /// Code block JSON roundtrip - serialize and deserialize should preserve data.
        #[test]
        fn code_block_json_roundtrip(code in arb_code_block()) {
            let json = serde_json::to_string(&code).unwrap();
            let parsed: Block = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(code, parsed);
        }

        /// Math block JSON roundtrip - serialize and deserialize should preserve data.
        #[test]
        fn math_block_json_roundtrip(math in arb_math_block()) {
            let json = serde_json::to_string(&math).unwrap();
            let parsed: Block = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(math, parsed);
        }

        /// Heading level is always clamped to valid range 1-6.
        #[test]
        fn heading_level_clamped(level in any::<u8>()) {
            let block = Block::heading(level, vec![Text::plain("Test")]);
            if let Block::Heading { level: actual, .. } = block {
                prop_assert!((1..=6).contains(&actual));
            } else {
                prop_assert!(false, "Expected Heading block");
            }
        }

        /// Content version is always set correctly.
        #[test]
        fn content_version_is_spec_version(blocks in prop::collection::vec(arb_paragraph(), 0..5)) {
            let blocks_len = blocks.len();
            let content = Content::new(blocks);
            prop_assert_eq!(&content.version, crate::SPEC_VERSION);
            prop_assert_eq!(content.len(), blocks_len);
        }

        /// Block type returns correct string for paragraphs.
        #[test]
        fn paragraph_block_type(para in arb_paragraph()) {
            prop_assert_eq!(para.block_type(), "paragraph");
        }

        /// Block type returns correct string for headings.
        #[test]
        fn heading_block_type(heading in arb_heading()) {
            prop_assert_eq!(heading.block_type(), "heading");
        }
    }

    #[test]
    fn code_block_highlighting_and_tokens_roundtrip() {
        let json = serde_json::json!({
            "type": "codeBlock",
            "value": "let x = 1;",
            "language": "rust",
            "children": [],
            "highlighting": "monokai",
            "tokens": [
                { "tokenType": "keyword", "value": "let" },
                { "tokenType": "identifier", "value": "x", "scope": "variable" }
            ]
        });
        let block: Block = serde_json::from_value(json).unwrap();
        if let Block::CodeBlock {
            highlighting,
            tokens,
            ..
        } = &block
        {
            assert_eq!(highlighting.as_deref(), Some("monokai"));
            let toks = tokens.as_ref().unwrap();
            assert_eq!(toks.len(), 2);
            assert_eq!(toks[0].token_type, "keyword");
            assert_eq!(toks[1].scope.as_deref(), Some("variable"));
        } else {
            panic!("Expected CodeBlock");
        }
        // Roundtrip
        let serialized = serde_json::to_value(&block).unwrap();
        let parsed: Block = serde_json::from_value(serialized).unwrap();
        assert_eq!(block, parsed);
    }

    #[test]
    fn code_block_without_new_fields_defaults_to_none() {
        let json = serde_json::json!({
            "type": "codeBlock",
            "value": "print('hello')",
            "language": "python",
            "children": []
        });
        let block: Block = serde_json::from_value(json).unwrap();
        if let Block::CodeBlock {
            highlighting,
            tokens,
            ..
        } = &block
        {
            assert!(highlighting.is_none());
            assert!(tokens.is_none());
        } else {
            panic!("Expected CodeBlock");
        }
    }

    #[test]
    fn figure_numbering_serialization() {
        // Auto
        let json = serde_json::to_value(FigureNumbering::Auto).unwrap();
        assert_eq!(json, serde_json::json!("auto"));

        // Unnumbered
        let json = serde_json::to_value(FigureNumbering::Unnumbered).unwrap();
        assert_eq!(json, serde_json::json!("none"));

        // Number
        let json = serde_json::to_value(FigureNumbering::Number(3)).unwrap();
        assert_eq!(json, serde_json::json!(3));

        // Roundtrip
        let auto: FigureNumbering = serde_json::from_str("\"auto\"").unwrap();
        assert_eq!(auto, FigureNumbering::Auto);
        let unnumbered: FigureNumbering = serde_json::from_str("\"none\"").unwrap();
        assert_eq!(unnumbered, FigureNumbering::Unnumbered);
        let num: FigureNumbering = serde_json::from_str("3").unwrap();
        assert_eq!(num, FigureNumbering::Number(3));
    }

    #[test]
    fn figure_with_numbering_and_subfigures_roundtrip() {
        let json = serde_json::json!({
            "type": "figure",
            "children": [
                { "type": "image", "src": "fig1.png", "alt": "Figure 1" }
            ],
            "numbering": "auto",
            "subfigures": [
                {
                    "id": "sub-a",
                    "label": "(a)",
                    "children": [
                        { "type": "paragraph", "children": [{ "type": "text", "value": "Sub A" }] }
                    ]
                }
            ]
        });
        let block: Block = serde_json::from_value(json).unwrap();
        if let Block::Figure(fig) = &block {
            assert_eq!(fig.numbering, Some(FigureNumbering::Auto));
            let subs = fig.subfigures.as_ref().unwrap();
            assert_eq!(subs.len(), 1);
            assert_eq!(subs[0].id.as_deref(), Some("sub-a"));
            assert_eq!(subs[0].label.as_deref(), Some("(a)"));
        } else {
            panic!("Expected Figure block");
        }
        let serialized = serde_json::to_value(&block).unwrap();
        let parsed: Block = serde_json::from_value(serialized).unwrap();
        assert_eq!(block, parsed);
    }

    #[test]
    fn figure_without_new_fields_defaults_to_none() {
        let json = serde_json::json!({
            "type": "figure",
            "children": []
        });
        let block: Block = serde_json::from_value(json).unwrap();
        if let Block::Figure(fig) = &block {
            assert!(fig.numbering.is_none());
            assert!(fig.subfigures.is_none());
        } else {
            panic!("Expected Figure block");
        }
    }
}
