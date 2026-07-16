# Integration: Android

How the Android Jova app consumes `jovawallet-core`. Companion to `integration-ios.md`; the structure mirrors that document.

The Android app gets the SDK as a Maven Central dependency. The artifact contains:

- A native library (`libjova_core_ffi.so`) for each ABI: `arm64-v8a`, `armeabi-v7a`, `x86_64`, `x86`.
- A generated `JovaCore.kt` file with the public API.
- A small `Convenience.kt` ergonomics layer.

## Adding the dependency

### Today (pre-v1.0): GitHub Releases AAR

Until v1.0.0 ships to Maven Central (gated on the external audit — see [issue #8](https://github.com/jovachain/jovawallet-core/issues/8)), the AAR is distributed as a release asset on each tag.

1. Download the AAR for the version you want from [https://github.com/jovachain/jovawallet-core/releases](https://github.com/jovachain/jovawallet-core/releases). The asset is named `jova-core-<version>.aar` and is accompanied by a `.sha256` file.
2. Verify the checksum and drop the AAR into your app module's `libs/` folder.
3. In `app/build.gradle.kts`:

```kotlin
dependencies {
    implementation(files("libs/jova-core-0.5.0.aar"))
    // Required transitive deps — the file-based AAR import doesn't propagate them.
    implementation("net.java.dev.jna:jna:5.14.0@aar")
    implementation("androidx.security:security-crypto:1.1.0-alpha06")
}
```

Each version bump is a swap of the AAR file + a one-line edit. The `.github/workflows/release-aar.yml` workflow builds and attaches the AAR automatically on every `v*.*.*` tag push, so consumers always have a versioned artifact to pull.

### Post-v1.0: Maven Central

Once the audit closes and v1.0.0 ships, the same SDK will be available as a normal Maven Central dependency:

```kotlin
dependencies {
    implementation("io.jovachain:jova-core:1.0.0")
}
```

Maven Central is preconfigured in most Android projects (AGP includes it by default). If yours doesn't:

```kotlin
// settings.gradle.kts
dependencyResolutionManagement {
    repositories {
        mavenCentral()
    }
}
```

Either way — no manual JNI setup, no NDK in your build, no `web3j` / `bitcoinj` / `bdk-android` to pull in.

## Minimum platform versions

- `compileSdk = 34` (Android 14)
- `minSdk = 24` (Android 7)
- Gradle 8.0+
- Kotlin 1.9+

These match the AAR's own constraints. The native libraries are built for Android NDK r26+ targeting API 21+, so the `minSdk = 24` is the AAR's manifest constraint, not a JNI constraint.

## The Hello-World

```kotlin
import io.jova.core.JovaWallet
import io.jova.core.JovaChain
import io.jova.core.Strength

fun helloWorld() {
    val mnemonic = JovaWallet.createMnemonic(Strength.BITS128)
    println("Generated 12 words: ${mnemonic.words}")

    JovaWallet.fromMnemonic(mnemonic).use { wallet ->
        val ethAddress = wallet.address(JovaChain.Ethereum, 0u)
        val btcAddress = wallet.address(JovaChain.Bitcoin, 0u)
        println("ETH: ${ethAddress.value}")
        println("BTC: ${btcAddress.value}")
    }   // wallet.close() runs → seed zeroized in Rust
}
```

`use { … }` is Kotlin's try-with-resources. Always wrap a wallet in it.

---

## The recommended app architecture

```
┌─────────────────────────────────────────────────────────┐
│              Android Jova App                            │
│                                                          │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ UI layer (Compose)                                   │ │
│ │  - SignTransactionScreen                             │ │
│ │  - ImportWalletScreen                                │ │
│ └─────────────────────────────────────────────────────┘ │
│                       ▲                                 │
│                       │ ViewModel calls                 │
│                       ▼                                 │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ WalletRepository (app-owned facade)                  │ │
│ │  - storeMnemonic(...) → EncryptedSharedPreferences  │ │
│ │  - signTransaction(...) → JovaWallet                 │ │
│ │  - signMessage(...) → JovaWallet                     │ │
│ └─────────────────────────────────────────────────────┘ │
│                       ▲                                 │
│                       │ uses                            │
│                       ▼                                 │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ Android Keystore + EncryptedSharedPreferences        │ │
│ │  - mnemonic encrypted with hardware-backed key       │ │
│ │  - biometric prompt gates decryption                 │ │
│ └─────────────────────────────────────────────────────┘ │
│                       ▲                                 │
│                       │ reads on demand                 │
│                       ▼                                 │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ JovaCore (this SDK)                                  │ │
│ │  - JovaWallet.fromMnemonic(...)                      │ │
│ │  - signTx(unsigned)                                  │ │
│ │  - signMessage(msg)                                  │ │
│ └─────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

Notes:

- `WalletRepository` is the app's facade. The UI never touches `JovaCore` directly.
- The mnemonic lives in `EncryptedSharedPreferences` (or DataStore with crypto), with a hardware-backed key from Android Keystore.
- `WalletRepository` is **not** a singleton holding a long-lived `JovaWallet`. Long-lived wallets keep secret material in memory longer than necessary.

---

## `WalletRepository` reference implementation

```kotlin
package io.jovachain.app.wallet

import android.content.Context
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import io.jova.core.*

class WalletRepository(context: Context) {

    private val masterKey = MasterKey.Builder(context)
        .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
        .setUserAuthenticationRequired(true, 0)   // require fresh biometric per access
        .build()

    private val prefs = EncryptedSharedPreferences.create(
        context,
        "io.jovachain.wallet",
        masterKey,
        EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
        EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM
    )

    fun storeMnemonic(mnemonic: Mnemonic) {
        prefs.edit().putString(KEY_MNEMONIC, mnemonic.words).apply()
    }

    private fun loadMnemonicBytes(): ByteArray {
        val str = prefs.getString(KEY_MNEMONIC, null)
            ?: throw WalletError.NoMnemonic
        return str.toByteArray(Charsets.UTF_8)
    }

    // `account` selects the HD account and MUST match the account whose
    // address(chain, account) the app shows as the "from" address. Defaults
    // to 0 (uniffi default), so single-account callers can omit it.
    fun signTransaction(tx: UnsignedTx, account: UInt = 0u): SignedTx {
        val bytes = loadMnemonicBytes()
        try {
            JovaWallet.fromMnemonicBuffer(MnemonicBuffer(bytes, ByteArray(0))).use { wallet ->
                return wallet.signTx(tx, account)
            }
        } finally {
            bytes.fill(0)
        }
    }

    fun signMessage(msg: SignableMessage, account: UInt = 0u): Signature {
        val bytes = loadMnemonicBytes()
        try {
            JovaWallet.fromMnemonicBuffer(MnemonicBuffer(bytes, ByteArray(0))).use { wallet ->
                return wallet.signMessage(msg, account)
            }
        } finally {
            bytes.fill(0)
        }
    }

    fun address(chain: JovaChain, account: UInt = 0u): Address {
        val bytes = loadMnemonicBytes()
        try {
            JovaWallet.fromMnemonicBuffer(MnemonicBuffer(bytes, ByteArray(0))).use { wallet ->
                return wallet.address(chain, account)
            }
        } finally {
            bytes.fill(0)
        }
    }

    private companion object {
        const val KEY_MNEMONIC = "primary"
    }
}

sealed class WalletError : Exception() {
    object NoMnemonic : WalletError()
}
```

What this is doing on purpose:

- Reads mnemonic into a `ByteArray`, fills with zeros in `finally`.
- Uses `MnemonicBuffer` so the SDK gets bytes (not a `String`) — bytes can be cleared.
- `use { … }` ensures `wallet.close()` runs even on exception.
- No retained wallet anywhere in the type.

---

## Threading

`JovaWallet` is **not** thread-safe. Do not share an instance across threads or coroutines.

Recommended pattern: run signing on `Dispatchers.Default` and marshal results back via a coroutine:

```kotlin
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class WalletViewModel(private val repo: WalletRepository) : ViewModel() {

    suspend fun sign(tx: UnsignedTx): SignedTx = withContext(Dispatchers.Default) {
        repo.signTransaction(tx)
    }
}
```

Wallet construction is fast (microseconds); signing is fast (single-digit milliseconds). Main thread is fine for development; production should stay on a background dispatcher.

---

## WalletConnect (Reown) integration

The app uses Reown (formerly WalletConnect Kotlin SDK). When a dApp sends a request:

```kotlin
fun onSessionRequest(request: SignClient.Model.SessionRequest): Result<String> {
    return runCatching {
        when (request.request.method) {
            "personal_sign" -> {
                val msg = SignableMessage.EvmPersonalSign(extractMessage(request))
                repo.signMessage(msg).hex
            }
            "eth_sendTransaction" -> {
                val tx = parseAsUnsignedEvm(request)
                repo.signTransaction(tx).rawHex
            }
            "eth_signTypedData_v4" -> {
                val msg = SignableMessage.EvmTypedDataV4(extractTypedDataJson(request))
                repo.signMessage(msg).hex
            }
            else -> throw IllegalArgumentException("unsupported method: ${request.request.method}")
        }
    }
}
```

The SDK does not know about Reown. The app does.

---

## Migration from legacy crypto stack

The legacy Android app used `web3j` + `bitcoinj` + `bdk-android` + a hand-rolled `EvmSigner` and `SecureWalletDerivation`. The migration plan:

1. **Phase 4 of `plan.md`** — both legacy and new code paths exist. New code routes through `WalletRepository`; old code through `EvmSigner`/`BitcoinWalletManager`.
2. **Per-chain cutover.** EVM first (most-used), then BTC, SOL, XRP. After each cutover, validate against `spec/test-vectors.json` plus a manual mainnet smoke test.
3. **BTC address reconciliation.** The legacy app already used BIP-84 (`bc1q…`). Verify both old and new derive the same address from the same seed phrase. Spot-check 100 known addresses against the legacy app's storage.
4. **Removal.** Delete `EvmSigner.kt`, `SecureWalletDerivation.kt`, `BitcoinWalletManager.kt`. Remove `web3j`, `bitcoinj`, `bdk-android` from `build.gradle.kts`.
5. **Verification.** APK shrinks by ~15 MB (legacy stack was big). Public behavior identical to users.

Mnemonic storage location stays the same (`EncryptedSharedPreferences`). No user-facing migration step.

---

## ProGuard / R8

The auto-generated `JovaCore.kt` uses standard Kotlin idioms — no reflection, no dynamic class loading. Default R8 rules work.

We ship a `consumer-rules.pro` in the AAR that ensures the JNI binding classes aren't obfuscated:

```proguard
-keep class io.jova.core.** { *; }
-keepclasseswithmembernames class * {
    native <methods>;
}
```

Apps don't need to add anything beyond pulling the AAR.

---

## Common patterns

### Show the user their address

```kotlin
viewModelScope.launch {
    val address = withContext(Dispatchers.Default) {
        repo.address(JovaChain.Ethereum)
    }
    addressFlow.value = address.value   // "0xAbC1..." — already EIP-55 checksummed
}
```

### Validate an address the user typed

```kotlin
val valid = JovaWallet.isValidAddress(pasted, JovaChain.Ethereum)
if (!valid) showError("Invalid Ethereum address.")
```

### Sign a Bitcoin PSBT received from the backend

```kotlin
val unsigned = UnsignedTx.Bitcoin(psbtBase64 = backendResponse.psbt)
val signed = repo.signTransaction(unsigned)
broadcast(signed.rawHex)
```

### Sign EIP-712 typed data from a dApp

```kotlin
val msg = SignableMessage.EvmTypedDataV4(json = typedDataJson)
val signature = repo.signMessage(msg)
return signature.hex
```

---

## Don'ts

- Don't retain a `JovaWallet` between user actions.
- Don't forget `use { … }` — without it, the native handle leaks until the GC eventually finalizes (which may be never, on a healthy app).
- Don't pass the mnemonic `String` across module boundaries — pass `ByteArray` and zero it.
- Don't log `mnemonic.words`, `address.value`, or `signed.rawHex`.
- Don't share `JovaWallet` across threads or coroutines.
- Don't catch `JovaException(JovaError.Internal)` and silently retry. Internal errors mean SDK bugs.

---

## Telemetry guidelines

Safe to log:

- `JovaError` variant name (e.g., `MalformedUnsignedTx`).
- The `reason` string on errors (stable identifier).
- The `chain` value involved.
- SDK version (`JovaCore.VERSION`).

Never log:

- `mnemonic.words`.
- `address.value`.
- `signed.rawHex` or `signature.hex`.
- `unsignedTx` payload contents.
- Stack traces if they may include addresses or amounts.

---

## Sample app

`examples/android-sample/` is a complete Compose app demonstrating the full flow: import wallet → store in encrypted prefs → derive addresses → sign a demo EIP-1559 transaction → render result. Use it as a copy-and-modify reference for the production Android app's integration.
