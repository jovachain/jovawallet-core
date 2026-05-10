#![no_std]
extern crate alloc;

pub fn smoke() -> &'static str {
    let _ = core::any::type_name::<bip39::Mnemonic>();
    let _ = core::any::type_name::<secp256k1::SecretKey>();
    let _ = core::any::type_name::<ed25519_dalek::SigningKey>();
    "nostd-ok"
}
