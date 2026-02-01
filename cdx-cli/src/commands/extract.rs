//! Extract command implementation.

use anyhow::{Context, Result};
use cdx_core::archive::CdxReader;
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

    // Asset extraction
    if asset_name.is_some() || all_assets {
        let asset_results = extract_assets(
            &file,
            &output_dir,
            asset_name.as_deref(),
            all_assets,
            config,
        )?;
        extracted_items.extend(asset_results);
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

/// Extract assets from a Codex document.
fn extract_assets(
    file: &PathBuf,
    output_dir: &PathBuf,
    asset_name: Option<&str>,
    all_assets: bool,
    config: &OutputConfig,
) -> Result<Vec<serde_json::Value>> {
    let mut reader = CdxReader::open(file)
        .with_context(|| format!("Failed to open archive: {}", file.display()))?;

    let mut extracted = Vec::new();

    // Get list of all files in the archive
    let file_names = reader.file_names();

    // Filter to asset paths (assets/ directory)
    let asset_files: Vec<&String> = file_names
        .iter()
        .filter(|name| name.starts_with("assets/"))
        .collect();

    let total_assets = asset_files.len();

    if asset_files.is_empty() {
        if config.json {
            extracted.push(serde_json::json!({
                "type": "assets",
                "count": 0,
                "note": "No assets found in document"
            }));
        } else {
            config.info("No assets found in document");
        }
        return Ok(extracted);
    }

    // Determine which assets to extract
    let to_extract: Vec<&String> = if all_assets {
        asset_files
    } else if let Some(name) = asset_name {
        // Find the asset by name (can be full path or just filename)
        let matches: Vec<&String> = asset_files
            .iter()
            .filter(|path| {
                // Match by full path or by filename
                ***path == format!("assets/{name}")
                    || path.ends_with(&format!("/{name}"))
                    || ***path == name
            })
            .copied()
            .collect();

        if matches.is_empty() {
            // Try partial match
            let partial_matches: Vec<&String> = asset_files
                .iter()
                .filter(|path| path.contains(name))
                .copied()
                .collect();

            if partial_matches.is_empty() {
                if config.json {
                    extracted.push(serde_json::json!({
                        "type": "assets",
                        "error": format!("Asset '{}' not found", name),
                        "available": asset_files
                    }));
                } else {
                    config.warning(&format!("Asset '{}' not found", name));
                    config.info("Available assets:");
                    for asset in &asset_files {
                        println!("  - {}", asset);
                    }
                }
                return Ok(extracted);
            }
            partial_matches
        } else {
            matches
        }
    } else {
        Vec::new()
    };

    // Extract each asset
    let mut extracted_count = 0;
    for asset_path in to_extract {
        let data = reader
            .read_file(asset_path)
            .with_context(|| format!("Failed to read asset: {}", asset_path))?;

        // Create the output path, preserving directory structure under assets/
        let relative_path = asset_path.strip_prefix("assets/").unwrap_or(asset_path);
        let output_path = output_dir.join(relative_path);

        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        fs::write(&output_path, &data)
            .with_context(|| format!("Failed to write asset: {}", output_path.display()))?;

        if !config.json {
            config.success(&format!(
                "Extracted: {} ({} bytes)",
                output_path.display(),
                data.len()
            ));
        }
        extracted_count += 1;
    }

    if config.json {
        extracted.push(serde_json::json!({
            "type": "assets",
            "count": extracted_count,
            "total_available": total_assets
        }));
    } else if extracted_count > 0 {
        config.success(&format!(
            "Extracted {} asset(s) to {}",
            extracted_count,
            output_dir.display()
        ));
    }

    Ok(extracted)
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
        Block::Admonition(adm) => {
            // Include admonition title if present
            if let Some(ref title) = adm.title {
                output.push(format!("[{}] {}", adm.variant, title));
            } else {
                output.push(format!("[{}]", adm.variant));
            }
            // Extract text from children
            for child in &adm.children {
                extract_block_text(child, output);
            }
        }
        _ => {}
    }
}

fn extract_text_nodes(nodes: &[Text]) -> String {
    nodes.iter().map(|node| node.value.clone()).collect()
}
