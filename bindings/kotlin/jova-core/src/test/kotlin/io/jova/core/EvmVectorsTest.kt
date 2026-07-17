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
import uniffi.jova_core_ffi.UnsignedTx

class EvmVectorsTest {

    private fun loadVectors(): JSONArray {
        val raw = javaClass.getResourceAsStream("/test-vectors.json")!!
            .bufferedReader().readText()
        return JSONObject(raw).getJSONArray("vectors")
    }

    @Test
    fun evmAddressVectors() {
        val vectors = loadVectors()
        var ran = 0
        for (i in 0 until vectors.length()) {
            val v = vectors.getJSONObject(i)
            if (v.getString("kind") != "address") continue
            val input = v.getJSONObject("input")
            val chain = decodeChain(input.getJSONObject("chain"))
            // EVM-only in this test; non-EVM chains get their own test in Phase 2/3.
            when (chain) {
                JovaChain.Ethereum, JovaChain.Polygon, JovaChain.Bsc,
                JovaChain.Arbitrum, JovaChain.Optimism, JovaChain.Base,
                is JovaChain.CustomEvm -> { /* keep */ }
                else -> continue
            }

            val mnemonic = input.getString("mnemonic")
            val pass = if (input.has("passphrase")) input.getString("passphrase") else ""
            val expected = v.getJSONObject("expected").getString("address")

            val wallet = JovaWallet.fromMnemonic(mnemonic, pass)
            val got = wallet.address(chain, 0u)
            assertEquals(
                "address mismatch for vector ${v.getString("id")}",
                expected.lowercase(),
                got.value.lowercase()
            )
            ran++
        }
        assertTrue("No address vectors ran — check vector file", ran > 0)
    }

    @Test
    fun evmSignTxVectors() {
        val vectors = loadVectors()
        var ran = 0
        for (i in 0 until vectors.length()) {
            val v = vectors.getJSONObject(i)
            if (v.getString("kind") != "sign_tx") continue
            val input = v.getJSONObject("input")
            val mnemonic = input.getString("mnemonic")
            val pass = if (input.has("passphrase")) input.getString("passphrase") else ""
            val unsignedDict = input.getJSONObject("unsigned_tx")

            if (unsignedDict.getString("kind") != "evm") continue
            val unsigned = UnsignedTx.Evm(tx = decodeEvmUnsigned(unsignedDict))
            val expected = v.getJSONObject("expected")

            val wallet = JovaWallet.fromMnemonic(mnemonic, pass)
            val signed = wallet.signTx(unsigned, 0u)
            assertEquals(
                "signed_hex mismatch for vector ${v.getString("id")}",
                expected.getString("signed_hex").lowercase(),
                signed.rawHex.lowercase()
            )
            assertEquals(
                "tx_hash mismatch for vector ${v.getString("id")}",
                expected.getString("tx_hash").lowercase(),
                signed.txHash.lowercase()
            )
            ran++
        }
        assertTrue("No sign_tx vectors ran — check vector file", ran > 0)
    }

    @Test
    fun evmSignMessageVectors() {
        val vectors = loadVectors()
        var ran = 0
        for (i in 0 until vectors.length()) {
            val v = vectors.getJSONObject(i)
            if (v.getString("kind") != "sign_message") continue
            val input = v.getJSONObject("input")
            val mnemonic = input.getString("mnemonic")
            val pass = if (input.has("passphrase")) input.getString("passphrase") else ""
            val messageDict = input.getJSONObject("message")

            // Phase 2 added BTC sign_message vectors with `message.kind == "bitcoin"`;
            // those are exercised by BtcVectorsTest. Skip them here. (The decoder
            // used to throw on non-EVM kinds; now it returns Bitcoin variants too.)
            val msgKind = messageDict.getString("kind")
            if (msgKind != "evmPersonalSign" && msgKind != "evmTypedDataV4") continue

            val signable = try {
                decodeSignableMessage(messageDict)
            } catch (e: VectorDecodeException) {
                continue
            }

            val expectedSig = v.getJSONObject("expected").getString("signature").lowercase()

            val wallet = JovaWallet.fromMnemonic(mnemonic, pass)
            val sig = wallet.signMessage(signable, 0u)
            assertEquals(
                "signature mismatch for vector ${v.getString("id")}",
                expectedSig,
                sig.hex.lowercase()
            )
            ran++
        }
        assertTrue("No sign_message vectors ran — check vector file", ran > 0)
    }

    @Test
    fun evmErrorVectors() {
        val vectors = loadVectors()
        var ran = 0
        for (i in 0 until vectors.length()) {
            val v = vectors.getJSONObject(i)
            if (v.getString("kind") != "error") continue
            val input = v.getJSONObject("input")
            val mnemonic = input.getString("mnemonic")
            val pass = if (input.has("passphrase")) input.getString("passphrase") else ""
            val expected = v.getJSONObject("expected")
            val expectedVariant = expected.getString("error_variant")

            // All Phase 1 error vectors go through sign_tx with a malformed tx.
            if (!input.has("unsigned_tx")) continue
            val unsignedDict = input.getJSONObject("unsigned_tx")
            if (unsignedDict.getString("kind") != "evm") continue

            val unsigned = UnsignedTx.Evm(tx = decodeEvmUnsigned(unsignedDict))
            val wallet = JovaWallet.fromMnemonic(mnemonic, pass)
            try {
                wallet.signTx(unsigned, 0u)
                fail("Expected FfiException.$expectedVariant for vector ${v.getString("id")}, but no exception was thrown")
            } catch (err: FfiException) {
                val actualVariant = err::class.simpleName ?: "Unknown"
                assertEquals(
                    "Wrong error variant for vector ${v.getString("id")}",
                    expectedVariant,
                    actualVariant
                )
                ran++
            }
        }
        assertTrue("No error vectors ran — check vector file", ran > 0)
    }
}
