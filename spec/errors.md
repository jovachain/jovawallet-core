# Error Taxonomy — frozen reference

This file is the spec-side mirror of `docs/error-model.md`. At Phase 0 it documents only the trivial stub variant.

## Variants

- (Phase 0 has no error path on `is_valid_mnemonic`; the function returns bool.)

## Reason vocabulary

(Phase 1 fills this in for malformed-tx and malformed-message reasons.)

## Phase 2 — Bitcoin (`malformed_unsigned_tx` reasons)

| Reason | Means |
|---|---|
| `psbt_invalid_base64` | PSBT base64 decode failed |
| `psbt_invalid_serialization` | base64 decoded but PSBT structure malformed |
| `psbt_no_signable_inputs` | none of the PSBT inputs are signable by this wallet's key |
| `expected_bitcoin_variant` | Internal: routing mismatch (caller invoked BtcSigner with a non-Bitcoin UnsignedTx) |
| `expected_evm_variant` | Internal: routing mismatch (caller invoked EvmSigner with a non-EVM UnsignedTx) |

## Phase 2 — Bitcoin (`malformed_signable_message` reasons)

| Reason | Means |
|---|---|
| `btc_message_address_mismatch` | the supplied address does not correspond to the wallet's derived BIP-84 key |
| `btc_message_address_invalid` | the supplied address is not a parseable Bitcoin address |
| `btc_message_address_wrong_network` | the supplied address is for a non-mainnet network (testnet/signet/regtest) |
| `btc_message_address_not_p2wpkh` | the supplied address is a valid Bitcoin address but not a BIP-84 P2WPKH (v1 only signs for native SegWit) |
| `expected_bitcoin_message` | Internal: routing mismatch |
| `expected_evm_message` | Internal: routing mismatch |

## Phase 2 — Bitcoin (`signing_failed` reasons)

These indicate an internal crypto failure that should be unreachable in
practice; if any of them surfaces to a caller it represents a bug to file.
The bare reasons map 1:1 to a fixed error path; the prefixed reasons embed an
upstream error message after the `:` (e.g. `sighash_failed:Invalid SIGHASH`).

| Reason | Where | Means |
|---|---|---|
| `xprv_invalid_scalar` | `crates/jova-core-chains/src/btc/psbt.rs`, `…/btc/message.rs` | the supplied XPrv's private key bytes are not a valid secp256k1 scalar (zero or >= group order); should be unreachable for keys derived from a valid mnemonic |
| `pubkey_invalid` | `crates/jova-core-chains/src/btc/psbt.rs`, `…/btc/message.rs`, `…/btc/address.rs` | the compressed public key bytes from the XPrv fail to parse as a valid secp256k1 point |
| `pubkey_compress_failed` | `crates/jova-core-chains/src/btc/address.rs` | producing the 33-byte compressed encoding of the derived public key failed |
| `sighash_failed:{upstream}` | `crates/jova-core-chains/src/btc/psbt.rs` | computing the BIP-143 sighash for a P2WPKH input we own failed in `bitcoin::sighash::SighashCache::p2wpkh_signature_hash` |
| `tx_encode_failed:{upstream}` | `crates/jova-core-chains/src/btc/psbt.rs` | consensus-encoding the finalized transaction failed |
| `bip322_sighash_failed:{upstream}` | `crates/jova-core-chains/src/btc/message.rs` | computing the BIP-143 sighash for the BIP-322 `to_sign` virtual tx failed |
| `bip322_witness_encode_failed:{upstream}` | `crates/jova-core-chains/src/btc/message.rs` | consensus-encoding the BIP-322 witness for base64 output failed |
| `legacy_recid_out_of_range:{recid}` | `crates/jova-core-chains/src/btc/message.rs` | the recovery id from the legacy `signmessage` ECDSA signing path was outside `0..=3` (libsecp256k1 invariant violation; unreachable in practice) |

## Phase 3 — XRP (`malformed_unsigned_tx` reasons)

| Reason | Means |
|---|---|
| `xrp_invalid_json` | `tx_json` failed JSON parse, or parsed to a non-object value |
| `xrp_missing_required_field:<Field>` | a required XRPL field (currently `TransactionType` or `Account`) is absent from the parsed object |
| `expected_xrp_variant` | Internal: routing mismatch (caller invoked `XrpSigner` with a non-XRP `UnsignedTx`) |

## Phase 3 — XRP (`malformed_signable_message` reasons)

| Reason | Means |
|---|---|
| `xrp_message_signing_unsupported` | XRPL does not define a standard message-signing scheme equivalent to BIP-322 / EIP-191; the SDK rejects every `SignableMessage` variant when routed to `XrpSigner`. There is currently no `SignableMessage::Xrp` variant, so this is reachable only by future XRP message-signing inputs. |

## Phase 3 — XRP (`signing_failed` reasons)

Like the BTC `signing_failed` reasons, these indicate an internal crypto or
serialization failure that should be unreachable for a well-formed XRPL
transaction; if any surfaces it represents a bug to file. The bare reasons map
1:1 to a fixed error path; the prefixed reasons embed an upstream error message
or hex-decode failure after the `:`.

| Reason | Where | Means |
|---|---|---|
| `xrp_secret_invalid` | `crates/jova-core-chains/src/xrp/tx.rs` | the derived XPrv's private-key bytes are not a valid secp256k1 scalar (zero or >= group order); unreachable for keys derived from a valid mnemonic |
| `xrp_encode_for_signing:{upstream}` | `crates/jova-core-chains/src/xrp/tx.rs` | `xrpl-rust::core::binarycodec::encode_for_signing` rejected the injected signing payload (typically an unknown field or out-of-range value that survived JSON parse) |
| `xrp_encode_for_signing_hex:{upstream}` | `crates/jova-core-chains/src/xrp/tx.rs` | the hex string returned by `encode_for_signing` failed `hex::decode` (canonically unreachable; would indicate an upstream contract violation) |
| `xrp_encode:{upstream}` | `crates/jova-core-chains/src/xrp/tx.rs` | `xrpl-rust::core::binarycodec::encode` rejected the fully signed payload after `TxnSignature` injection |
| `xrp_encode_hex:{upstream}` | `crates/jova-core-chains/src/xrp/tx.rs` | the hex string returned by `encode` failed `hex::decode` while preparing the `TXN\0` prefixed payload for the tx_hash digest (canonically unreachable) |
| `xrp_address_encode_failed:{upstream}` | `crates/jova-core-chains/src/xrp/address.rs` | `xrpl-rust` failed to base58check-encode the 20-byte AccountID into a classic `r…` address (canonically unreachable) |

