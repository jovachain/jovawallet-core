//! Multi-account (HD account index) key ↔ address parity tests.
//!
//! The critical correctness rule for the account parameter: for any
//! `(chain, account)`, the private key used by `sign_tx` / `sign_message`
//! MUST derive from the same path that `address(chain, account)` uses. If
//! they diverge, a signature would not correspond to the address the app
//! shows as the "from" address.
//!
//! These tests prove that rule end-to-end for accounts 0, 1, 2 on every chain
//! family, by independently recovering / extracting the signer from a
//! signature or signed tx and comparing it to `address(chain, account)`:
//!
//! * **EVM** — ecrecover the signer address from an EIP-191 personal-sign
//!   signature (via `alloy`) and compare to `address(Ethereum, N)`.
//! * **Bitcoin** — the SDK's `sign_message` rejects any request whose target
//!   address isn't the wallet's own derived P2WPKH; signing account `N`'s
//!   message *for* `address(Bitcoin, N)` succeeding (and failing for a
//!   different account) is itself the parity proof.
//! * **XRP** — extract `SigningPubKey` from the signed tx binary, re-derive
//!   the classic address, and compare to `address(Xrp, N)`.
//! * **Solana** — verify the ed25519 signature against the public key decoded
//!   from `address(Solana, N)` (a Solana address *is* the base58 pubkey).
//!
//! Account 0 is additionally pinned byte-for-byte by the existing vector
//! tests in `vectors_*.rs`; here we also assert accounts 0/1/2 produce
//! distinct addresses, proving the index actually selects different keys.

use jova_core::{BtcMsgScheme, JovaChain, JovaWallet, SignableMessage, UnsignedTx};

/// BIP-39 standard test mnemonic (same value used by the BTC/EVM vectors).
const MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

const ACCOUNTS: [u32; 3] = [0, 1, 2];

fn wallet() -> JovaWallet {
    JovaWallet::from_mnemonic(MNEMONIC, "").expect("valid mnemonic")
}

// ── EVM ───────────────────────────────────────────────────────────────────────

#[test]
fn evm_sign_matches_address_for_each_account() {
    use alloy::primitives::Signature;
    use std::str::FromStr;

    let w = wallet();
    let mut seen = std::collections::HashSet::new();
    let message = "jova multi-account parity";

    for n in ACCOUNTS {
        let addr = w.address(&JovaChain::Ethereum, n).expect("address").value;
        assert!(
            seen.insert(addr.to_lowercase()),
            "account {n}: EVM address not distinct"
        );

        let sig_hex = w
            .sign_message(
                &SignableMessage::EvmPersonalSign {
                    message: message.to_string(),
                },
                n,
            )
            .expect("sign_message")
            .hex;

        let sig = Signature::from_str(&sig_hex).expect("parse signature");
        let recovered = sig
            .recover_address_from_msg(message)
            .expect("recover signer address");

        assert_eq!(
            recovered.to_string().to_lowercase(),
            addr.to_lowercase(),
            "EVM account {n}: recovered signer != address(Ethereum, {n})",
        );
    }
}

// ── Bitcoin ─────────────────────────────────────────────────────────────────

#[test]
fn btc_sign_matches_address_for_each_account() {
    let w = wallet();
    let mut seen = std::collections::HashSet::new();

    for n in ACCOUNTS {
        let addr = w.address(&JovaChain::Bitcoin, n).expect("address").value;
        assert!(
            seen.insert(addr.clone()),
            "account {n}: BTC address not distinct"
        );

        // Signing succeeds ONLY when the target address equals the address
        // derived from the account-N key — this is the SDK's own guard, so a
        // success is a direct proof of key/address parity at account N.
        let msg = SignableMessage::Bitcoin {
            message: "jova multi-account parity".to_string(),
            address: addr.clone(),
            scheme: BtcMsgScheme::Legacy,
        };
        w.sign_message(&msg, n)
            .unwrap_or_else(|e| panic!("BTC account {n}: sign for own address must succeed: {e}"));
    }

    // Negative control: signing account 0's address with the account-1 key
    // must be rejected (proves the account index genuinely changes the key).
    let addr0 = w.address(&JovaChain::Bitcoin, 0).unwrap().value;
    let wrong = SignableMessage::Bitcoin {
        message: "jova multi-account parity".to_string(),
        address: addr0,
        scheme: BtcMsgScheme::Legacy,
    };
    assert!(
        w.sign_message(&wrong, 1).is_err(),
        "BTC: signing account 0's address with the account-1 key must fail",
    );
}

// ── XRP ───────────────────────────────────────────────────────────────────────

#[test]
fn xrp_sign_matches_address_for_each_account() {
    let w = wallet();
    let mut seen = std::collections::HashSet::new();

    for n in ACCOUNTS {
        let addr = w.address(&JovaChain::Xrp, n).expect("address").value;
        assert!(
            seen.insert(addr.clone()),
            "account {n}: XRP address not distinct"
        );

        // Minimal valid Payment; Account is cosmetic here (the signer does not
        // validate it), but the injected SigningPubKey is what we verify.
        let tx_json = format!(
            "{{\"TransactionType\":\"Payment\",\"Account\":\"{addr}\",\
             \"Destination\":\"rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe\",\
             \"Amount\":\"1000000\",\"Fee\":\"12\",\"Sequence\":1,\"Flags\":0}}"
        );
        let signed = w
            .sign_tx(&UnsignedTx::Xrp { tx_json }, n)
            .expect("sign_tx")
            .raw_hex;

        let pubkey = extract_xrp_signing_pubkey(&signed);
        let derived = xrp_classic_address(&pubkey);
        assert_eq!(
            derived, addr,
            "XRP account {n}: address derived from SigningPubKey != address(Xrp, {n})",
        );
    }
}

/// Extract the 33-byte compressed `SigningPubKey` from a serialized signed
/// XRPL tx (uppercase hex). The field is encoded as `73` (SigningPubKey type
/// code) + `21` (VL length = 33) + 33 pubkey bytes.
fn extract_xrp_signing_pubkey(signed_hex: &str) -> [u8; 33] {
    let marker = signed_hex
        .find("7321")
        .expect("SigningPubKey field present in signed tx");
    let start = marker + 4;
    let pk_hex = &signed_hex[start..start + 66];
    let bytes = hex::decode(pk_hex).expect("valid pubkey hex");
    let mut out = [0u8; 33];
    out.copy_from_slice(&bytes);
    out
}

/// Derive an XRPL classic address (`r…`) from a compressed secp256k1 pubkey,
/// mirroring `jova_core_chains::xrp::address` (RIPEMD160(SHA256(pubkey)) then
/// XRPL base58check with version byte 0x00).
fn xrp_classic_address(pubkey: &[u8; 33]) -> String {
    use ripemd::Ripemd160;
    use sha2::{Digest, Sha256};

    let account_id = Ripemd160::digest(Sha256::digest(pubkey));
    let mut payload = Vec::with_capacity(1 + 20 + 4);
    payload.push(0x00);
    payload.extend_from_slice(&account_id);
    let checksum = Sha256::digest(Sha256::digest(&payload));
    payload.extend_from_slice(&checksum[..4]);
    bs58::encode(payload)
        .with_alphabet(bs58::Alphabet::RIPPLE)
        .into_string()
}

// ── Solana ──────────────────────────────────────────────────────────────────

#[test]
fn sol_sign_matches_address_for_each_account() {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    let w = wallet();
    let mut seen = std::collections::HashSet::new();

    // Raw bytes the "dApp" asks to sign, base64-encoded (Solana convention).
    let message = b"jova multi-account parity";
    let message_base64 = {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(message)
    };

    for n in ACCOUNTS {
        let addr = w.address(&JovaChain::Solana, n).expect("address").value;
        assert!(
            seen.insert(addr.clone()),
            "account {n}: SOL address not distinct"
        );

        let sig_b58 = w
            .sign_message(
                &SignableMessage::Solana {
                    message_base64: message_base64.clone(),
                },
                n,
            )
            .expect("sign_message")
            .hex;

        // A Solana address IS the base58-encoded 32-byte ed25519 public key.
        let pk_bytes: [u8; 32] = bs58::decode(&addr)
            .into_vec()
            .expect("decode address")
            .try_into()
            .expect("32-byte pubkey");
        let sig_bytes: [u8; 64] = bs58::decode(&sig_b58)
            .into_vec()
            .expect("decode signature")
            .try_into()
            .expect("64-byte signature");

        let vk = VerifyingKey::from_bytes(&pk_bytes).expect("valid ed25519 pubkey");
        let sig = Signature::from_bytes(&sig_bytes);
        vk.verify(message, &sig).unwrap_or_else(|_| {
            panic!("SOL account {n}: signature does not verify under address(Solana, {n})")
        });
    }
}
