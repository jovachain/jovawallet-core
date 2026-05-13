use jova_core_chains::{
    evm::EvmSigner, Address, ChainSigner, SignableMessage, Signature, SignedTx, UnsignedTx,
};
use jova_core_primitives::{derive_secp256k1, DerivationPath, Mnemonic, Seed};

use crate::chain::JovaChain;
use crate::error::JovaError;

pub struct JovaWallet {
    seed: Seed,
}

impl JovaWallet {
    /// Create a wallet from a mnemonic phrase and optional passphrase.
    pub fn from_mnemonic(words: &str, passphrase: &str) -> Result<Self, JovaError> {
        let seed = Mnemonic::to_seed(words, passphrase)?;
        Ok(Self { seed })
    }

    /// Derive the canonical address for the given chain.
    ///
    /// `_account` is reserved for future multi-account support;
    /// in v1 all EVM chains use the account at index 0 within
    /// the m/44'/60'/0'/0/0 path.
    pub fn address(&self, chain: &JovaChain, _account: u32) -> Result<Address, JovaError> {
        let signer = self.evm_signer(chain)?;
        let xprv = self.derive_for(chain)?;
        Ok(signer.derive_address(&xprv)?)
    }

    /// Sign a transaction. For EVM, the chain ID inside the variant is authoritative.
    pub fn sign_tx(&self, unsigned: &UnsignedTx) -> Result<SignedTx, JovaError> {
        match unsigned {
            UnsignedTx::Evm(evm) => {
                // Derive the canonical chain label from the chain ID in the tx.
                let chain_label = chain_label_from_evm_chain_id(evm.chain_id);
                // We need a `&'static str` for EvmSigner; map to one.
                let static_label = static_chain_label(evm.chain_id);
                let signer = EvmSigner { chain_label: static_label };
                let xprv = self.derive_path("m/44'/60'/0'/0/0")?;
                let mut signed = signer.sign_tx(&xprv, unsigned)?;
                signed.chain = chain_label;
                Ok(signed)
            }
            // Phase 2+ adds Bitcoin, Solana, XRP arms here.
        }
    }

    /// Sign a message. Chain is implicit in the `SignableMessage` variant.
    pub fn sign_message(&self, msg: &SignableMessage) -> Result<Signature, JovaError> {
        match msg {
            SignableMessage::EvmPersonalSign { .. } | SignableMessage::EvmTypedDataV4 { .. } => {
                let signer = EvmSigner { chain_label: "ethereum" };
                let xprv = self.derive_path("m/44'/60'/0'/0/0")?;
                Ok(signer.sign_message(&xprv, msg)?)
            }
            // Phase 2+ adds Solana and Bitcoin arms.
        }
    }

    fn evm_signer(&self, chain: &JovaChain) -> Result<EvmSigner, JovaError> {
        if chain.evm_chain_id().is_none() {
            return Err(JovaError::UnsupportedChain(format!("{:?}", chain)));
        }
        Ok(EvmSigner { chain_label: chain.label() })
    }

    fn derive_for(&self, chain: &JovaChain) -> Result<jova_core_primitives::XPrv, JovaError> {
        self.derive_path(chain.derivation_path())
    }

    fn derive_path(&self, path_str: &str) -> Result<jova_core_primitives::XPrv, JovaError> {
        let path = DerivationPath::parse(path_str)
            .map_err(|_| JovaError::Internal { reason: "bad_path".into() })?;
        derive_secp256k1(&self.seed, &path)
            .map_err(|_| JovaError::Internal { reason: "derive_failed".into() })
    }
}

/// Map an EVM chain ID back to the canonical chain label string.
/// Unknown chain IDs return `"customEvm"`.
fn chain_label_from_evm_chain_id(id: u64) -> String {
    match id {
        1 => "ethereum".to_string(),
        137 => "polygon".to_string(),
        56 => "bsc".to_string(),
        42161 => "arbitrum".to_string(),
        10 => "optimism".to_string(),
        8453 => "base".to_string(),
        _ => "customEvm".to_string(),
    }
}

/// Map an EVM chain ID to a `&'static str` label for `EvmSigner`.
/// Unknown chain IDs map to `"customEvm"`.
fn static_chain_label(id: u64) -> &'static str {
    match id {
        1 => "ethereum",
        137 => "polygon",
        56 => "bsc",
        42161 => "arbitrum",
        10 => "optimism",
        8453 => "base",
        _ => "customEvm",
    }
}

/// Return `true` if `addr` is a valid address for `chain`.
pub fn is_valid_address(addr: &str, chain: &JovaChain) -> bool {
    if chain.evm_chain_id().is_some() {
        jova_core_chains::evm::validate_address(addr)
    } else {
        false
    }
}
