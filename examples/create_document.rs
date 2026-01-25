//! Example: Create a Codex document from scratch.
//!
//! This example demonstrates how to use the DocumentBuilder API
//! to create a new document with content, metadata, and save it.

use cdx_core::{Document, Result};

fn main() -> Result<()> {
    // Create a new document using the builder API
    let document = Document::builder()
        .title("My First Codex Document")
        .creator("Example Author")
        .add_heading(1, "Introduction")
        .add_paragraph("This is an example Codex document created with cdx-core.")
        .add_heading(2, "Features")
        .add_paragraph("Codex documents provide:")
        .add_paragraph("- Semantic content structure")
        .add_paragraph("- Verifiable integrity via content-addressable IDs")
        .add_paragraph("- Digital signatures for authenticity")
        .add_heading(2, "Conclusion")
        .add_paragraph("The Codex format is designed for the future of digital documents.")
        .build()?;

    // Get document information
    println!("Document created successfully!");
    println!("State: {:?}", document.state());
    println!("Title: {:?}", document.dublin_core().terms.title);
    println!("Creator: {:?}", document.dublin_core().terms.creator);

    // Compute the document ID
    let doc_id = document.compute_id()?;
    println!("Document ID: {doc_id}");

    // Save the document
    let output_path = "example_output.cdx";
    document.save(output_path)?;
    println!("Document saved to: {output_path}");

    Ok(())
}
