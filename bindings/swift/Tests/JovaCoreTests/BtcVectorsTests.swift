import XCTest
@testable import JovaCore

/// Phase 2 Bitcoin vector parity tests.
///
/// Loads `spec/test-vectors.json` and iterates every `btc.*` vector,
/// exercising the Swift uniffi binding through the same paths the Rust core
/// already verifies in `crates/jova-core/tests/vectors_btc.rs`.
///
/// **Not executed on Linux CI** — the Swift toolchain only runs on the
/// `macos-latest` GitHub Actions matrix entry; this file is exercised there.
///
/// Mirrors `EvmVectorsTests`; filters narrowly on `bitcoin` discriminators so
/// adding more BTC vectors later doesn't pollute the EVM test runs.
final class BtcVectorsTests: XCTestCase {

    // MARK: - Address derivation

    func testBtcAddressVectors() throws {
        var ran = 0
        for v in try loadVectors() where (v["kind"] as? String) == "address" {
            let input = v["input"] as! [String: Any]
            let chainDict = input["chain"] as! [String: Any]
            guard (chainDict["kind"] as? String) == "bitcoin" else { continue }

            let id = (v["id"] as? String) ?? "?"
            // `JovaWallet.address` ignores its `account` argument today and
            // always derives m/84'/0'/0'/0/0. Skip address-index >0 vectors;
            // the spec keeps them for the day the API grows an index arg.
            guard id.hasSuffix("_account0_index0") else { continue }

            let mnemonic = input["mnemonic"] as! String
            let pass = (input["passphrase"] as? String) ?? ""
            let expected = (v["expected"] as! [String: Any])["address"] as! String

            let wallet = try JovaWallet.fromMnemonic(words: mnemonic, passphrase: pass)
            let got = try wallet.address(chain: .bitcoin, account: 0)
            XCTAssertEqual(got.value, expected, "BTC address mismatch for vector \(id)")
            ran += 1
        }
        XCTAssertGreaterThanOrEqual(ran, 3, "Expected at least 3 BTC address vectors, ran \(ran)")
    }

    // MARK: - Transaction signing

    func testBtcSignTxVectors() throws {
        var ran = 0
        for v in try loadVectors() where (v["kind"] as? String) == "sign_tx" {
            let input = v["input"] as! [String: Any]
            let unsignedDict = input["unsigned_tx"] as! [String: Any]
            guard (unsignedDict["kind"] as? String) == "bitcoin" else { continue }

            let id = (v["id"] as? String) ?? "?"
            let mnemonic = input["mnemonic"] as! String
            let pass = (input["passphrase"] as? String) ?? ""
            let unsigned = try decodeUnsignedTx(unsignedDict)
            let expectedHex = ((v["expected"] as! [String: Any])["signed_hex"] as! String).lowercased()

            let wallet = try JovaWallet.fromMnemonic(words: mnemonic, passphrase: pass)
            let signed = try wallet.signTx(tx: unsigned)
            // The captured value for multi-party PSBTs is `psbt:<base64>`;
            // for finalized owned PSBTs it's pure lowercase hex. Compare
            // case-insensitively to absorb capture-tool casing differences.
            XCTAssertEqual(
                signed.rawHex.lowercased(),
                expectedHex,
                "signed_hex mismatch for vector \(id)"
            )
            ran += 1
        }
        XCTAssertGreaterThanOrEqual(ran, 3, "Expected at least 3 BTC sign_tx vectors, ran \(ran)")
    }

    // MARK: - Message signing

    func testBtcSignMessageVectors() throws {
        var ran = 0
        for v in try loadVectors() where (v["kind"] as? String) == "sign_message" {
            let input = v["input"] as! [String: Any]
            let messageDict = input["message"] as! [String: Any]
            guard (messageDict["kind"] as? String) == "bitcoin" else { continue }

            let id = (v["id"] as? String) ?? "?"
            let mnemonic = input["mnemonic"] as! String
            let pass = (input["passphrase"] as? String) ?? ""
            let signable = try decodeSignableMessage(messageDict)
            // Spec field name is `signature_hex` even though BTC carries base64.
            let expectedSig = (v["expected"] as! [String: Any])["signature_hex"] as! String

            let wallet = try JovaWallet.fromMnemonic(words: mnemonic, passphrase: pass)
            let sig = try wallet.signMessage(msg: signable)
            // base64 is case-sensitive; compare exactly.
            XCTAssertEqual(sig.hex, expectedSig, "signature mismatch for vector \(id)")
            ran += 1
        }
        XCTAssertGreaterThanOrEqual(
            ran, 2, "Expected at least 2 BTC sign_message vectors, ran \(ran)"
        )
    }

    // MARK: - Error vectors

    func testBtcErrorVectors() throws {
        var ran = 0
        for v in try loadVectors() where (v["kind"] as? String) == "error" {
            let id = (v["id"] as? String) ?? "?"
            guard id.hasPrefix("btc.") else { continue }

            let input = v["input"] as! [String: Any]
            let mnemonic = input["mnemonic"] as! String
            let pass = (input["passphrase"] as? String) ?? ""
            let expectedDict = v["expected"] as! [String: Any]
            let expectedVariant = expectedDict["error_variant"] as! String
            let expectedReason = expectedDict["reason"] as! String

            let wallet = try JovaWallet.fromMnemonic(words: mnemonic, passphrase: pass)

            // BTC error vectors split: PSBT-shaped go through signTx, message-
            // shaped through signMessage. Pick the path by which key is present.
            do {
                if let unsignedDict = input["unsigned_tx"] as? [String: Any] {
                    let unsigned = try decodeUnsignedTx(unsignedDict)
                    _ = try wallet.signTx(tx: unsigned)
                } else if let messageDict = input["message"] as? [String: Any] {
                    let msg = try decodeSignableMessage(messageDict)
                    _ = try wallet.signMessage(msg: msg)
                } else {
                    XCTFail("vector \(id) has neither unsigned_tx nor message in input")
                    continue
                }
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
        XCTAssertGreaterThanOrEqual(ran, 3, "Expected at least 3 BTC error vectors, ran \(ran)")
    }
}
