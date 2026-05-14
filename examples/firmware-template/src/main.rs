//! Phase 7 reference firmware-template.
//!
//! Bare-metal Cortex-M binary linking `jova-core-primitives` and signing a
//! synthetic EVM digest. Demonstrates:
//!
//! 1. The `JovaRng` trait — pass a real hardware TRNG here; this template
//!    uses a hardcoded test seed for hermetic CI.
//! 2. `Mnemonic::to_seed` and `derive_secp256k1` working on `thumbv7em-none-eabihf`.
//! 3. secp256k1 signing on the embedded target via the `lowmemory` feature.
//!
//! **Not production firmware.** Real firmware adds:
//! - Glitch / voltage monitoring.
//! - Secure-element protocol (ATECC608, OPTIGA Trust M).
//! - Display + user-confirmation UI.
//! - USB / BLE host-protocol layer the phone speaks.
//! - Vector parity against `spec/test-vectors.json`.
//! See `docs/integration-hardware.md` for guidance.

#![no_std]
#![no_main]
// `deny` (not `forbid`) so heap_init::init_heap can locally allow unsafe for
// the embedded-alloc::Heap::init call. Every other module in this binary
// stays unsafe-free; the crate-level deny catches accidental escapes.
#![deny(unsafe_code)]

extern crate alloc;

use alloc::string::String;
use core::mem::MaybeUninit;
use cortex_m::asm;
use cortex_m_rt::entry;
use embedded_alloc::LlffHeap as Heap;
use panic_halt as _;

use jova_core_primitives::{DerivationPath, Mnemonic, derive_secp256k1};

#[global_allocator]
static HEAP: Heap = Heap::empty();

/// Place the heap statically in RAM. 16 KiB is enough for the test workload
/// (jova-core-primitives' alloc footprint is mostly transient String for
/// mnemonic + derivation-path parsing).
const HEAP_SIZE: usize = 16 * 1024;
static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];

#[entry]
fn main() -> ! {
    // Initialise the heap before any alloc::* type is touched.
    {
        // SAFETY: `forbid(unsafe_code)` is at crate scope; the global allocator
        // setup requires raw pointer + size, which is structurally `unsafe`. We
        // gate this in a separate inner module so the crate-level forbid still
        // catches accidental `unsafe` elsewhere.
        init_heap();
    }

    // Use the BIP-39 standard test mnemonic so CI is reproducible.
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    // Mnemonic -> seed (PBKDF2-SHA512, alloc-only, no_std-safe).
    let seed = match Mnemonic::to_seed(mnemonic, "") {
        Ok(s) => s,
        Err(_) => loop_forever(),
    };

    // Derive Ethereum BIP-44 path.
    let path = match DerivationPath::parse("m/44'/60'/0'/0/0") {
        Ok(p) => p,
        Err(_) => loop_forever(),
    };

    let xprv = match derive_secp256k1(&seed, &path) {
        Ok(x) => x,
        Err(_) => loop_forever(),
    };

    // Get the compressed pubkey — proves the derivation pipeline ran.
    let pubkey = xprv.public_key_compressed();

    // Sign a synthetic 32-byte digest with secp256k1 directly.
    let digest = [0xABu8; 32];
    let secp = secp256k1::Secp256k1::signing_only();
    let sk_bytes = xprv.private_key_bytes();
    let sk = match secp256k1::SecretKey::from_byte_array(*sk_bytes) {
        Ok(k) => k,
        Err(_) => loop_forever(),
    };
    let msg = secp256k1::Message::from_digest(digest);
    let sig = secp.sign_ecdsa(msg, &sk);
    let sig_der = sig.serialize_der();

    // Discard the values so the optimizer doesn't strip them. Real firmware
    // hands the signature back to the host over USB/BLE; this template just
    // proves the math executed.
    let _ = core::hint::black_box((pubkey, sig_der, String::new()));

    loop_forever();
}

fn loop_forever() -> ! {
    loop {
        asm::wfi();
    }
}

mod heap_init {
    use super::{HEAP, HEAP_MEM, HEAP_SIZE};

    // The only unsafe in the crate. Isolated for review.
    #[allow(unsafe_code)]
    pub fn init_heap() {
        // SAFETY: HEAP.init is called once before any allocation. HEAP_MEM is
        // a static mut, but no aliases exist at this point — we're still in
        // _start before the entry handler executes user code.
        unsafe {
            HEAP.init(core::ptr::addr_of_mut!(HEAP_MEM) as usize, HEAP_SIZE);
        }
    }
}

use heap_init::init_heap;
