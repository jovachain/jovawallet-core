//! Hardware-RNG trait for firmware integrations.
//!
//! Firmware targets (Cortex-M, RISC-V, etc.) usually have a hardware TRNG
//! exposed via an HAL crate and don't want to depend on `getrandom` (which
//! pulls in `std` on most embedded targets). Implement `JovaRng` against
//! whatever entropy source the platform provides and call
//! [`Mnemonic::generate_with`](crate::Mnemonic::generate_with).
//!
//! # Example: STM32 TRNG via stm32f4xx-hal
//!
//! ```ignore
//! use stm32f4xx_hal::rng::Rng;
//! use jova_core_primitives::{JovaRng, Mnemonic, Strength};
//!
//! struct StmTrng<'a>(&'a mut Rng);
//! impl<'a> JovaRng for StmTrng<'a> {
//!     fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), RngError> {
//!         self.0.read(dest).map_err(|_| RngError::Unavailable)
//!     }
//! }
//!
//! let mut trng = StmTrng(&mut hal_rng);
//! let mnemonic = Mnemonic::generate_with(Strength::Bits128, &mut trng)?;
//! ```
//!
//! # Example: software PRNG seeded from a secure element
//!
//! For platforms without a TRNG, draw a seed from an ATECC608 / OPTIGA Trust M
//! / similar secure element and feed it to a CSPRNG (e.g. `rand_chacha`).
//! Document the entropy chain — auditors check this.

#[cfg(feature = "std")]
extern crate std;

/// Errors a [`JovaRng`] implementation may return.
///
/// Kept minimal so this trait is `no_std`-clean. Implementations attach their
/// own diagnostic context out-of-band (e.g. firmware logging).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RngError {
    /// Entropy source is temporarily unavailable (busy, locked, etc.).
    /// Caller may retry.
    Unavailable,
    /// Entropy source produced an output the caller considers low-quality.
    /// Implementations should self-test (e.g. NIST SP 800-90B health checks)
    /// and return this if a test fails.
    HealthCheckFailed,
}

impl core::fmt::Display for RngError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Unavailable => f.write_str("RNG unavailable"),
            Self::HealthCheckFailed => f.write_str("RNG health check failed"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for RngError {}

/// A random-number generator the firmware provides.
///
/// `fill_bytes` writes `dest.len()` bytes of entropy. The implementation is
/// responsible for cryptographic quality — `jova-core-primitives` only consumes
/// the bytes, it does not post-process them.
pub trait JovaRng {
    /// Fill `dest` with entropy. Returns `Err` if the source is unavailable
    /// or self-detected as broken.
    fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), RngError>;
}
