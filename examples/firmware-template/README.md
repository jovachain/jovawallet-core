# Firmware template

Reference Cortex-M (`thumbv7em-none-eabihf`) firmware demonstrating `jova-core-primitives` integration on bare metal. Phase 7 deliverable.

## What this template proves

1. `jova-core-primitives` builds cleanly for `thumbv7em-none-eabihf` with `default-features = false, features = ["external-rng"]`.
2. The full BIP-39 → seed → BIP-44 secp256k1 derivation pipeline runs in `no_std`.
3. ECDSA signing on Cortex-M works via `secp256k1 = { default-features = false, features = ["alloc", "lowmemory"] }`.
4. Final stripped ELF is ~394 KB — fits any STM32F4 / nRF52840 / SAMD51 flash with room to spare.

## What this template does NOT do

This is a **reference scaffolding**, not production firmware. Real firmware adds:

- Hardware TRNG wired into `JovaRng` (this template hardcodes the BIP-39 test mnemonic for hermetic CI).
- Secure-element protocol for seed storage (ATECC608 / OPTIGA Trust M / SE050).
- Glitch-detection, voltage monitoring, retry-on-mismatch.
- Display + user-confirmation UI for transaction details.
- Host-protocol layer (USB / BLE / NFC) the phone speaks.
- Vector parity against `spec/test-vectors.json`.

See [`docs/integration-hardware.md`](../../docs/integration-hardware.md) for the full integration guide.

## Build

```bash
cd examples/firmware-template
cargo build --target thumbv7em-none-eabihf --release
```

Artifact: `target/thumbv7em-none-eabihf/release/jova-firmware-template`.

CI builds this on every PR via `.github/workflows/ci-no-std.yml`.

## Memory layout

`memory.x` declares a generic 512 KB flash / 128 KB RAM map sized to fit any common Cortex-M4 dev board. Production firmware replaces it with the exact platform's memory map (consult the chip datasheet).

## Crate-level invariants

- `#![no_std]`, `#![no_main]` — bare metal.
- `#![deny(unsafe_code)]` — every module except `heap_init` is unsafe-free. `heap_init` carries one localized `unsafe` block for `embedded_alloc::Heap::init`, which is required by the global allocator's API.
- `panic = "abort"` in release profile — no unwinding through FFI.
- `lto = "fat"`, `opt-level = "z"`, `codegen-units = 1`, `strip = "symbols"` — flash budget minimization.
