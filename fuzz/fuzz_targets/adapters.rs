#![no_main]
use libfuzzer_sys::fuzz_target;
use restqs_robustness::adapter_input_contract;

fuzz_target!(|data: &[u8]| {
    if data.len() > 4096 {
        return;
    }
    let raw = String::from_utf8_lossy(data);
    let first = usize::from(data.first().copied().unwrap_or(0)) + 1;
    assert!(adapter_input_contract(&raw, first));
});
