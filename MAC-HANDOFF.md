# Mac handoff — for the next agent working from macOS

The previous Linux session shipped Phases 2 through 7 to GitHub. `main` is at `v0.3.0`. The Mac-only deliverables are why a Mac agent is needed.

**Copy-paste the prompt below into the next agent's session.** Two things to fill in first:

1. Replace `<repo-path>` with the actual local clone path on the Mac (after `git clone`).
2. The Mac agent will need to authenticate `gh` CLI fresh (`gh auth login`) — the Linux VM's token doesn't transfer.

---

```
You are continuing the jovawallet-core SDK on a macOS dev machine. The previous
Linux session shipped Phases 2 through 7 to GitHub. `main` is at v0.3.0; the
Mac-only deliverables are why you're here.

═══════════════════════════════════════════════════════════════
READ FIRST, in this order, before any tool use other than these reads:
═══════════════════════════════════════════════════════════════
1. <repo-path>/HANDOFF.md       — current state + open release gates
2. <repo-path>/docs/phase-4-status.md — explicit Mac-required boundary
3. <repo-path>/CLAUDE.md        — orientation + don'ts
4. <repo-path>/docs/integration-ios.md
5. <repo-path>/examples/ios-sample/README.md

═══════════════════════════════════════════════════════════════
WHAT YOU OWN ON MAC
═══════════════════════════════════════════════════════════════
The Mac-required work, in priority order:

1. **Build the iOS XCFramework.**
   - Script: `bindings/swift/scripts/build-xcframework.sh`
   - Targets: iOS device (arm64), iOS simulator (arm64 + x86_64), macOS arm64
   - Output: `JovaCore.xcframework` containing libjova_core_ffi.a per slice
   - Requires: macOS 13+, Xcode 15+, Rust 1.95+ with iOS targets
     (`rustup target add aarch64-apple-ios aarch64-apple-ios-sim aarch64-apple-darwin x86_64-apple-darwin`)

2. **Publish the SwiftPM satellite repo `jovachain/jovawallet-core-swift`** at
   `v0.3.0` (matching the SDK tag). The satellite carries:
   - The XCFramework as a binaryTarget
   - Generated `JovaCore.swift` (from uniffi-bindgen-swift)
   - The Convenience.swift ergonomics layer
   Tag, push, verify SwiftPM resolves it: `swift package describe` from a
   fresh consumer project.

3. **Build and run the iOS sample.**
   - `cd examples/ios-sample && open Package.swift` in Xcode → Cmd-R
   - Verifies the integration shape works end-to-end on iOS simulator
   - The sample's WalletService.swift mirrors the production app's intended
     JovaWallet lifecycle. Confirm address derivation works for every chain;
     confirm signTx + signMessage round-trip cleanly.

4. **(Optional, if you have Android tooling) Build and verify the Android
   sample** against the AAR produced by `bindings/kotlin/scripts/build-aar.sh`.
   This runs on Linux too but Mac with Android Studio is fine.

═══════════════════════════════════════════════════════════════
WHAT YOU DO NOT OWN (Mac doesn't matter for these)
═══════════════════════════════════════════════════════════════
These are still gated on humans / external work, tracked as GitHub issues:
- #3, #4 — BTC migration CSV + mainnet smoke
- #8 — External security audit firm engagement
- #9 — Reproducible-build dual-engineer pairing (requires two machines, but
        a Mac alone doesn't unblock it)
- #10 — Threat-model walkthrough (could start, but needs a second engineer
         signoff for the release gate)
- #11 — App-team RC validation (app team work, not SDK)
- #12 — Bug bounty funding
- #13 — Phase 4 100% rollout soak (app team work)
- #14 — 14-day fuzz soak (CI does it; you confirm 14 consecutive days green)

If your time is well-spent on items 1-4 above, finish those first. Don't
unilaterally engage an audit firm or commit bug bounty funding.

═══════════════════════════════════════════════════════════════
WHAT IS ALREADY ON MAIN (so you don't re-do it)
═══════════════════════════════════════════════════════════════
- Phases 2-7 merged. Tags v0.0.1, v0.1.0, v0.2.0, v0.3.0.
- The cryptographic surface (EVM 7 chains, BTC, XRP, SOL) is complete, tested
  byte-equal against external signers (cast/embit/xrpl-py/solders), and
  verified across Linux x86_64, macOS, Windows, Android NDK x4, WASM, and
  Cortex-M thumbv7em-none-eabihf.
- Phase 6 (WASM functional EVM+SOL, BTC/XRP browser deferred per 2026-05-11
  user decision) is merged but not yet tagged — that tag is v1.1.0, after
  v1.0.0 ships.
- Phase 7 (hardware-readiness, no_std + external-rng + firmware-template) is
  merged but not yet tagged — that tag is v1.2.0.
- CI is green on every PR: 8 workflows pass (audit, kotlin, no_std, swift,
  test ubuntu/macos/windows, wasm).
- Every Phase 2-7 PR (#5, #6, #7, #15, #16, #17) had swift pass on
  macos-latest, so the Swift parity tests already work. Your job is the
  binary distribution (XCFramework + SwiftPM satellite), not the test
  pass itself.

═══════════════════════════════════════════════════════════════
HOW TO WORK
═══════════════════════════════════════════════════════════════
- One subagent per substantial task (XCFramework build → review → publish
  → review → sample run → review). Two-stage review per task: spec then
  code quality. Use superpowers:subagent-driven-development if available.
- One PR per deliverable. Conventional Commits. NO AI attribution
  (no 🤖, no Co-Authored-By: Claude, no Generated with Claude Code footer).
- Push every commit as it lands. The user wants to see progress, not a
  bundled drop at the end.
- Branch naming: feat/phase-N-<topic> (e.g., feat/phase-4-ios-xcframework).
- Pause before merging to let the user review the binary output.

═══════════════════════════════════════════════════════════════
AUTHORIZATION
═══════════════════════════════════════════════════════════════
You are launched with --dangerously-skip-permissions. Proceed without
confirmation prompts for installs, edits, and pushes. The user is in tmux
and will check back.

═══════════════════════════════════════════════════════════════
DECISIONS ALREADY MADE — DO NOT RE-LITIGATE
═══════════════════════════════════════════════════════════════
- WASM scope: BTC/XRP browser signing is DEFERRED beyond v1.1.
- Test-as-contract: vectors from cast, bdk-cli/embit, solana-cli/solders,
  xrpl-py. Your Rust code matches the captured vector byte-for-byte.
- Engine confinement: bdk_wallet, alloy, bitcoin, solana-*, xrpl-rust may
  ONLY be in crates/jova-core-chains.
- Git identity is xhuman <xhuman.77x@gmail.com>. gh CLI is authenticated
  on the Linux VM; you'll need to authenticate gh CLI locally on the Mac.
- No --version pins on cargo; --locked only.

═══════════════════════════════════════════════════════════════
COMMON MAC-SPECIFIC GOTCHAS (from the project's spike notes)
═══════════════════════════════════════════════════════════════
- secp256k1-sys WASM build on macOS needs Homebrew LLVM, NOT Apple clang.
  `brew install llvm` and `bindings/wasm/scripts/build-wasm.sh` exports
  CC_wasm32_unknown_unknown to point at Homebrew's clang automatically.
- uniffi-bindgen-swift is shipped as a binary in the `uniffi` crate (v0.31+),
  install via `cargo install uniffi --features cli --locked`.
- iOS targets must be added: `rustup target add aarch64-apple-ios
  aarch64-apple-ios-sim aarch64-apple-darwin x86_64-apple-darwin`.
- Xcode command-line tools must be installed and selected: `xcode-select -p`
  should point to /Applications/Xcode.app/Contents/Developer.

═══════════════════════════════════════════════════════════════
BEGIN
═══════════════════════════════════════════════════════════════
Start by reading HANDOFF.md and docs/phase-4-status.md. Then verify your
Mac environment has Xcode 15+, Rust 1.95+ with iOS targets, and Homebrew
LLVM. Then dispatch the first subagent for item 1: build the iOS XCFramework.
```
