//! XRP classic address derivation + validation tests.

use jova_core_chains::xrp::{derive_xrp_address, validate_xrp_address};
use jova_core_primitives::{DerivationPath, Mnemonic, derive_secp256k1};

const MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

#[test]
fn xrp_address_for_abandon_seed_account_0_matches_xrpl_py() {
    let seed = Mnemonic::to_seed(MNEMONIC, "").expect("seed");
    let path = DerivationPath::parse("m/44'/144'/0'/0/0").expect("path");
    let xprv = derive_secp256k1(&seed, &path).expect("derive");
    let addr = derive_xrp_address(&xprv).expect("derive_xrp_address");

    // Captured from xrpl-py 4.5 + bip_utils 2.x against the same BIP-39 seed
    // at coin type 144 (Bip44Coins.RIPPLE). See
    // tools/xrp-vector-capture/capture.sh.
    let expected =
        include_str!("../../../tools/xrp-vector-capture/captures/abandon_account0.address");
    assert_eq!(addr, expected.trim());
}

#[test]
fn validates_known_xrp_addresses() {
    // The capture address (deterministically valid).
    let expected =
        include_str!("../../../tools/xrp-vector-capture/captures/abandon_account0.address").trim();
    assert!(validate_xrp_address(expected));

    // XRPL spec example addresses (xrpl-py docs).
    assert!(validate_xrp_address("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"));
    assert!(validate_xrp_address("rpGaCyHRYbgKhErgFih3RdjJqXDsYBouz3"));
}

#[test]
fn rejects_malformed_xrp_addresses() {
    // Empty.
    assert!(!validate_xrp_address(""));
    // Random garbage.
    assert!(!validate_xrp_address("xxx"));
    // EVM-shaped.
    assert!(!validate_xrp_address(
        "0x0000000000000000000000000000000000000000"
    ));
    // BTC P2WPKH-shaped.
    assert!(!validate_xrp_address(
        "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
    ));
    // Bad checksum (flipped last char of a valid address).
    assert!(!validate_xrp_address("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYf"));
}
