import XCTest
@testable import JovaCore

/// Phase 3c Solana vector parity tests.
///
/// Loads `spec/test-vectors.json` and iterates every `sol.*` vector,
/// exercising the Swift uniffi binding through the same paths the Rust core
/// already verifies in `crates/jova-core/tests/vectors_sol.rs`.
///
/// **Not executed on Linux CI** — the Swift toolchain only runs on the
/// `macos-latest` GitHub Actions matrix entry; this file is exercised there.
///
/// Mirrors `XrpVectorsTests`. Covers address derivation, tx signing (both
/// no-ALT and with-ALT v0 messages), raw ed25519 message signing, and the
/// MalformedUnsignedTx / MalformedSignableMessage error paths.
final class SolVectorsTests: XCTestCase {

    // MARK: - Address derivation

    func testSolAddressVectors() throws {
        var ran = 0
        for v in try loadVectors() where (v["kind"] as? String) == "address" {
            let input = v["input"] as! [String: Any]
            let chainDict = input["chain"] as! [String: Any]
            guard (chainDict["kind"] as? String) == "solana" else { continue }

            let id = (v["id"] as? String) ?? "?"
            let mnemonic = input["mnemonic"] as! String
            let pass = (input["passphrase"] as? String) ?? ""
            let expected = (v["expected"] as! [String: Any])["address"] as! String

            let wallet = try JovaWallet.fromMnemonic(words: mnemonic, passphrase: pass)
            let got = try wallet.address(chain: .solana, account: 0)
            XCTAssertEqual(got.value, expected, "SOL address mismatch for vector \(id)")
            ran += 1
        }
        XCTAssertGreaterThanOrEqual(ran, 1, "Expected at least 1 SOL address vector, ran \(ran)")
    }

    // MARK: - Transaction signing

    func testSolSignTxVectors() throws {
        var ran = 0
        for v in try loadVectors() where (v["kind"] as? String) == "sign_tx" {
            let input = v["input"] as! [String: Any]
            let unsignedDict = input["unsigned_tx"] as! [String: Any]
            guard (unsignedDict["kind"] as? String) == "solana" else { continue }

            let id = (v["id"] as? String) ?? "?"
            let mnemonic = input["mnemonic"] as! String
            let pass = (input["passphrase"] as? String) ?? ""
            let unsigned = try decodeUnsignedTx(unsignedDict)
            let expectedDict = v["expected"] as! [String: Any]
            let expectedHex = expectedDict["signed_hex"] as! String
            let expectedHash = expectedDict["tx_hash"] as! String

            let wallet = try JovaWallet.fromMnemonic(words: mnemonic, passphrase: pass)
            let signed = try wallet.signTx(tx: unsigned)
            XCTAssertEqual(signed.rawHex, expectedHex, "signed_hex mismatch for vector \(id)")
            XCTAssertEqual(signed.txHash, expectedHash, "tx_hash mismatch for vector \(id)")
            ran += 1
        }
        XCTAssertGreaterThanOrEqual(ran, 2, "Expected at least 2 SOL sign_tx vectors, ran \(ran)")
    }

    // MARK: - Message signing

    func testSolSignMessageVectors() throws {
        var ran = 0
        for v in try loadVectors() where (v["kind"] as? String) == "sign_message" {
            let input = v["input"] as! [String: Any]
            let msgDict = input["message"] as! [String: Any]
            guard (msgDict["kind"] as? String) == "solana" else { continue }

            let id = (v["id"] as? String) ?? "?"
            let mnemonic = input["mnemonic"] as! String
            let pass = (input["passphrase"] as? String) ?? ""
            let msg = try decodeSignableMessage(msgDict)
            // signature_hex carries the base58 sig string per cross-phase convention.
            let expected = (v["expected"] as! [String: Any])["signature_hex"] as! String

            let wallet = try JovaWallet.fromMnemonic(words: mnemonic, passphrase: pass)
            let sig = try wallet.signMessage(msg: msg)
            XCTAssertEqual(sig.hex, expected, "SOL signature mismatch for vector \(id)")
            ran += 1
        }
        XCTAssertGreaterThanOrEqual(ran, 1, "Expected at least 1 SOL sign_message vector, ran \(ran)")
    }

    // MARK: - Error vectors

    func testSolErrorVectors() throws {
        var ran = 0
        for v in try loadVectors() where (v["kind"] as? String) == "error" {
            let id = (v["id"] as? String) ?? "?"
            guard id.hasPrefix("sol.") else { continue }

            let input = v["input"] as! [String: Any]
            let mnemonic = input["mnemonic"] as! String
            let pass = (input["passphrase"] as? String) ?? ""
            let expectedDict = v["expected"] as! [String: Any]
            let expectedVariant = expectedDict["error_variant"] as! String
            let expectedReason = expectedDict["reason"] as! String

            let wallet = try JovaWallet.fromMnemonic(words: mnemonic, passphrase: pass)

            do {
                if let unsignedDict = input["unsigned_tx"] as? [String: Any] {
                    let unsigned = try decodeUnsignedTx(unsignedDict)
                    _ = try wallet.signTx(tx: unsigned)
                } else if let msgDict = input["message"] as? [String: Any] {
                    let msg = try decodeSignableMessage(msgDict)
                    _ = try wallet.signMessage(msg: msg)
                } else {
                    XCTFail("SOL error vector \(id) must carry an unsigned_tx or message")
                    continue
                }
                XCTFail("Expected FfiError.\(expectedVariant) for vector \(id), but no error was thrown")
            } catch let err as FfiError {
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
        XCTAssertGreaterThanOrEqual(ran, 4, "Expected at least 4 SOL error vectors, ran \(ran)")
    }
}
