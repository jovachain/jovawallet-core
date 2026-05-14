//! Phase 7: `external-rng` feature tests.
//!
//! Validates `Mnemonic::generate_with` against a deterministic RNG so the
//! output is reproducible — production firmware uses a hardware TRNG and the
//! mnemonic is consequently non-deterministic; the test substitutes a fixed
//! byte stream so the assertion is hermetic.

#![cfg(feature = "external-rng")]

use jova_core_primitives::{JovaRng, Mnemonic, RngError, Strength};

/// Deterministic RNG: writes the same byte stream every call.
struct FixedRng<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> FixedRng<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }
}

impl JovaRng for FixedRng<'_> {
    fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), RngError> {
        if self.bytes.len() - self.pos < dest.len() {
            return Err(RngError::Unavailable);
        }
        dest.copy_from_slice(&self.bytes[self.pos..self.pos + dest.len()]);
        self.pos += dest.len();
        Ok(())
    }
}

/// Always-fails RNG, for the error path.
struct FailingRng;

impl JovaRng for FailingRng {
    fn fill_bytes(&mut self, _dest: &mut [u8]) -> Result<(), RngError> {
        Err(RngError::HealthCheckFailed)
    }
}

#[test]
fn generate_with_all_zero_entropy_produces_abandon_about() {
    // 128 bits of zeros → the BIP-39 standard "abandon abandon … about" mnemonic.
    let mut rng = FixedRng::new(&[0u8; 32]);
    let m = Mnemonic::generate_with(Strength::Bits128, &mut rng).expect("generate");
    assert_eq!(
        m.words,
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
    );
    assert_eq!(m.passphrase, "");
}

#[test]
fn generate_with_24_words_at_bits256() {
    let mut rng = FixedRng::new(&[0u8; 32]);
    let m = Mnemonic::generate_with(Strength::Bits256, &mut rng).expect("generate");
    assert_eq!(m.words.split_whitespace().count(), 24);
}

#[test]
fn generate_with_propagates_rng_failure() {
    let mut rng = FailingRng;
    let result = Mnemonic::generate_with(Strength::Bits128, &mut rng);
    match result {
        Ok(_) => panic!("expected RngError::HealthCheckFailed, got Ok"),
        Err(e) => assert_eq!(e, RngError::HealthCheckFailed),
    }
}

#[test]
fn generate_with_uses_only_requested_entropy_bytes() {
    // 128-bit strength must consume exactly 16 bytes, not 32.
    let mut rng = FixedRng::new(&[0u8; 32]);
    let _m = Mnemonic::generate_with(Strength::Bits128, &mut rng).expect("generate");
    assert_eq!(rng.pos, 16, "Bits128 should consume 16 entropy bytes");
}

#[test]
fn rng_error_implements_display() {
    let s = format!("{}", RngError::Unavailable);
    assert!(s.contains("unavailable"));
    let s = format!("{}", RngError::HealthCheckFailed);
    assert!(s.contains("health"));
}
