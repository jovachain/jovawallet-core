use alloc::string::{String, ToString};
use bip39::Mnemonic as Bip39Mnemonic;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::MnemonicError;
use crate::seed::Seed;

#[derive(Clone, Copy, Debug)]
pub enum Strength {
    Bits128,
    Bits256,
}

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Mnemonic {
    pub words: String,
    pub passphrase: String,
}

impl Mnemonic {
    /// Generate a new random mnemonic.
    ///
    /// Uses OS entropy via `getrandom`. Only available when the `std` feature is enabled.
    #[cfg(feature = "std")]
    pub fn generate(strength: Strength) -> Self {
        // bip39 2.2.x `generate()` requires the `rand` feature which we don't enable
        // (it poisons no_std). Instead, we draw raw entropy via getrandom and call
        // `from_entropy()` which is always available.
        let entropy_bytes = match strength {
            Strength::Bits128 => 16, // 128 bits → 12 words
            Strength::Bits256 => 32, // 256 bits → 24 words
        };
        let mut entropy = [0u8; 32];
        getrandom::fill(&mut entropy[..entropy_bytes]).expect("OS entropy available");
        let m = Bip39Mnemonic::from_entropy(&entropy[..entropy_bytes])
            .expect("entropy length is valid");
        Self { words: m.to_string(), passphrase: String::new() }
    }

    pub fn validate(words: &str, _passphrase: &str) -> Result<(), MnemonicError> {
        // `parse()` is available because bip39's `alloc` feature enables `unicode-normalization`.
        match Bip39Mnemonic::parse(words) {
            Ok(_) => Ok(()),
            Err(bip39::Error::BadWordCount(_)) => Err(MnemonicError::InvalidWordCount),
            Err(bip39::Error::UnknownWord(idx)) => {
                let word = words.split_whitespace().nth(idx).unwrap_or("?").to_string();
                Err(MnemonicError::InvalidWord(word))
            }
            Err(bip39::Error::BadEntropyBitCount(_)) => Err(MnemonicError::InvalidWordCount),
            Err(bip39::Error::InvalidChecksum) => Err(MnemonicError::InvalidChecksum),
            Err(_) => Err(MnemonicError::InvalidChecksum),
        }
    }

    pub fn to_seed(words: &str, passphrase: &str) -> Result<Seed, MnemonicError> {
        Self::validate(words, passphrase)?;
        if passphrase.len() > 256 {
            return Err(MnemonicError::PassphraseTooLong);
        }
        let m = Bip39Mnemonic::parse(words).expect("validated above");
        // `to_seed()` is available because bip39's `alloc` feature enables `unicode-normalization`.
        let mut bytes = m.to_seed(passphrase);
        let seed = Seed::from_bytes(bytes);
        bytes.zeroize();
        Ok(seed)
    }
}
