//! Fuzz target for Content deserialization.
//!
//! This target tests parsing of the full `Content` structure which
//! contains a version field and a blocks array. Exercises the complete
//! content deserialization pipeline including nested blocks and text
//! nodes with marks.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Try parsing as a full Content structure
    let _ = serde_json::from_slice::<cdx_core::content::Content>(data);
});
