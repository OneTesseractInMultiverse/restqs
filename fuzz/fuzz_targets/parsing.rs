//! Bounded parser fuzz entry point: vary input budgets and check every accepted plan's field/operator invariants.

#![no_main]
use libfuzzer_sys::fuzz_target;
use restqs_robustness::{fields_are_resolved, limits_from_seed, operators_are_valid, parse};

fuzz_target!(|data: &[u8]| {
    if data.len() > 4096 {
        return;
    }
    let raw = String::from_utf8_lossy(data);
    if let Ok(query) = parse(&raw, limits_from_seed(data)) {
        assert!(fields_are_resolved(&query) && operators_are_valid(&query));
    }
});
