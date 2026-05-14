# Android sample app — `jovawallet-core` integration reference

Minimal Compose app demonstrating the SDK-side integration shape that the production Android Jova app should mirror. Reference companion to [`docs/integration-android.md`](../../docs/integration-android.md).

## What this sample shows

1. **Constructing a `JovaWallet`** from a mnemonic + passphrase (EncryptedSharedPreferences → `ByteArray` → `JovaWallet`, then `.use {}` for explicit cleanup).
2. **Deriving an address** for every supported chain (`JovaChain.{Ethereum, Polygon, Bsc, Arbitrum, Optimism, Base, CustomEvm, Bitcoin, Xrp, Solana}`).
3. **Signing an EIP-1559 transaction** (Ethereum example).
4. **Signing a PSBT** (Bitcoin single-party flow; verifies the result does NOT carry the `psbt:` prefix that the multi-party flow uses).
5. **Signing a BIP-322 message**.
6. **Signing an XRPL Payment** via `UnsignedTx.Xrp`.
7. **Signing a Solana v0 VersionedTransaction**.
8. **Error handling**: each call site uses a `try/catch FfiException` block with per-variant `when` matching.

## What this sample does NOT show

- **Feature-flag gating per chain.** The production app must gate each chain's SDK call behind a server-controlled flag (`useJovaCoreForEthereum`, …).
- **Address reconciliation against the legacy app's stored values.** Before flipping any user to the SDK BTC path, the app team must run `tools/btc-migration-check` against ≥100 production mnemonic→address pairs and confirm 100/100 match. See [`docs/btc-migration-check.md`](../../docs/btc-migration-check.md) and [issue #3](https://github.com/jovachain/jovawallet-core/issues/3).
- **Telemetry / Reown WalletConnect bridge.** Production-only concerns.

## Status (2026-05-14): Linux-buildable, but unwired

This sample's source compiles against the AAR produced by `bindings/kotlin/scripts/build-aar.sh` (which runs on this Linux dev VM). The Gradle scaffolding here is **deliberately minimal** — it does not declare an Android Application target because the SDK team's Linux dev VM doesn't have an Android emulator. The production app team plugs this sample's `WalletRepository.kt` into their own Android Application Module.

To run as a standalone app: add a fresh Android Studio project (Empty Compose Activity template), drop in `WalletRepository.kt` + `MainActivity.kt`, declare the SDK dependency:

```kotlin
// app/build.gradle.kts
dependencies {
    implementation("io.jovachain:jova-core:0.3.0")
}
```

…or for a local AAR before Maven publication (Phase 5):

```kotlin
dependencies {
    implementation(files("../../bindings/kotlin/jova-core/build/outputs/aar/jova-core-debug.aar"))
}
```

## Files

| Path | What |
|---|---|
| `WalletRepository.kt` | Integration layer. Mirrors the production Android app's intended shape. |
| `MainActivity.kt` | Compose UI demo — derive address, sign per chain. |
| `README.md` | This file. |

## SDK team responsibilities (per the Phase 4 plan)

- Keep this sample current as the SDK evolves.
- Stand by for bug reports from the app team.
- Tag patch releases for production-discovered edge cases.
- Hold an office-hour the first week of each chain's rollout.
