# Memory and Key Handling

How `jovawallet-core` handles secret material — mnemonics, seeds, derived private keys — across every layer of the stack and every binding.

This document is the source of truth for the secret-clearing contract. `security.md` quotes it; auditors test against it.

## Threat scope

- **In scope:** ensuring secret bytes are zeroized as soon as they're no longer needed in process memory the SDK controls.
- **Out of scope:** secret bytes the *host language* copied before they reached us, or that the OS swapped to disk before zeroization. Apps and operating systems own these boundaries; we honestly document where our guarantees end.

## Lifecycle of a secret

There are four kinds of secret material the SDK touches:

1. **Mnemonic words** (UTF-8 string). Lives in app memory until passed to us, briefly in our memory during `JovaWallet::from_mnemonic`, then we discard.
2. **BIP-39 passphrase** (UTF-8 string). Same as mnemonic.
3. **Seed** (64 bytes, derived from mnemonic via PBKDF2-HMAC-SHA512). Held inside `JovaWalletInner` for the lifetime of the wallet handle.
4. **Derived `XPrv`** (32 bytes + 32-byte chain code). Materialized per signing call, discarded when the call returns.

Public material — public keys, addresses, signatures — is **not** secret and crosses FFI freely.

---

## Rust-side guarantees

Every secret-bearing type implements `zeroize::Zeroize` and `zeroize::ZeroizeOnDrop`. Construction sites wrap them in `Zeroizing<>` containers so accidental moves don't leave copies behind.

### Types

```rust
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Mnemonic {
    pub(crate) words: String,        // zeroized via String → Vec<u8> drop hook
    pub(crate) passphrase: String,
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Seed(pub(crate) [u8; 64]);

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct XPrv {
    pub(crate) key: [u8; 32],
    pub(crate) chain_code: [u8; 32],
}

pub(crate) struct JovaWalletInner {
    seed: Zeroizing<Seed>,
    // ... no other secret state
}
```

### Construction

`JovaWallet::from_mnemonic(words, passphrase)`:

1. Validate words on a borrowed `&str` — no copy.
2. Derive 64-byte seed via `bip39::Mnemonic::to_seed_normalized(passphrase)` → `Zeroizing<[u8; 64]>`.
3. Move into `JovaWalletInner`.
4. Drop the `Mnemonic` (which zeroizes `words` and `passphrase`).

The mnemonic struct lives less than a millisecond; the seed lives as long as the wallet.

### Per-call derivation

`JovaWallet::address` and `JovaWallet::sign_*`:

1. Derive the chain-specific `XPrv` from the seed inside a `Zeroizing<XPrv>` container.
2. Pass `&XPrv` (borrowed reference) into the chain signer — the signer cannot move it.
3. When the call returns, the `Zeroizing<XPrv>` drops and zeroizes.

No `XPrv` outlives the call that derived it.

### Drop

`JovaWalletInner::drop` zeroizes the seed. This is automatic via `ZeroizeOnDrop`. There is no explicit `clear()` method needed — Rust's RAII handles it.

### What `zeroize` does and doesn't do

- **Does:** writes zero bytes into the memory backing the value, with `volatile` semantics so the optimizer can't elide the write.
- **Doesn't:** prevent the compiler from putting copies in registers, on the stack during inlining, or in CPU caches. We mitigate by avoiding `#[inline(always)]` on the hot path and by minimizing copies (`&` references everywhere possible).
- **Doesn't:** prevent the OS from swapping the page to disk before drop. On platforms where this matters (uncommon for phone apps; relevant for desktop), bindings should call `mlock` on the relevant memory before construction. The SDK does not do this by default because it requires elevated privileges; firmware integrations that need it (rare) must arrange it themselves.

### Allocator behavior

The default Rust allocator (`std::alloc::System`) does not zero on free. That's why we explicit-zeroize. We do **not** swap to a custom secure allocator (e.g., `secrecy`'s `SecretVec`) because:

- It complicates `no_std` compilation (firmware uses `linked_list_allocator` or similar).
- `Zeroizing<>` already gives us the contract we need.
- A custom allocator can mask bugs (failure to zeroize manually that the allocator papers over).

---

## FFI-side guarantees

Across the FFI, the question becomes: what does the host language do with values it copies out of WASM/Rust memory?

### Strings: an honest disclaimer

Both Swift `String` and Kotlin/Java `String` are immutable on the JVM/CoreFoundation level — there is **no public API** to overwrite their backing storage. Once a mnemonic is in a Swift `String`, the OS controls when that memory is freed and whether it's zeroed.

For apps that care, we expose `MnemonicBuffer`:

```
MnemonicBuffer { bytes: Bytes; passphrase: Bytes }
```

The app stores the mnemonic in a `[UInt8]` (Swift) or `ByteArray` (Kotlin) it owns and clears manually after passing to `JovaWallet`. We document this prominently in the integration guides.

In v1, **`Mnemonic` (string) and `MnemonicBuffer` (bytes) coexist.** Apps choose. The Jova iOS and Android apps will use `MnemonicBuffer` for the import/restore flow and `Mnemonic` (string) for create-new-wallet (where the bytes were generated inside the SDK and the app needs them as a displayable string anyway).

### Wallet handle: explicit lifecycle in every binding

Every binding exposes the wallet handle as a managed resource:

| Binding | Pattern | What happens |
|---|---|---|
| Swift | `final class JovaWallet`, `deinit` calls `jova_wallet_drop` | ARC frees the wrapper; deinit FFI-calls into Rust which drops the inner `Box`, zeroizing the seed. |
| Kotlin | `class JovaWallet : AutoCloseable`, `close()` calls `jova_wallet_drop` | Apps must call `close()` (or use `wallet.use { … }`). The native handle is also freed in the Cleaner / finalizer as a safety net, but apps should not rely on it. |
| JavaScript | `class JovaWallet { destroy(): void }` | Apps must call `destroy()`. There is no GC-driven zeroization — the JS GC does not run finalizers synchronously enough for crypto-grade clearing. |
| Rust | `struct JovaWallet { … }` with `Drop` | Standard RAII; nothing for the consumer to do. |

The Kotlin and JS variants document the manual `close()` / `destroy()` requirement loudly. Examples in `integration-android.md` and `integration-web.md` show the pattern.

### Signature outputs are not secret

`Signature.hex` and `SignedTx.rawHex` may be logged, transmitted, and stored. They reveal nothing the user wouldn't be revealing the moment they broadcast the transaction.

### Public keys are not secret

`Address.value` is public. Apps log addresses freely (subject to PII concerns about user identification — that's an app-policy decision, not a crypto one).

---

## Per-binding secret-clearing contract

### Swift

```swift
import JovaCore

let mnemonicBytes: [UInt8] = readUserInputAsBytes()  // app owns
defer { mnemonicBytes.withUnsafeMutableBufferPointer { ptr in
    ptr.update(repeating: 0, count: ptr.count)
}}

let buf = MnemonicBuffer(bytes: Data(mnemonicBytes), passphrase: Data())
let wallet = try JovaWallet(mnemonic: buf)   // SDK consumes buf
defer { /* wallet's deinit clears Rust-side seed */ }

let signed = try wallet.signTx(unsigned: tx)
broadcast(signed)
// at end of scope, wallet deinit fires → Rust drops seed
```

Limitation: `Data` backing storage is not guaranteed to be zeroed by Swift's runtime even after we overwrite it; an OS-level memory-pressure swap could have copied the bytes to disk between read and zero. This is a platform limit; we document it.

### Kotlin

```kotlin
import io.jova.core.JovaWallet
import io.jova.core.MnemonicBuffer

val mnemonicBytes = readUserInputAsBytes()  // ByteArray; app owns
try {
    JovaWallet(MnemonicBuffer(mnemonicBytes, ByteArray(0))).use { wallet ->
        val signed = wallet.signTx(tx)
        broadcast(signed)
    }   // wallet.close() runs → seed zeroized
} finally {
    mnemonicBytes.fill(0)
}
```

`use` is Kotlin's try-with-resources. It guarantees `close()` runs even on exception.

### JavaScript

```typescript
import { JovaWallet, MnemonicBuffer } from '@jovachain/wallet-core';

const mnemonicBytes = readUserInputAsBytes();   // Uint8Array; app owns
let wallet: JovaWallet | null = null;
try {
    wallet = new JovaWallet(new MnemonicBuffer(mnemonicBytes, new Uint8Array()));
    const signed = wallet.signTx(tx);
    await broadcast(signed);
} finally {
    wallet?.destroy();              // zeroizes Rust-side seed
    mnemonicBytes.fill(0);          // app-side best effort
}
```

### Rust

```rust
use jova_core::{JovaWallet, MnemonicBuffer};

let mnemonic_bytes = read_user_input_as_bytes();
let wallet = JovaWallet::from_mnemonic_buffer(MnemonicBuffer {
    bytes: mnemonic_bytes,
    passphrase: vec![],
})?;
let signed = wallet.sign_tx(&tx)?;
broadcast(signed);
// wallet drops at end of scope; seed zeroizes
// mnemonic_bytes was moved into MnemonicBuffer which dropped immediately after construction
```

---

## What the SDK explicitly does NOT do

- **Does not call `mlock`.** Locking pages prevents swap, but requires `CAP_IPC_LOCK` on Linux and is not portable to iOS/Android. Apps that need this protection (uncommon) can wrap their host-language buffer in a platform-specific locked region before calling us.
- **Does not call `madvise(DONTDUMP)`.** Same portability concern; not part of v1.
- **Does not interact with hardware-backed key stores.** That's `integration-ios.md` and `integration-android.md` territory — apps put the mnemonic in Keychain/Keystore and read it out at sign time.
- **Does not implement constant-time comparison of secrets** at our layer — the underlying crypto crates (`secp256k1`, `ed25519-dalek`) already use constant-time primitives. We do not introduce timing channels by adding equality checks on secrets.
- **Does not log secret material.** The Rust core has no logger; bindings choose theirs, and our `Display` impls on secret types intentionally do not reveal contents.

---

## Audit checklist

When auditing this layer, verify:

- [ ] Every type holding key material derives `Zeroize` and `ZeroizeOnDrop`, *or* is wrapped in `Zeroizing<>` at every construction site.
- [ ] No `Clone` impl exists on `Seed`, `XPrv`, or any inner secret type. (`Mnemonic` is `Clone` because the words may need to be returned to the user; the clone is also `Zeroize`.)
- [ ] No `#[derive(Debug)]` on secret types reveals contents — `Debug` impls are hand-written to redact.
- [ ] No `Display` impl on secret types exists at all.
- [ ] FFI surface does not expose any function returning seed or `XPrv` bytes.
- [ ] `JovaWalletInner::drop` is reachable on every termination path (no `mem::forget`, no `Box::leak`).
- [ ] Every binding's `MemoryTests` confirms post-drop the underlying byte region is zeroed (verifiable by reading the freed memory before allocator reuses it; tests use `nix::sys::mman::mprotect` or equivalent).
- [ ] `cargo miri test` on `jova-core-primitives` passes — catches use-after-free across `unsafe` boundaries.
- [ ] Reason strings in errors do not contain secret content.

---

## Hardware-wallet implications

`jova-core-primitives` is `no_std`, no global state, and uses `Zeroizing<>` exclusively. It is suitable for use in firmware that performs key derivation in a secure element or in a software-isolated security domain. Firmware using it does not gain WASM or chain-specific signing — those layers stay on the device's communication-side processor or on the companion app.

`integration-hardware.md` covers the firmware story in detail.
