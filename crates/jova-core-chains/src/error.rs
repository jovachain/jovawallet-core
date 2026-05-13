use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ChainError {
    #[error("invalid address for chain")]
    InvalidAddress,
    #[error("malformed unsigned tx: {0}")]
    MalformedUnsignedTx(String),
    #[error("malformed signable message: {0}")]
    MalformedSignableMessage(String),
    #[error("signing failed: {0}")]
    SigningFailed(String),
    #[error("internal: {0}")]
    Internal(String),
}
