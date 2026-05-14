# iOS sample app — `jovawallet-core` integration reference

Minimal SwiftUI app demonstrating the SDK-side integration shape that the production iOS Jova app should mirror. Reference companion to [`docs/integration-ios.md`](../../docs/integration-ios.md).

## What this sample shows

1. **Constructing a `JovaWallet`** from a mnemonic + passphrase (Keychain → `String` → `JovaWallet`, then immediate scope exit).
2. **Deriving an address** for every supported chain (`JovaChain.{ethereum, polygon, bsc, arbitrum, optimism, base, customEvm, bitcoin, xrp, solana}`).
3. **Signing an EIP-1559 transaction** (Ethereum example).
4. **Signing a PSBT** (Bitcoin single-party flow; verifies the result does NOT carry the `psbt:` prefix that the multi-party flow uses).
5. **Signing a BIP-322 message**.
6. **Signing an XRPL Payment** via `UnsignedTx.xrp(txJson:)`.
7. **Signing a Solana v0 VersionedTransaction**.
8. **Error handling**: each call site uses `do/catch` against `FfiException` (the uniffi-generated error type) with per-variant pattern matching.

## What this sample does NOT show

- **Feature-flag gating per chain.** The production app must gate each chain's SDK call behind a server-controlled flag (`useJovaCoreForEthereum`, …) so rollout can be staged from 1% → 10% → 50% → 100% per chain. See [`docs/superpowers/plans/2026-05-05-phase-4-app-integration.md`](../../docs/superpowers/plans/2026-05-05-phase-4-app-integration.md).
- **Address reconciliation against the legacy app's stored values.** The production app must derive the address with both SDK and legacy code, assert byte-identical, and refuse to ship if they differ.
- **Telemetry.** First two weeks of any chain's rollout, the production app should log every `FfiException` variant (no payload) with the chain ID + SDK version for comparison against the legacy code's error rate.
- **WalletConnect routing.** WalletConnect requests are routed through the SDK once feature flags are on.

## Status (2026-05-14): Mac-required to build

**This Linux dev VM cannot build or run the iOS sample.** The Swift toolchain, Xcode, and the SDK's iOS XCFramework are all macOS-only. The sample's source files are committed here; building / running requires:

- macOS 13+ (the host running Xcode)
- Xcode 15+
- A checkout of this repo
- Either:
  - The `jovawallet-core-swift` satellite repo at `v0.3.0` (Phase 5 will publish the SwiftPM package), OR
  - A local XCFramework built from this repo via `bindings/swift/scripts/build-xcframework.sh` on a Mac.

Once the satellite repo is published (Phase 5), the sample's `Package.swift` will declare it as a dependency and Xcode handles the rest.

## How to build & run (on a Mac)

```bash
cd examples/ios-sample
open JovaSample.xcodeproj  # or open Package.swift in Xcode
# Cmd-R to run on the iOS simulator.
```

## Files

| Path | What |
|---|---|
| `Package.swift` | SwiftPM manifest. Declares dependency on the `jovawallet-core-swift` package. |
| `Sources/JovaSample/App.swift` | `@main` entry point. |
| `Sources/JovaSample/WalletService.swift` | The integration layer that wraps `JovaWallet`. Mirrors the production iOS app's intended shape. |
| `Sources/JovaSample/ContentView.swift` | SwiftUI demo screen — derive address, sign tx/message per chain. |

## SDK team responsibilities (per the Phase 4 plan)

- Keep this sample current as the SDK evolves.
- Stand by for bug reports from the app team.
- Tag patch releases (`v0.3.1`, …) for production-discovered edge cases.
- Hold an office-hour the first week of each chain's rollout.
