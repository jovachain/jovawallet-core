use zeroize::{Zeroize, ZeroizeOnDrop};

// NOT Clone — `docs/memory-and-keys.md` audit checklist requires this.
// Anything that needs a seed takes &Seed; ownership is unique to JovaWalletInner.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Seed([u8; 64]);

impl Seed {
    pub(crate) fn from_bytes(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    /// Construct a `Seed` from an externally-supplied 64-byte buffer.
    ///
    /// Available when the `external-rng` feature is enabled (Phase 7 hardware
    /// integration). Firmware that derives its own BIP-39 → seed conversion in
    /// a secure element can call this to wrap the bytes in the same zeroizing
    /// container the SDK uses internally.
    ///
    /// The caller is responsible for the seed's provenance — this function
    /// does not validate that `bytes` is a real BIP-39-derived seed.
    #[cfg(feature = "external-rng")]
    pub fn from_external_bytes(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }
}

impl core::fmt::Debug for Seed {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Seed(<redacted>)")
    }
}
