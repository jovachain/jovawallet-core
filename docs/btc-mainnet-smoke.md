# BTC Mainnet Smoke Test

Final pre-release gate for Phase 2 (`v0.2.0`): a real mainnet transaction signed end-to-end by the SDK, broadcast, and confirmed on Bitcoin mainnet. Confirms captured vectors hold against live consensus.

This is **human-driven** — it costs ~$5 in BTC and requires a confirmation window of 10-30 minutes. Cannot be done in CI; cannot be done on the Linux dev VM.

Tracking issue: <https://github.com/jovachain/jovawallet-core/issues/4>.

## Procedure

1. Generate a fresh mnemonic via the SDK: `JovaWallet::create_mnemonic(.bits128)`.
2. Derive the first BIP-84 address: `wallet.address(.bitcoin, 0)`.
3. Send ~10,000 sats from an exchange withdrawal (≈\$5) to that address.
4. Construct a PSBT spending most of the sats to a destination address with a small fee.
   - Either via the backend's PSBT builder, or `bdk-cli` against the same mainnet UTXO set.
5. Sign via SDK:
   ```rust
   let signed = wallet.sign_tx(&UnsignedTx::Bitcoin { psbt_base64 })?;
   ```
6. Verify `signed.raw_hex` does **not** start with `psbt:` — single-party flow must finalize to broadcast-ready hex.
7. Broadcast via mempool.space or blockstream.info.
8. Watch mempool until confirmation (~10-30 min).

## Result (to be filled in)

**Date:** _TBD_  
**SDK version:** v0.2.0-rc  
**Engineer:** _TBD_  
**Tx hash:** _TBD_  
**Block height confirmed:** _TBD_  

This document is filled in when the smoke test passes.

## Why this gate exists

The SDK's BTC signing is fully tested against vectors captured from embit 0.8.0 (which matches Bitcoin Core ≥ v0.17 defaults). Those vectors are deterministic and high-confidence, but they don't exercise the live network's policy rules (minrelaytxfee, dust thresholds, mempool acceptance heuristics) or any infra-side encoding gotchas (e.g., serialization of witness elements through the broadcast endpoint).

A single mainnet confirmation closes that gap.
