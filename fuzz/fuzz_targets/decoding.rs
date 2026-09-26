//! Bounded decoding fuzz entry point: exercise hostile raw text and exact encoded Unicode round trips.

#![no_main]
use libfuzzer_sys::fuzz_target;
use restqs_robustness::{decoded_text_matches, generous_limits, parse};

fuzz_target!(|data: &[u8]| {
    if data.len() > 4096 {
        return;
    }
    let raw = String::from_utf8_lossy(data);
    let _ = parse(&format!("text={raw}"), generous_limits());
    assert!(decoded_text_matches(&raw));
});
