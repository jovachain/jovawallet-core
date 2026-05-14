//! SLIP-10 ed25519 derivation parity tests.
//!
//! Cross-checks `derive_ed25519` against the bip_utils 2.x reference for
//! `Bip44Coins.SOLANA` (path `m/44'/501'/0'/0'/0'`, all hardened). The
//! captured pubkey lives at
//! `tools/sol-vector-capture/captures/abandon_account_0.pubkey_hex`; the
//! address (base58 form) lives at `.pubkey_b58` and is asserted by the
//! companion address test in `jova-core-chains`.

use jova_core_primitives::{Ed25519DeriveError, Mnemonic, derive_ed25519};

const HARDENED: u32 = 0x8000_0000;
const MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// Standard Solana BIP-44 path used by Phantom / Solflare and bip_utils:
/// m/44'/501'/0'/0'/0' (all hardened).
fn sol_path() -> [u32; 5] {
    [HARDENED | 44, HARDENED | 501, HARDENED, HARDENED, HARDENED]
}

#[test]
fn slip10_ed25519_matches_bip_utils_for_abandon_seed() {
    let seed = Mnemonic::to_seed(MNEMONIC, "").expect("seed");
    let xprv = derive_ed25519(&seed, &sol_path()).expect("derive");

    let expected_pub_hex =
        include_str!("../../../tools/sol-vector-capture/captures/abandon_account_0.pubkey_hex")
            .trim();
    let expected_priv_hex = include_str!(
        "../../../tools/sol-vector-capture/captures/abandon_account_0.private_key_hex"
    )
    .trim();

    assert_eq!(
        hex::encode(xprv.public_key()),
        expected_pub_hex,
        "ed25519 pubkey mismatch vs bip_utils reference",
    );
    assert_eq!(
        hex::encode(xprv.secret_bytes()),
        expected_priv_hex,
        "ed25519 secret mismatch vs bip_utils reference",
    );
}

#[test]
fn slip10_ed25519_rejects_non_hardened_path() {
    let seed = Mnemonic::to_seed(MNEMONIC, "").expect("seed");
    // m/44'/501'/0'/0/0  — last two components NOT hardened (would be valid
    // for secp256k1 BIP-44 but is invalid under SLIP-10 ed25519).
    let bad_path = [HARDENED | 44, HARDENED | 501, HARDENED, 0, 0];
    let err = derive_ed25519(&seed, &bad_path).expect_err("should reject non-hardened");
    assert_eq!(err, Ed25519DeriveError::HardenedRequired);
}

#[test]
fn slip10_ed25519_rejects_single_non_hardened_component() {
    let seed = Mnemonic::to_seed(MNEMONIC, "").expect("seed");
    // m/44/501'/0'/0'/0'  — first component NOT hardened.
    let bad_path = [44, HARDENED | 501, HARDENED, HARDENED, HARDENED];
    let err = derive_ed25519(&seed, &bad_path).expect_err("should reject non-hardened");
    assert_eq!(err, Ed25519DeriveError::HardenedRequired);
}

#[test]
fn slip10_ed25519_empty_path_returns_master_key() {
    // Empty path == master key (still valid; matches BIP-32 semantics).
    let seed = Mnemonic::to_seed(MNEMONIC, "").expect("seed");
    let xprv = derive_ed25519(&seed, &[]).expect("derive master");
    // Sanity: master pubkey is 32 bytes and differs from derived pubkey.
    let pub_master = xprv.public_key();
    assert_eq!(pub_master.len(), 32);
    let xprv_sol = derive_ed25519(&seed, &sol_path()).expect("derive sol");
    assert_ne!(pub_master, xprv_sol.public_key());
}
