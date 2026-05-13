#![no_main]
use libfuzzer_sys::fuzz_target;
use jova_core::{is_valid_address, JovaChain};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = is_valid_address(s, &JovaChain::Ethereum);
    }
});
