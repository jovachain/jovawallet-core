# Integration: iOS

How the iOS Jova app consumes `jovawallet-core`. This guide is the canonical reference for the integration; it's also referenced by `examples/ios-sample/`.

The iOS app gets the SDK as a SwiftPM package. The package contains:

- A binary `JovaCore.xcframework` (the Rust core, compiled for iOS device, iOS simulator, and macOS).
- A generated `JovaCore.swift` file with the public API.
- A small `Convenience.swift` ergonomics layer.

## Adding the dependency

In Xcode → Project → Package Dependencies → `+` → enter:

```
https://github.com/jovachain/jovawallet-core-swift.git
```

Pin to a tagged version:

```swift
// Package.swift (if your app has one)
dependencies: [
    .package(url: "https://github.com/jovachain/jovawallet-core-swift.git", from: "1.0.0")
]
```

Add `JovaCore` as a target dependency. That's it — no `import TrustWalletCore`, no manual framework setup, no podfile edits.

## Minimum platform versions

- iOS 14+
- macOS 11+ (for any macOS-targeted Jova UI; not required for the iPhone app)
- Xcode 15+

These match what the underlying XCFramework is built for. Bumping these is a major version of the SDK.

## The Hello-World

```swift
import JovaCore

func helloWorld() throws {
    let mnemonic = JovaWallet.createMnemonic(strength: .bits128)
    print("Generated 12 words: \(mnemonic.words)")

    let wallet = try JovaWallet(mnemonic: mnemonic)
    let ethAddress = try wallet.address(on: .ethereum)
    let btcAddress = try wallet.address(on: .bitcoin)

    print("ETH: \(ethAddress.value)")
    print("BTC: \(btcAddress.value)")
    // wallet deinits at end of scope; seed zeroized
}
```

That's all you need to ship. The rest of this doc is about doing it right.

---

## The recommended app architecture

```
┌─────────────────────────────────────────────────────────┐
│                  iOS Jova App                            │
│                                                          │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ UI layer (SwiftUI)                                  │ │
│ │  - SignTransactionView                              │ │
│ │  - ImportWalletView                                 │ │
│ └─────────────────────────────────────────────────────┘ │
│                       ▲                                 │
│                       │ ViewModel calls                 │
│                       ▼                                 │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ WalletService (app-owned facade)                    │ │
│ │  - storeMnemonic(_:) → Keychain                     │ │
│ │  - signTransaction(_:) → JovaWallet                 │ │
│ │  - signMessage(_:) → JovaWallet                     │ │
│ └─────────────────────────────────────────────────────┘ │
│                       ▲                                 │
│                       │ uses                            │
│                       ▼                                 │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ Keychain (iOS Security framework)                   │ │
│ │  - mnemonic stored as Data with .biometric ACL      │ │
│ └─────────────────────────────────────────────────────┘ │
│                       ▲                                 │
│                       │ reads on demand                 │
│                       ▼                                 │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ JovaCore (this SDK)                                 │ │
│ │  - JovaWallet(mnemonic: …)                          │ │
│ │  - signTx(unsigned: …)                              │ │
│ │  - signMessage(msg: …)                              │ │
│ └─────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

Notes:

- `WalletService` is the app's facade. The UI never touches `JovaCore` directly.
- The mnemonic lives in Keychain. Each signing operation reads it, constructs a `JovaWallet`, signs, and lets the wallet deinit.
- `WalletService` is **not** a singleton holding a long-lived `JovaWallet`. Long-lived wallets keep secret material in memory longer than necessary.

---

## `WalletService` reference implementation

```swift
import Foundation
import JovaCore
import Security

public final class WalletService {

    public init() {}

    // MARK: - Mnemonic storage

    public func storeMnemonic(_ mnemonic: Mnemonic) throws {
        let data = mnemonic.words.data(using: .utf8)!
        let acl = SecAccessControlCreateWithFlags(
            kCFAllocatorDefault,
            kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
            .biometryCurrentSet,
            nil
        )!
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: "io.jovachain.wallet",
            kSecAttrAccount as String: "primary",
            kSecValueData as String: data,
            kSecAttrAccessControl as String: acl,
        ]
        SecItemDelete(query as CFDictionary)
        let status = SecItemAdd(query as CFDictionary, nil)
        guard status == errSecSuccess else { throw KeychainError.storeFailed(status) }
    }

    private func loadMnemonicWords() throws -> Data {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: "io.jovachain.wallet",
            kSecAttrAccount as String: "primary",
            kSecReturnData as String: true,
            kSecMatchLimit as String: kSecMatchLimitOne,
        ]
        var item: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &item)
        guard status == errSecSuccess, let data = item as? Data else {
            throw KeychainError.loadFailed(status)
        }
        return data
    }

    // MARK: - Signing

    /// Builds a fresh JovaWallet, signs, and deinits it. The wallet is never retained.
    public func signTransaction(_ tx: UnsignedTx) throws -> SignedTx {
        var bytes = try loadMnemonicWords()
        defer { bytes.resetBytes(in: 0..<bytes.count) }
        let buf = MnemonicBuffer(bytes: bytes, passphrase: Data())
        let wallet = try JovaWallet(mnemonic: buf)
        defer { /* wallet deinit zeroes Rust-side seed */ }
        return try wallet.signTx(unsigned: tx)
    }

    public func signMessage(_ msg: SignableMessage) throws -> Signature {
        var bytes = try loadMnemonicWords()
        defer { bytes.resetBytes(in: 0..<bytes.count) }
        let buf = MnemonicBuffer(bytes: bytes, passphrase: Data())
        let wallet = try JovaWallet(mnemonic: buf)
        return try wallet.signMessage(msg: msg)
    }

    public func address(for chain: JovaChain) throws -> Address {
        var bytes = try loadMnemonicWords()
        defer { bytes.resetBytes(in: 0..<bytes.count) }
        let buf = MnemonicBuffer(bytes: bytes, passphrase: Data())
        let wallet = try JovaWallet(mnemonic: buf)
        return try wallet.address(on: chain)
    }
}

public enum KeychainError: Error {
    case storeFailed(OSStatus)
    case loadFailed(OSStatus)
}
```

Things this code is doing on purpose:

- Reads mnemonic bytes into a mutable `Data`, zeroes via `resetBytes` on `defer`.
- Uses `MnemonicBuffer` so the SDK gets bytes (not a `String`) — bytes can be cleared.
- Constructs a fresh `JovaWallet` per call. Doesn't share across calls.
- No retained wallet anywhere in the type.
- No background queue retention — let the caller pick a queue.

---

## Threading

`JovaWallet` is **not** thread-safe. Do not share an instance across threads.

The recommended pattern: do the signing call on a background queue, marshal results back to main.

```swift
import JovaCore

func signOnBackground(_ tx: UnsignedTx) async throws -> SignedTx {
    return try await Task.detached(priority: .userInitiated) {
        try WalletService().signTransaction(tx)
    }.value
}
```

Wallet construction is fast (microseconds). Signing is fast (single-digit milliseconds). The main thread is fine for development; production should still go to background to keep the UI responsive in case of unexpected delays (e.g., CSPRNG on first call after wake).

---

## WalletConnect integration

The app speaks WalletConnect. When a dApp sends a `personal_sign` or `eth_sendTransaction` request:

1. App parses the WalletConnect message.
2. App displays the request to the user, shows what they're signing.
3. On user approval, app constructs the `UnsignedTx` or `SignableMessage`.
4. App calls `WalletService().signTransaction(tx)`.
5. App sends the resulting `SignedTx.rawHex` back to the dApp via WalletConnect — *or* hands `SignedTx.rawHex` to the backend for broadcast, depending on the protocol method.

The SDK does not know about WalletConnect. The app does.

```swift
func handleWalletConnectRequest(_ request: WCRequest) async throws -> WCResponse {
    switch request.method {
    case "personal_sign":
        let msg = SignableMessage.evmPersonalSign(message: request.params.message)
        let sig = try WalletService().signMessage(msg)
        return WCResponse.signature(sig.hex)

    case "eth_sendTransaction":
        let tx = try parseAsUnsignedEvm(request.params)
        let signed = try WalletService().signTransaction(tx)
        return WCResponse.signedTx(signed.rawHex)

    default:
        throw WCError.unsupportedMethod
    }
}
```

---

## Migration from legacy `CryptoWalletService.swift`

The legacy iOS app used Trust Wallet Core directly. The migration plan:

1. **Phase 3 of `plan.md`** — both legacy and new code paths exist. New code routes through `WalletService`; old code through `CryptoWalletService`.
2. **Per-chain cutover.** ETH first (most-used and most-tested), then BTC, SOL, XRP. After each cutover, validate against `spec/test-vectors.json` plus a manual mainnet smoke test with small amounts.
3. **Removal.** Once every call site is migrated, delete `CryptoWalletService.swift` and remove the `TrustWalletCore` SwiftPM dependency.
4. **Verification.** App size should shrink by several MB (TWC was a large binary). Public-facing behavior identical.

The migration does not require users to take any action. Mnemonic storage location, derivation paths, and addresses all match (BIP-84 was the standard the legacy app intended to use; it just hadn't fully wired BTC).

---

## Common patterns

### Show the user their address

```swift
let address = try WalletService().address(for: .ethereum)
addressLabel.text = address.value   // "0xAbC1...xyz" — already EIP-55 checksummed
```

### Validate an address the user typed

```swift
let valid = JovaWallet.isValidAddress(string: pasted, on: .ethereum)
if !valid { showError("Invalid Ethereum address.") }
```

### Sign a Bitcoin PSBT received from the backend

```swift
let unsigned = UnsignedTx.bitcoin(psbtBase64: backendResponse.psbt)
let signed = try WalletService().signTransaction(unsigned)
broadcast(signed.rawHex)
```

### Sign EIP-712 typed data from a dApp

```swift
let msg = SignableMessage.evmTypedDataV4(json: typedDataJSON)
let signature = try WalletService().signMessage(msg)
return signature.hex
```

---

## Don'ts

- Don't retain a `JovaWallet` between user actions.
- Don't pass the mnemonic `String` across module boundaries — pass `Data` and clear it.
- Don't log `mnemonic.words`, `wallet.address`, or `signed.rawHex` (the last is PII once associated with a tx).
- Don't share `JovaWallet` across threads.
- Don't assume `Mnemonic` clearing protects you fully — read `memory-and-keys.md` for the honest disclaimer.
- Don't catch `JovaError.internal` and silently retry. Internal errors mean SDK bugs; surface them and report telemetry.

---

## Telemetry guidelines

Safe to log:

- `JovaError` variant name (e.g., `malformedUnsignedTx`).
- The `reason` string on errors (it's a stable identifier, not user data).
- The `chain` value involved.
- SDK version (`JovaCore.version`).

Never log:

- `mnemonic.words`.
- `address.value` (PII).
- `signed.rawHex` or `signature.hex` (PII once linked to a user).
- `unsignedTx` payload contents.
- Exception backtraces if they include addresses or amounts.

---

## Sample app

`examples/ios-sample/` is a complete SwiftUI app demonstrating the full flow: import wallet → store in Keychain → derive addresses → sign a demo EIP-1559 transaction → render result. Use it as a copy-and-modify reference for the production iOS app's integration.
