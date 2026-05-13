import Foundation
@testable import JovaCore

enum VectorDecodeError: Error {
    case unknownChainKind(String)
    case missingField(String)
    case unknownMessageKind(String)
}

func decodeChain(_ dict: [String: Any]) throws -> JovaChain {
    guard let kind = dict["kind"] as? String else {
        throw VectorDecodeError.missingField("kind")
    }
    switch kind {
    case "ethereum":  return .ethereum
    case "polygon":   return .polygon
    case "bsc":       return .bsc
    case "arbitrum":  return .arbitrum
    case "optimism":  return .optimism
    case "base":      return .base
    case "bitcoin":   return .bitcoin
    case "solana":    return .solana
    case "xrp":       return .xrp
    case "customEvm":
        // JSON numbers decode as NSNumber; cast to the widest int first.
        guard let id = (dict["chainId"] as? NSNumber).map({ UInt64($0.uint64Value) })
                       ?? (dict["chainId"] as? UInt64) else {
            throw VectorDecodeError.missingField("chainId")
        }
        return .customEvm(chainId: id)
    default:
        throw VectorDecodeError.unknownChainKind(kind)
    }
}

func decodeEvmUnsigned(_ dict: [String: Any]) throws -> EvmUnsigned {
    // JSON numbers arrive as NSNumber; pull UInt64 via that bridge.
    func u64(_ key: String) throws -> UInt64 {
        guard let n = dict[key] as? NSNumber else { throw VectorDecodeError.missingField(key) }
        return n.uint64Value
    }
    func str(_ key: String) throws -> String {
        guard let s = dict[key] as? String else { throw VectorDecodeError.missingField(key) }
        return s
    }

    let accessListRaw = (dict["accessList"] as? [[String: Any]]) ?? []
    let accessList: [AccessListItem] = try accessListRaw.map { item in
        guard let addr = item["address"] as? String else {
            throw VectorDecodeError.missingField("accessList[].address")
        }
        // Vector JSON uses snake_case "storage_keys".
        let keys = (item["storage_keys"] as? [String]) ?? []
        return AccessListItem(address: addr, storageKeys: keys)
    }

    return EvmUnsigned(
        chainId:              try u64("chainId"),
        nonce:                try u64("nonce"),
        to:                   try str("to"),
        value:                try str("value"),
        gasLimit:             try u64("gasLimit"),
        maxFeePerGas:         try str("maxFeePerGas"),
        maxPriorityFeePerGas: try str("maxPriorityFeePerGas"),
        data:                 try str("data"),
        accessList:           accessList
    )
}

func decodeSignableMessage(_ dict: [String: Any]) throws -> SignableMessage {
    guard let kind = dict["kind"] as? String else {
        throw VectorDecodeError.missingField("kind")
    }
    switch kind {
    case "evmPersonalSign":
        guard let msg = dict["message"] as? String else {
            throw VectorDecodeError.missingField("message")
        }
        return .evmPersonalSign(message: msg)
    case "evmTypedDataV4":
        guard let json = dict["json"] as? String else {
            throw VectorDecodeError.missingField("json")
        }
        return .evmTypedDataV4(json: json)
    default:
        throw VectorDecodeError.unknownMessageKind(kind)
    }
}

/// Walk up from the current working directory and the source file location until
/// we find `spec/test-vectors.json`. swift test may run from bindings/swift or
/// the project root.
func findTestVectors() throws -> URL {
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

func loadVectors() throws -> [[String: Any]] {
    let url = try findTestVectors()
    let data = try Data(contentsOf: url)
    let json = try JSONSerialization.jsonObject(with: data) as! [String: Any]
    return json["vectors"] as! [[String: Any]]
}
