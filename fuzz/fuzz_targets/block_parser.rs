//! Fuzz target for Block deserialization.
//!
//! This target tests the custom `Deserialize` impl for `Block` which
//! dispatches on the `"type"` field to construct the correct variant.
//! Exercises all 20+ block type code paths with arbitrary JSON input.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Try parsing as a single Block
    let _ = serde_json::from_slice::<cdx_core::content::Block>(data);

    // Also try parsing as a Vec<Block> (nested children)
    let _ = serde_json::from_slice::<Vec<cdx_core::content::Block>>(data);
});
