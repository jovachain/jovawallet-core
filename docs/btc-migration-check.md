# BTC Migration Spot-Check

This SDK consumes existing user mnemonics from the production Android wallet. For BTC, the legacy app stores BIP-84 P2WPKH (`bc1q…`) addresses. A derivation mismatch between this SDK and the legacy app would mean funds land at the wrong address — catastrophic.

The spot-check is a one-time gate run by a trusted engineer before any wallet-app rollout uses the SDK for BTC.

## What runs

`tools/btc-migration-check/` ships a binary `jova-btc-migration-check` that:

1. Reads `tools/btc-migration-check/known-android-mappings.csv` — a 2-column `mnemonic,address` file.
2. For each row: derives `m/84'/0'/0'/0/0` from the mnemonic via the SDK, computes the P2WPKH bech32 address.
3. Compares to the legacy-stored address.
4. Prints `BTC migration check: N/N match`. Exits non-zero on any mismatch; logs the first-two-words preview of the offending mnemonic (never the full mnemonic).

## Inputs

The CSV is not in git — it contains user mnemonics. The directory's `.gitignore` excludes `known-android-mappings.csv`.

The Android team exports the file from production storage and hands it to the engineer driving the gate. Tracking issue: <https://github.com/jovachain/jovawallet-core/issues/3>.

Required: **≥100 rows** of real production mnemonic → BIP-84 address mappings.

## Procedure

```bash
# 1. Place the CSV at the expected path.
cp ~/Downloads/known-android-mappings.csv tools/btc-migration-check/

# 2. Run the check.
cargo run --release -p jova-btc-migration-check
# Expected: "BTC migration check: N/N match"
```

If any mismatch: STOP. Don't ship Phase 2. Investigate. Common causes:

- Legacy app used a different derivation path (verify it's `m/84'/0'/0'/0/0` and not e.g. `m/49'/0'/0'/0/0` for BIP-49 P2SH-wrapped SegWit).
- Legacy app applies a per-user passphrase (verify with the app team; the SDK currently passes empty passphrase).
- Bech32 encoding edge case (network prefix, witness version).

## Result (to be filled in)

**Date:** _TBD_  
**SDK version:** v0.2.0-rc  
**Engineer:** _TBD_  
**Result:** _N/N match_

This document is filled in when the gate closes. See [issue #3](https://github.com/jovachain/jovawallet-core/issues/3) for current status.

## Out of scope

- Mnemonics with non-empty passphrases (none in production at the time of writing).
- Account indices > 0 (the legacy app uses account 0 only).
- Multi-sig descriptor flows (v1 SDK is single-sig BIP-84).
