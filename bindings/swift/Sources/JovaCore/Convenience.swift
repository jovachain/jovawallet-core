import Foundation

public enum JovaCoreVersion {
    public static let value = "0.1.0"
}

// Helper for the most common case: build an EVM transfer without filling in the access list.
extension EvmUnsigned {
    public static func transfer(
        chainId: UInt64,
        nonce: UInt64,
        to: String,
        valueWei: String,
        gasLimit: UInt64 = 21_000,
        maxFeePerGas: String,
        maxPriorityFeePerGas: String
    ) -> EvmUnsigned {
        EvmUnsigned(
            chainId: chainId,
            nonce: nonce,
            to: to,
            value: valueWei,
            gasLimit: gasLimit,
            maxFeePerGas: maxFeePerGas,
            maxPriorityFeePerGas: maxPriorityFeePerGas,
            data: "0x",
            accessList: []
        )
    }
}
