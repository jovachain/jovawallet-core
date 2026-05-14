import Foundation
@testable import JovaCore

enum VectorDecodeError: Error {
    case unknownChainKind(String)
    case missingField(String)
    case unknownMessageKind(String)
    case unknownUnsignedTxKind(String)
    case unknownBtcMsgScheme(String)
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

/// Decode an `input.unsigned_tx` dict into the uniffi `UnsignedTx` enum.
/// Supports `"evm"`, `"bitcoin"`, and `"xrp"` kinds; later phases extend this
/// switch (e.g. `"solana"`).
func decodeUnsignedTx(_ dict: [String: Any]) throws -> UnsignedTx {
    guard let kind = dict["kind"] as? String else {
        throw VectorDecodeError.missingField("kind")
    }
    switch kind {
    case "evm":
        let evm = try decodeEvmUnsigned(dict)
        return .evm(tx: evm)
    case "bitcoin":
        guard let psbt = dict["psbt_base64"] as? String else {
            throw VectorDecodeError.missingField("psbt_base64")
        }
        return .bitcoin(psbtBase64: psbt)
    case "xrp":
        guard let txJson = dict["tx_json"] as? String else {
            throw VectorDecodeError.missingField("tx_json")
        }
        return .xrp(txJson: txJson)
    case "solana":
        guard let msg = dict["message_base64"] as? String else {
            throw VectorDecodeError.missingField("message_base64")
        }
        guard let bh = dict["recent_blockhash"] as? String else {
            throw VectorDecodeError.missingField("recent_blockhash")
        }
        return .solana(messageBase64: msg, recentBlockhash: bh)
    default:
        throw VectorDecodeError.unknownUnsignedTxKind(kind)
    }
}

/// Map the spec's camelCase `scheme` field to the uniffi `BtcMsgScheme` enum.
private func decodeBtcMsgScheme(_ s: String) throws -> BtcMsgScheme {
    switch s {
    case "bip322": return .bip322
    case "legacy": return .legacy
    default:       throw VectorDecodeError.unknownBtcMsgScheme(s)
    }
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
    case "bitcoin":
        guard let msg = dict["message"] as? String else {
            throw VectorDecodeError.missingField("message")
        }
        guard let addr = dict["address"] as? String else {
            throw VectorDecodeError.missingField("address")
        }
        guard let schemeStr = dict["scheme"] as? String else {
            throw VectorDecodeError.missingField("scheme")
        }
        let scheme = try decodeBtcMsgScheme(schemeStr)
        return .bitcoin(message: msg, address: addr, scheme: scheme)
    case "solana":
        guard let msgB64 = dict["message_base64"] as? String else {
            throw VectorDecodeError.missingField("message_base64")
        }
        return .solana(messageBase64: msgB64)
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
