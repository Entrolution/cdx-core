//! Example: Open an existing Codex document and verify its integrity.
//!
//! This example demonstrates how to open a .cdx file and perform
//! integrity verification.
//!
//! Usage: cargo run --example open_and_verify <path-to-cdx-file>

use cdx_core::{Document, Result};
use std::env;

fn main() -> Result<()> {
    // Get the file path from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path-to-cdx-file>", args[0]);
        eprintln!("Example: {} document.cdx", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    println!("Opening document: {file_path}");

    // Open the document
    let document = Document::open(file_path)?;
    println!("Document opened successfully!");

    // Display basic information
    println!("\n=== Document Information ===");
    println!("State: {:?}", document.state());
    println!("Title: {:?}", document.dublin_core().terms.title);
    println!("Creator: {:?}", document.dublin_core().terms.creator);

    // Verify document integrity
    println!("\n=== Verification ===");
    let report = document.verify()?;

    println!("Document ID valid: {}", report.id_valid);
    println!("Content valid: {}", report.content_valid);
    println!("Overall valid: {}", report.is_valid());

    if !report.errors.is_empty() {
        println!("\nErrors:");
        for error in &report.errors {
            println!("  - {error}");
        }
    }

    if report.is_valid() {
        println!("\nDocument integrity verified successfully!");
    } else {
        println!("\nWARNING: Document integrity verification failed!");
    }

    Ok(())
}
