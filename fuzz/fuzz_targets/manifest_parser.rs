//! Fuzz target for manifest JSON parsing.
//!
//! This target tests the robustness of the manifest parser when handling
//! arbitrary (potentially malformed) JSON data.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Attempt to parse arbitrary bytes as manifest JSON
    // This exercises the serde deserialization code paths
    let _ = serde_json::from_slice::<cdx_core::Manifest>(data);
});
