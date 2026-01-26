//! Content block types.

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
pub struct BlockAttributes {
    /// Text direction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dir: Option<String>,

    /// BCP 47 language tag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

impl BlockAttributes {
    /// Check if attributes are empty (all None).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.dir.is_none() && self.lang.is_none()
    }
}

/// A content block in the document tree.
///
/// Blocks are the structural elements of a document, containing
/// either other blocks (containers) or text content (leaves).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Block {
    /// Standard paragraph block.
    Paragraph {
        /// Optional unique identifier.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,

        /// Text content.
        children: Vec<Text>,

        /// Block attributes.
        #[serde(default, skip_serializing_if = "BlockAttributes::is_empty")]
        attributes: BlockAttributes,
    },

    /// Section heading (levels 1-6).
    Heading {
        /// Optional unique identifier.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,

        /// Heading level (1-6).
        level: u8,

        /// Text content.
        children: Vec<Text>,

        /// Block attributes.
        #[serde(default, skip_serializing_if = "BlockAttributes::is_empty")]
        attributes: BlockAttributes,
    },

    /// Ordered or unordered list.
    List {
        /// Optional unique identifier.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,

        /// Whether the list is ordered (numbered).
        ordered: bool,

        /// Starting number for ordered lists.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        start: Option<u32>,

        /// List items (must be `ListItem` blocks).
        children: Vec<Block>,

        /// Block attributes.
        #[serde(default, skip_serializing_if = "BlockAttributes::is_empty")]
        attributes: BlockAttributes,
    },

    /// Item within a list.
    ListItem {
        /// Optional unique identifier.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,

        /// Checkbox state (None = not a checkbox).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        checked: Option<bool>,

        /// Block content.
        children: Vec<Block>,

        /// Block attributes.
        #[serde(default, skip_serializing_if = "BlockAttributes::is_empty")]
        attributes: BlockAttributes,
    },

    /// Quoted content block.
    Blockquote {
        /// Optional unique identifier.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,

        /// Block content.
        children: Vec<Block>,

        /// Block attributes.
        #[serde(default, skip_serializing_if = "BlockAttributes::is_empty")]
        attributes: BlockAttributes,
    },

    /// Source code or preformatted text.
    CodeBlock {
        /// Optional unique identifier.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,

        /// Programming language identifier.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        language: Option<String>,

        /// Code content (single text node, no marks).
        children: Vec<Text>,

        /// Block attributes.
        #[serde(default, skip_serializing_if = "BlockAttributes::is_empty")]
        attributes: BlockAttributes,
    },

    /// Thematic break between sections.
    HorizontalRule {
        /// Optional unique identifier.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },

    /// Embedded or referenced image.
    Image(ImageBlock),

    /// Tabular data.
    Table {
        /// Optional unique identifier.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,

        /// Table rows.
        children: Vec<Block>,

        /// Block attributes.
        #[serde(default, skip_serializing_if = "BlockAttributes::is_empty")]
        attributes: BlockAttributes,
    },

    /// Row within a table.
    TableRow {
        /// Optional unique identifier.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,

        /// Whether this is a header row.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        header: bool,

        /// Table cells.
        children: Vec<Block>,

        /// Block attributes.
        #[serde(default, skip_serializing_if = "BlockAttributes::is_empty")]
        attributes: BlockAttributes,
    },

    /// Cell within a table row.
    TableCell(TableCellBlock),

    /// Mathematical content.
    Math(MathBlock),

    /// Line break within a block.
    Break {
        /// Optional unique identifier.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },

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

    /// Get the block type as a string.
    ///
    /// For extension blocks, this returns "extension". Use [`ExtensionBlock::full_type()`]
    /// to get the namespaced type like "forms:textInput".
    #[must_use]
    pub fn block_type(&self) -> &'static str {
        match self {
            Self::Paragraph { .. } => "paragraph",
            Self::Heading { .. } => "heading",
            Self::List { .. } => "list",
            Self::ListItem { .. } => "listItem",
            Self::Blockquote { .. } => "blockquote",
            Self::CodeBlock { .. } => "codeBlock",
            Self::HorizontalRule { .. } => "horizontalRule",
            Self::Image(_) => "image",
            Self::Table { .. } => "table",
            Self::TableRow { .. } => "tableRow",
            Self::TableCell(_) => "tableCell",
            Self::Math(_) => "math",
            Self::Break { .. } => "break",
            Self::Extension(_) => "extension",
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
            | Self::Break { id } => id.as_deref(),
            Self::Image(img) => img.id.as_deref(),
            Self::TableCell(cell) => cell.id.as_deref(),
            Self::Math(math) => math.id.as_deref(),
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
        assert_eq!(ext.block_type(), "extension");

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
}
