package io.jova.core

import org.junit.Test
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.json.JSONObject
import org.json.JSONArray
import uniffi.jova_core_ffi.FfiException
import uniffi.jova_core_ffi.JovaChain
import uniffi.jova_core_ffi.JovaWallet
import uniffi.jova_core_ffi.SignableMessage
import uniffi.jova_core_ffi.UnsignedTx

/**
 * Phase 3c Solana vector parity tests.
 *
 * Loads `spec/test-vectors.json` (copied into the JVM test resources by
 * `bindings/kotlin/scripts/build-aar.sh`) and iterates every `sol.*` vector,
 * exercising the Kotlin uniffi binding through the same paths the Rust core
 * already verifies in `crates/jova-core/tests/vectors_sol.rs`.
 *
 * Mirrors `XrpVectorsTest`. Includes both transaction and message coverage,
 * plus error vectors that exercise MalformedUnsignedTx and
 * MalformedSignableMessage variants.
 *
 * Filters narrowly on `solana` / `sol.` discriminators so SOL vectors don't
 * leak into the BTC/EVM/XRP test runs (and vice-versa).
 */
class SolVectorsTest {

    private fun loadVectors(): JSONArray {
        val raw = javaClass.getResourceAsStream("/test-vectors.json")!!
            .bufferedReader().readText()
        return JSONObject(raw).getJSONArray("vectors")
    }

    @Test
    fun solAddressVectors() {
        val vectors = loadVectors()
        var ran = 0
        for (i in 0 until vectors.length()) {
            val v = vectors.getJSONObject(i)
            if (v.getString("kind") != "address") continue
            val input = v.getJSONObject("input")
            if (input.getJSONObject("chain").getString("kind") != "solana") continue

            val id = v.getString("id")
            val mnemonic = input.getString("mnemonic")
            val pass = if (input.has("passphrase")) input.getString("passphrase") else ""
            val expected = v.getJSONObject("expected").getString("address")

            val wallet = JovaWallet.fromMnemonic(mnemonic, pass)
            val got = wallet.address(JovaChain.Solana, 0u)
            assertEquals(
                "SOL address mismatch for vector $id",
                expected,
                got.value
            )
            ran++
        }
        assertTrue("No SOL address vectors ran — check vector file", ran >= 1)
    }

    @Test
    fun solSignTxVectors() {
        val vectors = loadVectors()
        var ran = 0
        for (i in 0 until vectors.length()) {
            val v = vectors.getJSONObject(i)
            if (v.getString("kind") != "sign_tx") continue
            val input = v.getJSONObject("input")
            val unsignedDict = input.getJSONObject("unsigned_tx")
            if (unsignedDict.getString("kind") != "solana") continue

            val id = v.getString("id")
            val mnemonic = input.getString("mnemonic")
            val pass = if (input.has("passphrase")) input.getString("passphrase") else ""
            val unsigned: UnsignedTx = decodeUnsignedTx(unsignedDict)
            val expectedHex = v.getJSONObject("expected").getString("signed_hex")
            val expectedHash = v.getJSONObject("expected").getString("tx_hash")

            val wallet = JovaWallet.fromMnemonic(mnemonic, pass)
            val signed = wallet.signTx(unsigned, 0u)

            assertEquals("signed_hex mismatch for vector $id", expectedHex, signed.rawHex)
            assertEquals("tx_hash mismatch for vector $id", expectedHash, signed.txHash)
            ran++
        }
        assertTrue("No SOL sign_tx vectors ran — check vector file", ran >= 2)
    }

    @Test
    fun solSignMessageVectors() {
        val vectors = loadVectors()
        var ran = 0
        for (i in 0 until vectors.length()) {
            val v = vectors.getJSONObject(i)
            if (v.getString("kind") != "sign_message") continue
            val input = v.getJSONObject("input")
            val msgDict = input.getJSONObject("message")
            if (msgDict.getString("kind") != "solana") continue

            val id = v.getString("id")
            val mnemonic = input.getString("mnemonic")
            val pass = if (input.has("passphrase")) input.getString("passphrase") else ""
            val msg: SignableMessage = decodeSignableMessage(msgDict)
            // signature_hex carries the base58 string per the cross-phase
            // convention of using `signature_hex` regardless of encoding.
            val expected = v.getJSONObject("expected").getString("signature_hex")

            val wallet = JovaWallet.fromMnemonic(mnemonic, pass)
            val sig = wallet.signMessage(msg, 0u)

            assertEquals("SOL signature mismatch for vector $id", expected, sig.hex)
            ran++
        }
        assertTrue("No SOL sign_message vectors ran — check vector file", ran >= 1)
    }

    @Test
    fun solErrorVectors() {
        val vectors = loadVectors()
        var ran = 0
        for (i in 0 until vectors.length()) {
            val v = vectors.getJSONObject(i)
            if (v.getString("kind") != "error") continue
            val id = v.getString("id")
            if (!id.startsWith("sol.")) continue

            val input = v.getJSONObject("input")
            val mnemonic = input.getString("mnemonic")
            val pass = if (input.has("passphrase")) input.getString("passphrase") else ""
            val expectedVariant = v.getJSONObject("expected").getString("error_variant")
            val expectedReason = v.getJSONObject("expected").getString("reason")

            val wallet = JovaWallet.fromMnemonic(mnemonic, pass)
            try {
                when {
                    input.has("unsigned_tx") -> {
                        val unsigned = decodeUnsignedTx(input.getJSONObject("unsigned_tx"))
                        wallet.signTx(unsigned, 0u)
                    }
                    input.has("message") -> {
                        val msg = decodeSignableMessage(input.getJSONObject("message"))
                        wallet.signMessage(msg, 0u)
                    }
                    else -> fail("SOL error vector $id must carry an unsigned_tx or message in input")
                }
                fail("Expected FfiException.$expectedVariant for vector $id, but no exception was thrown")
            } catch (err: FfiException) {
                val actualVariant = err::class.simpleName ?: "Unknown"
                assertEquals(
                    "Wrong error variant for vector $id",
                    expectedVariant,
                    actualVariant
                )
                val msg = err.message ?: ""
                assertTrue(
                    "Error message for $id should contain reason '$expectedReason', got: $msg",
                    msg.contains(expectedReason)
                )
                ran++
            }
        }
        assertTrue("No SOL error vectors ran — check vector file", ran >= 4)
    }
}
