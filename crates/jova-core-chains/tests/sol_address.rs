//! Solana address derivation + validation tests.

use jova_core_chains::sol::{derive_sol_address, validate_sol_address};
use jova_core_primitives::{Mnemonic, derive_ed25519};

const HARDENED: u32 = 0x8000_0000;
const MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// Phantom/Solflare canonical path: m/44'/501'/0'/0'/0' (all hardened).
fn sol_path() -> [u32; 5] {
    [HARDENED | 44, HARDENED | 501, HARDENED, HARDENED, HARDENED]
}

#[test]
fn sol_address_for_abandon_seed_matches_bip_utils() {
    let seed = Mnemonic::to_seed(MNEMONIC, "").expect("seed");
    let xprv = derive_ed25519(&seed, &sol_path()).expect("derive");
    let addr = derive_sol_address(&xprv).expect("derive_sol_address");

    // Captured from bip_utils.Bip44Coins.SOLANA against the same BIP-39 seed.
    // See tools/sol-vector-capture/capture.sh.
    let expected =
        include_str!("../../../tools/sol-vector-capture/captures/abandon_account_0.pubkey_b58")
            .trim();
    assert_eq!(addr, expected);
}

#[test]
fn validates_known_sol_addresses() {
    // The captured abandon-mnemonic address (deterministically valid).
    let captured =
        include_str!("../../../tools/sol-vector-capture/captures/abandon_account_0.pubkey_b58")
            .trim();
    assert!(validate_sol_address(captured));

    // System program (all-zero pubkey). Always 32 ones in base58.
    assert!(validate_sol_address("11111111111111111111111111111111"));

    // Wrapped SOL mint (well-known mainnet account).
    assert!(validate_sol_address(
        "So11111111111111111111111111111111111111112"
    ));
}

#[test]
fn rejects_malformed_sol_addresses() {
    // Empty.
    assert!(!validate_sol_address(""));
    // Too short (28 chars).
    assert!(!validate_sol_address("1111111111111111111111111111"));
    // Too long (45 chars — outside the 32–44 envelope for a 32-byte payload).
    assert!(!validate_sol_address(
        "111111111111111111111111111111111111111111111"
    ));
    // EVM-shaped (contains 'x' which is in base58, but length is 42).
    // Pubkey parser rejects: the decoded payload isn't 32 bytes.
    assert!(!validate_sol_address(
        "0x0000000000000000000000000000000000000000"
    ));
    // BTC P2WPKH-shaped (`bc1q…`, length 42, contains '0' which is NOT in base58).
    assert!(!validate_sol_address(
        "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
    ));
    // XRP classic-shaped (`r…`, length 34, valid base58 alphabet but wrong
    // payload length when decoded).
    assert!(!validate_sol_address("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"));
}
