//! Content blocks and text representation.
//!
//! The content layer represents document content as a tree of semantic blocks.
//! This module provides types for all content blocks defined in the Codex specification.
//!
//! # Content Structure
//!
//! Content files have a root structure with a version and array of blocks:
//!
//! ```rust
//! use cdx_core::content::{Content, Block, Text};
//!
//! let content = Content {
//!     version: "0.1".to_string(),
//!     blocks: vec![
//!         Block::heading(1, vec![Text::plain("Hello World")]),
//!         Block::paragraph(vec![Text::plain("This is a paragraph.")]),
//!     ],
//! };
//! ```
//!
//! # Block Types
//!
//! - [`Block::Paragraph`] - Standard paragraph
//! - [`Block::Heading`] - Section heading (levels 1-6)
//! - [`Block::List`] - Ordered or unordered list
//! - [`Block::ListItem`] - Item within a list
//! - [`Block::Blockquote`] - Quoted content
//! - [`Block::CodeBlock`] - Source code or preformatted text
//! - [`Block::HorizontalRule`] - Thematic break
//! - [`Block::Image`] - Embedded or referenced image
//! - [`Block::Table`] - Tabular data
//! - [`Block::TableRow`] - Row within a table
//! - [`Block::TableCell`] - Cell within a row
//! - [`Block::Math`] - Mathematical content
//! - [`Block::Break`] - Line break
//!
//! # Text and Marks
//!
//! Text content is represented as [`Text`] nodes with optional [`Mark`]s for formatting:
//!
//! ```rust
//! use cdx_core::content::{Text, Mark};
//!
//! let bold_text = Text {
//!     value: "Important".to_string(),
//!     marks: vec![Mark::Bold],
//! };
//!
//! let link = Text {
//!     value: "Click here".to_string(),
//!     marks: vec![Mark::Link {
//!         href: "https://example.com".to_string(),
//!         title: Some("Example".to_string()),
//!     }],
//! };
//! ```

mod block;
mod text;
mod validation;

pub use block::{
    AdmonitionBlock, AdmonitionVariant, BarcodeBlock, BarcodeFormat, BarcodeSize, Block,
    BlockAttributes, BlockSignatureType, CellAlign, Content, DefinitionListBlock,
    ErrorCorrectionLevel, FigCaptionBlock, FigureBlock, ImageBlock, MathBlock, MathFormat,
    MeasurementBlock, SignatureBlock, SignaturePurpose, SignerDetails, SvgBlock, TableCellBlock,
    UncertaintyNotation, WritingMode,
};
pub use text::{ExtensionMark, Mark, MarkType, Text};
pub use validation::{validate_content, ValidationError};
