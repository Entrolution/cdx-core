//! Extract command implementation.

use anyhow::{Context, Result};
use cdx_core::content::{Block, Text};
use cdx_core::Document;
use std::fs;
use std::path::PathBuf;

use crate::output::OutputConfig;

pub fn run(
    file: PathBuf,
    output_dir: PathBuf,
    extract_content: bool,
    extract_text: bool,
    asset_name: Option<String>,
    all_assets: bool,
    config: &OutputConfig,
) -> Result<()> {
    config.verbose(&format!("Extracting from: {}", file.display()));

    // Open the document
    let doc = Document::open(&file)
        .with_context(|| format!("Failed to open document: {}", file.display()))?;

    // Ensure output directory exists
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir).with_context(|| {
            format!(
                "Failed to create output directory: {}",
                output_dir.display()
            )
        })?;
    }

    let mut extracted_items = Vec::new();

    // Extract content as JSON
    if extract_content {
        let content = doc.content();
        let content_json = serde_json::to_string_pretty(&content)?;

        if config.json {
            extracted_items.push(serde_json::json!({
                "type": "content",
                "format": "json",
                "blocks": content.len()
            }));
        } else {
            let output_path = output_dir.join("content.json");
            fs::write(&output_path, &content_json)
                .with_context(|| format!("Failed to write content: {}", output_path.display()))?;
            config.success(&format!("Extracted content to: {}", output_path.display()));
        }
    }

    // Extract as plain text
    if extract_text {
        let content = doc.content();
        let text = extract_plain_text(&content.blocks);

        if config.json {
            extracted_items.push(serde_json::json!({
                "type": "content",
                "format": "text",
                "length": text.len()
            }));
        } else {
            let output_path = output_dir.join("content.txt");
            fs::write(&output_path, &text)
                .with_context(|| format!("Failed to write text: {}", output_path.display()))?;
            config.success(&format!("Extracted text to: {}", output_path.display()));
        }
    }

    // Asset extraction is not fully implemented yet since we need archive access
    if asset_name.is_some() || all_assets {
        config.warning("Asset extraction not yet implemented (requires archive access)");
        if config.json {
            extracted_items.push(serde_json::json!({
                "type": "assets",
                "note": "Asset extraction not yet implemented"
            }));
        }
    }

    // If nothing specific requested, show help
    if !extract_content && !extract_text && asset_name.is_none() && !all_assets {
        if config.json {
            let result = serde_json::json!({
                "status": "info",
                "message": "No extraction options specified. Use --content, --text, --asset, or --all-assets."
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            config.info(
                "No extraction options specified. Use --content, --text, --asset, or --all-assets.",
            );
        }
    } else if config.json {
        let result = serde_json::json!({
            "status": "success",
            "file": file.display().to_string(),
            "extracted": extracted_items
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    }

    Ok(())
}

/// Extract plain text from document blocks.
fn extract_plain_text(blocks: &[Block]) -> String {
    let mut text_parts = Vec::new();

    for block in blocks {
        extract_block_text(block, &mut text_parts);
    }

    text_parts.join("\n\n")
}

fn extract_block_text(block: &Block, output: &mut Vec<String>) {
    match block {
        Block::Paragraph { children, .. } | Block::Heading { children, .. } => {
            let text = extract_text_nodes(children);
            if !text.is_empty() {
                output.push(text);
            }
        }
        Block::Blockquote { children, .. } => {
            for child in children {
                extract_block_text(child, output);
            }
        }
        Block::List { children, .. } => {
            for item in children {
                if let Block::ListItem { children, .. } = item {
                    for child in children {
                        extract_block_text(child, output);
                    }
                }
            }
        }
        Block::CodeBlock { children, .. } => {
            let text = extract_text_nodes(children);
            if !text.is_empty() {
                output.push(text);
            }
        }
        Block::Table { children, .. } => {
            for row in children {
                if let Block::TableRow { children, .. } = row {
                    let row_text: Vec<String> = children
                        .iter()
                        .filter_map(|cell| {
                            if let Block::TableCell(cell_block) = cell {
                                let cell_text = extract_text_nodes(&cell_block.children);
                                if !cell_text.is_empty() {
                                    Some(cell_text)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect();
                    if !row_text.is_empty() {
                        output.push(row_text.join("\t"));
                    }
                }
            }
        }
        Block::Math(math_block) => {
            output.push(math_block.value.clone());
        }
        _ => {}
    }
}

fn extract_text_nodes(nodes: &[Text]) -> String {
    nodes.iter().map(|node| node.value.clone()).collect()
}
