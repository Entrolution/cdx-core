//! Content validation.

use std::collections::HashSet;
use std::fmt;

use super::{Block, Content, Text};

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
        Block::HorizontalRule { .. } | Block::Break { .. } => {}
        Block::Image(img) => validate_image(img, path, ctx.errors),
        Block::Table { children, .. } => validate_table(children, path, &mut ctx),
        Block::TableRow { children, .. } => validate_table_row(children, path, parent, &mut ctx),
        Block::TableCell(cell) => validate_table_cell(cell, path, parent, ctx.errors),
        Block::Math(math) => validate_math(math, path, ctx.errors),
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
}
