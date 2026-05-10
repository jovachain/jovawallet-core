# jovawallet-core — Design Spec

**Date:** 2026-05-05
**Status:** Awaiting user review
**Driver:** Reframe `jovawallet-core` from "two thin native wrappers around Trust Wallet Core" to a Rust core with multiple thin bindings, sized for an enterprise multi-chain wallet that needs to scale to iOS, Android, web (WASM), backend, and eventually hardware-wallet firmware.

## Summary

The existing first-draft docs designed a two-wrappers-around-TWC SDK targeting iOS and Android. The user's stated future — web, backend, hardware wallets, and "all platforms eventually" — makes that shape the wrong choice. After web research into 2026 enterprise wallet architectures (BDK, Block's Bitkey, Foundation Devices, Mozilla Application Services), the dominant pattern is a **pure-Rust signing core wrapped with `uniffi-rs`** for Swift+Kotlin and `wasm-bindgen` for web, with a `no_std`-clean primitives sub-crate that drops into firmware.

This spec captures the new architecture, the layered crate structure, the public API contract (preserved from the original docs with minor extensions), the integration story for every target platform, and the phased plan from empty repo to v1.0 and beyond.

## Decisions locked in this spec

- **Engine:** pure-Rust (`bdk_wallet`, `alloy`, Anza's Solana split crates, `xrpl-rust`, `secp256k1`, `ed25519-dalek`, `bip39`, `slip-10`). Not Trust Wallet Core. (ADR D1)
- **Architecture:** one Rust workspace, multiple language bindings auto-generated from FFI surface. (ADR D3)
- **Bindings:** `uniffi-rs` → SwiftPM + Maven AAR; `wasm-bindgen` → npm. (ADR D3, D6)
- **Layering:** `primitives` ← `chains` ← `core` ← `ffi`/`wasm`. One-way dependencies, enforced by `cargo-deny`. `primitives` is `no_std`-clean. (ADR D9)
- **Releases:** lockstep semver across every binding from a single Git tag. (ADR D8)
- **API boundary:** plain language values only. No engine types in public. (ADR D5)
- **Bitcoin:** BIP-84 native SegWit at v1; Taproot when ready. (ADR D4)
- **Out-of-scope locked:** no RPC, no fee logic, no storage, no UI. (ADR D7)
- **Async:** binding-layer concern, not core. (ADR D10)
- **Secret memory:** `zeroize::Zeroizing` everywhere; bindings extend within language limits. (ADR D11)
- **Chain trait:** one `ChainSigner` trait per chain family. (ADR D12)

## Documents produced

The full design lives in `docs/`:

- `docs/README.md` — index
- `docs/overview.md` — what / why / consumers
- `docs/architecture.md` — Rust core + bindings architecture
- `docs/decisions.md` — 12 ADRs
- `docs/folder-structure.md` — file-by-file layout
- `docs/api.md` — public API contract
- `docs/chains.md` — chain registry
- `docs/flows.md` — sequence diagrams
- `docs/error-model.md` — `JovaError` taxonomy and per-binding mapping
- `docs/memory-and-keys.md` — secret-clearing contract
- `docs/testing.md` — vectors, property tests, fuzz, parity
- `docs/build-and-release.md` — CI, publishing, semver lockstep
- `docs/security.md` — threat model, audit posture, supply-chain
- `docs/integration-ios.md` — iOS app guide
- `docs/integration-android.md` — Android app guide
- `docs/integration-web.md` — browser/Node WASM guide
- `docs/integration-backend.md` — Rust/Node backend guide
- `docs/integration-hardware.md` — firmware guide
- `docs/plan.md` — phased plan from empty repo to v1.0+
- `docs/glossary.md` — terms

## Phased plan

Risk-first ordering: feasibility spike → bootstrap → EVM → BTC (highest funds-at-risk first) → SOL+XRP → app integration with feature flags → audit + v1.0 → WASM → hardware.

| Phase | Duration | Deliverable |
|---|---|---|
| -1 | 3–5 d | Feasibility spike: prove uniffi + Swift + Kotlin + WASM compile with chosen chain crates. |
| 0 | 3–5 d | Repo bootstrap, hello-world on every binding incl. WASM compile smoke (`v0.0.1`) |
| 1 | 10–14 d | Ethereum end-to-end on Rust+Swift+Kotlin; WASM compile-only (`v0.1.0`) |
| 2 | 3–4 wk | Bitcoin (highest funds-at-risk first) (`v0.2.0`) |
| 3 | 3–5 wk | Solana + XRP + remaining EVM chains (`v0.5.0`) |
| 4 | 3–4 wk | iOS + Android integration with per-chain feature flags |
| 5 | 3–4 wk | Audit, RC cycles, hardening, `v1.0.0` |
| 6 | 2–3 wk | WASM functional + npm publish (`v1.1.0`) |
| 7 | 4–6 wk | Hardware-wallet readiness (`v1.2.0`) |
| 8 | TBD | Custom Jova chain when it ships |

**Total to v1.0:** ~16–22 weeks (4–5.5 months) for a senior team. Backend Rust direct is available continuously from v0.1.0; it is not a separate phase.

## Research basis

Web research (May 2026) covered:

- `uniffi-rs` maturity and adopters (Mozilla AS, BDK, Bitkey, Foundation).
- TWC's portability story (no first-class Rust crate, no firmware story, beta WASM).
- Pure-Rust per-chain crates (BDK, alloy, Anza Solana split crates, xrpl-rust) — all production grade.
- KMP vs Rust+uniffi tradeoffs (Bitkey's hybrid pattern; KMP fails on hardware/web).
- WASM via `uniffi-bindgen-javascript` vs Kotlin/Wasm beta.
- `no_std` cleanliness of the candidate stack.
- BDK's CI / release pattern as the reference template.

## Open questions (none blocking, all post-Phase-0)

- Whether to ship a `viem`/`wagmi` adapter package alongside the WASM binding (Phase 6 candidate).
- Whether to ship a Go-via-cgo binding before Phase 7 (depends on whether a Go backend service materializes).
- Reproducible-build attestation level (SLSA 3 in Phase 5+).
- Bug-bounty platform choice for v1.0.

## What was not in scope of this spec

- Implementation. Phase 0 of `docs/plan.md` does that.
- App-side detail beyond the integration-guide level (apps have their own design docs).
- Backend service design (separate repo).
- Hardware-wallet firmware design (separate repo when it exists).
