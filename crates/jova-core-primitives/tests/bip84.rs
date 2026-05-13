use jova_core_primitives::DerivationPath;
#[cfg(not(miri))]
use jova_core_primitives::{Mnemonic, derive_secp256k1};

// BIP-84 official test vector mnemonic (12 words).
// Source: https://github.com/bitcoin/bips/blob/master/bip-0084.mediawiki
#[cfg(not(miri))]
const BIP84_VECTOR_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

#[test]
#[cfg(not(miri))] // secp256k1 uses extern static (C FFI); miri cannot analyse it
fn bip84_account0_external_index0_pubkey() {
    let seed = Mnemonic::to_seed(BIP84_VECTOR_MNEMONIC, "").expect("seed");
    let path = DerivationPath::parse("m/84'/0'/0'/0/0").expect("path");
    let xprv = derive_secp256k1(&seed, &path).expect("derive");
    let pubkey_compressed = xprv.public_key_compressed();
    // Expected first external pubkey from BIP-84 vectors.
    let expected_hex = "0330d54fd0dd420a6e5f8d3624f5f3482cae350f79d5f0753bf5beef9c2d91af3c";
    assert_eq!(hex::encode(pubkey_compressed), expected_hex);
}

#[test]
#[cfg(not(miri))]
fn bip84_account0_external_index1_pubkey() {
    let seed = Mnemonic::to_seed(BIP84_VECTOR_MNEMONIC, "").expect("seed");
    let path = DerivationPath::parse("m/84'/0'/0'/0/1").expect("path");
    let xprv = derive_secp256k1(&seed, &path).expect("derive");
    let pubkey_compressed = xprv.public_key_compressed();
    // BIP-84 vector second external pubkey.
    let expected_hex = "03e775fd51f0dfb8cd865d9ff1cca2a158cf651fe997fdc9fee9c1d3b5e995ea77";
    assert_eq!(hex::encode(pubkey_compressed), expected_hex);
}

#[test]
#[cfg(not(miri))]
fn bip84_account0_change_index0_pubkey() {
    let seed = Mnemonic::to_seed(BIP84_VECTOR_MNEMONIC, "").expect("seed");
    let path = DerivationPath::parse("m/84'/0'/0'/1/0").expect("path");
    let xprv = derive_secp256k1(&seed, &path).expect("derive");
    let pubkey_compressed = xprv.public_key_compressed();
    // BIP-84 vector first internal (change) pubkey.
    let expected_hex = "03025324888e429ab8e3dbaf1f7802648b9cd01e9b418485c5fa4c1b9b5700e1a6";
    assert_eq!(hex::encode(pubkey_compressed), expected_hex);
}

#[test]
fn bip84_path_helper_matches_parse() {
    let helper = DerivationPath::bip84_path(0, 0, 0).expect("helper");
    let parsed = DerivationPath::parse("m/84'/0'/0'/0/0").expect("parse");
    assert_eq!(helper, parsed);
}

#[test]
fn bip84_path_helper_change_branch() {
    let helper = DerivationPath::bip84_path(0, 1, 0).expect("helper");
    let parsed = DerivationPath::parse("m/84'/0'/0'/1/0").expect("parse");
    assert_eq!(helper, parsed);
}

#[test]
fn bip84_path_helper_account_5() {
    let helper = DerivationPath::bip84_path(5, 0, 0).expect("helper");
    let parsed = DerivationPath::parse("m/84'/0'/5'/0/0").expect("parse");
    assert_eq!(helper, parsed);
}

#[test]
fn bip84_path_helper_rejects_oversized_account() {
    // account >= 2^31 would alias hardened components and is rejected.
    let r = DerivationPath::bip84_path(0x8000_0000, 0, 0);
    assert!(r.is_err());
}

#[test]
fn bip84_path_helper_rejects_oversized_change() {
    let r = DerivationPath::bip84_path(0, 0x8000_0000, 0);
    assert!(r.is_err());
}

#[test]
fn bip84_path_helper_rejects_oversized_index() {
    let r = DerivationPath::bip84_path(0, 0, 0x8000_0000);
    assert!(r.is_err());
}
