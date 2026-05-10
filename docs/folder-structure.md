# Folder Structure

The complete repo layout, file by file. This is the target structure that Phase 0 of [`plan.md`](./plan.md) builds out.

## Top level

```
jovawallet-core/
├── README.md                       1-page overview, links into docs/
├── LICENSE                         MIT
├── CHANGELOG.md                    keepachangelog.com format, edited by humans
├── CODEOWNERS                      paths → reviewer teams
├── CONTRIBUTING.md                 PR/issue/test-vector contribution rules
├── SECURITY.md                     vulnerability disclosure policy
├── rust-toolchain.toml             pinned stable Rust + components
├── Cargo.toml                      workspace root + [workspace.dependencies]
├── Cargo.lock                      committed; required for reproducible builds
├── deny.toml                       cargo-deny config: licensing, advisories, layered deps
├── .gitignore
├── .gitattributes                  text=auto, lockfile diff hints
├── .editorconfig
├── crates/                         the Rust core — see "Crates" below
├── bindings/                       per-platform packaging — see "Bindings" below
├── spec/                           authoritative correctness oracle — see "Spec" below
├── docs/                           internal documentation (you are here)
├── examples/                       runnable sample apps per binding
├── fuzz/                           cargo-fuzz harnesses
├── tools/                          dev tooling: vector generators, lint scripts
└── .github/
    ├── workflows/                  CI pipelines
    ├── ISSUE_TEMPLATE/
    └── PULL_REQUEST_TEMPLATE.md
```

## Crates

```
crates/
├── jova-core-primitives/
│   ├── Cargo.toml
│   ├── README.md                   no_std, what's in here, what's not
│   └── src/
│       ├── lib.rs                  pub use re-exports + #![no_std]
│       ├── mnemonic.rs             Mnemonic, Strength, generate, validate, to_seed
│       ├── seed.rs                 Seed (Zeroizing<[u8; 64]>)
│       ├── path.rs                 DerivationPath, parsing, BIP-44/84/49/86 helpers
│       ├── derive.rs               BIP-32 (secp256k1) and SLIP-10 (ed25519) derivation
│       ├── keys.rs                 XPrv, XPub, PrivateKey, PublicKey
│       ├── signature.rs            raw signing primitives (r,s,v) + ed25519
│       ├── hashes.rs               Sha256, Sha512, Keccak256, Ripemd160 wrappers
│       ├── encoding.rs             hex, base58, base58check, bech32 (no_std-safe)
│       └── zeroize_ext.rs          extension impls for the workspace types
│
├── jova-core-chains/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs                  pub use signer trait + dispatch
│       ├── signer.rs               ChainSigner trait (see decisions.md D12)
│       ├── unsigned_tx.rs          UnsignedTx enum + per-variant payload structs
│       ├── signable_message.rs     SignableMessage enum
│       ├── address.rs              Address newtype + per-chain canonical form helpers
│       ├── btc/
│       │   ├── mod.rs              BtcSigner impl ChainSigner
│       │   ├── address.rs          P2WPKH derivation, bech32 encoding, validation
│       │   ├── psbt.rs             PSBT parse, sign-all, finalize
│       │   └── message.rs          BIP-322 + legacy signMessage
│       ├── evm/
│       │   ├── mod.rs              EvmSigner impl ChainSigner (parameterized by chainId)
│       │   ├── address.rs          keccak256 → EIP-55 checksum
│       │   ├── tx.rs               EIP-1559 type-2 tx encoding + signing
│       │   ├── eip191.rs           personal_sign
│       │   └── eip712.rs           typed-data v4
│       ├── sol/
│       │   ├── mod.rs              SolSigner impl ChainSigner
│       │   ├── address.rs          ed25519 pubkey → base58
│       │   ├── tx.rs               VersionedTransaction (v0) signing
│       │   └── message.rs          raw ed25519 over UTF-8 bytes
│       └── xrp/
│           ├── mod.rs              XrpSigner impl ChainSigner
│           ├── address.rs          base58check r-prefix
│           └── tx.rs               canonical XRPL serialize + sign
│
├── jova-core/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs                  pub re-exports; the public Rust API
│       ├── wallet.rs               JovaWallet struct, from_mnemonic, address, sign_*
│       ├── chain.rs                JovaChain enum
│       ├── error.rs                JovaError thiserror enum
│       └── prelude.rs              recommended `use jova_core::prelude::*;`
│
├── jova-core-ffi/
│   ├── Cargo.toml                  cdylib + staticlib crate-type
│   ├── README.md
│   ├── build.rs                    runs uniffi-bindgen generators
│   ├── uniffi.toml                 binding-language tuning
│   └── src/
│       ├── lib.rs                  #[uniffi::export] surface; re-exports jova-core
│       ├── jova_core.udl           OR proc-macro-only, see decisions
│       ├── error_map.rs            thiserror → uniffi error variants
│       └── ffi_types.rs            BigInteger, byte-array newtypes
│
└── jova-core-wasm/
    ├── Cargo.toml                  cdylib for wasm32-unknown-unknown
    ├── README.md
    └── src/
        ├── lib.rs                  #[wasm_bindgen] surface; JSON in / JSON out
        ├── error_map.rs            thiserror → JS Error
        └── types.rs                serde-wasm-bindgen helpers
```

## Bindings

These directories package the Rust artifacts into language-native consumer formats. None of them contain crypto code — only thin wrappers, build scripts, and tests.

```
bindings/
├── swift/
│   ├── Package.swift               only used in CI for build-from-source flow
│   ├── Sources/
│   │   ├── JovaCore/
│   │   │   ├── JovaCore.swift          uniffi-generated; do not edit
│   │   │   ├── Convenience.swift       hand-written ergonomics: typealiases,
│   │   │   │                          static helpers wrapping the generated API
│   │   │   └── Module.modulemap        generated header gluing
│   │   └── JovaCoreFFI/                XCFramework binary target placeholder in dev;
│   │                                  replaced with built artifact at release time
│   ├── Tests/
│   │   └── JovaCoreTests/
│   │       ├── VectorsTests.swift      loads ../../../spec/test-vectors.json
│   │       ├── ApiSurfaceTests.swift   ensures every documented method exists
│   │       ├── ErrorMappingTests.swift
│   │       └── MemoryTests.swift       deinit clears handle
│   ├── scripts/
│   │   └── build-xcframework.sh        used by CI; builds 5 targets + lipo
│   └── README.md                       SwiftPM consumption snippet
│
├── kotlin/
│   ├── settings.gradle.kts
│   ├── build.gradle.kts                root build, version pin
│   ├── jova-core/
│   │   ├── build.gradle.kts            module: AAR + JVM jar publish
│   │   └── src/
│   │       ├── main/
│   │       │   ├── kotlin/io/jova/core/
│   │       │   │   ├── JovaCore.kt         uniffi-generated; do not edit
│   │       │   │   ├── Convenience.kt      hand-written ergonomics
│   │       │   │   └── package-info.kt
│   │       │   └── jniLibs/                 cargo-ndk drop targets
│   │       │       ├── arm64-v8a/libjova_core_ffi.so
│   │       │       ├── armeabi-v7a/libjova_core_ffi.so
│   │       │       ├── x86_64/libjova_core_ffi.so
│   │       │       └── x86/libjova_core_ffi.so
│   │       └── test/kotlin/io/jova/core/
│   │           ├── VectorsTest.kt
│   │           ├── ApiSurfaceTest.kt
│   │           ├── ErrorMappingTest.kt
│   │           └── MemoryTest.kt           AutoCloseable behavior
│   ├── scripts/
│   │   └── build-aar.sh
│   └── README.md
│
└── wasm/
    ├── package.json                    @jovachain/wallet-core
    ├── tsconfig.json
    ├── pkg/                            wasm-pack build output, gitignored
    ├── src/
    │   ├── index.ts                    re-exports + TS-friendly facade
    │   └── types.ts                    TypeScript type definitions
    ├── tests/
    │   ├── vectors.test.ts             loads ../../spec/test-vectors.json
    │   ├── api-surface.test.ts
    │   └── error-mapping.test.ts
    ├── scripts/
    │   └── build-wasm.sh               wasm-pack + esbuild bundle
    └── README.md
```

## Spec

The single most important folder in the repo. Every binding's tests load from here.

```
spec/
├── api.md                          frozen copy of docs/api.md per release
├── chains.md                       frozen copy of docs/chains.md per release
├── errors.md                       frozen JovaError taxonomy
├── test-vectors.json               THE correctness oracle
├── test-vectors.schema.json        JSON Schema; CI validates the vector file
├── test-vectors/
│   ├── README.md                   how to author and add a vector
│   └── sources/
│       ├── btc/                    raw artifacts: PSBTs, expected hex, source URLs
│       ├── evm/                    raw RLP-encoded txs, EIP-712 typed data files
│       ├── sol/                    raw v0 messages, base64
│       └── xrp/                    raw canonical JSON
└── CHANGELOG.md                    spec-only changelog; correctness changes documented
```

`test-vectors.json` is read by:

- `crates/jova-core/tests/vectors.rs` — Rust integration tests.
- `bindings/swift/Tests/JovaCoreTests/VectorsTests.swift`
- `bindings/kotlin/jova-core/src/test/kotlin/.../VectorsTest.kt`
- `bindings/wasm/tests/vectors.test.ts`

If any binding produces a different result than the vector specifies, that binding's CI job fails and merge is blocked.

## Examples

Runnable sample applications. These are not published; they exist so contributors can verify the integration story works end-to-end and so the documentation can point to real code.

```
examples/
├── README.md                       what each example does, how to run it
├── ios-sample/                     SwiftUI app: enter mnemonic → show address → sign demo tx
│   ├── Sample.xcodeproj
│   └── Sources/
├── android-sample/                 Compose app: same flow as iOS
│   ├── build.gradle.kts
│   └── app/src/main/
├── web-sample/                     Vite + TypeScript: same flow in a browser
│   ├── package.json
│   └── src/
├── rust-cli/                       jova-cli: address, sign, validate via stdin/stdout
│   └── src/main.rs
└── backend-node/                   Express demo of the WASM build verifying signatures
    └── src/server.ts
```

## Fuzz harnesses

```
fuzz/
├── Cargo.toml                      cargo-fuzz workspace
└── fuzz_targets/
    ├── fuzz_psbt_sign.rs           random bytes → BTC sign; assert no panic
    ├── fuzz_eip1559_decode.rs
    ├── fuzz_eip712_typed.rs
    ├── fuzz_sol_versioned_tx.rs
    ├── fuzz_xrp_canonical.rs
    └── fuzz_mnemonic_parse.rs
```

Run nightly via `nightly-fuzz.yml` for 30 minutes per target. New crashes file an issue automatically.

## Tools

```
tools/
├── README.md
├── gen-vector/                     generate a test vector from a known signing operation
│   └── src/main.rs                 reads a Rust expression, emits the JSON entry
├── verify-spec/                    fail CI if docs/api.md and spec/api.md disagree
│   └── src/main.rs
├── audit-deps/                     wraps cargo-vet + cargo-deny + cargo-audit
│   └── audit.sh
└── release/                        release-time helpers
    ├── tag.sh                      validates lockstep tag preconditions
    └── publish.sh                  invoked by CI release workflow
```

## CI workflows

```
.github/workflows/
├── ci.yml                          push/PR: cargo fmt + clippy + test on Linux+macOS+Windows
├── ci-bindings-swift.yml           push/PR: build XCFramework, run JovaCoreTests
├── ci-bindings-kotlin.yml          push/PR: cargo-ndk × 4 ABIs, build AAR, run JUnit
├── ci-bindings-wasm.yml            push/PR: wasm-pack, run vitest
├── ci-no-std.yml                   push/PR: build jova-core-primitives for thumbv7em
├── nightly-fuzz.yml                cron 02:00 UTC: cargo-fuzz 30 min per target
├── nightly-miri.yml                cron 03:00 UTC: cargo miri test on jova-core-primitives
├── audit.yml                       push/PR + cron daily: cargo audit + cargo deny + cargo vet
└── release.yml                     on tag: publish crates.io + push satellite Swift repo
                                    + Maven Central + npm
```

`build-and-release.md` describes each workflow in detail.

## Naming conventions

- **Crate names** use kebab-case prefixed with `jova-core-`. Example: `jova-core-primitives`.
- **Rust modules** use snake_case. Example: `chains::btc::psbt`.
- **Public types** use PascalCase: `JovaWallet`, `UnsignedTx`, `SignedTx`.
- **Generated Swift module** is `JovaCore`. Generated Kotlin package is `io.jova.core`. npm package is `@jovachain/wallet-core`.
- **Test vectors** are keyed by `chain.operation.scenario` — e.g. `btc.psbt_sign.bip84_single_input`.

## What's *not* in the repo

- App code (iOS / Android / web wallets) — those are separate repos that depend on this one.
- Backend services (`jova-rpc`, `jova-fees`, future) — separate repos.
- TWC, web3j, bitcoinj, bdk-android, or any other engine the apps used historically. Those become uninstalled dependencies once Phase 3/4 of the plan completes.
