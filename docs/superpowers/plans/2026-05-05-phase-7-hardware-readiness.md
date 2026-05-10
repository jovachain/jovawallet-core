# Phase 7: Hardware-Wallet Readiness (Process Plan)

> **Status:** Process plan. Phase 7 is gated on having actual hardware to test against, which is a hardware-team deliverable not a software one. The SDK-side work is small; the firmware-side work belongs in a separate firmware repo.

> **For agentic workers:** Most of this plan is about preparing the SDK for hardware integration, not about building hardware-wallet firmware. The firmware itself is a different codebase entirely.

**Goal:** A reference Cortex-M firmware integrating `jova-core-primitives` exists and signs against the same vectors as the phone bindings. Tag `v1.2.0`.

**Preconditions:**
- `v1.1.0` shipped (WASM done).
- Hardware platform decided (STM32 dev board, BitBox-style ATSAMD51, etc.).
- Hardware team exists.

**Exit criteria:**
- `external-rng` feature on `jova-core-primitives` ships.
- `JovaWallet::from_seed_bytes(...)` exists in the Rust direct API (not exposed via FFI/WASM — firmware-only).
- `examples/firmware-template/` builds for `thumbv7em-none-eabihf` and signs an EVM digest correctly.
- `docs/integration-hardware.md` updated with production-grade integration patterns.
- `v1.2.0` tagged.

---

## SDK-side tasks (in this repo)

| # | Task | Roughly |
|---|---|---|
| 1 | Add `external-rng` feature to `jova-core-primitives`. When enabled, `Mnemonic::generate_with(strength, &mut impl JovaRng)` is callable. The default `getrandom`-based path is gated behind a non-default feature. | 2 days |
| 2 | Add `JovaWallet::from_seed_bytes(bytes: &[u8])` constructor in `jova-core` (not exposed via FFI). For hardware that already has the seed in a secure element. | 1 day |
| 3 | Document the `JovaRng` trait, examples for STM32 TRNG, Renesas TRH4, and software-PRNG-from-secure-element. | 1 day |
| 4 | Verify `jova-core-primitives` builds for `thumbv7em-none-eabihf` with `--no-default-features --features external-rng,alloc`. CI workflow `ci-no-std.yml` extended to test this configuration. | 1 day |
| 5 | Create `examples/firmware-template/`: a minimal `thumbv7em` Cargo project linking `jova-core-primitives`, demonstrating: deriving an EVM key from a hard-coded test seed, signing a digest with `secp256k1`, returning the signature. Builds in CI; no real hardware needed. | 5 days |
| 6 | Side-channel and glitch-protection guidance in `docs/integration-hardware.md`: reference patterns for ATECC608, OPTIGA Trust M, and software-only mitigations. | 3 days |
| 7 | Run `cargo miri test -p jova-core-primitives --features external-rng` to validate no UB across the new code paths. | 1 day |
| 8 | Vector parity: a hardware-side test runner reads `spec/test-vectors.json` and signs each vector using the firmware code path; compares to the canonical signed_hex. (Note: this requires the firmware repo to exist; it can be its own subagent task in the firmware repo.) | 2 days |
| 9 | Tag `v1.2.0`. | 0.5 day |

**Total in this repo: ~2 weeks.**

## Firmware-side work (separate repo, NOT this plan's scope)

For context only — the firmware repo (e.g., `jovachain/jova-firmware-reference`) does:

- Picks a hardware platform.
- Implements the secure-element protocol for seed storage.
- Wires `jova-core-primitives` into the firmware crate.
- Implements display + user-confirmation UI for transaction details.
- Implements glitch-detection, voltage monitoring, retry-on-mismatch.
- Implements the host-protocol layer (USB / BLE) the phone app speaks.
- Runs vector parity against `spec/test-vectors.json`.
- Has its own `v1.0` release independently of the SDK.

This is months of hardware-engineering work. Phase 7 in the SDK repo just makes sure we don't block that work — we provide a clean, tested, no_std primitives layer that the firmware repo imports as a Cargo dep.

---

## Risks

- **The hardware team doesn't exist yet.** If so, Phase 7 in the SDK is preparatory only — we publish `v1.2.0` with the `external-rng` feature and the firmware-template example, and wait. The SDK is hardware-ready; the hardware just isn't built.
- **A primitive crate accidentally pulls in `std`.** `ci-no-std.yml` should catch this on every PR. If it doesn't (it should), Phase 7's first job is fixing the CI to actually do.
- **`secp256k1`'s low-memory mode** is required for firmware with <512 KB RAM. We already have the `lowmemory` feature in `[workspace.dependencies]`; verify it actually gets compiled in for the firmware target.
- **Glitch-protection guidance is hard to write generically.** The `docs/integration-hardware.md` should reference specific reference platforms (Foundation Devices Passport, BitBox02, Trezor) and explicitly say "consult your secure element vendor's white papers; this guide is starting points, not exhaustive."

---

## What this plan does NOT do

- Does not build hardware. Hardware engineering is separate.
- Does not certify firmware to FIPS or CC. Certification is a 6+ month process with its own budget.
- Does not change the public API of `jova-core` or any binding. `from_seed_bytes` is a Rust-only addition; FFI stays at the `from_mnemonic` shape.
- Does not add hardware-specific transaction display formatting — that's firmware's job.

## Estimated time

2 weeks of SDK-team work in this repo. Firmware integration is a separate timeline that this plan deliberately doesn't predict.
