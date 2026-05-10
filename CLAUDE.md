# CLAUDE.md

Orientation for any AI agent working on `jovawallet-core`. **Read this first.**

---

## What this project is

`jovawallet-core` is a multi-chain transaction-signing SDK for Jova wallets.

**Architecture:** pure-Rust core + uniffi-rs (generates Swift + Kotlin bindings) + wasm-bindgen (generates JavaScript/WASM) + a `no_std`-clean primitives sub-crate for hardware-wallet firmware. Chains supported in v1: Bitcoin (BIP-84 native SegWit), Ethereum + the EVM family (Polygon, BSC, Arbitrum, Optimism, Base, customEvm), Solana (v0 versioned transactions), and XRP. v1 ships to iOS and Android. v1.1 adds web/Node WASM. v1.2 adds hardware-wallet readiness.

**Engine:** pure-Rust crates per chain — `bdk_wallet`, `alloy`, the Anza Solana split crates (`solana-keypair` / `solana-transaction` / `solana-message` / `solana-pubkey` / `solana-signature`), `xrpl`. NOT Trust Wallet Core.

## Status

**Pre-implementation.** Design docs and per-phase implementation plans are complete (~30 markdown files). **No code has been written.** **The repo is not yet a git repo** — the very first task in the very first plan runs `git init`.

## Where to start

Read in this order. Stop when you have what you need for your task.

| # | File | Why |
|---|---|---|
| 1 | `docs/overview.md` | What this is, what it isn't, who consumes it |
| 2 | `docs/architecture.md` | Layered Rust workspace + bindings shape |
| 3 | `docs/decisions.md` | 12 ADRs covering every load-bearing choice |
| 4 | `docs/superpowers/plans/README.md` | Execution roadmap; index of all phase plans |
| 5 | `docs/superpowers/plans/2026-05-05-phase-minus-1-feasibility-spike.md` | **The first phase to execute.** |

For chain-specific or contract-level reference: `docs/api.md`, `docs/chains.md`, `docs/error-model.md`, `docs/flows.md`, `docs/memory-and-keys.md`, `docs/testing.md`, `docs/build-and-release.md`, `docs/security.md`, `docs/glossary.md`.

For integration patterns (after the SDK ships): `docs/integration-{ios,android,web,backend,hardware}.md`.

## How to execute

1. Phases run in order: **-1 → 0 → 1 → 2 → 3 → 4 → 5 → 6 → 7**.
2. Each phase has its own plan file at `docs/superpowers/plans/2026-05-05-phase-N-*.md`. Phases -1, 0, 1, 2, 3, 6 are full TDD-level; Phases 4, 5, 7 are process plans (checklist-driven, partly outside this repo).
3. **Phase -1 first.** It is a 3–5 day feasibility spike that validates the toolchain and produces `docs/feasibility-report.md`. **Stop after Phase -1.** The user reads the report and decides go/no-go for Phase 0.
4. Recommended dispatch: `superpowers:subagent-driven-development` (one fresh subagent per task, two-stage review per task).
5. Confirm WHEN with the user before any first dispatch. **Do not autonomously start executing.**

## Environment

The user is on macOS arm64 with Xcode and Node + pnpm pre-installed. **Rust and Android NDK are NOT pre-installed.** Expect to install missing tools as preconditions to the relevant tasks.

**Versioning policy (decided 2026-05-10 at project start):** install latest stable for all tools; use `--locked` (so transitive deps are frozen for reproducibility) but omit `--version` (so cargo picks the current latest tool release). Concrete versions below are recorded as of 2026-05-10. Exception: `uniffi-bindgen` (the binary) and the `uniffi` crate (in `Cargo.toml`) **must be the exact same version** — sync at install time (both currently 0.31.1). See `docs/env-setup.md` for the full local-install walkthrough.

> **Install pattern change for uniffi 0.30+:** earlier plan files reference `cargo install uniffi-bindgen-cli`. That crate doesn't exist on crates.io. In modern uniffi the CLI binary ships inside the `uniffi` umbrella crate, gated by the `cli` feature: `cargo install uniffi --features cli --locked`. This installs two binaries: `uniffi-bindgen` (unified) and `uniffi-bindgen-swift` (Swift-optimized). Treat `uniffi-bindgen-cli` in older plan text as the old name for the binary `uniffi-bindgen`.

| Tool | Current (2026-05-10) | Install | Required by |
|---|---|---|---|
| Rust stable | **1.95.0** (released 2026-04-16) | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh -s -- -y --default-toolchain stable` | Phase 0 onward |
| iOS targets | (matches toolchain) | `rustup target add aarch64-apple-ios aarch64-apple-ios-sim aarch64-apple-darwin x86_64-apple-darwin` | Phase -1 Task 3, all native iOS |
| Android targets | (matches toolchain) | `rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android` | Phase -1 Task 4, all native Android |
| WASM target | (matches toolchain) | `rustup target add wasm32-unknown-unknown` | Phase -1 Task 5, Phase 6 |
| `no_std` target | (matches toolchain) | `rustup target add thumbv7em-none-eabihf` | Phase -1 Task 6, Phase 7 |
| `uniffi-bindgen` (binary) — installed via `uniffi --features cli` | **0.31.1** | `cargo install uniffi --features cli --locked` — installs both `uniffi-bindgen` and `uniffi-bindgen-swift` | iOS + Android binding generation |
| `cargo-ndk` | **4.1.2** | `cargo install cargo-ndk --locked` | Android cross-compile |
| `wasm-pack` | **0.14.0** | `cargo install wasm-pack --locked` | WASM build |
| `cargo-fuzz` | **0.13.1** | `cargo install cargo-fuzz --locked` (nightly Rust required) | Phase 1 fuzz harnesses |
| `just` | **1.51.0** | `cargo install just --locked` | Phase 0 task running |
| Android NDK | **r29 stable (`29.0.14206865`)** — Oct 2025; r30 still in beta as of 2026-05-10 | Studio SDK Manager → SDK Tools → NDK (Side by side) → tick `29.0.14206865` (NO `-beta` suffix); export `ANDROID_NDK_HOME=$HOME/Library/Android/sdk/ndk/29.0.14206865` | Phase -1 Task 4 (cross-compile), Android binding builds |
| Foundry / `cast` | (latest from script) | `curl -L https://foundry.paradigm.xyz \| bash; foundryup` | Phase 1 EVM vector capture |
| `bdk-cli` | **1.0.0** | `cargo install bdk-cli --locked` | Phase 2 BTC vector capture |
| `solana-cli` (Anza) | (latest from installer) | `sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"` | Phase 3c SOL vector capture |
| `rippled` or `xrpl-py` | (latest) | per XRPL docs | Phase 3b XRP vector capture |

If a tool is missing when a task needs it, install it as a precondition. Do not work around it. Do not skip a verification step because the tool isn't there.

## Conventions (locked — not up for debate during execution)

- **Test-as-contract.** Captured reference values in `spec/test-vectors.json` are invariant. If a code snippet in a plan doesn't compile against the actual crate version, fix the snippet, not the test. Reference values come from external signers (`cast`, `bdk-cli`, `solana-cli`, `rippled`/`xrpl-py`) — never invented.
- **TDD per task.** Failing test → minimal implementation → passing test → commit. One commit per logical task step. The plans are written in this shape; follow it.
- **No placeholders in committed artifacts.** `tools/verify-spec` (built in Phase 0) rejects any vector whose `expected` field contains `TODO`, `<capture`, or `REPLACE`. Capture real values before committing.
- **Typed FFI.** uniffi enums + records, not JSON-shaped String parameters.
- **Plain types at the boundary.** No `bdk_wallet`, `alloy`, Solana split crates, or `xrpl` types in `jova-core` or any binding. Engine types stay confined to `jova-core-chains`.
- **`zeroize` everywhere for secrets.** `Mnemonic`, `Seed`, `XPrv`, `Ed25519Xprv` all derive `Zeroize + ZeroizeOnDrop`. **`Seed` and `XPrv` are NOT `Clone`.** `Mnemonic` is `Clone` (intentional — see `docs/memory-and-keys.md`).
- **Lockstep versioning.** Every binding at every tag is built from the same commit. Release publish order is least-revertible-last: Swift satellite → Maven → npm → crates.io.
- **`cargo-vet`, `cargo-deny`, `cargo-audit` pass on every PR.** Don't bypass. Don't add a dep that fails any of them.
- **`panic = "abort"` in release profile.** Every fallible op uses `Result`. Panics indicate bugs and abort the host. Don't try-catch panics across FFI — that's UB.

## Don'ts

- **Don't autonomously dispatch subagents or run `git init` without explicit user confirmation.** The user's preference is to confirm WHEN before any first dispatch — see the saved memory `feedback_no_autonomous_execution.md`.
- **Don't substitute design choices.** Architecture and the 12 ADRs in `docs/decisions.md` are locked. If a task seems to violate one, stop and ask.
- **Don't add features beyond what a task specifies.** YAGNI is enforced.
- **Don't `git push` to remote** unless the plan explicitly calls for it. Phase 0 sets up the GitHub remote; before that, work is local.
- **Don't skip the spec-compliance + code-quality reviews per task.** Per `superpowers:subagent-driven-development`: implementer → spec reviewer → fix loop until clean → code quality reviewer → fix loop until clean → next task.
- **Don't commit the BTC migration CSV.** `tools/btc-migration-check/known-android-mappings.csv` contains user mnemonics. The directory's `.gitignore` excludes it.
- **Don't fight the test.** If a vector says `expected.signed_hex = 0x...abc` and your code produces `0x...abd`, your code is wrong, not the vector.

## When you finish a phase

Each plan ends with explicit exit criteria. Verify them all. Then:

- Phases ending in a tag (-1, 0, 1, 2, 3, 5, 6, 7): tag the version on `main`, push the tag, wait for the release pipeline.
- Phases ending in a milestone (4): document the milestone (e.g., app at 100% rollout) and surface to the user.

After Phase -1 specifically: stop. Surface `docs/feasibility-report.md`. Wait for go/no-go on Phase 0.

## If you get stuck

- **Toolchain issues:** install via standard channels (rustup, cargo install, brew). If a tool genuinely isn't available, surface to the user.
- **Crate API has changed since the plan was written:** trust the test, adjust the snippet.
- **Vector reference value can't be captured (external signer broken/missing):** stop. Surface to user. Don't invent values.
- **Spec/plan contradicts something:** stop. Surface to user. The user (or external review) decides.
- **You feel uncertain whether your approach is correct:** stop. Use `BLOCKED` or `NEEDS_CONTEXT` per the subagent-driven-development skill. Bad work is worse than no work.
