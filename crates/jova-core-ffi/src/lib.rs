//! jova-core-ffi — uniffi bindings for the public JovaWallet API.
//!
//! Phase 1: exposes JovaWallet, chain/tx/message types, and free helpers.
//! Phase 2: Bitcoin variants are now wired through to `jova-core` (BIP-84).
//! Phase 3b: Xrp variant is now wired through to `jova-core` (BIP-44 coin
//! type 144, canonical XRPL signing).
//! Phase 3c: Solana variant is now wired through (SLIP-10 ed25519 at
//! m/44'/501'/0'/0'/0', VersionedTransaction v0 signing, raw ed25519
//! message signing).

#![forbid(unsafe_code)]

use std::sync::Arc;

use jova_core::{JovaError, JovaWallet as InnerWallet, Strength};

// ---------- Chain enum ----------

#[derive(uniffi::Enum, Clone, Debug, PartialEq, Eq)]
pub enum JovaChain {
    Ethereum,
    Polygon,
    Bsc,
    Arbitrum,
    Optimism,
    Base,
    Bitcoin,
    Solana,
    Xrp,
    CustomEvm { chain_id: u64 },
}

// ---------- Unsigned tx types ----------

#[derive(uniffi::Record, Clone, Debug)]
pub struct AccessListItem {
    pub address: String,
    pub storage_keys: Vec<String>,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct EvmUnsigned {
    pub chain_id: u64,
    pub nonce: u64,
    pub to: String,
    pub value: String, // wei, decimal string (avoids u128/U256 marshalling)
    pub gas_limit: u64,
    pub max_fee_per_gas: String,
    pub max_priority_fee_per_gas: String,
    pub data: String, // 0x-prefixed hex
    pub access_list: Vec<AccessListItem>,
}

#[derive(uniffi::Enum, Clone, Debug)]
pub enum UnsignedTx {
    Evm {
        tx: EvmUnsigned,
    },
    Bitcoin {
        psbt_base64: String,
    },
    /// Phase 3c — declared for forward compatibility; returns UnsupportedChain until implemented.
    Solana {
        message_base64: String,
        recent_blockhash: String,
    },
    /// XRPL transaction (Phase 3b): a canonical XRPL JSON object as a string.
    /// The signer injects `SigningPubKey` and `TxnSignature`; callers must not
    /// pre-populate those fields.
    Xrp {
        tx_json: String,
    },
}

// ---------- Signable message types ----------

#[derive(uniffi::Enum, Clone, Debug)]
pub enum BtcMsgScheme {
    Bip322,
    Legacy,
}

#[derive(uniffi::Enum, Clone, Debug)]
pub enum SignableMessage {
    EvmPersonalSign {
        message: String,
    },
    EvmTypedDataV4 {
        json: String,
    }, // dynamic schema; String is intentional
    /// Phase 3 — declared for forward compatibility; returns UnsupportedChain until implemented.
    Solana {
        message_base64: String,
    },
    Bitcoin {
        message: String,
        address: String,
        scheme: BtcMsgScheme,
    },
}

// ---------- Output types ----------

#[derive(uniffi::Record, Clone, Debug)]
pub struct Address {
    pub chain: JovaChain,
    pub value: String,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct SignedTx {
    pub chain: JovaChain,
    pub raw_hex: String,
    pub tx_hash: String,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct Signature {
    pub hex: String,
}

// ---------- Errors ----------

#[derive(uniffi::Error, Debug, thiserror::Error)]
#[uniffi(flat_error)]
pub enum FfiError {
    #[error("invalid mnemonic")]
    InvalidMnemonic,
    #[error("invalid passphrase")]
    InvalidPassphrase,
    #[error("invalid address for {chain:?}")]
    InvalidAddress { chain: JovaChain },
    #[error("unsupported chain: {chain:?}")]
    UnsupportedChain { chain: JovaChain },
    #[error("malformed unsigned tx: {reason}")]
    MalformedUnsignedTx { reason: String },
    #[error("malformed signable message: {reason}")]
    MalformedSignableMessage { reason: String },
    #[error("signing failed: {reason}")]
    SigningFailed { reason: String },
    #[error("internal: {reason}")]
    Internal { reason: String },
    #[error("invalid private key: {reason}")]
    InvalidPrivateKey { reason: String },
}

// ---------- Conversions: JovaError → FfiError ----------

impl From<JovaError> for FfiError {
    fn from(e: JovaError) -> Self {
        match e {
            JovaError::InvalidMnemonic => Self::InvalidMnemonic,
            JovaError::InvalidPassphrase => Self::InvalidPassphrase,
            JovaError::InvalidAddress { chain } => Self::InvalidAddress {
                chain: parse_chain(&chain).unwrap_or(JovaChain::Ethereum),
            },
            JovaError::UnsupportedChain(s) => Self::UnsupportedChain {
                chain: parse_chain(&s).unwrap_or(JovaChain::Ethereum),
            },
            JovaError::MalformedUnsignedTx { reason } => Self::MalformedUnsignedTx { reason },
            JovaError::MalformedSignableMessage { reason } => {
                Self::MalformedSignableMessage { reason }
            }
            JovaError::SigningFailed { reason } => Self::SigningFailed { reason },
            JovaError::Internal { reason } => Self::Internal { reason },
            JovaError::InvalidPrivateKey { reason } => Self::InvalidPrivateKey { reason },
        }
    }
}

// ---------- Chain name parsing (FFI label ↔ JovaChain) ----------

fn parse_chain(s: &str) -> Option<JovaChain> {
    Some(match s {
        "ethereum" => JovaChain::Ethereum,
        "polygon" => JovaChain::Polygon,
        "bsc" => JovaChain::Bsc,
        "arbitrum" => JovaChain::Arbitrum,
        "optimism" => JovaChain::Optimism,
        "base" => JovaChain::Base,
        "bitcoin" => JovaChain::Bitcoin,
        "solana" => JovaChain::Solana,
        "xrp" => JovaChain::Xrp,
        _ => return None,
    })
}

// ---------- FFI JovaChain → core JovaChain ----------

fn ffi_chain_to_core(c: &JovaChain) -> Result<jova_core::JovaChain, FfiError> {
    Ok(match c {
        JovaChain::Ethereum => jova_core::JovaChain::Ethereum,
        JovaChain::Polygon => jova_core::JovaChain::Polygon,
        JovaChain::Bsc => jova_core::JovaChain::Bsc,
        JovaChain::Arbitrum => jova_core::JovaChain::Arbitrum,
        JovaChain::Optimism => jova_core::JovaChain::Optimism,
        JovaChain::Base => jova_core::JovaChain::Base,
        JovaChain::Bitcoin => jova_core::JovaChain::Bitcoin,
        JovaChain::Xrp => jova_core::JovaChain::Xrp,
        JovaChain::Solana => jova_core::JovaChain::Solana,
        JovaChain::CustomEvm { chain_id } => jova_core::JovaChain::CustomEvm {
            chain_id: *chain_id,
        },
    })
}

// ---------- core Address → FFI Address ----------

fn core_address_to_ffi(a: jova_core::Address) -> Address {
    Address {
        chain: parse_chain(&a.chain).unwrap_or(JovaChain::Ethereum),
        value: a.value,
    }
}

// ---------- core SignedTx → FFI SignedTx ----------

fn core_signed_tx_to_ffi(t: jova_core::SignedTx) -> SignedTx {
    SignedTx {
        chain: parse_chain(&t.chain).unwrap_or(JovaChain::Ethereum),
        raw_hex: t.raw_hex,
        tx_hash: t.tx_hash,
    }
}

// ---------- FFI UnsignedTx → core UnsignedTx ----------

fn ffi_unsigned_tx_to_core(t: UnsignedTx) -> Result<jova_core::UnsignedTx, FfiError> {
    match t {
        UnsignedTx::Evm { tx } => Ok(jova_core::UnsignedTx::Evm(jova_core::EvmUnsigned {
            chain_id: tx.chain_id,
            nonce: tx.nonce,
            to: tx.to,
            value: tx.value,
            gas_limit: tx.gas_limit,
            max_fee_per_gas: tx.max_fee_per_gas,
            max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
            data: tx.data,
            access_list: tx
                .access_list
                .into_iter()
                .map(|x| jova_core::AccessListItem {
                    address: x.address,
                    storage_keys: x.storage_keys,
                })
                .collect(),
        })),
        UnsignedTx::Bitcoin { psbt_base64 } => Ok(jova_core::UnsignedTx::Bitcoin { psbt_base64 }),
        UnsignedTx::Xrp { tx_json } => Ok(jova_core::UnsignedTx::Xrp { tx_json }),
        UnsignedTx::Solana {
            message_base64,
            recent_blockhash,
        } => Ok(jova_core::UnsignedTx::Solana {
            message_base64,
            recent_blockhash,
        }),
    }
}

// ---------- FFI SignableMessage → core SignableMessage ----------

fn ffi_signable_message_to_core(
    m: SignableMessage,
) -> Result<jova_core::SignableMessage, FfiError> {
    match m {
        SignableMessage::EvmPersonalSign { message } => {
            Ok(jova_core::SignableMessage::EvmPersonalSign { message })
        }
        SignableMessage::EvmTypedDataV4 { json } => {
            Ok(jova_core::SignableMessage::EvmTypedDataV4 { json })
        }
        SignableMessage::Bitcoin {
            message,
            address,
            scheme,
        } => {
            let core_scheme = match scheme {
                BtcMsgScheme::Bip322 => jova_core::BtcMsgScheme::Bip322,
                BtcMsgScheme::Legacy => jova_core::BtcMsgScheme::Legacy,
            };
            Ok(jova_core::SignableMessage::Bitcoin {
                message,
                address,
                scheme: core_scheme,
            })
        }
        SignableMessage::Solana { message_base64 } => {
            Ok(jova_core::SignableMessage::Solana { message_base64 })
        }
    }
}

// ---------- Free functions ----------

/// Generate a new random mnemonic. Returns the word string (12 or 24 words).
#[uniffi::export]
pub fn create_mnemonic(bits256: bool) -> String {
    let strength = if bits256 {
        Strength::Bits256
    } else {
        Strength::Bits128
    };
    jova_core::create_mnemonic(strength).words.clone()
}

/// Return `true` if `words` is a valid BIP-39 mnemonic with correct checksum.
#[uniffi::export]
pub fn is_valid_mnemonic(words: String, passphrase: String) -> bool {
    jova_core::is_valid_mnemonic(&words, &passphrase)
}

/// Return `true` if `addr` is a valid address for `chain`.
#[uniffi::export]
pub fn is_valid_address(addr: String, chain: JovaChain) -> bool {
    match ffi_chain_to_core(&chain) {
        Ok(core_chain) => jova_core::is_valid_address(&addr, &core_chain),
        Err(_) => false,
    }
}

// ---------- Wallet object ----------

#[derive(uniffi::Object, Debug)]
pub struct JovaWallet {
    inner: InnerWallet,
}

#[uniffi::export]
impl JovaWallet {
    /// Create a wallet from a BIP-39 mnemonic phrase and optional passphrase.
    #[uniffi::constructor]
    pub fn from_mnemonic(words: String, passphrase: String) -> Result<Arc<Self>, FfiError> {
        let inner = InnerWallet::from_mnemonic(&words, &passphrase).map_err(FfiError::from)?;
        Ok(Arc::new(Self { inner }))
    }

    /// Create a single-chain wallet from a raw private key (hex; optional 0x).
    /// Curve is chosen by `chain` (secp256k1 for EVM/BTC/XRP, ed25519 for Solana).
    #[uniffi::constructor]
    pub fn from_private_key(hex: String, chain: JovaChain) -> Result<Arc<Self>, FfiError> {
        let core_chain = ffi_chain_to_core(&chain)?;
        let inner = InnerWallet::from_private_key(&hex, &core_chain).map_err(FfiError::from)?;
        Ok(Arc::new(Self { inner }))
    }

    /// Derive the canonical address for the given chain and account index.
    pub fn address(&self, chain: JovaChain, account: u32) -> Result<Address, FfiError> {
        let core_chain = ffi_chain_to_core(&chain)?;
        Ok(core_address_to_ffi(
            self.inner
                .address(&core_chain, account)
                .map_err(FfiError::from)?,
        ))
    }

    /// Sign a transaction. For EVM, the chain ID inside the variant is authoritative.
    pub fn sign_tx(&self, tx: UnsignedTx) -> Result<SignedTx, FfiError> {
        let core_tx = ffi_unsigned_tx_to_core(tx)?;
        Ok(core_signed_tx_to_ffi(
            self.inner.sign_tx(&core_tx).map_err(FfiError::from)?,
        ))
    }

    /// Sign a message. Chain is implicit in the `SignableMessage` variant.
    pub fn sign_message(&self, msg: SignableMessage) -> Result<Signature, FfiError> {
        let core_msg = ffi_signable_message_to_core(msg)?;
        let sig = self.inner.sign_message(&core_msg).map_err(FfiError::from)?;
        Ok(Signature { hex: sig.hex })
    }
}

uniffi::setup_scaffolding!();
