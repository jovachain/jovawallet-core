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

/**
 * Phase 2 Bitcoin vector parity tests.
 *
 * Loads `spec/test-vectors.json` (copied into the JVM test resources by
 * `bindings/kotlin/scripts/build-aar.sh`) and iterates every `btc.*` vector,
 * exercising the Kotlin uniffi binding through the same paths the Rust core
 * already verifies in `crates/jova-core/tests/vectors_btc.rs`.
 *
 * Mirrors `EvmVectorsTest`; filters narrowly on `bitcoin` discriminators so
 * adding more BTC vectors later doesn't pollute the EVM test runs.
 */
class BtcVectorsTest {

    private fun loadVectors(): JSONArray {
        val raw = javaClass.getResourceAsStream("/test-vectors.json")!!
            .bufferedReader().readText()
        return JSONObject(raw).getJSONArray("vectors")
    }

    @Test
    fun btcAddressVectors() {
        val vectors = loadVectors()
        var ran = 0
        for (i in 0 until vectors.length()) {
            val v = vectors.getJSONObject(i)
            if (v.getString("kind") != "address") continue
            val input = v.getJSONObject("input")
            if (input.getJSONObject("chain").getString("kind") != "bitcoin") continue

            // `JovaWallet.address` ignores its `account` argument today and
            // always derives m/84'/0'/0'/0/0. Skip address-index >0 vectors;
            // the spec keeps them for the day the API grows an index arg.
            val id = v.getString("id")
            if (!id.endsWith("_account0_index0")) continue

            val mnemonic = input.getString("mnemonic")
            val pass = if (input.has("passphrase")) input.getString("passphrase") else ""
            val expected = v.getJSONObject("expected").getString("address")

            val wallet = JovaWallet.fromMnemonic(mnemonic, pass)
            val got = wallet.address(JovaChain.Bitcoin, 0u)
            assertEquals(
                "address mismatch for vector $id",
                expected,
                got.value
            )
            ran++
        }
        assertTrue("No BTC address vectors ran — check vector file", ran >= 3)
    }

    @Test
    fun btcSignTxVectors() {
        val vectors = loadVectors()
        var ran = 0
        for (i in 0 until vectors.length()) {
            val v = vectors.getJSONObject(i)
            if (v.getString("kind") != "sign_tx") continue
            val input = v.getJSONObject("input")
            val unsignedDict = input.getJSONObject("unsigned_tx")
            if (unsignedDict.getString("kind") != "bitcoin") continue

            val id = v.getString("id")
            val mnemonic = input.getString("mnemonic")
            val pass = if (input.has("passphrase")) input.getString("passphrase") else ""
            val unsigned: UnsignedTx = decodeUnsignedTx(unsignedDict)
            val expectedHex = v.getJSONObject("expected").getString("signed_hex")

            val wallet = JovaWallet.fromMnemonic(mnemonic, pass)
            val signed = wallet.signTx(unsigned, 0u)

            // The captured value for multi-party PSBTs is `psbt:<base64>`;
            // for finalized owned PSBTs it's pure lowercase hex. Compare
            // case-insensitively to absorb capture-tool casing differences.
            assertEquals(
                "signed_hex mismatch for vector $id",
                expectedHex.lowercase(),
                signed.rawHex.lowercase()
            )
            ran++
        }
        assertTrue("No BTC sign_tx vectors ran — check vector file", ran >= 3)
    }

    @Test
    fun btcSignMessageVectors() {
        val vectors = loadVectors()
        var ran = 0
        for (i in 0 until vectors.length()) {
            val v = vectors.getJSONObject(i)
            if (v.getString("kind") != "sign_message") continue
            val input = v.getJSONObject("input")
            val messageDict = input.getJSONObject("message")
            if (messageDict.getString("kind") != "bitcoin") continue

            val id = v.getString("id")
            val mnemonic = input.getString("mnemonic")
            val pass = if (input.has("passphrase")) input.getString("passphrase") else ""
            val signable = decodeSignableMessage(messageDict)
            // Spec field name is `signature_hex` even though BTC carries base64.
            val expectedSig = v.getJSONObject("expected").getString("signature_hex")

            val wallet = JovaWallet.fromMnemonic(mnemonic, pass)
            val sig = wallet.signMessage(signable, 0u)
            // base64 is case-sensitive; compare exactly.
            assertEquals("signature mismatch for vector $id", expectedSig, sig.hex)
            ran++
        }
        assertTrue("No BTC sign_message vectors ran — check vector file", ran >= 2)
    }

    @Test
    fun btcErrorVectors() {
        val vectors = loadVectors()
        var ran = 0
        for (i in 0 until vectors.length()) {
            val v = vectors.getJSONObject(i)
            if (v.getString("kind") != "error") continue
            val id = v.getString("id")
            if (!id.startsWith("btc.")) continue

            val input = v.getJSONObject("input")
            val mnemonic = input.getString("mnemonic")
            val pass = if (input.has("passphrase")) input.getString("passphrase") else ""
            val expectedVariant = v.getJSONObject("expected").getString("error_variant")
            val expectedReason = v.getJSONObject("expected").getString("reason")

            val wallet = JovaWallet.fromMnemonic(mnemonic, pass)

            // BTC error vectors split: PSBT-shaped go through signTx, message-
            // shaped through signMessage. Pick the path by which key is present.
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
                    else -> fail("vector $id has neither unsigned_tx nor message in input")
                }
                fail("Expected FfiException.$expectedVariant for vector $id, but no exception was thrown")
            } catch (err: FfiException) {
                val actualVariant = err::class.simpleName ?: "Unknown"
                assertEquals(
                    "Wrong error variant for vector $id",
                    expectedVariant,
                    actualVariant
                )
                // FfiError is `flat_error` in uniffi so the variant has no
                // `reason` field — the reason gets stringified into the
                // Exception message via thiserror, prefixed with the variant
                // description. Verify the reason appears in the message so a
                // regression in the wrapping shows up.
                val msg = err.message ?: ""
                assertTrue(
                    "Error message for $id should contain reason '$expectedReason', got: $msg",
                    msg.contains(expectedReason)
                )
                ran++
            }
        }
        assertTrue("No BTC error vectors ran — check vector file", ran >= 3)
    }
}
