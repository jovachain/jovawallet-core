use alloc::string::String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MnemonicError {
    InvalidWordCount,
    InvalidChecksum,
    InvalidWord(String),
    PassphraseTooLong,
}

#[cfg(feature = "std")]
impl std::error::Error for MnemonicError {}

impl core::fmt::Display for MnemonicError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidWordCount => f.write_str("invalid word count"),
            Self::InvalidChecksum => f.write_str("invalid checksum"),
            Self::InvalidWord(_) => f.write_str("invalid word"),
            Self::PassphraseTooLong => f.write_str("passphrase too long"),
        }
    }
}
