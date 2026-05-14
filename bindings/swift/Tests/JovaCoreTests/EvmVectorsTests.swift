import XCTest
@testable import JovaCore

final class EvmVectorsTests: XCTestCase {

    // MARK: - Address derivation

    func testEvmAddressVectors() throws {
        var ran = 0
        for v in try loadVectors() where (v["kind"] as? String) == "address" {
            let input = v["input"] as! [String: Any]
            let chain = try decodeChain(input["chain"] as! [String: Any])
            // This file covers EVM only; non-EVM chains get their own test in Phase 2/3.
            switch chain {
            case .ethereum, .polygon, .bsc, .arbitrum, .optimism, .base, .customEvm:
                break
            default:
                continue
            }

            let mnemonic = input["mnemonic"] as! String
            let pass = (input["passphrase"] as? String) ?? ""
            let expected = (v["expected"] as! [String: Any])["address"] as! String

            let wallet = try JovaWallet.fromMnemonic(words: mnemonic, passphrase: pass)
            let got = try wallet.address(chain: chain, account: 0)
            XCTAssertEqual(
                got.value.lowercased(),
                expected.lowercased(),
                "address mismatch for vector \(v["id"] ?? "?")"
            )
            ran += 1
        }
        XCTAssertGreaterThan(ran, 0, "No address vectors ran — check vector file")
    }

    // MARK: - Transaction signing

    func testEvmSignTxVectors() throws {
        var ran = 0
        for v in try loadVectors() where (v["kind"] as? String) == "sign_tx" {
            let input = v["input"] as! [String: Any]
            let mnemonic = input["mnemonic"] as! String
            let pass = (input["passphrase"] as? String) ?? ""
            let unsignedDict = input["unsigned_tx"] as! [String: Any]

            // EVM-only path in this test.
            guard (unsignedDict["kind"] as? String) == "evm" else { continue }
            let evm = try decodeEvmUnsigned(unsignedDict)
            let unsigned = UnsignedTx.evm(tx: evm)

            let expected = v["expected"] as! [String: Any]
            let expectedHex = (expected["signed_hex"] as! String).lowercased()
            let expectedHash = (expected["tx_hash"] as! String).lowercased()

            let wallet = try JovaWallet.fromMnemonic(words: mnemonic, passphrase: pass)
            let signed = try wallet.signTx(tx: unsigned)
            XCTAssertEqual(
                signed.rawHex.lowercased(),
                expectedHex,
                "signed_hex mismatch for vector \(v["id"] ?? "?")"
            )
            XCTAssertEqual(
                signed.txHash.lowercased(),
                expectedHash,
                "tx_hash mismatch for vector \(v["id"] ?? "?")"
            )
            ran += 1
        }
        XCTAssertGreaterThan(ran, 0, "No sign_tx vectors ran — check vector file")
    }

    // MARK: - Message signing

    func testEvmSignMessageVectors() throws {
        var ran = 0
        for v in try loadVectors() where (v["kind"] as? String) == "sign_message" {
            let input = v["input"] as! [String: Any]
            let mnemonic = input["mnemonic"] as! String
            let pass = (input["passphrase"] as? String) ?? ""
            let messageDict = input["message"] as! [String: Any]

            // Phase 2 added BTC sign_message vectors with `message.kind == "bitcoin"`;
            // those are exercised by BtcVectorsTests. Skip them here. (The decoder
            // used to throw on non-EVM kinds; now it returns Bitcoin variants too.)
            let msgKind = (messageDict["kind"] as? String) ?? ""
            guard msgKind == "evmPersonalSign" || msgKind == "evmTypedDataV4" else { continue }

            let signable: SignableMessage
            do {
                signable = try decodeSignableMessage(messageDict)
            } catch VectorDecodeError.unknownMessageKind {
                continue
            }

            let expected = v["expected"] as! [String: Any]
            let expectedSig = (expected["signature"] as! String).lowercased()

            let wallet = try JovaWallet.fromMnemonic(words: mnemonic, passphrase: pass)
            let sig = try wallet.signMessage(msg: signable)
            XCTAssertEqual(
                sig.hex.lowercased(),
                expectedSig,
                "signature mismatch for vector \(v["id"] ?? "?")"
            )
            ran += 1
        }
        XCTAssertGreaterThan(ran, 0, "No sign_message vectors ran — check vector file")
    }

    // MARK: - Error vectors

    func testEvmErrorVectors() throws {
        var ran = 0
        for v in try loadVectors() where (v["kind"] as? String) == "error" {
            let input = v["input"] as! [String: Any]
            let mnemonic = input["mnemonic"] as! String
            let pass = (input["passphrase"] as? String) ?? ""
            let expected = v["expected"] as! [String: Any]
            let expectedVariant = expected["error_variant"] as! String

            // All Phase 1 error vectors go through sign_tx with a malformed tx.
            guard let unsignedDict = input["unsigned_tx"] as? [String: Any],
                  (unsignedDict["kind"] as? String) == "evm" else { continue }

            let evm: EvmUnsigned
            do {
                evm = try decodeEvmUnsigned(unsignedDict)
            } catch {
                // If the decoder itself throws (not expected here), re-throw.
                throw error
            }
            let unsigned = UnsignedTx.evm(tx: evm)

            let wallet = try JovaWallet.fromMnemonic(words: mnemonic, passphrase: pass)
            do {
                _ = try wallet.signTx(tx: unsigned)
                XCTFail("Expected FfiError.\(expectedVariant) for vector \(v["id"] ?? "?"), but no error was thrown")
            } catch let err as FfiError {
                // Verify the variant matches.
                switch (expectedVariant, err) {
                case ("MalformedUnsignedTx", .MalformedUnsignedTx):
                    break  // correct
                case ("InvalidMnemonic", .InvalidMnemonic):
                    break
                case ("UnsupportedChain", .UnsupportedChain):
                    break
                case ("SigningFailed", .SigningFailed):
                    break
                default:
                    XCTFail("Wrong error variant for vector \(v["id"] ?? "?"): expected \(expectedVariant), got \(err)")
                }
                ran += 1
            }
        }
        XCTAssertGreaterThan(ran, 0, "No error vectors ran — check vector file")
    }
}
