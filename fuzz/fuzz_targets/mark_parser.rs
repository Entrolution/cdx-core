//! Fuzz target for Mark deserialization.
//!
//! This target tests the custom `Deserialize` impl for `Mark` which
//! handles both simple string marks (e.g., `"bold"`) and complex object
//! marks (e.g., `{"type": "link", "href": "..."}`). Exercises all 15+
//! mark type code paths with arbitrary input.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Try parsing as a single Mark
    let _ = serde_json::from_slice::<cdx_core::content::Mark>(data);

    // Also try parsing as a Vec<Mark> (text node marks array)
    let _ = serde_json::from_slice::<Vec<cdx_core::content::Mark>>(data);
});
