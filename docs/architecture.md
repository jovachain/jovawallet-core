# Architecture

## Goal

A signing SDK that runs identically on every platform Jova ships to — iOS, Android, web, desktop, backend, and eventually hardware-wallet firmware. One source of crypto truth in Rust; drift between bindings is detected by shared vectors at the binding boundary and prevented from merging.

## Shape

`jovawallet-core` is a Rust workspace. The workspace contains a layered set of crates. The lowest layer is `no_std`-clean and runs on hardware-wallet firmware unchanged. The top layer exposes a public API that is then re-exported through two FFI surfaces — `uniffi-rs` for Swift and Kotlin, `wasm-bindgen` for JavaScript and WebAssembly. Native consumer packages (SwiftPM, Maven AAR, npm) are assembled around those FFI outputs by CI.

```
                       ┌────────────────────────────────────────────┐
                       │  Rust workspace (jovawallet-core)           │
                       │                                             │
   FIRMWARE ─────────► │  jova-core-primitives  (no_std, no global) │
                       │      curves · BIP-32/39/44 · SLIP-10 ·     │
                       │      hashes · zeroizing key types          │
                       │                       ▲                     │
                       │                       │                     │
                       │  jova-core-chains  (std)                    │
                       │      btc · evm · sol · xrp                  │
                       │      one ChainSigner trait, one impl each   │
                       │                       ▲                     │
                       │                       │                     │
                       │  jova-core  (public Rust API, sync only)    │
                       │      JovaWallet · JovaChain · UnsignedTx    │
                       │      Address · Signature · JovaError        │
                       │                       ▲                     │
                       │       ┌───────────────┴───────────────┐     │
                       │       │                               │     │
                       │  jova-core-ffi                  jova-core-wasm
                       │  (uniffi-rs)                    (wasm-bindgen)
                       └───────┬───────────────┬───────────────┬─────┘
                               │               │               │
              ┌────────────────┘               │               └────────────────┐
              ▼                                ▼                                ▼
     SwiftPM "JovaCore"            Maven "io.jovachain:jova-core"      npm "@jovachain/wallet-core"
        iOS · macOS                    Android · JVM                       browser · Node ESM

                BACKEND in Rust ──► imports `jova-core` directly as a Cargo dep.
                BACKEND in Node ──► imports the npm WASM build.
                FIRMWARE        ──► imports `jova-core-primitives` only, builds for `thumbv7em-none-eabihf`.
```

## Design principles

### 1. One implementation, many bindings

Every binding is generated from the same Rust source. We do not write a Swift implementation of EIP-712 and a Kotlin implementation of EIP-712. We write one Rust implementation, and Swift+Kotlin call it through `uniffi-rs`. The bindings contain no cryptographic logic — only marshalling and idiomatic-language wrappers.

This is what eliminates *crypto-layer* drift: it is prevented by *not having two implementations to drift apart*. Drift in the smaller surfaces that remain — FFI marshalling, JSON enum mapping, hand-written convenience layers, vector-coverage gaps — is *detected* by shared vectors and parity tests, and prevented from merging when CI catches it.

### 2. Layered, with one-way dependencies

```
jova-core-ffi  ─┐                              jova-core-wasm
                ├──► jova-core ──► jova-core-chains ──► jova-core-primitives
```

- **`jova-core-primitives`** — pure cryptographic primitives. `no_std`. Heap is allowed where unavoidable (e.g., `bip39`'s wordlist string ops), avoided in the hot path. Everything here is `zeroize`-safe. This is the layer that lands on hardware wallets.
- **`jova-core-chains`** — chain-specific encoding and signing. Each chain family is one Rust module behind a shared `ChainSigner` trait. Depends on `bdk_wallet`, `alloy`, the Anza Solana split crates (`solana-keypair`, `solana-transaction`, `solana-message`, `solana-pubkey`, `solana-signature`), and `xrpl-rust`. Never holds keys; receives them as references for the duration of a single call.
- **`jova-core`** — the public Rust API. The only crate consumers see if they pull `jova-core` directly. Sync-only. No async runtime. No logging framework. No global state.
- **`jova-core-ffi`** — `uniffi-rs` annotations over `jova-core`. The only crate that knows about UDL/proc-macros/JNI/Swift bindings.
- **`jova-core-wasm`** — `wasm-bindgen` annotations over `jova-core`. JSON in, JSON out at the boundary.

`cargo-deny` and a workspace-level `[workspace.dependencies]` block enforce these arrows. A PR that adds an upward dependency fails CI.

### 3. Plain values at every boundary

No file outside `jova-core-ffi` or `jova-core-wasm` references those crates. No file outside `jova-core-chains` references the underlying `bdk_wallet`, `alloy`, etc. The binding-language facing API is plain strings, byte arrays, plain enums, plain structs. Apps never `import bdk` or `import TrustWalletCore`.

This is what makes engine swaps painless. Replacing the Solana stack with an alternative (or any chain crate with another) is a one-file change in `chains::sol` — no consumer notices.

### 4. Vector-first

Adding or changing behavior starts with adding or changing `spec/test-vectors.json`. The Rust test suite reads it. The Swift test suite reads it. The Kotlin test suite reads it. The JS test suite reads it. If any binding disagrees, the bug is in that binding, not in the spec.

The vectors file is the load-bearing document of this whole project. `architecture.md` is just commentary.

### 5. Stateless wallet objects

A `JovaWallet` is constructed from a mnemonic, used for one or more signing operations, then dropped. It owns no database, no cache, no network client, no listeners, no observers. Reconstructing one is cheap (key derivation is fast).

Concurrency is the consumer's problem: do not share a `JovaWallet` across threads. Construct fresh instances when you need parallelism.

### 6. Small surface forever

Every method that ships is locked into the contract — apps depend on it, drift between platforms must be prevented forever. Adding a method has a real cost: it locks behavior on every binding for the rest of the SDK's life. Refuse anything that isn't core signing primitives.

## What's in scope

- BIP-39 mnemonic generation, validation, seed derivation.
- BIP-32 HD derivation (and SLIP-10 for ed25519 chains).
- BIP-44 / BIP-84 / chain-specific derivation paths.
- Address derivation and address validation per chain.
- Transaction signing: EIP-1559 EVM, BIP-174 PSBT (P2WPKH), Solana v0 versioned tx, XRPL canonical.
- Message signing: EIP-191, EIP-712 v4, BIP-322 (preferred) + legacy `signMessage`, raw ed25519 for Solana.

## What's out of scope (locked)

- Network I/O of any kind. No RPC, no balance, no broadcast, no fee estimation, no nonce lookup. The backend builds the unsigned payload; the SDK signs it.
- Secret storage at rest. iOS Keychain, Android Keystore — apps own that boundary.
- WalletConnect protocol negotiation. Apps speak WalletConnect; once they have an unsigned payload, they hand it to `JovaWallet.signTx(unsigned:)`. Same boundary as backend-built tx.
- UI, biometrics, PIN, push notifications.
- PSBT construction, fee suggestion, gas oracle, RBF logic — these are payload-construction concerns owned by the backend.

## Threat model summary

What `jovawallet-core` defends against:

- **Drift between platforms.** Test-vector enforcement makes any divergence a CI failure.
- **Crypto bugs in user-facing app code.** All cryptographic operations route through audited Rust crates (`secp256k1`, `ed25519-dalek`, `bdk_wallet`, `alloy`). No rolling-our-own crypto. No language-specific reimplementations.
- **Type-leak refactor risk.** Underlying crate types are confined to `jova-core-chains`. A future engine swap is a one-file change.
- **Secret-clearing failures across FFI.** Sensitive types use `zeroize::Zeroizing` in Rust. Bindings extend this within language limits (Kotlin `Closeable` with explicit `close()`; Swift `deinit` clears its `Data`; JS uses `WeakRef` + explicit `.zeroize()`).
- **Supply-chain compromise.** All Rust dependencies pinned by `Cargo.lock` + checksum. `cargo-vet` audits applied. Every release tag is signed.

What it does **not** defend against — by design, since these are app or platform concerns:

- Compromised app process memory (rooted device, attached debugger, malicious in-process library).
- User entering a seed phrase into a phishing UI.
- Loss of the seed phrase (apps own backup flows).
- Compromised broadcast (MITM on tx submission — backend's TLS handles this).
- Side-channel attacks on shared hardware (a known limitation of any software signing on consumer phones).

`security.md` expands this section with concrete controls per category.

## Build and release model

- **Rust core** is published to `crates.io` as `jova-core`, `jova-core-primitives`, `jova-core-chains` on every tag. Direct Rust consumers (backends, CLIs, future desktop apps) depend on these.
- **Swift** is distributed via SwiftPM. The build emits an XCFramework; CI pushes it plus a `Package.swift` to a satellite repo `jovachain/jovawallet-core-swift` on every tag. (SwiftPM resolves better when `Package.swift` sits at a repo root — this is the BDK pattern.)
- **Kotlin** is published as an Android AAR plus a JVM jar to Maven Central as `io.jovachain:jova-core` and `io.jovachain:jova-core-jvm` on every tag.
- **WebAssembly** is published to npm as `@jovachain/wallet-core` on every tag (Phase 6 onward).
- **Lockstep versioning.** Every artifact at tag `v1.4.2` is built from the same commit. No mixed versions are ever published.
- **Trust roots.** `Cargo.lock` is committed and required for reproducible builds. Every release artifact's checksum is published to the GitHub release page.

`build-and-release.md` documents the CI matrix and the actual workflows.

## Engine choice

We do not wrap Trust Wallet Core. The four chains we support at v1 (BTC, EVM, SOL, XRP) all have first-rate pure-Rust crates that are equal-or-better than TWC's coverage:

| Chain | Crate | Why |
|---|---|---|
| Bitcoin | `bdk_wallet`, `rust-bitcoin`, `rust-miniscript` | Production standard. Used by Block's Bitkey, Bull Bitcoin Mobile. PSBT v2, descriptors, BIP-84 + BIP-86. |
| EVM | `alloy` (consensus + sol-types) | Replaces deprecated `ethers-rs`. EIP-1559, EIP-712 v4, EIP-191. |
| Solana | Anza split crates (`solana-keypair`, `solana-transaction`, `solana-message`, `solana-pubkey`, `solana-signature`) | First-party. VersionedTransaction (v0). Smaller dep tree than the monolithic `solana-sdk`; WASM-viable. |
| XRP | `xrpl-rust` | XRPL Foundation grant winner. Canonical serialization + signing. |
| Primitives | `secp256k1`, `k256`, `ed25519-dalek`, `bip39`, `slip-10`, `bip32`, `zeroize` | All `no_std`-clean. Used in `jova-core-primitives`. |

ADR D1 in `decisions.md` documents this choice and the conditions under which we'd revisit it.
