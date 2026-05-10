# Phase 4: iOS + Android App Integration (Process Plan)

> **Status:** Process plan. Most work happens in the iOS and Android app repositories, NOT in `jovawallet-core`. The agent operating in this repo cannot fully execute Phase 4 — coordinate with the app teams.

> **For agentic workers:** Read this as procedure, not as a TDD-step plan. The bulk of the implementation is in app code your agent doesn't have access to. Within this repo, your only Phase 4 deliverables are example apps (`examples/ios-sample/`, `examples/android-sample/`) and any SDK-side tweaks the app teams request.

**Goal:** Both Jova apps signing through `jovawallet-core` end-to-end on mainnet at 100% rollout, legacy crypto stacks deleted. No new SDK tag from this phase — the apps move; the SDK doesn't.

**Preconditions:**
- `v0.5.0` tagged. Maven Central has `io.jovachain:jova-core:0.5.0`. Satellite repo has `jovawallet-core-swift` at `0.5.0`.
- App teams have been pre-briefed on the migration plan and have allocated time.
- Feature-flag infrastructure exists in both apps (most modern wallet apps already have one; if not, add it first).

**Exit criteria:**
- iOS app at 100% rollout on every chain through `jovawallet-core`. Legacy `CryptoWalletService.swift` deleted. `TrustWalletCore` SwiftPM dependency removed.
- Android app at 100% rollout on every chain. Legacy `EvmSigner`, `SecureWalletDerivation`, `BitcoinWalletManager` deleted. `web3j`, `bitcoinj`, `bdk-android` removed from `build.gradle.kts`. APK shrinks ~15 MB.
- Both apps in production for at least one release cycle at 100% with no telemetry spike in `JovaError::Internal` rate or chain-specific `signingFailed` rate.

---

## Sub-phase 4a — iOS (~1.5–2 weeks)

Driven by the iOS app team. SDK team supports.

### Procedure

1. **Add the SDK dependency.** In the iOS app's `Package.swift` (or via Xcode UI):
   ```swift
   .package(url: "https://github.com/jovachain/jovawallet-core-swift.git", from: "0.5.0")
   ```
2. **Implement `WalletService`** per `docs/integration-ios.md`. Reads mnemonic from Keychain into a clearable `Data`, constructs `JovaWallet` per call, signs, releases.
3. **Add per-chain feature flags.** Server-controlled, per-user-cohort:
   ```swift
   if FeatureFlag.useJovaCoreForEthereum.isEnabled {
       return try walletService.signTransaction(tx)
   } else {
       return try legacyCryptoWalletService.signEthereum(tx)
   }
   ```
   Flags: `useJovaCoreForEthereum`, `useJovaCoreForPolygon`, etc., one per chain.
4. **Migrate one chain at a time.** Recommended order: Ethereum first (highest volume, most-tested upstream), then Polygon/BSC/Arbitrum/Optimism/Base in any order, then XRP, then SOL, then BTC last (highest funds-at-risk; we want our cumulative confidence highest before flipping it).
5. **Per-chain rollout cadence:**
   - Day 1: 1% of users.
   - Day 3 (if telemetry healthy): 10%.
   - Day 7: 50%.
   - Day 14: 100%.
   - Two weeks at 100% with no incidents → next chain.
6. **Reconcile addresses.** For each chain, before flipping any user to the SDK signing path, the app should derive the address with both SDK and legacy and assert byte-identical. A discrepancy means the user's funds are at a different address — never flip them; investigate.
7. **Update WalletConnect bridge.** WalletConnect requests are routed through `WalletService.signMessage` / `signTransaction` once feature flags are on for the relevant chain.
8. **Manual mainnet smoke per chain at 1% rollout.** Send a small amount from a test account; confirm broadcast and on-chain inclusion.
9. **Telemetry.** First two weeks of any chain's rollout, log every `JovaError` variant (no payload) with the chain ID and the SDK version. Compare against the legacy code's error rate.
10. **Cleanup after 100% on the last chain:**
    - Delete `CryptoWalletService.swift` and any chain-specific legacy signers.
    - Remove `TrustWalletCore` from `Package.swift`.
    - Remove the feature flags themselves (they're permanent at this point).
    - Tag the app release notes.

### What the SDK team does in this sub-phase

- **Stand by for bug reports.** Real production usage will surface FFI marshalling bugs, edge-case error variants the vectors didn't cover, and Swift-side ergonomics gaps. Each becomes a vector + a SDK patch release (`v0.5.1`, `v0.5.2`, …).
- **Update `examples/ios-sample/`** to mirror the production iOS app's integration shape. This is the SDK-repo deliverable.
- **Hold a SDK office-hour** with the app team for the first week of each chain's rollout.

### Exit checklist (sub-phase 4a)

- [ ] All EVM chains at 100% rollout for two weeks.
- [ ] XRP at 100% for two weeks.
- [ ] SOL at 100% for two weeks.
- [ ] BTC at 100% for two weeks.
- [ ] `CryptoWalletService.swift` deleted.
- [ ] `TrustWalletCore` removed from `Package.swift`.
- [ ] Feature flags removed.
- [ ] App release notes tagged.

---

## Sub-phase 4b — Android (~2 weeks)

Driven by the Android app team. SDK team supports.

### Procedure

Same shape as iOS, with these differences:

1. **Add the SDK dependency.** In the app's `build.gradle.kts`:
   ```kotlin
   implementation("io.jovachain:jova-core:0.5.0")
   ```
2. **Implement `WalletRepository`** per `docs/integration-android.md`. Reads mnemonic from `EncryptedSharedPreferences` into a `ByteArray`, wraps `JovaWallet` in `use { … }`, clears bytes after.
3. **BTC address reconciliation gate.** Before flipping any Android user to the SDK signing path for BTC, the app team must produce a list of N known seed→`bc1q…` mappings from the legacy storage and confirm SDK derivation matches every one. Document the script + results in `docs/btc-migration-check.md` (created in Phase 2).
4. **Same per-chain feature flags + same rollout cadence as iOS.**
5. **Update Reown (WalletConnect Kotlin SDK) bridge.**
6. **Per-chain smoke.** Same.
7. **Cleanup after 100% on the last chain:**
    - Delete `EvmSigner.kt`, `SecureWalletDerivation.kt`, `BitcoinWalletManager.kt`.
    - Remove `web3j`, `bitcoinj`, `bdk-android` from `build.gradle.kts`.
    - Verify APK size shrinks ~15 MB; capture the before/after in the release notes.
    - Remove feature flags.

### What the SDK team does in this sub-phase

- **Stand by for bug reports** (same as iOS).
- **Update `examples/android-sample/`** to mirror the production app's shape.
- **Office-hour** for the first week.

### Exit checklist (sub-phase 4b)

- [ ] All chains at 100% for two weeks each.
- [ ] BTC migration spot-check: 100% match between SDK and legacy storage.
- [ ] Legacy code deleted.
- [ ] Legacy crypto deps removed from `build.gradle.kts`.
- [ ] APK size reduction verified and documented.
- [ ] Feature flags removed.

---

## Coordination model

The SDK team **does not** drive Phase 4 directly. Roles:

| Team | Owns |
|---|---|
| iOS app team | Code changes in iOS repo, feature flag rollout, mainnet smoke tests, telemetry analysis |
| Android app team | Same for Android |
| SDK team | Bug fixes (patch releases), example apps, office-hours, vector additions for any production-discovered edge cases |

Cross-team standup once per week during Phase 4. SDK team participates as a support function.

---

## What happens if rollout reveals a SDK bug

1. The app teams pause further rollout (stay at the current cohort %).
2. SDK team adds a vector reproducing the bug.
3. SDK team fixes; tags `v0.5.X` patch release.
4. App teams pin the new version, re-test the affected cohort.
5. Resume rollout schedule.

This is why the rollout is staged — small cohorts contain blast radius.

---

## What this plan does NOT do

- Does not produce new SDK code beyond patch releases reactive to bugs.
- Does not produce a new SDK tag at the end of Phase 4. The next tag is `v1.0.0` from Phase 5.
- Does not change the API. Phase 4 is "use the existing SDK in production"; if the API needed a change, that's a Phase 5 concern.

---

## Estimated time

3–4 weeks total wall-clock if iOS and Android run in parallel with their own teams. Sequential would be 5–6 weeks.
