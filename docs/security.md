# Security

The threat model, audit posture, and supply-chain controls for `jovawallet-core`.

This document is consumed by:

- **External auditors** scoping a paid review.
- **App-team engineers** evaluating whether an SDK upgrade is safe.
- **Bug-bounty researchers** identifying in-scope and out-of-scope behavior.
- **Internal incident responders** during a suspected security event.

If you're filing a vulnerability disclosure, see `SECURITY.md` at the repo root.

---

## Threat model

### What we defend against

| Threat | Defense |
|---|---|
| Drift between platforms producing different signatures | All bindings load `spec/test-vectors.json`. CI fails on byte mismatch. (`testing.md`) |
| Crypto bugs in user-facing app code | All cryptographic operations run in audited Rust crates (`secp256k1`, `ed25519-dalek`, `bdk_wallet`, `alloy`). No language-specific reimplementations. (`architecture.md`) |
| Type-leak refactor risk | Engine-specific types confined to `jova-core-chains`. Public API exposes plain values only. (ADR D5) |
| Secret-clearing failures | `zeroize::Zeroizing` wrappers; `Drop` clears; bindings extend within language limits. (`memory-and-keys.md`) |
| Supply-chain compromise of Rust deps | `Cargo.lock` committed; `cargo-vet`, `cargo-deny`, `cargo-audit` in CI; license whitelist; advisory denylist. (`build-and-release.md`) |
| Malformed input causing a panic / abort | Fuzz harnesses on every parser; nightly cargo-fuzz; **the release profile sets `panic = "abort"`**, because every fallible operation already returns `Result` and unwinding through FFI is undefined behavior. A panic in production therefore aborts the host process — apps must rely on `Result` for recoverability and treat any panic as an SDK bug to file. Panic-free is the goal; aborting is the defence against UB if the goal slips. (`testing.md`) |
| Use-after-free or double-free across FFI | `Box::from_raw` is the unique owner; `cargo miri test` nightly catches UB. (`memory-and-keys.md`) |
| Side-channel timing leaks in our code | We don't introduce equality checks on secrets. Underlying crates use constant-time primitives. (`memory-and-keys.md`) |
| Unsafe code blocks introducing memory safety bugs | `#![forbid(unsafe_code)]` at every crate except `jova-core-ffi` (which has minimal `unsafe` for handle marshalling). What `unsafe` exists is reviewed PR-by-PR. |
| Outdated dependencies with known CVEs | `cargo audit` daily cron; release blocked if advisories present. |

### What we do NOT defend against

These are app, OS, or platform concerns. We document them so consumers know where their responsibility starts.

| Threat | Whose problem |
|---|---|
| Compromised app process memory (rooted device, attached debugger, malicious in-process library) | Host OS + app sandboxing |
| User entering seed phrase into a phishing UI | App (UI hardening, authenticity signaling) |
| Loss of seed phrase (no backup) | App (backup + recovery flows) |
| MITM on transaction broadcast | Backend (TLS pinning, request signing) |
| Side-channel attacks on shared hardware (cache timing, electromagnetic emanation) | Hardware platform; consumer-grade phones can't fully mitigate this |
| User pasting mnemonic into a system clipboard | App (warn user, restrict clipboard read) |
| App bug that logs the mnemonic | App; SDK refuses to log secrets, so it can only happen if the app explicitly reads-and-logs |
| Compromised CI signing keys for an app's release | App's release process; SDK has its own signing keys for *our* artifacts |
| Vulnerabilities in the underlying platform's crypto (e.g., a libcrypto bug under a chain crate) | Platform; we update dependencies promptly when CVEs land |

---

## Per-component audit notes

### `jova-core-primitives`

- `unsafe` count: **0**. `#![forbid(unsafe_code)]`.
- Allocator: assumes presence of `alloc` for heap-using paths. `no_std`-clean.
- Dependencies: all crypto-grade, all individually audited.
- The most security-sensitive layer; smallest, simplest, audited first.

### `jova-core-chains`

- `unsafe` count: **0** in our code.
- Trusted dependencies: `bdk_wallet`, `alloy`, Anza's split Solana crates, `xrpl-rust`. Each maintained by a team that has its own audit story (BDK has multiple paid audits; alloy is the de-facto post-ethers-rs replacement; the Solana split crates are first-party Anza; xrpl-rust is XRPL Foundation funded).
- Per-chain modules are independently testable.

### `jova-core`

- `unsafe` count: **0**.
- Surface is small and direct.

### `jova-core-ffi`

- `unsafe` count: **bounded**. Generated `uniffi-rs` glue contains some `unsafe` for handle marshalling. We do not write our own; `uniffi-rs` is itself Mozilla-audited.
- Primary risk is in handle lifecycle (`Box::from_raw` correctness) — `cargo miri test` exercises this.

### `jova-core-wasm`

- `unsafe` count: **0** in our code; some in `wasm-bindgen` glue (out of our control, audited upstream).
- WASM execution is sandboxed by the runtime — additional defense in depth.

### Bindings (Swift, Kotlin, JS)

- Entirely auto-generated except for the small `Convenience.{swift,kt,ts}` ergonomics layer.
- Convenience layer is reviewed against a "no business logic, only re-export" rule.
- JS does not use `eval`, `Function(...)`, or any string-as-code pattern.

---

## Secret-handling controls

(Cross-references `memory-and-keys.md`.)

- Every secret-bearing type implements `Zeroize` + `ZeroizeOnDrop`.
- `MnemonicBuffer` API exists for apps that want to control the input buffer's lifetime.
- Bindings expose the wallet handle as a managed resource: Swift `deinit`, Kotlin `AutoCloseable`, JS `destroy()`.
- No log call in the SDK touches secret material. The SDK has no logger by default.
- `Display` impls on secret types are absent. `Debug` impls redact.
- `Cargo.toml` of `jova-core-primitives` denies `derive(Clone)` on inner secret types via `cargo deny lints`.

---

## Supply chain

### Dependency policy

We accept new Rust dependencies only when:

1. The crate is on `crates.io` (no Git deps in production crates).
2. The crate has a license in our whitelist (MIT, Apache-2.0, BSD-3-Clause, ISC, MPL-2.0).
3. The crate has had a release in the past 12 months *or* is in stable maintenance mode with no open advisories.
4. We have written a `cargo-vet` audit row for the version we're using.
5. The dependency tree it brings doesn't add a transitive dependency we wouldn't accept directly.

`deny.toml` enforces points 2–4 in CI.

### Pinning

- **Production crates** (`jova-core-*`): exact versions in `Cargo.toml`. No `^` ranges.
- **Dev/test/fuzz dependencies**: `^` ranges allowed (these don't ship to consumers).
- `Cargo.lock` is committed and `cargo build --locked` is used in CI and release.

### Auditing dependencies

`cargo-vet` is the canonical tool. Every dependency we transitively pull in must have an audit row in `supply-chain/audits.toml`. Audits we trust transitively (Mozilla's `mozilla-vet`, Bytecode Alliance's, Google's) are imported via `imports`.

`cargo-deny` enforces the advisory denylist daily. A new RUSTSEC advisory blocks the release pipeline until we've patched or worked around it.

### Reproducible builds

See `build-and-release.md` for current state. SLSA Level 3 is a Phase 5 goal.

---

## Release-time integrity

- Every release tag is GPG-signed by the release manager.
- The GitHub release includes per-artifact SHA-256 checksums.
- The Maven Central artifacts are GPG-signed (required by Sonatype).
- The crates.io publish uses a CI-only API token that can publish but not yank/unyank.
- The npm publish uses an `@jovachain`-scoped token.

The signing keys are stored as GitHub Actions secrets, with rotation every 6 months.

---

## Disclosure policy

### Reporting

Email `security@jovachain.io` with a description of the issue. PGP key is published in `SECURITY.md`. We respond within 48 hours.

For high-severity issues we use a private GitHub Security Advisory.

### Severity classification

| Level | Definition | Example | SLA to fix |
|---|---|---|---|
| Critical | User funds at risk under realistic conditions | Bug produces a signature that is valid but spends more than the user authorized | 72 hours |
| High | Failure of a documented invariant | Mnemonic checksum bypass; address derivation differs between bindings | 7 days |
| Medium | Security-relevant but not directly exploitable | Memory not zeroed on a non-default code path; logged secret content | 30 days |
| Low | Defense-in-depth issue | Outdated dependency without a known exploit | next release |

### Bug bounty

A formal bounty program will launch alongside `v1.0.0`. Out-of-scope categories will include:

- Issues only reproducible with engineered debug builds.
- Issues that require root / jailbreak.
- Issues in upstream Rust crates we depend on (report to that crate's maintainers).
- Issues in app-side code (those are the app's bounty programs).

---

## Audit history

This section will be updated as audits are completed.

- _Phase 5_: First external paid audit. Scope: `jova-core-primitives`, `jova-core-chains`, `jova-core`. Excludes binding glue.
- _Phase 5+_: Bindings + FFI audit.

Reports will be published as PDFs in `docs/audits/`.

---

## Known security limitations

These are documented honestly because hiding them would be worse:

1. **No mlock by default.** Pages holding the seed can be swapped to disk by the OS. Mitigation: short-lived wallet handles; apps that want this protection can wrap their own buffers in OS-locked memory before calling us.
2. **JS GC is not deterministic.** Apps relying on `JovaWallet.destroy()` for clearing must call it explicitly. Failure to do so leaves seed bytes in WASM linear memory until GC, which is non-deterministic in JS.
3. **Swift / Kotlin String immutability.** A mnemonic passed in as a `String` cannot be zeroed by us. The `MnemonicBuffer` API exists for apps that need this guarantee.
4. **Side-channel exposure on shared hardware.** Standard limitation of any software signing on consumer phones. Hardware-wallet integration (Phase 7) is the long-term answer.
5. **Compiler-introduced register copies.** `zeroize` cannot prevent the compiler from putting copies in registers or the stack during inlining. We avoid `#[inline(always)]` on hot paths and minimize copies but cannot eliminate this entirely.
6. **TLS / certificate pinning.** N/A — the SDK does no I/O. Apps and backends own this.

---

## Pre-release security checklist

Before a release tag is pushed:

- [ ] No new `unsafe` blocks (per `cargo geiger`).
- [ ] `cargo audit` passes with no advisories at any severity.
- [ ] `cargo deny check` passes.
- [ ] `cargo vet` passes.
- [ ] Nightly fuzz has run for at least 7 consecutive days without new crashes.
- [ ] `cargo miri test` on `jova-core-primitives` passes.
- [ ] Vector tests pass on every binding.
- [ ] Memory tests confirm secret clearing on every binding.
- [ ] No new TODO/FIXME comments referencing security in the diff.
- [ ] CHANGELOG documents any security-relevant changes.
- [ ] If the release patches a CVE, a Security Advisory is drafted.

---

## Audit scope (recommended for paid review)

For an external audit, recommended scope in priority order:

1. **`jova-core-primitives`** (8–12 hours) — keys, derivation, hashes, encoding. The compromise-blast-radius focus.
2. **`jova-core-chains`** per chain (16–24 hours each) — chain-specific encoding/signing.
3. **`jova-core` + FFI handle lifecycle** (8 hours).
4. **`jova-core-ffi` and `jova-core-wasm` glue** (8 hours).
5. **Build and release pipeline** (4–8 hours) — supply-chain controls, signing, reproducibility.

Auditors should have a copy of `decisions.md`, `memory-and-keys.md`, this file, and `architecture.md` before starting.
