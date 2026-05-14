//! Multi-input + multi-party PSBT signing parity tests.
//!
//! Captures come from an INDEPENDENT signer (Python `embit` 0.8.0) operating
//! on the same BIP-84 test mnemonics. If `sign_psbt` produces a different
//! signed hex (multi-input fully-owned case) or a semantically different PSBT
//! (multi-party case), the SDK is wrong — not the vector.
//!
//! Capture procedure: `tools/btc-vector-capture/multi_input.sh` and
//! `tools/btc-vector-capture/multi_party.sh`.

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use bitcoin::psbt::Psbt;
use jova_core_chains::btc::sign_psbt;
use jova_core_primitives::{DerivationPath, Mnemonic, derive_secp256k1};

const MNEMONIC_A: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

const UNSIGNED_PSBT_B64_MULTI: &str =
    include_str!("../../../tools/btc-vector-capture/captures/multi_input.psbt.b64");
const EXPECTED_SIGNED_HEX_MULTI: &str =
    include_str!("../../../tools/btc-vector-capture/captures/multi_input.signed_hex");
const UNSIGNED_PSBT_B64_TWOPARTY: &str =
    include_str!("../../../tools/btc-vector-capture/captures/two_party.psbt.b64");
const EXPECTED_PSBT_AFTER_A_SIGNS: &str =
    include_str!("../../../tools/btc-vector-capture/captures/two_party.after_a.psbt.b64");

fn xprv_a() -> jova_core_primitives::XPrv {
    // The multi-input PSBT captures TWO UTXOs both locking to the SAME P2WPKH
    // address (m/84'/0'/0'/0/0). A wallet may legally hold many UTXOs at a
    // single address; this lets one XPrv sign both inputs without an API
    // change to `sign_psbt(xprv, ...)`.
    let seed = Mnemonic::to_seed(MNEMONIC_A, "").expect("seed");
    let path = DerivationPath::parse("m/84'/0'/0'/0/0").expect("path");
    derive_secp256k1(&seed, &path).expect("derive")
}

#[test]
fn signs_multi_input_psbt_we_own_all_inputs_to_finalized() {
    let result = sign_psbt(&xprv_a(), UNSIGNED_PSBT_B64_MULTI.trim()).expect("signs");
    assert!(result.is_finalized, "expected finalized, got: {result:?}");
    assert_eq!(
        result.tx_hex.expect("tx_hex present").trim().to_lowercase(),
        EXPECTED_SIGNED_HEX_MULTI.trim().to_lowercase(),
        "signed tx hex must match the captured reference byte-for-byte",
    );
    let tx_hash = result.tx_hash.expect("tx_hash present");
    assert_eq!(tx_hash.len(), 64);
    assert!(tx_hash.chars().all(|c| c.is_ascii_hexdigit()));
    assert!(result.psbt_base64.is_none());
}

#[test]
fn multi_party_psbt_returns_unfinalized_after_our_signature() {
    let result = sign_psbt(&xprv_a(), UNSIGNED_PSBT_B64_TWOPARTY.trim()).expect("signs");
    assert!(
        !result.is_finalized,
        "expected unfinalized, got: {result:?}"
    );
    assert!(
        result.tx_hex.is_none(),
        "tx_hex must be None when unfinalized"
    );
    assert!(
        result.tx_hash.is_none(),
        "tx_hash must be None when unfinalized"
    );
    let got_b64 = result.psbt_base64.expect("psbt_base64 present");

    // Option Y — semantic equality.
    //
    // embit and rust-bitcoin both produce BIP-174 PSBTs, but their default
    // field sets are not byte-identical for an in-progress (partially signed)
    // PSBT — small differences in optional metadata (sighash_type marker,
    // ordering of bip32_derivation, presence of unknown fields) prevent a
    // raw base64 comparison from being load-bearing across two heterogeneous
    // signers. The CRYPTOGRAPHIC contract is the partial signature for
    // input 0; that's what we assert byte-for-byte. Input 1 must remain
    // unsigned. The remainder of the PSBT envelope (witness_utxo, the global
    // unsigned tx, bip32 hints) must round-trip unchanged.
    let got = Psbt::deserialize(&BASE64.decode(got_b64.trim()).expect("decode got"))
        .expect("got is valid PSBT");
    let expected = Psbt::deserialize(
        &BASE64
            .decode(EXPECTED_PSBT_AFTER_A_SIGNS.trim())
            .expect("decode expected"),
    )
    .expect("expected is valid PSBT");

    // The unsigned-tx envelope is identical.
    assert_eq!(
        got.unsigned_tx, expected.unsigned_tx,
        "unsigned_tx must match the reference",
    );
    assert_eq!(got.inputs.len(), 2, "must still be a 2-input PSBT");
    assert_eq!(expected.inputs.len(), 2);

    // Input 0: A's partial signature is present and byte-equal to embit's.
    let got_in0 = &got.inputs[0];
    let exp_in0 = &expected.inputs[0];
    assert_eq!(
        got_in0.partial_sigs.len(),
        1,
        "input 0 must have exactly one partial sig (A's)",
    );
    assert_eq!(
        got_in0.partial_sigs, exp_in0.partial_sigs,
        "input 0 partial sig must match embit byte-for-byte (DER sig + sighash byte + pubkey)",
    );
    assert_eq!(
        got_in0.witness_utxo, exp_in0.witness_utxo,
        "input 0 witness_utxo must round-trip unchanged",
    );
    assert!(
        got_in0.final_script_witness.is_none(),
        "input 0 must NOT be finalized in a multi-party flow",
    );

    // Input 1: no signature from A — B has not yet signed.
    let got_in1 = &got.inputs[1];
    assert!(
        got_in1.partial_sigs.is_empty(),
        "input 1 must have NO partial signatures (B hasn't signed)",
    );
    assert!(
        got_in1.final_script_witness.is_none(),
        "input 1 must NOT be finalized",
    );
    assert_eq!(
        got_in1.witness_utxo, expected.inputs[1].witness_utxo,
        "input 1 witness_utxo must round-trip unchanged",
    );
}
