//! Fuzz target for CDX archive parsing.
//!
//! This target tests the robustness of the archive reader when handling
//! arbitrary (potentially malformed) byte sequences as CDX documents.

#![no_main]

use libfuzzer_sys::fuzz_target;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    // Attempt to parse arbitrary bytes as a CDX archive
    // This exercises the ZIP parsing and manifest validation code paths
    let cursor = Cursor::new(data);
    let _ = cdx_core::Document::open_from_reader(cursor);
});
