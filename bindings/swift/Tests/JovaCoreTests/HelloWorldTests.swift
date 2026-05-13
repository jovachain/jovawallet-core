import XCTest
@testable import JovaCore

final class HelloWorldTests: XCTestCase {
    func testNegativeMnemonicValidationVector() throws {
        let vectors = try loadVectors()
        let v = vectors.first { ($0["id"] as? String) == "phase0.mnemonic_validation_neg.gibberish" }!

        let input = v["input"] as! [String: Any]
        let words = input["words"] as! String
        let passphrase = (input["passphrase"] as? String) ?? ""
        let expected = (v["expected"] as! [String: Any])["valid"] as! Bool

        XCTAssertEqual(isValidMnemonic(words: words, passphrase: passphrase), expected)
    }
}
