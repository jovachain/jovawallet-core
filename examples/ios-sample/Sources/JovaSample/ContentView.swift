// ContentView — SwiftUI demo screen. Derives one address per chain and
// signs a personal_sign message to prove the FFI round-trip works.

import SwiftUI
import JovaCore

/// BIP-39 official test mnemonic — DEMO ONLY. Never hardcode in production.
private let DEMO_MNEMONIC =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"

struct ContentView: View {
    @State private var addresses: [(JovaChain, String)] = []
    @State private var error: String?

    var body: some View {
        NavigationView {
            List {
                Section(header: Text("Addresses (BIP-39 abandon-about)")) {
                    if addresses.isEmpty {
                        Text("Tap derive →")
                    }
                    ForEach(addresses, id: \.1) { (chain, value) in
                        VStack(alignment: .leading) {
                            Text(String(describing: chain)).font(.caption).foregroundColor(.secondary)
                            Text(value).font(.system(.body, design: .monospaced))
                        }
                    }
                }
                if let error {
                    Section(header: Text("Error")) {
                        Text(error).foregroundColor(.red)
                    }
                }
            }
            .navigationTitle("JovaSample")
            .toolbar {
                ToolbarItem(placement: .automatic) {
                    Button("Derive") { deriveAll() }
                }
            }
        }
    }

    private func deriveAll() {
        let service = WalletService(mnemonic: DEMO_MNEMONIC)
        var out: [(JovaChain, String)] = []
        for chain in [
            JovaChain.ethereum, .polygon, .bsc, .arbitrum, .optimism, .base,
            .bitcoin, .xrp, .solana,
        ] {
            do {
                let addr = try service.address(chain: chain)
                out.append((chain, addr.value))
            } catch {
                self.error = "\(chain): \(error)"
                return
            }
        }
        self.addresses = out
        self.error = nil
    }
}
