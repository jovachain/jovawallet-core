// WalletRepository — the integration layer that wraps JovaWallet.
//
// Mirrors the production Android app's intended shape per
// docs/integration-android.md. Real apps:
//
//   1. Read the mnemonic from EncryptedSharedPreferences (via the Tink/Jetpack
//      EncryptedSharedPreferences API) into a clearable ByteArray. This
//      sample uses a literal for demo purposes ONLY — never ship a
//      hardcoded mnemonic.
//   2. Construct JovaWallet per signing call; release immediately after.
//      JovaWallet implements AutoCloseable via uniffi; use `.use {}`
//      so the underlying Rust Seed is dropped (and zeroized) deterministically.
//   3. Each chain call site is gated behind a server-controlled feature flag
//      (see Phase 4 plan, section "Per-chain rollout cadence").

package io.jovachain.sample

import uniffi.jova_core_ffi.Address
import uniffi.jova_core_ffi.BtcMsgScheme
import uniffi.jova_core_ffi.EvmUnsigned
import uniffi.jova_core_ffi.FfiException
import uniffi.jova_core_ffi.JovaChain
import uniffi.jova_core_ffi.JovaWallet
import uniffi.jova_core_ffi.SignableMessage
import uniffi.jova_core_ffi.SignedTx
import uniffi.jova_core_ffi.Signature
import uniffi.jova_core_ffi.UnsignedTx

/** Errors surfaced to UI code. Wraps the FfiException variants with the chain hint. */
sealed class WalletError(message: String) : Exception(message) {
    object InvalidMnemonic : WalletError("invalid mnemonic")
    object InvalidPassphrase : WalletError("invalid passphrase")
    class InvalidAddress(val chain: JovaChain) : WalletError("invalid address for $chain")
    class UnsupportedChain(val chain: JovaChain) : WalletError("unsupported chain: $chain")
    class MalformedUnsignedTx(val reason: String) : WalletError("malformed unsigned tx: $reason")
    class MalformedSignableMessage(val reason: String) : WalletError("malformed signable message: $reason")
    class SigningFailed(val reason: String) : WalletError("signing failed: $reason")
    class Internal(val reason: String) : WalletError("internal: $reason")

    companion object {
        fun from(ffi: FfiException): WalletError = when (ffi) {
            is FfiException.InvalidMnemonic -> InvalidMnemonic
            is FfiException.InvalidPassphrase -> InvalidPassphrase
            is FfiException.InvalidAddress -> InvalidAddress(ffi.chain)
            is FfiException.UnsupportedChain -> UnsupportedChain(ffi.chain)
            is FfiException.MalformedUnsignedTx -> MalformedUnsignedTx(ffi.reason)
            is FfiException.MalformedSignableMessage -> MalformedSignableMessage(ffi.reason)
            is FfiException.SigningFailed -> SigningFailed(ffi.reason)
            is FfiException.Internal -> Internal(ffi.reason)
        }
    }
}

class WalletRepository(
    private val mnemonic: String,
    private val passphrase: String = "",
) {
    /** Derive the canonical address for a chain. In production, gate behind FeatureFlag.useJovaCoreFor<Chain>. */
    fun address(chain: JovaChain): Address = withWallet { it.address(chain, 0u) }

    /** Sign an EIP-1559 EVM transaction. */
    fun signEvm(tx: EvmUnsigned): SignedTx = withWallet { it.signTx(UnsignedTx.Evm(tx)) }

    /**
     * Sign a Bitcoin PSBT. If the returned [SignedTx.rawHex] starts with `psbt:`,
     * the wallet returned an updated PSBT (multi-party flow); the production app
     * must hand it to the next signer rather than broadcast. For single-party
     * flows the rawHex is the broadcast-ready transaction hex.
     */
    fun signBitcoin(psbtBase64: String): SignedTx =
        withWallet { it.signTx(UnsignedTx.Bitcoin(psbtBase64 = psbtBase64)) }

    /** Sign an XRPL transaction (JSON-encoded). */
    fun signXrp(txJson: String): SignedTx =
        withWallet { it.signTx(UnsignedTx.Xrp(txJson = txJson)) }

    /** Sign a Solana VersionedTransaction (v0). */
    fun signSolana(messageBase64: String, recentBlockhash: String): SignedTx =
        withWallet { it.signTx(UnsignedTx.Solana(messageBase64 = messageBase64, recentBlockhash = recentBlockhash)) }

    /** Sign a personal_sign EIP-191 message. */
    fun signEvmPersonal(message: String): Signature =
        withWallet { it.signMessage(SignableMessage.EvmPersonalSign(message)) }

    /** Sign an EIP-712 typed-data message. */
    fun signEvmTypedData(json: String): Signature =
        withWallet { it.signMessage(SignableMessage.EvmTypedDataV4(json)) }

    /** Sign a BIP-322 or legacy Bitcoin message. */
    fun signBitcoinMessage(message: String, address: String, scheme: BtcMsgScheme): Signature =
        withWallet { it.signMessage(SignableMessage.Bitcoin(message = message, address = address, scheme = scheme)) }

    /** Sign raw bytes for Solana (no canonical message scheme). */
    fun signSolanaMessage(messageBase64: String): Signature =
        withWallet { it.signMessage(SignableMessage.Solana(messageBase64 = messageBase64)) }

    // --- internals ---

    private inline fun <R> withWallet(block: (JovaWallet) -> R): R {
        val wallet = try {
            JovaWallet.fromMnemonic(mnemonic, passphrase)
        } catch (e: FfiException) {
            throw WalletError.from(e)
        }
        return try {
            wallet.use(block)
        } catch (e: FfiException) {
            throw WalletError.from(e)
        }
    }
}
