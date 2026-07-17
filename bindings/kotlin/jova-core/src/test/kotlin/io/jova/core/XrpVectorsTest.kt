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
 * Phase 3b XRP vector parity tests.
 *
 * Loads `spec/test-vectors.json` (copied into the JVM test resources by
 * `bindings/kotlin/scripts/build-aar.sh`) and iterates every `xrp.*` vector,
 * exercising the Kotlin uniffi binding through the same paths the Rust core
 * already verifies in `crates/jova-core/tests/vectors_xrp.rs`.
 *
 * Mirrors `BtcVectorsTest`. XRPL has no canonical message-signing scheme
 * equivalent to BIP-322 / EIP-191, so there is no `xrpSignMessageVectors`
 * test; XRP error coverage is purely transaction-side.
 *
 * Filters narrowly on `xrp` / `xrp.` discriminators so adding more XRP
 * vectors later doesn't pollute the BTC/EVM test runs (and vice-versa).
 */
class XrpVectorsTest {

    private fun loadVectors(): JSONArray {
        val raw = javaClass.getResourceAsStream("/test-vectors.json")!!
            .bufferedReader().readText()
        return JSONObject(raw).getJSONArray("vectors")
    }

    @Test
    fun xrpAddressVectors() {
        val vectors = loadVectors()
        var ran = 0
        for (i in 0 until vectors.length()) {
            val v = vectors.getJSONObject(i)
            if (v.getString("kind") != "address") continue
            val input = v.getJSONObject("input")
            if (input.getJSONObject("chain").getString("kind") != "xrp") continue

            val id = v.getString("id")
            val mnemonic = input.getString("mnemonic")
            val pass = if (input.has("passphrase")) input.getString("passphrase") else ""
            val expected = v.getJSONObject("expected").getString("address")

            val wallet = JovaWallet.fromMnemonic(mnemonic, pass)
            val got = wallet.address(JovaChain.Xrp, 0u)
            assertEquals(
                "XRP address mismatch for vector $id",
                expected,
                got.value
            )
            ran++
        }
        assertTrue("No XRP address vectors ran — check vector file", ran >= 1)
    }

    @Test
    fun xrpSignTxVectors() {
        val vectors = loadVectors()
        var ran = 0
        for (i in 0 until vectors.length()) {
            val v = vectors.getJSONObject(i)
            if (v.getString("kind") != "sign_tx") continue
            val input = v.getJSONObject("input")
            val unsignedDict = input.getJSONObject("unsigned_tx")
            if (unsignedDict.getString("kind") != "xrp") continue

            val id = v.getString("id")
            val mnemonic = input.getString("mnemonic")
            val pass = if (input.has("passphrase")) input.getString("passphrase") else ""
            val unsigned: UnsignedTx = decodeUnsignedTx(unsignedDict)
            val expectedHex = v.getJSONObject("expected").getString("signed_hex")
            val expectedHash = v.getJSONObject("expected").getString("tx_hash")

            val wallet = JovaWallet.fromMnemonic(mnemonic, pass)
            val signed = wallet.signTx(unsigned, 0u)

            // xrpl-py emits uppercase hex; the SDK normalizes to uppercase too.
            // Compare case-insensitively for resilience.
            assertEquals(
                "signed_hex mismatch for vector $id",
                expectedHex.uppercase(),
                signed.rawHex.uppercase()
            )
            assertEquals(
                "tx_hash mismatch for vector $id",
                expectedHash.uppercase(),
                signed.txHash.uppercase()
            )
            ran++
        }
        assertTrue("No XRP sign_tx vectors ran — check vector file", ran >= 2)
    }

    @Test
    fun xrpErrorVectors() {
        val vectors = loadVectors()
        var ran = 0
        for (i in 0 until vectors.length()) {
            val v = vectors.getJSONObject(i)
            if (v.getString("kind") != "error") continue
            val id = v.getString("id")
            if (!id.startsWith("xrp.")) continue

            val input = v.getJSONObject("input")
            val mnemonic = input.getString("mnemonic")
            val pass = if (input.has("passphrase")) input.getString("passphrase") else ""
            val expectedVariant = v.getJSONObject("expected").getString("error_variant")
            val expectedReason = v.getJSONObject("expected").getString("reason")

            val wallet = JovaWallet.fromMnemonic(mnemonic, pass)

            // XRP error vectors all carry an `unsigned_tx` (XRPL has no message
            // signing surface in this SDK), so the dispatch path is signTx.
            try {
                if (input.has("unsigned_tx")) {
                    val unsigned = decodeUnsignedTx(input.getJSONObject("unsigned_tx"))
                    wallet.signTx(unsigned, 0u)
                } else {
                    fail("XRP error vector $id must carry an unsigned_tx in input")
                }
                fail("Expected FfiException.$expectedVariant for vector $id, but no exception was thrown")
            } catch (err: FfiException) {
                val actualVariant = err::class.simpleName ?: "Unknown"
                assertEquals(
                    "Wrong error variant for vector $id",
                    expectedVariant,
                    actualVariant
                )
                // FfiError is `flat_error` in uniffi so the variant carries no
                // structured `reason` field — the reason gets stringified into
                // the Exception message via thiserror. Verify it appears so a
                // regression in the wrapping shows up.
                val msg = err.message ?: ""
                assertTrue(
                    "Error message for $id should contain reason '$expectedReason', got: $msg",
                    msg.contains(expectedReason)
                )
                ran++
            }
        }
        assertTrue("No XRP error vectors ran — check vector file", ran >= 2)
    }
}
