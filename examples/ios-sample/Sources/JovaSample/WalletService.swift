// WalletService — the integration layer that wraps JovaWallet.
//
// Mirrors the production iOS app's intended shape per
// docs/integration-ios.md. Real apps:
//
//   1. Read the mnemonic from Keychain (kSecAttrAccessibleWhenUnlockedThisDeviceOnly)
//      into a clearable Data. This sample uses a literal for demo purposes
//      ONLY — never ship a hardcoded mnemonic.
//   2. Construct JovaWallet per signing call; release immediately after.
//      JovaWallet owns the Seed; freeing the wallet zeroizes the seed bytes.
//   3. Each chain call site is gated behind a server-controlled feature flag
//      (see Phase 4 plan, section "Per-chain rollout cadence").

import Foundation
import JovaCore

/// Errors surfaced to UI code. Wraps the FfiException variants with a chain
/// hint for the per-chain telemetry the production app emits.
enum WalletError: Error {
    case invalidMnemonic
    case invalidPassphrase
    case invalidAddress(chain: JovaChain)
    case unsupportedChain(chain: JovaChain)
    case malformedUnsignedTx(reason: String)
    case malformedSignableMessage(reason: String)
    case signingFailed(reason: String)
    case `internal`(reason: String)

    init(_ ffi: FfiException) {
        switch ffi {
        case .InvalidMnemonic:          self = .invalidMnemonic
        case .InvalidPassphrase:        self = .invalidPassphrase
        case let .InvalidAddress(chain): self = .invalidAddress(chain: chain)
        case let .UnsupportedChain(chain): self = .unsupportedChain(chain: chain)
        case let .MalformedUnsignedTx(reason): self = .malformedUnsignedTx(reason: reason)
        case let .MalformedSignableMessage(reason): self = .malformedSignableMessage(reason: reason)
        case let .SigningFailed(reason): self = .signingFailed(reason: reason)
        case let .Internal(reason):     self = .internal(reason: reason)
        }
    }
}

final class WalletService {
    private let mnemonic: String
    private let passphrase: String

    /// In production, init from Keychain bytes — never a literal.
    init(mnemonic: String, passphrase: String = "") {
        self.mnemonic = mnemonic
        self.passphrase = passphrase
    }

    /// Derive the canonical address for a chain. In production, gate the call
    /// site behind FeatureFlag.useJovaCoreFor<Chain>.
    func address(chain: JovaChain) throws -> Address {
        let wallet: JovaWallet
        do {
            wallet = try JovaWallet.fromMnemonic(words: mnemonic, passphrase: passphrase)
        } catch let e as FfiException {
            throw WalletError(e)
        }
        do {
            return try wallet.address(chain: chain, account: 0)
        } catch let e as FfiException {
            throw WalletError(e)
        }
    }

    /// Sign an EIP-1559 EVM transaction.
    func signEvm(tx: EvmUnsigned) throws -> SignedTx {
        try sign(unsigned: .evm(tx: tx))
    }

    /// Sign a Bitcoin PSBT. If the returned SignedTx.rawHex starts with
    /// "psbt:", the wallet returned an updated PSBT (multi-party flow);
    /// the production app must hand it to the next signer rather than
    /// broadcast. For single-party flows the rawHex is the broadcast-ready
    /// transaction hex.
    func signBitcoin(psbtBase64: String) throws -> SignedTx {
        try sign(unsigned: .bitcoin(psbtBase64: psbtBase64))
    }

    /// Sign an XRPL transaction (JSON encoded).
    func signXrp(txJson: String) throws -> SignedTx {
        try sign(unsigned: .xrp(txJson: txJson))
    }

    /// Sign a Solana VersionedTransaction (v0). The unsigned message is the
    /// bincode-serialized solana_message::VersionedMessage, base64-encoded.
    func signSolana(messageBase64: String, recentBlockhash: String) throws -> SignedTx {
        try sign(unsigned: .solana(messageBase64: messageBase64, recentBlockhash: recentBlockhash))
    }

    /// Sign a personal_sign EIP-191 message.
    func signEvmPersonal(message: String) throws -> Signature {
        try signMessage(msg: .evmPersonalSign(message: message))
    }

    /// Sign an EIP-712 typed-data message.
    func signEvmTypedData(json: String) throws -> Signature {
        try signMessage(msg: .evmTypedDataV4(json: json))
    }

    /// Sign a BIP-322 or legacy Bitcoin message.
    func signBitcoinMessage(message: String, address: String, scheme: BtcMsgScheme) throws -> Signature {
        try signMessage(msg: .bitcoin(message: message, address: address, scheme: scheme))
    }

    /// Sign raw bytes for Solana (no canonical message scheme).
    func signSolanaMessage(messageBase64: String) throws -> Signature {
        try signMessage(msg: .solana(messageBase64: messageBase64))
    }

    // MARK: - Internals

    private func sign(unsigned: UnsignedTx) throws -> SignedTx {
        let wallet: JovaWallet
        do {
            wallet = try JovaWallet.fromMnemonic(words: mnemonic, passphrase: passphrase)
        } catch let e as FfiException {
            throw WalletError(e)
        }
        do {
            return try wallet.signTx(tx: unsigned)
        } catch let e as FfiException {
            throw WalletError(e)
        }
    }

    private func signMessage(msg: SignableMessage) throws -> Signature {
        let wallet: JovaWallet
        do {
            wallet = try JovaWallet.fromMnemonic(words: mnemonic, passphrase: passphrase)
        } catch let e as FfiException {
            throw WalletError(e)
        }
        do {
            return try wallet.signMessage(msg: msg)
        } catch let e as FfiException {
            throw WalletError(e)
        }
    }
}
