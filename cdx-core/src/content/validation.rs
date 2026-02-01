//! Content validation.

use std::collections::HashSet;
use std::fmt;

use super::{Block, Content, Text};
use crate::extensions::ExtensionBlock;

/// Content validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// Path to the invalid element (e.g., `blocks[0].children[1]`).
    pub path: String,

    /// Description of the validation failure.
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Validate content structure and rules.
///
/// This validates:
/// - Block structure (correct children types)
/// - Unique block IDs
/// - Required fields
/// - Heading levels (1-6)
/// - List items only in lists
/// - Table rows only in tables
/// - Table cells only in rows
///
/// # Errors
///
/// Returns a vector of validation errors if any are found.
#[must_use]
pub fn validate_content(content: &Content) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let mut seen_ids = HashSet::new();

    for (i, block) in content.blocks.iter().enumerate() {
        let path = format!("blocks[{i}]");
        validate_block(block, &path, &mut errors, &mut seen_ids, None);
    }

    errors
}

/// Parent context for validating child blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParentContext {
    List,
    Table,
    TableRow,
    DefinitionList,
    Figure,
}

/// Context passed through validation.
struct ValidationContext<'a> {
    errors: &'a mut Vec<ValidationError>,
    seen_ids: &'a mut HashSet<String>,
}

impl ValidationContext<'_> {
    fn add_error(&mut self, path: &str, message: impl Into<String>) {
        self.errors.push(ValidationError {
            path: path.to_string(),
            message: message.into(),
        });
    }
}

fn validate_block(
    block: &Block,
    path: &str,
    errors: &mut Vec<ValidationError>,
    seen_ids: &mut HashSet<String>,
    parent: Option<ParentContext>,
) {
    let mut ctx = ValidationContext { errors, seen_ids };

    // Check ID uniqueness
    if let Some(id) = block.id() {
        if !ctx.seen_ids.insert(id.to_string()) {
            ctx.add_error(path, format!("duplicate block ID: {id}"));
        }
    }

    match block {
        Block::Paragraph { children, .. } => validate_text_children(children, path, ctx.errors),
        Block::Heading {
            level, children, ..
        } => {
            validate_heading(*level, children, path, ctx.errors);
        }
        Block::List { children, .. } => validate_list(children, path, &mut ctx),
        Block::ListItem { children, .. } => validate_list_item(children, path, parent, &mut ctx),
        Block::Blockquote { children, .. } => validate_container(children, path, &mut ctx),
        Block::CodeBlock { children, .. } => validate_code_block(children, path, ctx.errors),
        Block::HorizontalRule { .. } | Block::Break { .. } | Block::Signature(_) => {}
        Block::Image(img) => validate_image(img, path, ctx.errors),
        Block::Table { children, .. } => validate_table(children, path, &mut ctx),
        Block::TableRow { children, .. } => validate_table_row(children, path, parent, &mut ctx),
        Block::TableCell(cell) => validate_table_cell(cell, path, parent, ctx.errors),
        Block::Math(math) => validate_math(math, path, ctx.errors),
        Block::Extension(ext) => validate_extension(ext, path, &mut ctx),
        // New block types
        Block::DefinitionList(dl) => validate_definition_list(&dl.children, path, &mut ctx),
        Block::DefinitionItem { children, .. } => {
            validate_definition_item(children, path, parent, &mut ctx);
        }
        Block::DefinitionTerm { children, .. } => {
            validate_text_children(children, path, ctx.errors);
        }
        Block::DefinitionDescription { children, .. } => {
            validate_container(children, path, &mut ctx);
        }
        Block::Measurement(m) => validate_measurement(m, path, ctx.errors),
        Block::Svg(svg) => validate_svg(svg, path, ctx.errors),
        Block::Barcode(bc) => validate_barcode(bc, path, ctx.errors),
        Block::Figure(fig) => validate_figure(&fig.children, path, &mut ctx),
        Block::FigCaption(fc) => validate_figcaption(&fc.children, path, parent, ctx.errors),
    }
}

fn validate_heading(level: u8, children: &[Text], path: &str, errors: &mut Vec<ValidationError>) {
    if !(1..=6).contains(&level) {
        errors.push(ValidationError {
            path: path.to_string(),
            message: format!("heading level must be 1-6, got {level}"),
        });
    }
    validate_text_children(children, path, errors);
}

fn validate_list(children: &[Block], path: &str, ctx: &mut ValidationContext<'_>) {
    for (i, child) in children.iter().enumerate() {
        let child_path = format!("{path}.children[{i}]");
        if !matches!(child, Block::ListItem { .. }) {
            ctx.add_error(
                &child_path,
                format!("list children must be listItem, got {}", child.block_type()),
            );
        }
        validate_block(
            child,
            &child_path,
            ctx.errors,
            ctx.seen_ids,
            Some(ParentContext::List),
        );
    }
}

fn validate_list_item(
    children: &[Block],
    path: &str,
    parent: Option<ParentContext>,
    ctx: &mut ValidationContext<'_>,
) {
    if parent != Some(ParentContext::List) {
        ctx.add_error(path, "listItem must be a child of list");
    }
    for (i, child) in children.iter().enumerate() {
        let child_path = format!("{path}.children[{i}]");
        validate_block(child, &child_path, ctx.errors, ctx.seen_ids, None);
    }
}

fn validate_container(children: &[Block], path: &str, ctx: &mut ValidationContext<'_>) {
    for (i, child) in children.iter().enumerate() {
        let child_path = format!("{path}.children[{i}]");
        validate_block(child, &child_path, ctx.errors, ctx.seen_ids, None);
    }
}

fn validate_code_block(children: &[Text], path: &str, errors: &mut Vec<ValidationError>) {
    if children.len() != 1 {
        errors.push(ValidationError {
            path: path.to_string(),
            message: format!(
                "codeBlock should have exactly 1 text node, got {}",
                children.len()
            ),
        });
    }
    for child in children {
        if !child.marks.is_empty() {
            errors.push(ValidationError {
                path: path.to_string(),
                message: "codeBlock text should not have marks".to_string(),
            });
        }
    }
}

fn validate_image(img: &super::block::ImageBlock, path: &str, errors: &mut Vec<ValidationError>) {
    if img.src.is_empty() {
        errors.push(ValidationError {
            path: path.to_string(),
            message: "image src is required".to_string(),
        });
    }
    if img.alt.is_empty() {
        errors.push(ValidationError {
            path: path.to_string(),
            message: "image alt is required".to_string(),
        });
    }
}

fn validate_table(children: &[Block], path: &str, ctx: &mut ValidationContext<'_>) {
    for (i, child) in children.iter().enumerate() {
        let child_path = format!("{path}.children[{i}]");
        if !matches!(child, Block::TableRow { .. }) {
            ctx.add_error(
                &child_path,
                format!(
                    "table children must be tableRow, got {}",
                    child.block_type()
                ),
            );
        }
        validate_block(
            child,
            &child_path,
            ctx.errors,
            ctx.seen_ids,
            Some(ParentContext::Table),
        );
    }
}

fn validate_table_row(
    children: &[Block],
    path: &str,
    parent: Option<ParentContext>,
    ctx: &mut ValidationContext<'_>,
) {
    if parent != Some(ParentContext::Table) {
        ctx.add_error(path, "tableRow must be a child of table");
    }
    for (i, child) in children.iter().enumerate() {
        let child_path = format!("{path}.children[{i}]");
        if !matches!(child, Block::TableCell(_)) {
            ctx.add_error(
                &child_path,
                format!(
                    "tableRow children must be tableCell, got {}",
                    child.block_type()
                ),
            );
        }
        validate_block(
            child,
            &child_path,
            ctx.errors,
            ctx.seen_ids,
            Some(ParentContext::TableRow),
        );
    }
}

fn validate_table_cell(
    cell: &super::block::TableCellBlock,
    path: &str,
    parent: Option<ParentContext>,
    errors: &mut Vec<ValidationError>,
) {
    if parent != Some(ParentContext::TableRow) {
        errors.push(ValidationError {
            path: path.to_string(),
            message: "tableCell must be a child of tableRow".to_string(),
        });
    }
    if cell.colspan == 0 {
        errors.push(ValidationError {
            path: path.to_string(),
            message: "tableCell colspan must be at least 1".to_string(),
        });
    }
    if cell.rowspan == 0 {
        errors.push(ValidationError {
            path: path.to_string(),
            message: "tableCell rowspan must be at least 1".to_string(),
        });
    }
    validate_text_children(&cell.children, path, errors);
}

fn validate_math(math: &super::block::MathBlock, path: &str, errors: &mut Vec<ValidationError>) {
    if math.value.is_empty() {
        errors.push(ValidationError {
            path: path.to_string(),
            message: "math value is required".to_string(),
        });
    }
}

fn validate_extension(ext: &ExtensionBlock, path: &str, ctx: &mut ValidationContext<'_>) {
    // Validate extension namespace and type
    if ext.namespace.is_empty() {
        ctx.add_error(path, "extension namespace is required");
    }
    if ext.block_type.is_empty() {
        ctx.add_error(path, "extension block type is required");
    }

    // Validate children recursively
    for (i, child) in ext.children.iter().enumerate() {
        let child_path = format!("{path}.children[{i}]");
        validate_block(child, &child_path, ctx.errors, ctx.seen_ids, None);
    }

    // Validate fallback content if present
    if let Some(fallback) = &ext.fallback {
        let fallback_path = format!("{path}.fallback");
        validate_block(fallback, &fallback_path, ctx.errors, ctx.seen_ids, None);
    }
}

fn validate_text_children(children: &[Text], path: &str, errors: &mut Vec<ValidationError>) {
    for (i, text) in children.iter().enumerate() {
        if text.value.is_empty() {
            errors.push(ValidationError {
                path: format!("{path}.children[{i}]"),
                message: "text value cannot be empty".to_string(),
            });
        }
    }
}

fn validate_definition_list(children: &[Block], path: &str, ctx: &mut ValidationContext<'_>) {
    for (i, child) in children.iter().enumerate() {
        let child_path = format!("{path}.children[{i}]");
        if !matches!(child, Block::DefinitionItem { .. }) {
            ctx.add_error(
                &child_path,
                format!(
                    "definitionList children must be definitionItem, got {}",
                    child.block_type()
                ),
            );
        }
        validate_block(
            child,
            &child_path,
            ctx.errors,
            ctx.seen_ids,
            Some(ParentContext::DefinitionList),
        );
    }
}

fn validate_definition_item(
    children: &[Block],
    path: &str,
    parent: Option<ParentContext>,
    ctx: &mut ValidationContext<'_>,
) {
    if parent != Some(ParentContext::DefinitionList) {
        ctx.add_error(path, "definitionItem must be a child of definitionList");
    }
    for (i, child) in children.iter().enumerate() {
        let child_path = format!("{path}.children[{i}]");
        validate_block(child, &child_path, ctx.errors, ctx.seen_ids, None);
    }
}

fn validate_measurement(
    m: &super::block::MeasurementBlock,
    path: &str,
    errors: &mut Vec<ValidationError>,
) {
    if m.display.is_empty() {
        errors.push(ValidationError {
            path: path.to_string(),
            message: "measurement display is required".to_string(),
        });
    }
}

fn validate_svg(svg: &super::block::SvgBlock, path: &str, errors: &mut Vec<ValidationError>) {
    // SVG must have exactly one of src or content
    match (&svg.src, &svg.content) {
        (Some(_), Some(_)) => {
            errors.push(ValidationError {
                path: path.to_string(),
                message: "svg must have either src or content, not both".to_string(),
            });
        }
        (None, None) => {
            errors.push(ValidationError {
                path: path.to_string(),
                message: "svg must have either src or content".to_string(),
            });
        }
        _ => {}
    }
}

fn validate_barcode(
    bc: &super::block::BarcodeBlock,
    path: &str,
    errors: &mut Vec<ValidationError>,
) {
    if bc.data.is_empty() {
        errors.push(ValidationError {
            path: path.to_string(),
            message: "barcode data is required".to_string(),
        });
    }
    // Check for generic/placeholder alt text
    let alt_lower = bc.alt.to_lowercase();
    if bc.alt.is_empty()
        || alt_lower == "barcode"
        || alt_lower == "qr code"
        || alt_lower == "image"
    {
        errors.push(ValidationError {
            path: path.to_string(),
            message: "barcode alt must be meaningful (not just 'barcode' or 'image')".to_string(),
        });
    }
}

fn validate_figure(children: &[Block], path: &str, ctx: &mut ValidationContext<'_>) {
    for (i, child) in children.iter().enumerate() {
        let child_path = format!("{path}.children[{i}]");
        validate_block(
            child,
            &child_path,
            ctx.errors,
            ctx.seen_ids,
            Some(ParentContext::Figure),
        );
    }
}

fn validate_figcaption(
    children: &[Text],
    path: &str,
    parent: Option<ParentContext>,
    errors: &mut Vec<ValidationError>,
) {
    if parent != Some(ParentContext::Figure) {
        errors.push(ValidationError {
            path: path.to_string(),
            message: "figCaption should be a child of figure".to_string(),
        });
    }
    validate_text_children(children, path, errors);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{BlockAttributes, Mark, Text};

    #[test]
    fn test_valid_content() {
        let content = Content::new(vec![
            Block::heading(1, vec![Text::plain("Title")]),
            Block::paragraph(vec![Text::plain("Body")]),
        ]);
        let errors = validate_content(&content);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_duplicate_ids() {
        let content = Content::new(vec![
            Block::Paragraph {
                id: Some("dup".to_string()),
                children: vec![Text::plain("First")],
                attributes: BlockAttributes::default(),
            },
            Block::Paragraph {
                id: Some("dup".to_string()),
                children: vec![Text::plain("Second")],
                attributes: BlockAttributes::default(),
            },
        ]);
        let errors = validate_content(&content);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("duplicate"));
    }

    #[test]
    fn test_invalid_heading_level() {
        let content = Content::new(vec![Block::Heading {
            id: None,
            level: 7,
            children: vec![Text::plain("Too deep")],
            attributes: BlockAttributes::default(),
        }]);
        let errors = validate_content(&content);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("level"));
    }

    #[test]
    fn test_list_item_outside_list() {
        let content = Content::new(vec![Block::list_item(vec![Block::paragraph(vec![
            Text::plain("Orphan"),
        ])])]);
        let errors = validate_content(&content);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("child of list"));
    }

    #[test]
    fn test_list_with_wrong_children() {
        let content = Content::new(vec![Block::List {
            id: None,
            ordered: false,
            start: None,
            children: vec![Block::paragraph(vec![Text::plain("Wrong")])],
            attributes: BlockAttributes::default(),
        }]);
        let errors = validate_content(&content);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("listItem"));
    }

    #[test]
    fn test_code_block_with_marks() {
        let content = Content::new(vec![Block::CodeBlock {
            id: None,
            language: Some("rust".to_string()),
            children: vec![Text::with_marks("code", vec![Mark::Bold])],
            attributes: BlockAttributes::default(),
        }]);
        let errors = validate_content(&content);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("marks"));
    }

    #[test]
    fn test_empty_image() {
        let content = Content::new(vec![Block::Image(super::super::block::ImageBlock {
            id: None,
            src: String::new(),
            alt: String::new(),
            title: None,
            width: None,
            height: None,
        })]);
        let errors = validate_content(&content);
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_valid_table() {
        let content = Content::new(vec![Block::table(vec![Block::table_row(
            vec![Block::table_cell(vec![Text::plain("Cell")])],
            false,
        )])]);
        let errors = validate_content(&content);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_table_row_outside_table() {
        let content = Content::new(vec![Block::table_row(
            vec![Block::table_cell(vec![Text::plain("Orphan")])],
            false,
        )]);
        let errors = validate_content(&content);
        assert!(errors.iter().any(|e| e.message.contains("child of table")));
    }

    // Tests for new block type validation

    #[test]
    fn test_valid_definition_list() {
        let content = Content::new(vec![Block::definition_list(vec![Block::definition_item(
            vec![
                Block::definition_term(vec![Text::plain("Term")]),
                Block::definition_description(vec![Block::paragraph(vec![Text::plain("Desc")])]),
            ],
        )])]);
        let errors = validate_content(&content);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_definition_item_outside_list() {
        let content = Content::new(vec![Block::definition_item(vec![Block::definition_term(
            vec![Text::plain("Orphan term")],
        )])]);
        let errors = validate_content(&content);
        assert!(errors
            .iter()
            .any(|e| e.message.contains("child of definitionList")));
    }

    #[test]
    fn test_definition_list_with_wrong_children() {
        let content = Content::new(vec![Block::DefinitionList(
            super::super::block::DefinitionListBlock::new(vec![Block::paragraph(vec![
                Text::plain("Wrong"),
            ])]),
        )]);
        let errors = validate_content(&content);
        assert!(errors.iter().any(|e| e.message.contains("definitionItem")));
    }

    #[test]
    fn test_svg_with_both_src_and_content() {
        let content = Content::new(vec![Block::Svg(super::super::block::SvgBlock {
            id: None,
            src: Some("file.svg".to_string()),
            content: Some("<svg>...</svg>".to_string()),
            width: None,
            height: None,
            alt: None,
        })]);
        let errors = validate_content(&content);
        assert!(errors
            .iter()
            .any(|e| e.message.contains("either src or content, not both")));
    }

    #[test]
    fn test_svg_with_neither_src_nor_content() {
        let content = Content::new(vec![Block::Svg(super::super::block::SvgBlock {
            id: None,
            src: None,
            content: None,
            width: None,
            height: None,
            alt: None,
        })]);
        let errors = validate_content(&content);
        assert!(errors
            .iter()
            .any(|e| e.message.contains("either src or content")));
    }

    #[test]
    fn test_barcode_with_generic_alt() {
        use super::super::block::{BarcodeBlock, BarcodeFormat};

        let content = Content::new(vec![Block::Barcode(BarcodeBlock::new(
            BarcodeFormat::Qr,
            "https://example.com",
            "barcode", // Generic alt text should fail
        ))]);
        let errors = validate_content(&content);
        assert!(errors.iter().any(|e| e.message.contains("meaningful")));
    }

    #[test]
    fn test_barcode_with_good_alt() {
        use super::super::block::{BarcodeBlock, BarcodeFormat};

        let content = Content::new(vec![Block::Barcode(BarcodeBlock::new(
            BarcodeFormat::Qr,
            "https://example.com",
            "Link to example.com homepage",
        ))]);
        let errors = validate_content(&content);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_valid_figure() {
        let content = Content::new(vec![Block::figure(vec![
            Block::image("photo.png", "A photo"),
            Block::figcaption(vec![Text::plain("Figure 1")]),
        ])]);
        let errors = validate_content(&content);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_figcaption_outside_figure() {
        let content = Content::new(vec![Block::figcaption(vec![Text::plain("Orphan caption")])]);
        let errors = validate_content(&content);
        assert!(errors
            .iter()
            .any(|e| e.message.contains("child of figure")));
    }

    #[test]
    fn test_measurement_empty_display() {
        let content = Content::new(vec![Block::Measurement(
            super::super::block::MeasurementBlock {
                id: None,
                value: 42.0,
                uncertainty: None,
                uncertainty_notation: None,
                exponent: None,
                display: String::new(), // Empty display should fail
                unit: None,
            },
        )]);
        let errors = validate_content(&content);
        assert!(errors.iter().any(|e| e.message.contains("display")));
    }
}
