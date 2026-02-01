//! Example: Extract text content from a Codex document.
//!
//! This example demonstrates how to iterate over the content blocks
//! in a document and extract text.
//!
//! Usage: cargo run --example extract_content <path-to-cdx-file>

use cdx_core::content::{Block, Text};
use cdx_core::{Document, Result};
use std::env;

fn main() -> Result<()> {
    // Get the file path from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        // Create a sample document for demonstration
        println!("No file specified, creating sample document...\n");
        run_with_sample_document()
    } else {
        let file_path = &args[1];
        println!("Opening document: {file_path}");
        let document = Document::open(file_path)?;
        extract_content(&document)
    }
}

fn run_with_sample_document() -> Result<()> {
    let document = Document::builder()
        .title("Sample Document")
        .creator("Extract Example")
        .add_heading(1, "Main Title")
        .add_paragraph("This is the first paragraph with some important text.")
        .add_heading(2, "Section One")
        .add_paragraph("Content in section one discusses various topics.")
        .add_heading(2, "Section Two")
        .add_paragraph("Another section with different content.")
        .build()?;

    extract_content(&document)
}

fn extract_content(document: &Document) -> Result<()> {
    println!("=== Document Content ===\n");

    for (i, block) in document.content().blocks.iter().enumerate() {
        print_block(block, i, 0);
    }

    Ok(())
}

fn print_block(block: &Block, index: usize, depth: usize) {
    let indent = "  ".repeat(depth);

    match block {
        Block::Paragraph { children, .. } => {
            println!("{indent}[{index}] Paragraph: {}", extract_text(children));
        }
        Block::Heading {
            level, children, ..
        } => {
            println!("{indent}[{index}] H{level}: {}", extract_text(children));
        }
        Block::List {
            ordered, children, ..
        } => {
            let list_type = if *ordered { "Ordered" } else { "Unordered" };
            println!("{indent}[{index}] {list_type} List:");
            for (j, child) in children.iter().enumerate() {
                print_block(child, j, depth + 1);
            }
        }
        Block::ListItem { children, .. } => {
            println!("{indent}[{index}] List Item:");
            for (j, child) in children.iter().enumerate() {
                print_block(child, j, depth + 1);
            }
        }
        Block::CodeBlock {
            language, children, ..
        } => {
            let lang = language.as_deref().unwrap_or("plain");
            let code_text = extract_text(children);
            println!("{indent}[{index}] Code ({lang}):");
            for line in code_text.lines() {
                println!("{indent}  | {line}");
            }
        }
        Block::Blockquote { children, .. } => {
            println!("{indent}[{index}] Blockquote:");
            for (j, child) in children.iter().enumerate() {
                print_block(child, j, depth + 1);
            }
        }
        Block::Image(img) => {
            println!("{indent}[{index}] Image: {} ({})", img.alt, img.src);
        }
        Block::HorizontalRule { .. } => {
            println!("{indent}[{index}] ---");
        }
        Block::Math(math) => {
            println!("{indent}[{index}] Math ({:?}): {}", math.format, math.value);
        }
        Block::Table { children, .. } => {
            println!("{indent}[{index}] Table:");
            for (j, child) in children.iter().enumerate() {
                print_block(child, j, depth + 1);
            }
        }
        Block::TableRow { children, .. } => {
            println!("{indent}[{index}] Row:");
            for (j, child) in children.iter().enumerate() {
                print_block(child, j, depth + 1);
            }
        }
        Block::TableCell(cell) => {
            let text = extract_text(&cell.children);
            println!("{indent}[{index}] Cell: {text}");
        }
        Block::Break { .. } => {
            println!("{indent}[{index}] <br>");
        }
        Block::Extension(ext) => {
            println!(
                "{indent}[{index}] Extension: {}:{}",
                ext.namespace, ext.block_type
            );
            for (j, child) in ext.children.iter().enumerate() {
                print_block(child, j, depth + 1);
            }
        }
        Block::DefinitionList(dl) => {
            println!("{indent}[{index}] Definition List:");
            for (j, child) in dl.children.iter().enumerate() {
                print_block(child, j, depth + 1);
            }
        }
        Block::DefinitionItem { children, .. } => {
            println!("{indent}[{index}] Definition Item:");
            for (j, child) in children.iter().enumerate() {
                print_block(child, j, depth + 1);
            }
        }
        Block::DefinitionTerm { children, .. } => {
            println!(
                "{indent}[{index}] Definition Term: {}",
                extract_text(children)
            );
        }
        Block::DefinitionDescription { children, .. } => {
            println!("{indent}[{index}] Definition Description:");
            for (j, child) in children.iter().enumerate() {
                print_block(child, j, depth + 1);
            }
        }
        Block::Measurement(m) => {
            println!("{indent}[{index}] Measurement: {}", m.display);
        }
        Block::Signature(sig) => {
            println!("{indent}[{index}] Signature: {:?}", sig.signature_type);
        }
        Block::Svg(svg) => {
            let src = svg.src.as_deref().unwrap_or("[inline]");
            println!("{indent}[{index}] SVG: {src}");
        }
        Block::Barcode(bc) => {
            println!("{indent}[{index}] Barcode ({:?}): {}", bc.format, bc.alt);
        }
        Block::Figure(fig) => {
            println!("{indent}[{index}] Figure:");
            for (j, child) in fig.children.iter().enumerate() {
                print_block(child, j, depth + 1);
            }
        }
        Block::FigCaption(fc) => {
            println!(
                "{indent}[{index}] Figure Caption: {}",
                extract_text(&fc.children)
            );
        }
        Block::Admonition(adm) => {
            let title = adm.title.as_deref().unwrap_or_default();
            println!("{indent}[{index}] Admonition ({:?}): {title}", adm.variant);
            for (j, child) in adm.children.iter().enumerate() {
                print_block(child, j, depth + 1);
            }
        }
    }
}

fn extract_text(texts: &[Text]) -> String {
    texts
        .iter()
        .map(|t| t.value.as_str())
        .collect::<Vec<_>>()
        .join("")
}
