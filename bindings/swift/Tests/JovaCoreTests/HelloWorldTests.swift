import XCTest
@testable import JovaCore

final class HelloWorldTests: XCTestCase {
    func testNegativeMnemonicValidationVector() throws {
        // Find spec/test-vectors.json relative to the repo root.
        // swift test runs with cwd at bindings/swift; walk up to find the spec dir.
        let url = try Self.findTestVectors()
        let data = try Data(contentsOf: url)
        let json = try JSONSerialization.jsonObject(with: data) as! [String: Any]
        let vectors = json["vectors"] as! [[String: Any]]
        let v = vectors.first { ($0["id"] as? String) == "phase0.mnemonic_validation_neg.gibberish" }!

        let input = v["input"] as! [String: Any]
        let words = input["words"] as! String
        let passphrase = (input["passphrase"] as? String) ?? ""
        let expected = (v["expected"] as! [String: Any])["valid"] as! Bool

        XCTAssertEqual(isValidMnemonic(words: words, passphrase: passphrase), expected)
    }

    /// Walk up from the current working directory and from the source file until we find
    /// `spec/test-vectors.json`. Swift tests may run from bindings/swift or the project root.
    private static func findTestVectors() throws -> URL {
        // Strategy 1: walk up from CWD.
        var dir = URL(fileURLWithPath: FileManager.default.currentDirectoryPath)
        for _ in 0..<6 {
            let candidate = dir.appendingPathComponent("spec/test-vectors.json")
            if FileManager.default.fileExists(atPath: candidate.path) {
                return candidate
            }
            dir = dir.deletingLastPathComponent()
        }
        // Strategy 2: walk up from the source file location.
        var src = URL(fileURLWithPath: #file)
        for _ in 0..<8 {
            let candidate = src.appendingPathComponent("spec/test-vectors.json")
            if FileManager.default.fileExists(atPath: candidate.path) {
                return candidate
            }
            src = src.deletingLastPathComponent()
        }
        throw NSError(
            domain: "JovaCoreTests",
            code: 1,
            userInfo: [NSLocalizedDescriptionKey:
                "test-vectors.json not found; searched from \(FileManager.default.currentDirectoryPath)"]
        )
    }
}
