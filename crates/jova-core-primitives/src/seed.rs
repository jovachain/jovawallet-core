use zeroize::{Zeroize, ZeroizeOnDrop};

// NOT Clone — `docs/memory-and-keys.md` audit checklist requires this.
// Anything that needs a seed takes &Seed; ownership is unique to JovaWalletInner.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Seed([u8; 64]);

impl Seed {
    pub(crate) fn from_bytes(bytes: [u8; 64]) -> Self {
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
