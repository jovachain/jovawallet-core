# Glossary

Terms used across the `jovawallet-core` documentation. If you find yourself confused by a word, look here first.

## A

**ABI (Application Binary Interface).** The contract between compiled artifacts. We use the term in two senses: the C ABI exposed by `jova-core-ffi` for foreign-language consumption, and the Android NDK ABIs (`arm64-v8a`, etc.) we cross-compile to.

**Account index.** The integer in `m/44'/<coin>'/<account>'/...` that lets a user have multiple addresses on the same chain. v1 apps always use `0`. Reserved on the API for a future advanced-mode feature.

**ADR (Architecture Decision Record).** A short document capturing a load-bearing design choice. See `decisions.md`.

**Alloy.** The current pure-Rust EVM toolkit (`alloy-rs/core`, `alloy-rs/alloy`). Replaces deprecated `ethers-rs`. We use it in `jova-core-chains::evm`.

**ALT (Address Lookup Table).** A Solana feature where a transaction references accounts via index into a pre-published table, reducing tx size. v0 versioned transactions support ALTs; the SDK signs whatever message the backend produced (ALTs are a payload concern).

## B

**Base58 / Base58check.** Compact alphanumeric encodings used by Bitcoin (legacy `1…` addresses), Solana (32-byte pubkeys), and XRP (`r…` addresses). Base58check adds a 4-byte checksum.

**BDK (Bitcoin Dev Kit).** The production-grade Rust crate ecosystem for Bitcoin (`bdk_wallet`, `bdk-tx`, `bdk-miniscript`). Funded by OpenSats and Btrust. Used by Block's Bitkey, Bull Bitcoin Mobile, and others. We use it in `jova-core-chains::btc`.

**bech32.** The address encoding for Bitcoin native SegWit (BIP-84) and Taproot (BIP-86). Format: `bc1q…` for SegWit, `bc1p…` for Taproot.

**BIP-32.** Hierarchical Deterministic (HD) key derivation for secp256k1. Defines `XPrv`/`XPub` and the `m/...` derivation path syntax.

**BIP-39.** Mnemonic-phrase encoding of a seed. 12 or 24 English words from a standard wordlist. The SDK supports it via the `bip39` crate.

**BIP-44.** Standard derivation path layout: `m/44'/<coin>'/<account>'/<change>/<index>`. Used by EVM, XRP, and Solana (with hardened-only variant).

**BIP-84.** Native SegWit derivation path: `m/84'/0'/<account>'/<change>/<index>` for Bitcoin. Produces P2WPKH `bc1q…` addresses. The SDK's BTC default. ADR D4.

**BIP-86.** Taproot derivation path: `m/86'/0'/...`. Produces P2TR `bc1p…` addresses. Roadmap; not v1.

**BIP-174 (PSBT).** Partially Signed Bitcoin Transaction format. The backend constructs a PSBT; the SDK signs the inputs it can; the resulting PSBT (or fully-signed tx) is returned.

**BIP-322.** Modern Bitcoin message-signing standard. Preferred over the legacy `signMessage` scheme. The SDK supports both.

**Binding.** A language-specific package built from the Rust core: Swift, Kotlin, JavaScript/WASM. `bindings/swift/`, `bindings/kotlin/`, `bindings/wasm/`.

## C

**`cargo-deny`.** A Rust tool enforcing license, advisory, and dependency-graph rules. Configured via `deny.toml`.

**`cargo-fuzz`.** Wrapper around libFuzzer for Rust. Used in `fuzz/`.

**`cargo-vet`.** Mozilla's tool for auditing the dependency tree. Audit rows live in `supply-chain/audits.toml`.

**Chain code.** The 32-byte half of an `XPrv` / `XPub` used in BIP-32 derivation. Not a secret per se but should be protected the same as the private key.

**`ChainSigner`.** The Rust trait every chain family implements in `jova-core-chains`. ADR D12. See `decisions.md`.

**Convenience layer.** The hand-written `Convenience.{swift,kt,ts}` files in each binding that add language-idiomatic helpers on top of the auto-generated API. Strict rule: no business logic, only re-export and ergonomics.

**Custodial.** A backend that holds and uses user secrets. Distinct from non-custodial, where users hold their own keys. The SDK supports custodial flows but does not encourage them — we ship a non-custodial-first design.

## D

**Derivation path.** A string like `m/44'/60'/0'/0/0` describing how to derive a child key from a master seed. See `chains.md` for the canonical paths per chain.

**Determinism.** A property the SDK enforces: the same input always produces the same output, on every binding, every platform, every release. Vector tests verify this.

**Drift.** When two implementations of the same contract disagree. The hazard the entire project exists to prevent.

## E

**ECDSA.** The signing algorithm used on secp256k1 (BTC, EVM, XRP). The SDK uses `secp256k1` and `alloy`'s implementations — never rolls its own.

**ed25519.** The elliptic curve used by Solana. The SDK uses `ed25519-dalek`. Not the same as secp256k1; distinct derivation (SLIP-10 not BIP-32) and distinct signing.

**EIP-55.** Ethereum's address checksum scheme. Mixed-case hex; valid Ethereum addresses pass validation when the case matches the checksum.

**EIP-191.** Ethereum's `personal_sign` standard. Prepends a `\x19Ethereum Signed Message:\n<len>` prefix before keccak hashing.

**EIP-712.** Ethereum's typed-data signing standard. Structured JSON with a domain and message; hashed with `hashStruct`. We support v4.

**EIP-1559.** Ethereum's type-2 transaction format with `maxFeePerGas` and `maxPriorityFeePerGas`. The default for all EVM chains we support. Legacy (type-0) txs are refused.

**EIP-2930.** Access lists. Optional in EIP-1559 txs.

## F

**FFI (Foreign Function Interface).** The boundary between Rust and a calling language. Our FFI is generated by `uniffi-rs` (Swift, Kotlin) and `wasm-bindgen` (JavaScript).

**Fuzzing.** Random-input testing to find crashes, panics, or undefined behavior. Run nightly via `cargo-fuzz`.

## G

**`getrandom`.** Rust's cross-platform secure random source. Wraps OS primitives. Replaced by an injected RNG on hardware.

## H

**Hardware wallet.** A dedicated device that holds the seed in tamper-resistant memory and signs transactions on user confirmation. Phase 7 target.

**HD (Hierarchical Deterministic) wallet.** A wallet where every key is derived from one seed via a structured derivation tree. Standardized by BIP-32 (secp256k1) and SLIP-10 (ed25519).

**HMAC.** A keyed message-authentication code used inside PBKDF2 (mnemonic-to-seed) and BIP-32 derivation.

## J

**`JovaError`.** The exhaustive error enum. See `error-model.md`.

**`JovaWallet`.** The public entry point. Construct from a mnemonic; use for signing; let it drop. See `api.md`.

**JNI.** Java Native Interface — how Kotlin calls into the Rust `.so` files on Android. Generated by `uniffi-rs`.

## K

**Keccak256.** The hash function used by Ethereum (it's not the same as standard SHA-3 due to a padding difference). Used in EIP-55 address checksums and EIP-1559 tx hashing.

**KMP (Kotlin Multiplatform).** A Kotlin feature for sharing code across JVM, iOS, and other targets. Considered in ADR D3 and rejected for v1.

## L

**libsecp256k1.** Bitcoin Core's reference C implementation of secp256k1. The `secp256k1` Rust crate wraps it. Constant-time, side-channel-resistant.

**Lockstep versioning.** Every binding is published at the same version from the same commit. ADR D8.

## M

**Maven Central.** The standard Java/Kotlin package registry. We publish `io.jovachain:jova-core` there.

**Mnemonic.** A BIP-39 phrase. 12 or 24 English words encoding a seed.

**`MnemonicBuffer`.** Byte-array variant of `Mnemonic` for apps that want to control the buffer's lifetime and clear it explicitly.

**Monorepo.** One Git repo containing multiple related projects (in our case: Rust workspace + every binding's packaging). ADR D2.

## N

**`no_std`.** A Rust build flag indicating the crate doesn't use the `std` crate. `jova-core-primitives` is `no_std`-clean. Required for hardware-wallet firmware.

## P

**P2PKH.** Pay-to-Public-Key-Hash, legacy Bitcoin script type. `1…` addresses.

**P2WPKH.** Pay-to-Witness-Public-Key-Hash, native SegWit. `bc1q…` addresses (BIP-84).

**P2TR.** Pay-to-Taproot. `bc1p…` addresses (BIP-86).

**Parity.** Two implementations producing identical output for identical input. The fundamental property the test vectors enforce across bindings.

**PBKDF2.** The KDF used to turn a mnemonic + passphrase into a seed (`bip39::Mnemonic::to_seed`). HMAC-SHA512 with 2048 iterations.

**PRF (WebAuthn).** A WebAuthn extension that returns a stable per-credential secret. Used for browser-side mnemonic encryption.

**Primitives.** The crypto building blocks (`jova-core-primitives`). `no_std`-clean. Hardware-wallet target.

**Property test.** Test that asserts an invariant over many randomly generated inputs. Run via `proptest`.

**PSBT.** See BIP-174.

## R

**RLP.** Recursive Length Prefix encoding, used by Ethereum to serialize transactions and other structured data.

**RUSTSEC.** The Rust ecosystem's security advisory database. `cargo-audit` checks against it daily.

## S

**Satellite repo.** `jovachain/jovawallet-core-swift`. Auto-generated SwiftPM package repo populated by CI on every release. ADR D6.

**Seed.** The 64-byte output of PBKDF2 over a mnemonic + passphrase. Held inside `JovaWalletInner` for the wallet's lifetime; zeroized on drop.

**secp256k1.** The elliptic curve used by Bitcoin, Ethereum, and XRP for signing. Implemented by the `secp256k1` crate (Bitcoin Core's libsecp256k1) and `k256` (pure Rust).

**SegWit.** "Segregated witness" — a Bitcoin upgrade enabling smaller, cheaper transactions. Native SegWit (BIP-84) is the SDK default for BTC.

**SLIP-10.** The ed25519 analog of BIP-32. Hardened-only derivation; used for Solana.

**Solana split crates.** Anza's first-party Rust crates for Solana, used in `jova-core-chains::sol`: `solana-keypair`, `solana-transaction`, `solana-message`, `solana-pubkey`, `solana-signature`. Replaced the monolithic `solana-sdk` because the split versions have a much smaller dep tree and are WASM-viable.

**Spec.** `spec/` directory in the repo. The authoritative source of correctness — vectors, API frozen-copy, error taxonomy. If code disagrees with spec, code is wrong.

**Static class methods.** `JovaWallet.createMnemonic`, `JovaWallet.isValidMnemonic`, `JovaWallet.isValidAddress`. Don't require a wallet instance.

**`Strength`.** Enum for mnemonic length: `bits128` (12 words) or `bits256` (24 words).

**Supply chain.** The dependency tree. We control it via pinning, `cargo-vet`, `cargo-deny`, `cargo-audit`, and license whitelisting.

**SwiftPM (Swift Package Manager).** Apple's package manager. Our canonical iOS/macOS distribution.

## T

**Test vector.** A `(input, expected_output)` pair encoded in `spec/test-vectors.json`. The correctness oracle.

**TWC (Trust Wallet Core).** A C++ multi-chain crypto library (mid-migration to Rust). Considered as the engine in an earlier draft of ADR D1; rejected. We use pure-Rust crates instead.

## U

**`uniffi-rs`.** Mozilla's tool for generating idiomatic Swift / Kotlin / Python / Ruby bindings from a Rust crate. Used in `crates/jova-core-ffi`.

**`UnsignedTx`.** The discriminated union of unsigned transaction shapes per chain family. Apps construct from backend payloads; the SDK signs.

## V

**`v0` versioned tx (Solana).** Solana's transaction format that supports ALTs. The format we sign.

**Vector-first.** Adding or changing behavior starts with adding or changing a vector. ADR-style design principle. See `architecture.md`.

## W

**WalletConnect.** A protocol for connecting wallets to dApps. Apps speak it; the SDK does not. Once an app extracts an `UnsignedTx` from a WalletConnect request, it hands it to `wallet.signTx(unsigned:)`.

**`wasm-bindgen`.** The Rust → JavaScript binding generator. Used in `crates/jova-core-wasm`.

**WASM (WebAssembly).** Sandboxed bytecode runtime. Our browser binding compiles `jova-core-wasm` to WASM.

## X

**XCFramework.** Apple's binary distribution format combining multiple architectures and platforms. We build one containing iOS device, iOS simulator, and macOS slices.

**XRPL.** The XRP Ledger. The chain XRP transactions broadcast to.

**XRP `r…` address.** Base58check-encoded address for XRP. Always starts with `r`.

**`XPrv` / `XPub`.** Extended private / public key in BIP-32. `XPrv` includes the chain code; the SDK never serializes `XPrv` to a string (no `xprv...` ever leaves Rust memory).

## Z

**`Zeroize` / `Zeroizing`.** Rust crate that overwrites memory on drop, with `volatile` semantics so the compiler can't elide the write. Every secret-bearing type in `jova-core-primitives` and `jova-core` uses it. See `memory-and-keys.md`.

**Zero-state SDK.** `JovaWallet` holds no global state, no cache, no logger, no async runtime. ADR D10 and `architecture.md`.
