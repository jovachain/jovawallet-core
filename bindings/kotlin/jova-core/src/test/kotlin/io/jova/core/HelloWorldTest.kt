package io.jova.core

import org.junit.Test
import org.junit.Assert.assertEquals
import org.json.JSONObject
import uniffi.jova_core_ffi.isValidMnemonic

class HelloWorldTest {
    @Test
    fun negativeMnemonicValidationVector() {
        val json = JSONObject(
            javaClass.getResourceAsStream("/test-vectors.json")!!.bufferedReader().readText()
        )
        val vectors = json.getJSONArray("vectors")
        var v: JSONObject? = null
        for (i in 0 until vectors.length()) {
            val candidate = vectors.getJSONObject(i)
            if (candidate.getString("id") == "phase0.mnemonic_validation_neg.gibberish") {
                v = candidate; break
            }
        }
        requireNotNull(v) { "vector phase0.mnemonic_validation_neg.gibberish missing" }

        val input = v.getJSONObject("input")
        val words = input.getString("words")
        val passphrase = if (input.has("passphrase")) input.getString("passphrase") else ""
        val expected = v.getJSONObject("expected").getBoolean("valid")

        assertEquals(expected, isValidMnemonic(words, passphrase))
    }
}
