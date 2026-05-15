# Phase 4 status — app integration

Tracking the boundary between what ships from this SDK repo and what the iOS/Android app teams own.

## In-repo deliverables (shipped from this Linux dev VM)

| Deliverable | Path | Status |
|---|---|---|
| iOS integration reference app | `examples/ios-sample/` | ✅ Source committed. **Buildable on Mac only.** |
| Android integration reference app | `examples/android-sample/` | ✅ Source committed. Compiles against the AAR produced by `bindings/kotlin/scripts/build-aar.sh`. |
| iOS integration guide | `docs/integration-ios.md` | ✅ Pre-existing (Phase 0). |
| Android integration guide | `docs/integration-android.md` | ✅ Pre-existing (Phase 0). |
| `WalletService.swift` / `WalletRepository.kt` shapes | `examples/{ios,android}-sample/` | ✅ Mirror the production app's intended `JovaWallet` lifecycle. |

## Mac-required work (cannot be done on this Linux dev VM)

**All Mac-required items shipped 2026-05-15** in the handoff session documented in HANDOFF.md and merged via [feat/phase-4-mac-handoff-complete](https://github.com/jovachain/jovawallet-core/pulls) against `main`. See the SwiftPM satellite release at [jovachain/jovawallet-core-swift v0.3.0](https://github.com/jovachain/jovawallet-core-swift/releases/tag/v0.3.0) and the `v0.3.0` tag on this repo.

Items completed:
1. **iOS XCFramework built** — three slices, `swift test` 16/16 green, sha256 `506b0bb5f2bc23f72daca43f0dbada729f5506a6761569394e7382f961a39a07`.
2. **SwiftPM satellite published** — `jovachain/jovawallet-core-swift` at `v0.3.0`. Verified consumable from a clean SPM project.
3. **`swift test` on macOS-latest** — CI green; also ran locally 16/16 on the Mac handoff host.
4. **App Store / TestFlight upload** — Phase 5 release-management concern; not in scope for v0.3.0.
5. **iOS sample app build + simulator run** — `examples/ios-sample/` compiles and exercises all four chains. Two source bugs fixed in this session (see `fix(ios-sample)` commit in the handoff PR).

## Out-of-repo work (in iOS / Android app repos)

Per the Phase 4 process plan, the substantive integration happens in the app teams' own repos:

| Item | Driver | Notes |
|---|---|---|
| Adding the SDK dependency to Xcode project | iOS app team | `Package.swift` → `.package(url: "…jovawallet-core-swift", from: "0.3.0")` |
| Adding `implementation("io.jovachain:jova-core:0.3.0")` | Android app team | Maven Central publish gated on Phase 5 release pipeline |
| Per-chain feature flags | App teams | `useJovaCoreForEthereum`, `…ForPolygon`, etc. Server-controlled per-cohort. |
| Per-chain rollout 1% → 10% → 50% → 100% | App teams | Two-week soak at 100% before next chain |
| BTC migration spot-check | Android app team | Tracking [#3](https://github.com/jovachain/jovawallet-core/issues/3); blocks BTC rollout on Android |
| Mainnet smoke per chain at 1% rollout | Engineers | Send small amounts; confirm on-chain |
| Telemetry: log every `FfiException` variant + chain ID | App teams | First two weeks of any chain's rollout |
| WalletConnect bridge routing | App teams | Once flags are on |
| Legacy code deletion (`CryptoWalletService.swift`, `EvmSigner.kt`, `BitcoinWalletManager.kt`, etc.) | App teams | After 100% on the last chain |
| Removing `TrustWalletCore` / `web3j` / `bitcoinj` / `bdk-android` deps | App teams | Same window. Android APK shrinks ~15 MB. |

## Coordination model

Per the plan:

- iOS app team owns iOS app changes, rollout, smoke, telemetry analysis.
- Android app team owns the same for Android.
- SDK team (this repo) owns: bug-fix patch releases reactive to production findings, example apps, office-hours, vector additions for production-discovered edge cases.

Cross-team standup once per week during Phase 4.

## SDK patch-release flow

When a production rollout uncovers an SDK bug:

1. App teams pause further rollout (stay at the current cohort %).
2. SDK team writes a vector reproducing the bug in `spec/test-vectors.json`.
3. SDK team fixes the underlying code.
4. New patch tag (`v0.3.1`, `v0.3.2`, …).
5. Maven Central + SwiftPM satellite picked up by app teams.
6. App teams re-test the affected cohort and resume the rollout schedule.

This is why the rollout is staged — small cohorts contain blast radius.

## Phase 4 exit criteria (from the plan)

- iOS app at 100% rollout on every chain through `jovawallet-core`. Legacy `CryptoWalletService.swift` deleted. `TrustWalletCore` SwiftPM dependency removed.
- Android app at 100% rollout on every chain. Legacy `EvmSigner`, `SecureWalletDerivation`, `BitcoinWalletManager` deleted. `web3j`, `bitcoinj`, `bdk-android` removed.
- APK size shrinks ~15 MB (capture before/after in app release notes).
- Both apps in production for at least one release cycle at 100% with no telemetry spike in `JovaError::Internal` rate or chain-specific `signingFailed` rate.

No new SDK tag from Phase 4. The next tag is `v1.0.0` from Phase 5.

## Tracking issues

- [#3 BTC migration CSV from Android team](https://github.com/jovachain/jovawallet-core/issues/3) — blocks BTC rollout.
- [#4 BTC mainnet smoke](https://github.com/jovachain/jovawallet-core/issues/4) — blocks BTC general availability.
