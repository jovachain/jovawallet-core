#![no_main]
use jova_core::{JovaChain, is_valid_address};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = is_valid_address(s, &JovaChain::Bitcoin);
    }
});
