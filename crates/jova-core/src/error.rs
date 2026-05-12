use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum JovaError {
    #[error("invalid mnemonic")]
    InvalidMnemonic,
    #[error("invalid passphrase")]
    InvalidPassphrase,
    #[error("invalid address for {chain}")]
    InvalidAddress { chain: String },
    #[error("unsupported chain: {0}")]
    UnsupportedChain(String),
    #[error("malformed unsigned tx: {reason}")]
    MalformedUnsignedTx { reason: String },
    #[error("malformed signable message: {reason}")]
    MalformedSignableMessage { reason: String },
    #[error("signing failed: {reason}")]
    SigningFailed { reason: String },
    #[error("internal: {reason}")]
    Internal { reason: String },
}

impl From<jova_core_primitives::MnemonicError> for JovaError {
    fn from(_: jova_core_primitives::MnemonicError) -> Self {
        JovaError::InvalidMnemonic
    }
}

impl From<jova_core_chains::ChainError> for JovaError {
    fn from(e: jova_core_chains::ChainError) -> Self {
        match e {
            jova_core_chains::ChainError::InvalidAddress => {
                JovaError::InvalidAddress { chain: "evm".into() }
            }
            jova_core_chains::ChainError::MalformedUnsignedTx(r) => {
                JovaError::MalformedUnsignedTx { reason: r }
            }
            jova_core_chains::ChainError::MalformedSignableMessage(r) => {
                JovaError::MalformedSignableMessage { reason: r }
            }
            jova_core_chains::ChainError::SigningFailed(r) => {
                JovaError::SigningFailed { reason: r }
            }
            jova_core_chains::ChainError::Internal(r) => {
                JovaError::Internal { reason: r }
            }
        }
    }
}
