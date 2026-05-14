// MainActivity — Compose UI demo. Derives one address per chain on Tap.
//
// Production code reads the mnemonic from EncryptedSharedPreferences — never
// hardcoded. The literal here is the BIP-39 abandon-about test vector, which
// has no funds attached and is published in every wallet spec.

package io.jovachain.sample

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import uniffi.jova_core_ffi.JovaChain

private const val DEMO_MNEMONIC =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"

private val ALL_CHAINS = listOf(
    JovaChain.Ethereum, JovaChain.Polygon, JovaChain.Bsc,
    JovaChain.Arbitrum, JovaChain.Optimism, JovaChain.Base,
    JovaChain.Bitcoin, JovaChain.Xrp, JovaChain.Solana,
)

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            MaterialTheme {
                Scaffold { padding ->
                    AddressDemo(modifier = Modifier.padding(padding))
                }
            }
        }
    }
}

@Composable
private fun AddressDemo(modifier: Modifier = Modifier) {
    var rows by remember { mutableStateOf<List<Pair<JovaChain, String>>>(emptyList()) }
    var err by remember { mutableStateOf<String?>(null) }

    Column(modifier = modifier.padding(16.dp)) {
        Button(onClick = {
            val repo = WalletRepository(DEMO_MNEMONIC)
            try {
                rows = ALL_CHAINS.map { it to repo.address(it).value }
                err = null
            } catch (e: WalletError) {
                err = e.message
            }
        }) { Text("Derive addresses") }

        Spacer(Modifier.height(16.dp))

        err?.let { Text("Error: $it") }

        LazyColumn {
            items(rows) { (chain, value) ->
                Text(chain.toString(), style = MaterialTheme.typography.labelSmall)
                Text(value, fontFamily = FontFamily.Monospace)
                Spacer(Modifier.height(8.dp))
            }
        }
    }
}
