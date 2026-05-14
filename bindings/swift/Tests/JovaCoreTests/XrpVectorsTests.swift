import XCTest
@testable import JovaCore

/// Phase 3b XRP vector parity tests.
///
/// Loads `spec/test-vectors.json` and iterates every `xrp.*` vector,
/// exercising the Swift uniffi binding through the same paths the Rust core
/// already verifies in `crates/jova-core/tests/vectors_xrp.rs`.
///
/// **Not executed on Linux CI** — the Swift toolchain only runs on the
/// `macos-latest` GitHub Actions matrix entry; this file is exercised there.
///
/// Mirrors `BtcVectorsTests`. XRPL has no canonical message-signing scheme
/// equivalent to BIP-322 / EIP-191, so there is no `testXrpSignMessageVectors`
/// test; XRP error coverage is purely transaction-side.
final class XrpVectorsTests: XCTestCase {

    // MARK: - Address derivation

    func testXrpAddressVectors() throws {
        var ran = 0
        for v in try loadVectors() where (v["kind"] as? String) == "address" {
            let input = v["input"] as! [String: Any]
            let chainDict = input["chain"] as! [String: Any]
            guard (chainDict["kind"] as? String) == "xrp" else { continue }

            let id = (v["id"] as? String) ?? "?"
            let mnemonic = input["mnemonic"] as! String
            let pass = (input["passphrase"] as? String) ?? ""
            let expected = (v["expected"] as! [String: Any])["address"] as! String

            let wallet = try JovaWallet.fromMnemonic(words: mnemonic, passphrase: pass)
            let got = try wallet.address(chain: .xrp, account: 0)
            XCTAssertEqual(got.value, expected, "XRP address mismatch for vector \(id)")
            ran += 1
        }
        XCTAssertGreaterThanOrEqual(ran, 1, "Expected at least 1 XRP address vector, ran \(ran)")
    }

    // MARK: - Transaction signing

    func testXrpSignTxVectors() throws {
        var ran = 0
        for v in try loadVectors() where (v["kind"] as? String) == "sign_tx" {
            let input = v["input"] as! [String: Any]
            let unsignedDict = input["unsigned_tx"] as! [String: Any]
            guard (unsignedDict["kind"] as? String) == "xrp" else { continue }

            let id = (v["id"] as? String) ?? "?"
            let mnemonic = input["mnemonic"] as! String
            let pass = (input["passphrase"] as? String) ?? ""
            let unsigned = try decodeUnsignedTx(unsignedDict)
            let expectedDict = v["expected"] as! [String: Any]
            let expectedHex = (expectedDict["signed_hex"] as! String).uppercased()
            let expectedHash = (expectedDict["tx_hash"] as! String).uppercased()

            let wallet = try JovaWallet.fromMnemonic(words: mnemonic, passphrase: pass)
            let signed = try wallet.signTx(tx: unsigned)
            // xrpl-py emits uppercase hex; the SDK normalizes to uppercase too.
            // Compare case-insensitively for resilience.
            XCTAssertEqual(
                signed.rawHex.uppercased(),
                expectedHex,
                "signed_hex mismatch for vector \(id)"
            )
            XCTAssertEqual(
                signed.txHash.uppercased(),
                expectedHash,
                "tx_hash mismatch for vector \(id)"
            )
            ran += 1
        }
        XCTAssertGreaterThanOrEqual(ran, 2, "Expected at least 2 XRP sign_tx vectors, ran \(ran)")
    }

    // MARK: - Error vectors

    func testXrpErrorVectors() throws {
        var ran = 0
        for v in try loadVectors() where (v["kind"] as? String) == "error" {
            let id = (v["id"] as? String) ?? "?"
            guard id.hasPrefix("xrp.") else { continue }

            let input = v["input"] as! [String: Any]
            let mnemonic = input["mnemonic"] as! String
            let pass = (input["passphrase"] as? String) ?? ""
            let expectedDict = v["expected"] as! [String: Any]
            let expectedVariant = expectedDict["error_variant"] as! String
            let expectedReason = expectedDict["reason"] as! String

            let wallet = try JovaWallet.fromMnemonic(words: mnemonic, passphrase: pass)

            // XRP error vectors all carry an `unsigned_tx` (XRPL has no message
            // signing surface in this SDK), so the dispatch path is signTx.
            do {
                guard let unsignedDict = input["unsigned_tx"] as? [String: Any] else {
                    XCTFail("XRP error vector \(id) must carry an unsigned_tx in input")
                    continue
                }
                let unsigned = try decodeUnsignedTx(unsignedDict)
                _ = try wallet.signTx(tx: unsigned)
                XCTFail("Expected FfiError.\(expectedVariant) for vector \(id), but no error was thrown")
            } catch let err as FfiError {
                // Verify both the variant and that the reason flows through
                // the stringified `errorDescription` (FfiError is flat_error
                // in uniffi, so there's no structured reason field).
                let msg: String
                switch (expectedVariant, err) {
                case ("MalformedUnsignedTx", .MalformedUnsignedTx(let m)):
                    msg = m
                case ("MalformedSignableMessage", .MalformedSignableMessage(let m)):
                    msg = m
                case ("InvalidMnemonic", .InvalidMnemonic(let m)):
                    msg = m
                case ("UnsupportedChain", .UnsupportedChain(let m)):
                    msg = m
                case ("SigningFailed", .SigningFailed(let m)):
                    msg = m
                default:
                    XCTFail("Wrong error variant for vector \(id): expected \(expectedVariant), got \(err)")
                    continue
                }
                XCTAssertTrue(
                    msg.contains(expectedReason),
                    "Error message for \(id) should contain reason '\(expectedReason)', got: \(msg)"
                )
                ran += 1
            }
        }
        XCTAssertGreaterThanOrEqual(ran, 2, "Expected at least 2 XRP error vectors, ran \(ran)")
    }
}
