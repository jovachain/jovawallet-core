# Testing Strategy

The full test pyramid for `jovawallet-core`. Every layer below is enforced in CI; a PR that fails any of them does not merge.

## Why testing matters more here than in most repos

This is a signing SDK. A bug doesn't crash an app — it produces a different signature, which broadcasts to a public chain, which loses real money irrevocably. Every test exists to make that outcome impossible.

There are three distinct correctness questions the test suite must answer:

1. **Does the SDK sign correctly?** — vector tests against known-good outputs, usually copied from BIPs / EIPs / chain reference implementations.
2. **Does every binding behave identically?** — every binding loads the same `spec/test-vectors.json` and must produce byte-identical output.
3. **Does the SDK handle malformed input safely?** — fuzzing, property tests, malformed-input integration tests.

---

## Layer 1: vector tests (the correctness oracle)

`spec/test-vectors.json` is the single source of correctness truth. Every binding loads it and runs the same suite.

### Vector file shape

```json
{
  "version": "1.0",
  "vectors": [
    {
      "id": "btc.address.bip84_account0_index0",
      "kind": "address",
      "input": {
        "mnemonic": "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        "passphrase": "",
        "chain": "bitcoin",
        "account": 0
      },
      "expected": {
        "address": "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu"
      },
      "source": "BIP-84 official test vectors",
      "source_url": "https://github.com/bitcoin/bips/blob/master/bip-0084.mediawiki"
    },
    {
      "id": "evm.tx.eip1559_simple_transfer",
      "kind": "sign_tx",
      "input": {
        "mnemonic": "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        "passphrase": "",
        "unsigned_tx": {
          "kind": "evm",
          "chainId": 1,
          "nonce": 0,
          "to": "0x0000000000000000000000000000000000000000",
          "value": "1000000000000000000",
          "gasLimit": 21000,
          "maxFeePerGas": "30000000000",
          "maxPriorityFeePerGas": "2000000000",
          "data": "0x"
        }
      },
      "expected": {
        "signed_hex": "0x02f873018084773594008506fc23ac008252089400000000000000000000000000000000000000000880de0b6b3a76400008080c001a0...",
        "tx_hash": "0xabc...123"
      },
      "source": "captured from local geth signing of identical input",
      "source_url": "internal"
    }
  ]
}
```

### Vector kinds

- `address` — derive an address; expected `address` string.
- `sign_tx` — sign a tx; expected `signed_hex` and `tx_hash`.
- `sign_message` — sign a message; expected `signature_hex`.
- `validate_address_pos` — `isValidAddress` returns `true` for given input.
- `validate_address_neg` — `isValidAddress` returns `false` for given input.
- `mnemonic_validation_pos` / `mnemonic_validation_neg` — `isValidMnemonic` results.
- `error` — operation must fail with the expected `JovaError` variant and `reason`.

### Coverage requirements per chain

Before a chain is "supported," it must have:

- **Three** `address` vectors (account 0, 1, 5, with two different mnemonics).
- **Two** `sign_tx` vectors covering distinct scenarios (simple transfer + something chain-specific: PSBT multi-input, ALT-using SOL tx, EIP-712 typed data, XRP with destination tag).
- **One** `sign_message` vector per supported scheme on the chain.
- **At least three** `error` vectors covering the most likely malformed inputs (`invalid_address`, `chainid_mismatch`, `invalid_base64`, etc.).
- **Address-validation positives and negatives.**

A new-chain PR that doesn't include this fails CI on the spec validator (`tools/verify-spec`).

### Sources for vectors

In priority order:

1. **Official BIP / EIP test vectors.** BIP-39, BIP-84 official vectors. EIP-712 reference. EIP-1559 reference.
2. **Reference-implementation captures.** Local `geth` signing for EVM. `bdk` test vectors for BTC. `solana-cli sign-only` for SOL. `xrpl-cli` for XRP.
3. **Differential captures.** Sign the same input with TWC (Swift) and a pure-Rust impl (Rust) — if they agree, both vectors are credible.
4. **Production captures from the existing apps' known-good behavior.** Specifically scoped: only behaviors we want to keep. Documented as such.

### What's *not* a vector source

- The SDK's own output. Vectors must come from outside the SDK. Otherwise we're testing that we agree with ourselves.
- Hand-written hex by an engineer. If you can't get the value from a reference impl, you don't have a credible vector.

---

## Layer 2: per-binding parity

Every binding has a `VectorsTests` suite that:

1. Loads `spec/test-vectors.json` via the binding's local file-loading idiom.
2. For each vector, executes the corresponding SDK call.
3. Asserts byte-identical output (or expected error).

If Swift produces `0x...abc` and Kotlin produces `0x...abd` for the same vector, both jobs fail and a PR cannot merge.

### Implementation per binding

| Binding | Test framework | Vector loading | Path |
|---|---|---|---|
| Rust | `cargo test` | `serde_json` | `crates/jova-core/tests/vectors.rs` |
| Swift | `XCTest` | `Bundle.module + JSONDecoder` | `bindings/swift/Tests/JovaCoreTests/VectorsTests.swift` |
| Kotlin | `JUnit5` + `kotlinx.serialization` | classpath resource | `bindings/kotlin/jova-core/src/test/kotlin/io/jova/core/VectorsTest.kt` |
| WASM | `vitest` | `import vectors from '../../spec/test-vectors.json'` | `bindings/wasm/tests/vectors.test.ts` |

The vector file is included in each binding's package via build-time copy or symlink — never duplicated by hand.

### Selective per-binding skip

Some vectors are valid only on certain bindings (e.g., a vector that tests behavior of `MnemonicBuffer` is meaningless in the Rust direct API where there's nothing to clear). A vector entry can declare `"applies_to": ["swift", "kotlin", "wasm"]`. Default is "all."

---

## Layer 3: property-based tests

`proptest` (Rust) generates randomized inputs and asserts invariants. Run on every push as part of `cargo test`.

### Properties tested

- **Address determinism**: for any mnemonic + chain + account, two calls return the same address.
- **Address ↔ pubkey**: derived address matches `chain_specific_encode(derive_pubkey(seed, path))`.
- **PSBT round-trip**: `parse(serialize(psbt))` equals the original PSBT structurally (BTC).
- **EIP-1559 RLP round-trip**: encoded tx decodes to the same fields (EVM).
- **Mnemonic round-trip**: `from_words(to_words(seed))` over the wordlist preserves entropy bits.
- **Sign + verify**: every signed tx, when re-parsed and verified against the derived public key, validates.
- **Validate-then-derive**: `isValidAddress(derive(seed, chain), chain)` is always `true`.

### Where they live

`crates/jova-core/tests/properties/` — one file per property family. Run with `cargo test --test properties`. Run for 256 cases per property by default; 4096 cases on the nightly fuzz job.

---

## Layer 4: fuzz harnesses

`cargo-fuzz` (libFuzzer underneath) targets every parser entry point. Fuzz nightly for 30 minutes per target. New crashes auto-file an issue.

### Targets

```
fuzz/fuzz_targets/
├── fuzz_psbt_sign.rs            random bytes → BTC sign; assert no panic
├── fuzz_eip1559_decode.rs       random bytes → EVM tx parse
├── fuzz_eip712_typed.rs         random JSON → EIP-712 hash
├── fuzz_sol_versioned_tx.rs     random bytes → SOL message parse
├── fuzz_xrp_canonical.rs        random JSON → XRP serialize
├── fuzz_mnemonic_parse.rs       random Unicode → mnemonic validate
├── fuzz_path_parse.rs           random strings → derivation path parse
└── fuzz_address_parse.rs        random strings → address validate per chain
```

### What a fuzz target asserts

The targets do **not** assert correctness (no oracle for random input). They assert:

1. **No panic.** A `panic!` in the SDK is a bug.
2. **No abort.** No `std::process::abort` reachable.
3. **No infinite loop.** Fuzz harness times out at 5 seconds per case; longer is a bug.
4. **No memory leak.** Run with `address-sanitizer` periodically.
5. **No `unsafe` UB.** Run on `cargo-fuzz`'s ASAN+UBSAN build.

### Differential fuzzing

For chains where we have a credible reference implementation locally:

- **EVM**: differential against `alloy` direct (we *use* alloy, so this is round-tripping; useful as a sanity check).
- **BTC**: differential against `rust-bitcoin` directly.
- **SOL**: differential against `solana-cli sign-only`.
- **XRP**: differential against the official Python `xrpl-py` library when feasible.

Differential fuzz targets live in `fuzz/fuzz_targets/diff_*.rs`. They sign the same input two ways and assert byte-identical output. Run weekly, not nightly.

---

## Layer 5: API surface tests

Verify every documented method exists with the documented signature. Ensures we haven't accidentally removed or renamed something.

```
bindings/swift/Tests/JovaCoreTests/ApiSurfaceTests.swift
bindings/kotlin/jova-core/src/test/kotlin/io/jova/core/ApiSurfaceTest.kt
bindings/wasm/tests/api-surface.test.ts
crates/jova-core/tests/api_surface.rs
```

These tests reference every public method and would fail to compile if a method were removed or renamed. They serve as compiler-checked documentation of the surface.

---

## Layer 6: error-mapping tests

Ensure every `JovaError` variant maps correctly through every binding.

```
crates/jova-core/tests/errors.rs           every variant: construct → display → debug
bindings/*/...ErrorMappingTests/           every variant: trigger → catch → variant equal
```

Triggers come from intentionally-malformed vectors in `spec/test-vectors.json` of `kind: error`.

---

## Layer 7: memory tests

Verify the secret-clearing contract from `memory-and-keys.md`.

```
crates/jova-core/tests/memory.rs           drop → underlying memory zeroed
bindings/swift/Tests/.../MemoryTests.swift
bindings/kotlin/.../MemoryTest.kt
bindings/wasm/tests/memory.test.ts         destroy() → linear-memory region zeroed
```

Verification techniques:

- **Rust**: read the byte buffer from a raw pointer after drop, confirm zero. Done via `mprotect` to detect use-after-zero.
- **Swift / Kotlin / JS**: best-effort — verify the SDK's `clear()` was called by hooking the FFI boundary in tests.

`miri` is run nightly on `jova-core-primitives` to catch UB, double-frees, and use-after-free.

---

## Layer 8: no-std build

`jova-core-primitives` is built for `thumbv7em-none-eabihf` in `ci-no-std.yml`. If anything in the primitives crate accidentally pulls in `std`, the build fails. This protects the firmware-readiness contract from regressing.

---

## Layer 9: integration smoke tests

The four sample apps under `examples/` each have a smoke test that:

1. Builds the binding from local source (not from the published artifact).
2. Constructs a `JovaWallet` from a known-good mnemonic.
3. Derives one address per supported chain.
4. Signs one transaction per chain.
5. Asserts results match `spec/test-vectors.json`.

Run on every PR. They are slower than unit tests but catch packaging-level mistakes that unit tests can't (e.g., a missing JNI symbol in the Android AAR).

---

## Continuous fuzzing — clusterfuzz-lite

`nightly-fuzz.yml` uses GitHub's `actions/clusterfuzz-lite` to:

- Run all fuzz targets for 30 minutes each.
- Carry over corpora between runs (the corpus accumulates in a separate `jovawallet-core-fuzz-corpus` repo).
- File an issue on each new crash with the minimized reproducer attached.

Crashes block the next release until triaged.

---

## Audit-grade reproducibility

For audits and incident response:

- `Cargo.lock` is committed. `cargo build --locked` reproduces the exact dependency tree.
- Every release artifact's checksum is published to the GitHub release page.
- The build is reproducible — given the same `Cargo.lock` and the same compiler version (pinned in `rust-toolchain.toml`), `cargo build --release --locked` produces identical bytes.
- `cargo-vet` audits are committed to `supply-chain/` (one-line approval per vetted dependency).
- `cargo-deny` enforces license whitelist (MIT, Apache-2.0, BSD-3, ISC) and rejects RUSTSEC-known-vulnerable versions.

---

## What we don't test

- **Correctness of the underlying crates** (`secp256k1`, `bdk_wallet`, `alloy`, etc.). They have their own test suites and audits. We assume they're correct and validate that *we use them correctly* via vectors.
- **The Rust compiler.** Same rationale.
- **Operating system primitives** (file I/O, network) — we don't use them.

This is the boundary between "SDK testing" and "Rust ecosystem testing." We don't reinvent it.

---

## A new chain's testing checklist

Before merging a new chain:

1. ✅ Three `address` vectors.
2. ✅ Two `sign_tx` vectors (simple + chain-specific edge case).
3. ✅ One `sign_message` vector per scheme.
4. ✅ Three `error` vectors covering common malformed inputs.
5. ✅ Property tests in `crates/jova-core/tests/properties/<chain>.rs`.
6. ✅ Fuzz target in `fuzz/fuzz_targets/fuzz_<chain>_*.rs`.
7. ✅ All vectors load and pass on Rust + Swift + Kotlin. WASM compile smoke is required from day one; WASM functional vector parity is required from v1.1.0 onward (a chain landing pre-v1.1.0 may have its WASM functional vectors deferred but **not** its compile smoke).
8. ✅ Documentation updates in `chains.md`, `api.md` (if `UnsignedTx` grew).
9. ✅ Sample app updated to demo the new chain.

PRs that don't tick all nine fail review.
