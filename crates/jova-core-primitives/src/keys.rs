use zeroize::{Zeroize, ZeroizeOnDrop};

// NOT Clone. Per-call derivation produces a fresh XPrv that lives only for the
// duration of the signing call; chain signers take &XPrv and don't need to copy.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct XPrv {
    pub(crate) key: [u8; 32],
    pub(crate) chain_code: [u8; 32],
}

impl XPrv {
    /// Construct an `XPrv` directly from raw key material.
    ///
    /// **Use only in tests and test-vector harnesses.**  Normal code should
    /// produce an `XPrv` via `derive_secp256k1`.
    pub fn from_raw_key_and_chain_code(key: [u8; 32], chain_code: [u8; 32]) -> Self {
        Self { key, chain_code }
    }

    pub fn private_key_bytes(&self) -> &[u8; 32] {
        &self.key
    }

    pub fn public_key_uncompressed(&self) -> [u8; 65] {
        let secp = secp256k1::Secp256k1::signing_only();
        let sk = secp256k1::SecretKey::from_byte_array(self.key).expect("valid sk");
        let pk = sk.public_key(&secp);
        pk.serialize_uncompressed()
    }

    pub fn public_key_compressed(&self) -> [u8; 33] {
        let secp = secp256k1::Secp256k1::signing_only();
        let sk = secp256k1::SecretKey::from_byte_array(self.key).expect("valid sk");
        let pk = sk.public_key(&secp);
        pk.serialize()
    }
}

impl core::fmt::Debug for XPrv {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("XPrv(<redacted>)")
    }
}
