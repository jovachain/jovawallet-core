//! Track 0: private-key import unit tests (secp256k1 chains in this file;
//! Solana ed25519 covered in the same file once Task 3 lands).
//!
//! Known vector: the secp256k1 test scalar
//! 0x4646464646464646464646464646464646464646464646464646464646464646
//! (EIP-155 example key) maps to EVM address
//! 0x9d8A62f656a8d1615C1294fd71e9CFb3E4855A4F.

use jova_core::{JovaChain, JovaError, JovaWallet, is_valid_address};

/// EIP-155 example private key (32 bytes of 0x46).
const EIP155_KEY: &str =
    "4646464646464646464646464646464646464646464646464646464646464646";
const EIP155_EVM_ADDR: &str = "0x9d8A62f656a8d1615C1294fd71e9CFb3E4855A4F";

#[test]
fn from_private_key_evm_derives_known_address() {
    let wallet = JovaWallet::from_private_key(EIP155_KEY, &JovaChain::Ethereum)
        .expect("valid secp256k1 key");
    let addr = wallet.address(&JovaChain::Ethereum, 0).expect("derive");
    assert_eq!(
        addr.value.to_lowercase(),
        EIP155_EVM_ADDR.to_lowercase(),
        "EVM address from imported key must match the EIP-155 example"
    );
}

#[test]
fn from_private_key_accepts_0x_prefix() {
    let with_prefix = format!("0x{EIP155_KEY}");
    let wallet = JovaWallet::from_private_key(&with_prefix, &JovaChain::Ethereum)
        .expect("0x prefix tolerated");
    let addr = wallet.address(&JovaChain::Ethereum, 0).expect("derive");
    assert_eq!(addr.value.to_lowercase(), EIP155_EVM_ADDR.to_lowercase());
}

#[test]
fn from_private_key_rejects_bad_hex() {
    let err = JovaWallet::from_private_key("zz46", &JovaChain::Ethereum).unwrap_err();
    assert!(
        matches!(err, JovaError::InvalidPrivateKey { ref reason } if reason == "not_hex"),
        "non-hex must be InvalidPrivateKey{{reason=not_hex}}, got {err:?}"
    );
}

#[test]
fn from_private_key_rejects_wrong_length() {
    // 31 bytes.
    let short = "46".repeat(31);
    let err = JovaWallet::from_private_key(&short, &JovaChain::Ethereum).unwrap_err();
    assert!(
        matches!(err, JovaError::InvalidPrivateKey { ref reason } if reason == "expected_32_bytes"),
        "31-byte key must be InvalidPrivateKey{{reason=expected_32_bytes}}, got {err:?}"
    );
}

#[test]
fn from_private_key_rejects_zero_scalar() {
    let zero = "00".repeat(32);
    let err = JovaWallet::from_private_key(&zero, &JovaChain::Ethereum).unwrap_err();
    assert!(
        matches!(err, JovaError::InvalidPrivateKey { ref reason } if reason == "secp256k1_scalar_out_of_range"),
        "zero scalar is not a valid secp256k1 key, got {err:?}"
    );
}

#[test]
fn key_material_wallet_rejects_unbound_chain() {
    // An Ethereum-bound key must refuse to derive a Polygon address even
    // though both are secp256k1 — strict single-chain scoping.
    let wallet = JovaWallet::from_private_key(EIP155_KEY, &JovaChain::Ethereum)
        .expect("valid key");
    let err = wallet.address(&JovaChain::Polygon, 0).unwrap_err();
    assert!(
        matches!(err, JovaError::UnsupportedChain(_)),
        "bound=ethereum, asked=polygon must be UnsupportedChain, got {err:?}"
    );
}

/// Pinned snapshot: the EIP-155 secp256k1 key (32 bytes of 0x46) imported as
/// a Bitcoin wallet produces this P2WPKH (BIP-84 bech32) address.
///
/// Why this is sound: BTC address derivation from an imported leaf key reads
/// only the raw secp256k1 secret → public key → hash160 → bech32 encode.
/// The chain code set to zero in `derive_for(Bitcoin)` is never used for
/// leaf-address derivation (only for BIP-32 child key derivation, which does
/// not occur here).  The EVM known-vector test (`from_private_key_evm_derives_known_address`)
/// already proves that this key's public key is correct (the EVM address is
/// Keccak of the same uncompressed public key point).  Two assertions:
/// (a) the address passes `is_valid_address` so it is syntactically well-formed;
/// (b) the exact string is pinned so any future regression breaks a test.
///
/// No leaf private key appears in the BTC spec test-vectors.json (those vectors
/// are mnemonic-derived, not raw-key-derived), so a cross-check against
/// vectors_btc.rs is not possible.  Pinned snapshot is the correct approach.
const EIP155_BTC_ADDR: &str = "bc1qhkfq3zahaqkkzx5mjnamwjsfpq2jk7z00ppggv";

#[test]
fn from_private_key_bitcoin_derives_valid_address() {
    let wallet = JovaWallet::from_private_key(EIP155_KEY, &JovaChain::Bitcoin)
        .expect("valid secp256k1 key");
    let addr = wallet.address(&JovaChain::Bitcoin, 0).expect("derive");

    // (a) Well-formed P2WPKH address
    assert!(
        is_valid_address(&addr.value, &JovaChain::Bitcoin),
        "derived BTC address must pass is_valid_address, got: {}",
        addr.value
    );
    // (b) Pinned regression snapshot (see module-level comment above)
    assert_eq!(
        addr.value, EIP155_BTC_ADDR,
        "BTC address from imported EIP-155 key must match pinned snapshot"
    );
}

/// Pinned snapshot: the EIP-155 secp256k1 key (32 bytes of 0x46) imported as
/// an XRP wallet produces this classic XRPL address.
///
/// Why this is sound: XRP address derivation reads only the secp256k1
/// compressed public key → SHA256 → RIPEMD-160 → XRPL base58check encode.
/// The chain code is unused for leaf address derivation (same reasoning as
/// the Bitcoin test above).  Two assertions:
/// (a) the address passes `is_valid_address` (r-prefix, valid checksum);
/// (b) the exact string is pinned as a regression snapshot.
///
/// No leaf private key is exposed in vectors_xrp.rs (those vectors are
/// mnemonic-derived), so a cross-check against that file is not possible.
/// Pinned snapshot is the correct approach.
const EIP155_XRP_ADDR: &str = "rJHMeqKu8Ep7Fazx8MQG6JunaafBXz93YQ";

#[test]
fn from_private_key_xrp_derives_valid_address() {
    let wallet = JovaWallet::from_private_key(EIP155_KEY, &JovaChain::Xrp)
        .expect("valid secp256k1 key");
    let addr = wallet.address(&JovaChain::Xrp, 0).expect("derive");

    // (a) Well-formed r-prefix XRPL classic address
    assert!(
        is_valid_address(&addr.value, &JovaChain::Xrp),
        "derived XRP address must pass is_valid_address, got: {}",
        addr.value
    );
    // (b) Pinned regression snapshot (see module-level comment above)
    assert_eq!(
        addr.value, EIP155_XRP_ADDR,
        "XRP address from imported EIP-155 key must match pinned snapshot"
    );
}

// ── Solana (ed25519) ──────────────────────────────────────────────────────

/// 32-byte ed25519 secret scalar of all 0x01. Its base58 Solana address is
/// deterministic; we assert via round-trip (validate_sol_address) rather than
/// hardcoding, then lock the exact value once observed.
const ED25519_KEY: &str =
    "0101010101010101010101010101010101010101010101010101010101010101";

#[test]
fn from_private_key_solana_derives_valid_address() {
    let wallet = JovaWallet::from_private_key(ED25519_KEY, &JovaChain::Solana)
        .expect("valid ed25519 key");
    let addr = wallet.address(&JovaChain::Solana, 0).expect("derive");
    assert!(
        jova_core::is_valid_address(&addr.value, &JovaChain::Solana),
        "derived Solana address must validate: {}",
        addr.value
    );
    // Lock the deterministic value (base58 of ed25519 pubkey for 0x01*32).
    // Observed from SDK's derive_sol_address output; authoritative per Step 6.
    assert_eq!(
        addr.value, "AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9",
        "Solana address for the all-0x01 ed25519 secret"
    );
}

#[test]
fn from_private_key_solana_accepts_any_32_bytes() {
    // ed25519 has no scalar-range restriction; the zero secret is valid.
    let zero = "00".repeat(32);
    assert!(JovaWallet::from_private_key(&zero, &JovaChain::Solana).is_ok());
}

#[test]
fn from_private_key_solana_rejects_wrong_length() {
    let short = "01".repeat(31);
    let err = JovaWallet::from_private_key(&short, &JovaChain::Solana).unwrap_err();
    assert!(matches!(err, JovaError::InvalidPrivateKey { .. }), "got {err:?}");
}

#[test]
fn secp_key_cannot_sign_solana_and_vice_versa() {
    use jova_core::{SignableMessage, UnsignedTx};
    // An EVM-bound key signing a Solana tx must be UnsupportedChain.
    let evm_wallet = JovaWallet::from_private_key(EIP155_KEY, &JovaChain::Ethereum)
        .expect("valid key");
    let sol_tx = UnsignedTx::Solana {
        message_base64: "AA==".into(),
        recent_blockhash: "11111111111111111111111111111111".into(),
    };
    let err = evm_wallet.sign_tx(&sol_tx).unwrap_err();
    assert!(
        matches!(err, JovaError::UnsupportedChain(_)),
        "evm key signing solana tx must be UnsupportedChain, got {err:?}"
    );

    // A Solana-bound key signing an EVM message must be UnsupportedChain.
    let sol_wallet = JovaWallet::from_private_key(ED25519_KEY, &JovaChain::Solana)
        .expect("valid key");
    let evm_msg = SignableMessage::EvmPersonalSign { message: "hi".into() };
    let err2 = sol_wallet.sign_message(&evm_msg).unwrap_err();
    assert!(
        matches!(err2, JovaError::UnsupportedChain(_)),
        "solana key signing evm message must be UnsupportedChain, got {err2:?}"
    );
}

// ── Positive same-chain signing tests ────────────────────────────────────

#[test]
fn imported_evm_key_can_sign_evm_tx() {
    use jova_core::{EvmUnsigned, UnsignedTx};
    let wallet = JovaWallet::from_private_key(EIP155_KEY, &JovaChain::Ethereum)
        .expect("valid key");
    let unsigned = UnsignedTx::Evm(EvmUnsigned {
        chain_id: 1,
        nonce: 0,
        to: "0x9d8A62f656a8d1615C1294fd71e9CFb3E4855A4F".into(),
        value: "0".into(),
        gas_limit: 21000,
        max_fee_per_gas: "1000000000".into(),
        max_priority_fee_per_gas: "1000000000".into(),
        data: "0x".into(),
        access_list: vec![],
    });
    let signed = wallet.sign_tx(&unsigned).expect("imported EVM key must sign its own chain");
    assert!(
        signed.raw_hex.starts_with("0x"),
        "signed EVM tx raw_hex must be 0x-prefixed, got: {}",
        signed.raw_hex
    );
    assert!(!signed.raw_hex.is_empty(), "signed EVM tx raw_hex must be non-empty");
}

#[test]
fn imported_solana_key_can_sign_solana_message() {
    use jova_core::SignableMessage;
    let wallet = JovaWallet::from_private_key(ED25519_KEY, &JovaChain::Solana)
        .expect("valid key");
    // Sign a trivial 1-byte payload (0x00) encoded as base64 "AA==".
    let msg = SignableMessage::Solana { message_base64: "AA==".into() };
    let signed = wallet.sign_message(&msg).expect("imported Solana key must sign its own chain");
    // Signature should be a non-empty base58 string (64 bytes → ~88 base58 chars).
    assert!(
        !signed.hex.is_empty(),
        "Solana signature must be non-empty"
    );
}
