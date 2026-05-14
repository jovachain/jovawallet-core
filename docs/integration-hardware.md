# Integration: Hardware Wallet Firmware

How a hardware wallet's firmware uses `jovawallet-core`. Phase 7 of the plan; this document is forward-looking.

The firmware target is an embedded device — typically Cortex-M4 or M33 — with no operating system, a few hundred kB of RAM, a megabyte of flash, and a secure element for key custody. Common reference platforms: Foundation Devices Passport (STM32H7), BitBox02 (ATSAMD51), Trezor Safe (STM32U5).

The shape of the integration is fundamentally different from a phone or backend:

- Only the `jova-core-primitives` crate is usable. The chain-specific crates (`bdk_wallet`, `alloy`, the Solana split crates, `xrpl-rust`) all require `std`.
- The firmware does not import `jova-core` (which uses `jova-core-chains` and therefore drags in `std`).
- Chain-specific encoding lives in firmware-side code or is delegated entirely to the companion phone app.

## What the firmware uses from us

```toml
# firmware/Cargo.toml
[dependencies]
jova-core-primitives = { version = "1.2", default-features = false, features = ["external-rng"] }
zeroize = { version = "1", default-features = false }
secp256k1 = { version = "0.31", default-features = false, features = ["alloc", "lowmemory"] }
```

`jova-core-primitives` is `no_std`-clean. `default-features = false` disables the `std` feature (which would pull in `getrandom`). The `external-rng` feature is the Phase 7 deliverable — it swaps in [`JovaRng`](#jovarng-the-hardware-rng-trait) so firmware supplies entropy from its own hardware TRNG. The crate uses `alloc` transitively because BIP-39 wordlist operations need a heap; firmware uses `embedded-alloc` / `linked_list_allocator` / `talc`.

What you get:

- `Mnemonic::generate_with(strength, &mut rng)` — generate a fresh mnemonic using firmware-supplied randomness. **Phase 7 deliverable; requires the `external-rng` feature.**
- `Mnemonic::validate(words, passphrase)` — checksum + wordlist validation.
- `Mnemonic::to_seed(words, passphrase)` — PBKDF2-HMAC-SHA512.
- `Seed`, `XPrv`, `Ed25519Xprv` — HD types. `Seed::from_external_bytes(bytes)` constructs from a secure-element-supplied 64-byte BIP-39 seed (Phase 7).
- `derive_secp256k1(seed, path)` — BIP-32 secp256k1 derivation.
- `derive_ed25519(seed, path)` — SLIP-10 ed25519 derivation (Solana).
- `DerivationPath::{bip44_path, bip84_path, parse}` — path constructors.

For raw signing, hashing, and encoding, firmware imports `secp256k1`, `ed25519-dalek`, `sha2`, `sha3`, `ripemd`, `hex`, `bs58`, `bech32` directly with `no_std + alloc` configurations.

## `JovaRng`: the hardware RNG trait (Phase 7)

```rust
pub trait JovaRng {
    fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), RngError>;
}

pub enum RngError {
    /// Source busy / locked — caller may retry.
    Unavailable,
    /// Source self-detected as broken (e.g. NIST SP 800-90B health check failed).
    HealthCheckFailed,
}
```

The implementation is responsible for cryptographic quality — `jova-core-primitives` consumes the bytes without post-processing.

### Reference patterns

**STM32 TRNG (stm32f4xx-hal):**
```rust
use stm32f4xx_hal::rng::Rng as HalRng;
use jova_core_primitives::{JovaRng, RngError};

struct StmTrng<'a>(&'a mut HalRng);
impl<'a> JovaRng for StmTrng<'a> {
    fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), RngError> {
        self.0.read(dest).map_err(|_| RngError::Unavailable)
    }
}
```

**nRF52840 RNG (nrf52840-hal):** similar shape via `nrf52840_hal::rng::Rng`.

**Secure-element-seeded CSPRNG (no on-chip TRNG):** draw a 32-byte seed from the secure element (ATECC608 `Random` command, OPTIGA Trust M `GetRandom`), feed to `rand_chacha::ChaCha20Rng`, wrap in `JovaRng`. Document the entropy chain — auditors check this.

## Bypassing the mnemonic step

When the firmware has the seed in a secure element (typical for production — the user enters their mnemonic on first setup and the device stores the derived seed, never the words), wrap it via `Seed::from_external_bytes`:

```rust
use jova_core_primitives::{Seed, derive_secp256k1, DerivationPath};

let seed_bytes: [u8; 64] = secure_element_read_seed();
let seed = Seed::from_external_bytes(seed_bytes);  // Zeroize on drop
let path = DerivationPath::parse("m/44'/60'/0'/0/0").unwrap();
let xprv = derive_secp256k1(&seed, &path).unwrap();
```

For host-side (non-bare-metal) consumers that want the full `JovaWallet` API, `jova-core` ships `JovaWallet::from_seed_bytes(bytes)` behind the same `external-rng` feature. **Rust-only — not exposed via FFI/WASM.**

## Reference firmware template

[`examples/firmware-template/`](../examples/firmware-template/) is a working `thumbv7em-none-eabihf` binary linking `jova-core-primitives` and signing a synthetic EVM digest. Builds in CI; no real hardware needed. Demonstrates:

- Heap setup via `embedded-alloc` (16 KiB static buffer).
- BIP-39 mnemonic → seed → BIP-44 derivation in `no_std`.
- secp256k1 ECDSA signing on Cortex-M with the `lowmemory` feature.
- ~394 KB stripped ELF — fits any STM32F4 / nRF52840 / SAMD51 flash.

The template hardcodes the BIP-39 test mnemonic for hermetic CI; production firmware substitutes `Mnemonic::generate_with` against the platform's TRNG.

## Side-channel and glitch protection

Generic guidance — consult your secure element vendor's white papers for platform-specific recommendations:

**Secure-element-backed key custody:**
- **ATECC608** (Microchip): I2C/SWI, slot-based key storage with HMAC authentication. Use for the seed; the SDK never sees the raw key.
- **OPTIGA Trust M** (Infineon): I2C, ECC P-256 native + RSA-2048. Slower but more flexible.
- **SE050** (NXP): broader algorithm set including secp256k1 native; can perform the full signing operation on the secure element.

**Glitch-detection patterns:**
- Voltage monitoring via on-chip ADC (rail droop indicates a power-glitch attempt).
- Clock-frequency self-check before each crypto operation.
- Recompute critical decisions twice with different timing and assert agreement.
- TXM/RXM cross-check for I2C/SPI commands to the secure element.

**User-confirmation UI:**
- Display transaction details (chain, to-address, amount, fee) on the device's own screen, not the phone's.
- Require explicit button press to confirm. Cancellation aborts; do not auto-time-out into approval.
- Display the address in a checksummed form (EIP-55 for EVM, bech32 for BTC) and have the user verify the first/last 4 characters against the phone display.

Reference platforms with public design documents:
- **Foundation Devices Passport** — STM32H7, dual-secure-element architecture.
- **BitBox02** — ATSAMD51 + ATECC608.
- **Trezor Safe 5** — STM32U5, on-chip secure storage.

This guide is **starting points, not exhaustive**; certification (FIPS 140-3, CC EAL5+) is a 6+ month process with its own budget.

What you don't get (and what to do instead):

| Need | Source |
|---|---|
| EIP-1559 RLP encoding | Implement in firmware using `rlp` crate (no_std-clean) or use what the phone app already produced |
| EIP-712 typed-data hashing | Implement in firmware or ask the phone for the digest |
| BIP-174 PSBT parsing | Use `bitcoin` crate's `no_std` mode + `psbt` feature |
| Solana v0 message encoding | Companion app sends pre-encoded bytes; firmware just signs |
| XRPL canonical serialization | Companion app sends pre-encoded bytes |

The general pattern: the phone (running the full `jova-core` SDK) constructs and serializes the unsigned transaction. The phone sends serialized bytes + the digest to the hardware wallet. The hardware wallet validates the digest matches the bytes (sanity check), confirms with the user, signs, and returns the signature.

This is the same model every modern hardware wallet uses. We don't reinvent it.

---

## Custom RNG injection

Firmware can't use `getrandom` from the OS. `jova-core-primitives` accepts an injected RNG via a feature:

```toml
[dependencies.jova-core-primitives]
version = "1.0"
default-features = false
features = ["alloc", "external-rng"]
```

```rust
use jova_core_primitives::rng::JovaRng;

struct HwRng;
impl JovaRng for HwRng {
    fn fill(&mut self, dst: &mut [u8]) -> Result<(), JovaError> {
        // Read from the chip's TRNG peripheral
        unsafe { stm32::TRNG::read_bytes(dst) }
    }
}

let mnemonic = Mnemonic::generate_with(Strength::Bits256, &mut HwRng)?;
```

The RNG must be a true random source (TRNG peripheral, not a seeded PRNG).

---

## Memory: even more careful than the phone

On hardware:

- Stack and heap are the *same* RAM segment as everything else. There's no OS isolation.
- Power-glitch attacks are a real threat — voltage/frequency changes can flip bits during signing.
- Side-channel resistance matters more than on phones (an attacker has physical access).

What `jova-core-primitives` does well here:

- `#![forbid(unsafe_code)]` — no UB to chain into a glitch.
- `Zeroizing<>` everywhere — secrets are not lingering after use.
- Constant-time underlying primitives (`secp256k1`'s constant-time mode, `ed25519-dalek`'s).

What the firmware adds:

- Glitch detection: voltage monitor, double-check critical results.
- Side-channel mitigations: blinding scalars before secp256k1 multiplication.
- Power-glitch retries: re-do the signing if any consistency check fails.

These are firmware-side concerns; the SDK's job is to provide a clean primitives layer that doesn't fight them.

---

## Reproducible firmware builds

Hardware-wallet firmware must be reproducible — users should be able to verify the binary they're running matches the source. The pattern:

- Pin Rust toolchain (`rust-toolchain.toml`).
- `Cargo.lock` committed.
- Build with `RUSTFLAGS="-C codegen-units=1 -C debuginfo=0"`.
- Strip the resulting ELF deterministically.
- Publish the SHA-256 hash on the firmware release page.

`jova-core-primitives` is reproducible (see `build-and-release.md`). Firmware builds inherit that property.

---

## Companion app integration model

The full picture:

```
┌──────────────────────────┐         USB / BLE         ┌────────────────────────┐
│   iOS / Android app       │ ◄──────────────────────► │   Hardware wallet      │
│                          │                            │   firmware             │
│  - jova-core (full SDK)  │                            │  - jova-core-primitives│
│  - Constructs unsigned   │  unsigned tx + digest +    │  - Validates digest    │
│    tx with               │  derivation path           │  - Shows on screen     │
│  - Computes digest       │ ──────────────────────────►│  - User confirms       │
│  - Sends to firmware     │                            │  - Derives XPrv via    │
│                          │                            │    BIP-32 / SLIP-10    │
│                          │                            │  - secp256k1.sign or   │
│                          │                            │    ed25519.sign        │
│  - Receives signature    │ ◄──────────────────────────│  - Returns signature   │
│  - Assembles signed tx   │  signature bytes           │                        │
│  - Broadcasts via        │                            │                        │
│    backend               │                            │                        │
└──────────────────────────┘                            └────────────────────────┘
```

The phone app does ~95% of the work. The hardware wallet's job is:

1. Hold the seed in a tamper-resistant secure element.
2. Show the user what's being signed (chain-specific UI; small screen).
3. On confirmation, derive the per-tx key, sign the digest, return the signature.
4. Refuse to sign anything where the displayed details don't match the actual digest.

Step 4 is the whole reason hardware wallets exist. Firmware code that uses our SDK is responsible for getting it right; we provide the primitives that make it correct (digest computation, derivation), but the policy enforcement is the firmware's problem.

---

## A minimal firmware-side `sign_tx` example

```rust
#![no_std]
#![no_main]
extern crate alloc;

use jova_core_primitives::{Mnemonic, DerivationPath, Curve, secp256k1};
use zeroize::Zeroizing;

fn handle_evm_sign_request(req: &SignRequest) -> Result<[u8; 65], FwError> {
    // 1. Sanity-check digest matches the unsigned-tx bytes the phone sent.
    let computed = keccak256_evm_digest(&req.unsigned_tx_bytes, &req.access_list);
    if computed != req.digest {
        return Err(FwError::DigestMismatch);
    }

    // 2. Show on screen.
    display::show_evm_tx(&req.unsigned_tx_bytes)?;
    if !user::confirm()? {
        return Err(FwError::UserDeclined);
    }

    // 3. Load seed from secure element.
    let seed = Zeroizing::new(secure_element::load_seed()?);

    // 4. Derive m/44'/60'/0'/0/0.
    let path = DerivationPath::parse("m/44'/60'/0'/0/0")?;
    let xprv = jova_core_primitives::derive(&seed, &path, Curve::Secp256k1)?;
    // xprv is Zeroizing<XPrv> — drops at end of scope

    // 5. Sign.
    let sig = secp256k1::sign(&req.digest, &xprv)?;

    // 6. Encode (r,s,v) — 65 bytes.
    Ok(sig.to_evm_rsv())
}
```

Notes:

- The firmware reconstructs the digest *itself* from the unsigned bytes (step 1). It does not trust the phone's digest. This is critical — it's the only thing standing between the user and a malicious phone signing whatever it wants.
- Step 2 displays the human-readable transaction. The display logic is per-chain firmware code; we don't ship it.
- Steps 4–6 use `jova-core-primitives` exclusively.
- `Zeroizing<>` ensures derived keys clear when the function returns.

---

## What we'll add for hardware in Phase 7

Provisional commitments:

- `external-rng` feature on `jova-core-primitives` (replacing `getrandom`).
- `from_seed_bytes(bytes: &[u8])` constructor on `JovaWallet` (Rust-only for now; not on FFI bindings) — for hardware that already has the seed in a secure element and wants to skip mnemonic-to-seed derivation.
- A reference `firmware-template/` example using `jova-core-primitives` on `thumbv7em-none-eabihf`, demonstrating the full sign-flow loop.
- Documentation on glitch protection, side-channel mitigation, and how to integrate with common secure elements (ATECC, OPTIGA, TRH4-equivalent).

These are scoped to Phase 7 because they require hardware to test against. Until then, the `no_std` build target validates the API surface compiles for embedded; actual hardware integration begins when there's hardware to integrate with.

---

## Don'ts

- Don't trust digests from the host. Recompute them from the bytes the host sent.
- Don't display the unsigned tx without parsing it. A blob the user can't read offers no security benefit over a software wallet.
- Don't pull in `std`-using crates by accident (`bdk_wallet`, `alloy`, etc.). The build will silently inflate or fail in linker if any sneak in.
- Don't reuse derived keys across calls. Derive freshly per signing operation; let `Zeroizing` clean up.
- Don't skip the user confirmation step. Ever.
