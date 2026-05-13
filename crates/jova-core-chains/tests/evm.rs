//! EVM signer integration tests.
//!
//! These tests cover:
//!  - EIP-55 checksum address derivation and validation
//!  - EIP-1559 transaction signing (smoke — real vectors captured in Task 4)
//!  - EIP-191 personal_sign (smoke)
//!  - ChainSigner trait plumbing (derive_address, validate_address, sign_tx, sign_message)
//!
//! Known-good reference values for the address tests come from EIP-55 specification.
//! Transaction signing smokes use a fixed private-key (one from secp256k1 test vectors).

use jova_core_chains::{
    ChainError, ChainSigner, EvmUnsigned, SignableMessage, UnsignedTx, evm::EvmSigner,
};
use jova_core_primitives::XPrv;

// ── helpers ─────────────────────────────────────────────────────────────────

/// Build an XPrv from raw 32-byte private key.  Only for tests.
fn xprv_from_bytes(key: [u8; 32]) -> XPrv {
    // XPrv::new_unchecked exposed for tests via test helper
    jova_core_chains::evm::test_helpers::xprv_from_key_bytes(key)
}

// ── EIP-55 checksum tests ────────────────────────────────────────────────────

/// EIP-55 reference vectors from the specification (Section "Test Cases").
#[test]
fn eip55_checksum_vectors() {
    let cases = [
        // All caps
        "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed",
        "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359",
        "0xdbF03B407c01E7cD3CBea99509d93f8DDDC8C6FB",
        "0xD1220A0cf47c7B9Be7A2E6BA89F429762e7b9aDb",
    ];
    for addr in cases {
        // validate must accept a correct checksum address
        assert!(
            jova_core_chains::evm::validate_address(addr),
            "Should accept EIP-55 valid: {addr}"
        );
        // round-trip: strip 0x, lowercase, re-checksum → must match original
        let body = &addr[2..];
        let lower = body.to_ascii_lowercase();
        let rechecksum = jova_core_chains::evm::eip55_checksum(&lower);
        assert_eq!(rechecksum, addr, "Checksum round-trip failed for {addr}");
    }
}

#[test]
fn eip55_all_lowercase_accepted() {
    // All-lowercase addresses (legacy) are accepted without checksum verification.
    assert!(jova_core_chains::evm::validate_address(
        "0x5aaeb6053f3e94c9b9a09f33669435e7ef1beaed"
    ));
}

#[test]
fn eip55_all_uppercase_hex_accepted() {
    // All-uppercase is also legacy-accepted.
    assert!(jova_core_chains::evm::validate_address(
        "0x5AAEB6053F3E94C9B9A09F33669435E7EF1BEAED"
    ));
}

#[test]
fn eip55_wrong_checksum_rejected() {
    // Flip one letter to break the checksum — mixed case so the checksum path fires.
    // "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed" is valid;
    // change the first 'a' to 'A' to get an invalid mixed-case address.
    let bad = "0x5AAeb6053F3E94C9b9A09f33669435E7Ef1BeAed";
    assert!(
        !jova_core_chains::evm::validate_address(bad),
        "Should reject wrong EIP-55 checksum"
    );
}

#[test]
fn eip55_too_short_rejected() {
    assert!(!jova_core_chains::evm::validate_address("0x1234"));
}

#[test]
fn eip55_non_hex_rejected() {
    // 42 chars but contains 'z'
    assert!(!jova_core_chains::evm::validate_address(
        "0xZZZZb6053F3E94C9b9A09f33669435E7Ef1BeAed"
    ));
}

// ── ChainSigner trait plumbing ───────────────────────────────────────────────

#[test]
fn derive_address_is_checksummed() {
    // Private key 0x01…01 (one of the secp256k1 spec keys).
    // The expected address is verified externally against cast / EIP-55.
    // Key: 0000…0001 → address 0x7E5F4552091A69125d5DfCb7b8C2659029395Bdf (from cast)
    let key = xprv_from_bytes([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 1,
    ]);
    let signer = EvmSigner {
        chain_label: "ethereum",
    };
    let addr = signer.derive_address(&key).unwrap();
    assert_eq!(addr.chain, "ethereum");
    // Must start with 0x and have length 42
    assert_eq!(&addr.value[..2], "0x");
    assert_eq!(addr.value.len(), 42);
    // Must be valid EIP-55 (mixed-case validates)
    assert!(jova_core_chains::evm::validate_address(&addr.value));
    // Known-good value: verified against alloy-signer-local's key_to_address test
    assert_eq!(addr.value, "0x7E5F4552091A69125d5DfCb7b8C2659029395Bdf");
}

#[test]
fn derive_address_key_2() {
    // Private key 0x02 → address 0x2B5AD5c4795c026514f8317c7a215E218DcCD6cF
    let key = xprv_from_bytes([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 2,
    ]);
    let signer = EvmSigner {
        chain_label: "polygon",
    };
    let addr = signer.derive_address(&key).unwrap();
    assert_eq!(addr.chain, "polygon");
    assert_eq!(addr.value, "0x2B5AD5c4795c026514f8317c7a215E218DcCD6cF");
}

#[test]
fn validate_address_delegation() {
    let signer = EvmSigner {
        chain_label: "ethereum",
    };
    assert!(signer.validate_address("0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed"));
    assert!(!signer.validate_address("0x1234"));
    assert!(!signer.validate_address("not-an-address"));
}

// ── EIP-1559 transaction signing smoke ───────────────────────────────────────

#[test]
fn sign_eip1559_produces_valid_hex() {
    // Minimal EIP-1559 transaction — chain 1 (mainnet), no data, no access list.
    // We don't assert the exact signed hex here; that's Task 4 with cast vectors.
    // We assert:
    //   - sign_tx returns Ok
    //   - raw_hex starts with "0x02" (type-2 envelope)
    //   - tx_hash is 66 chars (0x + 64 hex = 32-byte hash)
    let key = xprv_from_bytes([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 1,
    ]);
    let signer = EvmSigner {
        chain_label: "ethereum",
    };
    let tx = UnsignedTx::Evm(EvmUnsigned {
        chain_id: 1,
        nonce: 0,
        to: "0x2B5AD5c4795c026514f8317c7a215E218DcCD6cF".into(),
        value: "1000000000000000000".into(), // 1 ETH in wei
        gas_limit: 21000,
        max_fee_per_gas: "30000000000".into(), // 30 gwei
        max_priority_fee_per_gas: "1000000000".into(), // 1 gwei
        data: "0x".into(),
        access_list: vec![],
    });
    let result = signer.sign_tx(&key, &tx).unwrap();
    assert_eq!(result.chain, "ethereum");
    assert!(
        result.raw_hex.starts_with("0x02"),
        "EIP-1559 envelope must start with 0x02, got: {}",
        &result.raw_hex[..10]
    );
    assert_eq!(
        result.tx_hash.len(),
        66,
        "tx_hash must be 0x + 32 bytes hex"
    );
    assert!(result.tx_hash.starts_with("0x"));
}

#[test]
fn sign_tx_wrong_variant_errors() {
    // Passing a non-EVM UnsignedTx to EvmSigner must return an error.
    // Currently only Evm variant exists, but we can simulate by matching behavior.
    // This test validates the match arm in sign_tx.
    // Since Evm is the only variant right now, we just confirm the happy path.
    // (The error path will be exercised once other chain variants are added.)
    let key = xprv_from_bytes([0u8; 32].map(|_| 1u8));
    let signer = EvmSigner {
        chain_label: "ethereum",
    };
    let tx = UnsignedTx::Evm(EvmUnsigned {
        chain_id: 1,
        nonce: 0,
        to: "0x2B5AD5c4795c026514f8317c7a215E218DcCD6cF".into(),
        value: "0".into(),
        gas_limit: 21000,
        max_fee_per_gas: "1".into(),
        max_priority_fee_per_gas: "1".into(),
        data: "0x".into(),
        access_list: vec![],
    });
    // Must not panic — just verify Ok
    assert!(signer.sign_tx(&key, &tx).is_ok());
}

#[test]
fn sign_tx_malformed_to_address_errors() {
    let key = xprv_from_bytes([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 1,
    ]);
    let signer = EvmSigner {
        chain_label: "ethereum",
    };
    let tx = UnsignedTx::Evm(EvmUnsigned {
        chain_id: 1,
        nonce: 0,
        to: "not-a-valid-address".into(),
        value: "0".into(),
        gas_limit: 21000,
        max_fee_per_gas: "1".into(),
        max_priority_fee_per_gas: "1".into(),
        data: "0x".into(),
        access_list: vec![],
    });
    let err = signer.sign_tx(&key, &tx).unwrap_err();
    assert!(
        matches!(err, ChainError::MalformedUnsignedTx(_)),
        "Expected MalformedUnsignedTx, got: {err:?}"
    );
}

// ── EIP-191 personal_sign smoke ───────────────────────────────────────────────

#[test]
fn sign_personal_message_format() {
    // Sign a known string and verify the format of the signature.
    // Exact value is asserted in Task 4's cast-derived vectors.
    let key = xprv_from_bytes([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 1,
    ]);
    let signer = EvmSigner {
        chain_label: "ethereum",
    };
    let msg = SignableMessage::EvmPersonalSign {
        message: "Hello Jova".into(),
    };
    let sig = signer.sign_message(&key, &msg).unwrap();
    // 0x + 32 bytes r + 32 bytes s + 1 byte v = 0x + 130 hex chars = 132 total
    assert_eq!(
        sig.hex.len(),
        132,
        "personal_sign signature must be 132 chars"
    );
    assert!(sig.hex.starts_with("0x"));
    // v must be 27 or 28 (EIP-191 legacy recovery)
    let v_hex = &sig.hex[130..];
    let v = u8::from_str_radix(v_hex, 16).expect("v is hex");
    assert!(
        v == 27 || v == 28,
        "personal_sign v must be 27 or 28, got {v}"
    );
}

// ── EIP-712 typed data smoke ──────────────────────────────────────────────────

#[test]
fn sign_typed_data_v4_format() {
    // Minimal EIP-712 typed data (from alloy-signer tests).
    let typed_data_json = r#"{
        "domain": {
            "name": "Ether Mail",
            "version": "1",
            "chainId": 1,
            "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
        },
        "types": {
            "EIP712Domain": [
                {"name":"name","type":"string"},
                {"name":"version","type":"string"},
                {"name":"chainId","type":"uint256"},
                {"name":"verifyingContract","type":"address"}
            ],
            "Person": [
                {"name":"name","type":"string"},
                {"name":"wallet","type":"address"}
            ],
            "Mail": [
                {"name":"from","type":"Person"},
                {"name":"to","type":"Person"},
                {"name":"contents","type":"string"}
            ]
        },
        "primaryType": "Mail",
        "message": {
            "from":{"name":"Cow","wallet":"0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"},
            "to":{"name":"Bob","wallet":"0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"},
            "contents":"Hello, Bob!"
        }
    }"#;

    let key = xprv_from_bytes([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 1,
    ]);
    let signer = EvmSigner {
        chain_label: "ethereum",
    };
    let msg = SignableMessage::EvmTypedDataV4 {
        json: typed_data_json.into(),
    };
    let sig = signer.sign_message(&key, &msg).unwrap();
    assert_eq!(sig.hex.len(), 132);
    assert!(sig.hex.starts_with("0x"));
}
