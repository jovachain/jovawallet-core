use jova_core_chains::{
    Address, BtcSigner, ChainSigner, SignableMessage, Signature, SignedTx, UnsignedTx, XrpSigner,
    evm::EvmSigner, sol::SolSigner,
};
use jova_core_primitives::{
    DerivationPath, Ed25519Xprv, Mnemonic, Seed, derive_ed25519, derive_secp256k1,
};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::chain::JovaChain;
use crate::error::JovaError;

/// What a `JovaWallet` derives from: either a full BIP-39 seed (HD wallet)
/// or a single imported leaf private key bound to one chain.
///
/// `Zeroize` + `ZeroizeOnDrop` ensure the raw 32-byte key material is
/// overwritten when the enum drops. `JovaChain` is not secret and is
/// skipped. The `Seed` variant is already `ZeroizeOnDrop` via
/// `jova_core_primitives::Seed`.
#[derive(Zeroize, ZeroizeOnDrop)]
enum KeyMaterial {
    /// HD wallet: per-chain keys are BIP-32 / SLIP-10 derived from this seed.
    Seed(Seed),
    /// Imported secp256k1 leaf key (EVM family, Bitcoin, or XRP). Serves only `chain`.
    Secp256k1 { key: [u8; 32], #[zeroize(skip)] chain: JovaChain },
    /// Imported ed25519 leaf key (Solana). Serves only `chain`.
    Ed25519 { key: [u8; 32], #[zeroize(skip)] chain: JovaChain },
}

pub struct JovaWallet {
    material: KeyMaterial,
}

impl core::fmt::Debug for JovaWallet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let kind = match &self.material {
            KeyMaterial::Seed(_) => "Seed",
            KeyMaterial::Secp256k1 { .. } => "Secp256k1",
            KeyMaterial::Ed25519 { .. } => "Ed25519",
        };
        write!(f, "JovaWallet(<{kind}, redacted>)")
    }
}

impl JovaWallet {
    /// Create a wallet from a mnemonic phrase and optional passphrase.
    pub fn from_mnemonic(words: &str, passphrase: &str) -> Result<Self, JovaError> {
        let seed = Mnemonic::to_seed(words, passphrase)?;
        Ok(Self {
            material: KeyMaterial::Seed(seed),
        })
    }

    /// Create a single-chain wallet from a raw 32-byte private key (hex).
    ///
    /// Accepts an optional `0x` prefix. The curve is chosen by `chain`:
    /// EVM family / Bitcoin / XRP use secp256k1; Solana uses ed25519. The
    /// resulting wallet serves ONLY `chain` — any other chain returns
    /// `UnsupportedChain`.
    pub fn from_private_key(hex: &str, chain: &JovaChain) -> Result<Self, JovaError> {
        let key = parse_private_key_hex(hex)?;
        let is_secp = matches!(
            chain,
            JovaChain::Bitcoin | JovaChain::Xrp
        ) || chain.evm_chain_id().is_some();
        if is_secp {
            // Reject scalars outside the secp256k1 group order (and zero).
            secp256k1::SecretKey::from_byte_array(key).map_err(|_| {
                JovaError::InvalidPrivateKey {
                    reason: "secp256k1_scalar_out_of_range".into(),
                }
            })?;
            Ok(Self {
                material: KeyMaterial::Secp256k1 {
                    key,
                    chain: chain.clone(),
                },
            })
        } else if matches!(chain, JovaChain::Solana) {
            // ed25519: every 32-byte value is a valid secret (length already
            // checked by parse_private_key_hex). SigningKey::from_bytes is
            // infallible, so no scalar-range check is needed.
            Ok(Self {
                material: KeyMaterial::Ed25519 {
                    key,
                    chain: chain.clone(),
                },
            })
        } else {
            Err(JovaError::UnsupportedChain(format!(
                "from_private_key unsupported chain {:?}",
                chain
            )))
        }
    }

    /// Create a wallet directly from a 64-byte BIP-39 seed.
    ///
    /// **Hardware-wallet integrations only.** This bypasses the mnemonic →
    /// seed PBKDF2 step. Use it from firmware that has already derived the
    /// seed in a secure element and only needs the signing surface.
    ///
    /// Not exposed via FFI/WASM — the FFI/WASM API stays at `from_mnemonic`.
    ///
    /// Available when the `external-rng` feature is enabled.
    #[cfg(feature = "external-rng")]
    pub fn from_seed_bytes(bytes: [u8; 64]) -> Self {
        Self {
            material: KeyMaterial::Seed(Seed::from_external_bytes(bytes)),
        }
    }

    /// Derive the canonical address for the given chain.
    ///
    /// `_account` is reserved for future multi-account support;
    /// in v1 all EVM chains use the account at index 0 within
    /// the m/44'/60'/0'/0/0 path, and Bitcoin uses the BIP-84
    /// m/84'/0'/0'/0/0 path.
    pub fn address(&self, chain: &JovaChain, _account: u32) -> Result<Address, JovaError> {
        self.ensure_chain_allowed(chain)?;
        match chain {
            JovaChain::Bitcoin => {
                let xprv = self.derive_for(chain)?;
                Ok(BtcSigner.derive_address(&xprv)?)
            }
            JovaChain::Xrp => {
                let xprv = self.derive_for(chain)?;
                Ok(XrpSigner.derive_address(&xprv)?)
            }
            JovaChain::Solana => {
                // Solana uses an ed25519 leaf key (Ed25519Xprv); SolSigner is
                // not a ChainSigner impl (the trait is locked to secp256k1
                // XPrv), so route directly.
                let xprv = self.derive_ed25519_for(chain)?;
                Ok(SolSigner.derive_address(&xprv)?)
            }
            c if c.evm_chain_id().is_some() => {
                let signer = self.evm_signer(c)?;
                let xprv = self.derive_for(c)?;
                Ok(signer.derive_address(&xprv)?)
            }
            other => Err(JovaError::UnsupportedChain(format!("{:?}", other))),
        }
    }

    /// Sign a transaction. For EVM, the chain ID inside the variant is authoritative.
    /// For Bitcoin, the PSBT carries its own input descriptors.
    pub fn sign_tx(&self, unsigned: &UnsignedTx) -> Result<SignedTx, JovaError> {
        self.ensure_chain_allowed(&chain_of_unsigned_tx(unsigned))?;
        match unsigned {
            UnsignedTx::Evm(evm) => {
                // Derive the canonical chain label from the chain ID in the tx.
                let chain_label = chain_label_from_evm_chain_id(evm.chain_id);
                // We need a `&'static str` for EvmSigner; map to one.
                let static_label = static_chain_label(evm.chain_id);
                let signer = EvmSigner {
                    chain_label: static_label,
                };
                let xprv = self.derive_for(&chain_of_unsigned_tx(unsigned))?;
                let mut signed = signer.sign_tx(&xprv, unsigned)?;
                signed.chain = chain_label;
                Ok(signed)
            }
            UnsignedTx::Bitcoin { .. } => {
                let xprv = self.derive_for(&chain_of_unsigned_tx(unsigned))?;
                Ok(BtcSigner.sign_tx(&xprv, unsigned)?)
            }
            UnsignedTx::Xrp { .. } => {
                let xprv = self.derive_for(&chain_of_unsigned_tx(unsigned))?;
                Ok(XrpSigner.sign_tx(&xprv, unsigned)?)
            }
            UnsignedTx::Solana { .. } => {
                let xprv = self.derive_ed25519_for(&JovaChain::Solana)?;
                Ok(SolSigner.sign_tx(&xprv, unsigned)?)
            }
        }
    }

    /// Sign a message. Chain is implicit in the `SignableMessage` variant.
    pub fn sign_message(&self, msg: &SignableMessage) -> Result<Signature, JovaError> {
        self.ensure_chain_allowed(&chain_of_signable_message(msg))?;
        match msg {
            SignableMessage::EvmPersonalSign { .. } | SignableMessage::EvmTypedDataV4 { .. } => {
                let signer = EvmSigner {
                    chain_label: "ethereum",
                };
                let xprv = self.derive_for(&chain_of_signable_message(msg))?;
                Ok(signer.sign_message(&xprv, msg)?)
            }
            SignableMessage::Bitcoin { .. } => {
                let xprv = self.derive_for(&chain_of_signable_message(msg))?;
                Ok(BtcSigner.sign_message(&xprv, msg)?)
            }
            SignableMessage::Solana { .. } => {
                let xprv = self.derive_ed25519_for(&JovaChain::Solana)?;
                Ok(SolSigner.sign_message(&xprv, msg)?)
            }
        }
    }

    /// For a key-material wallet, the single chain it is bound to.
    /// `Seed` wallets return `None` (they serve every chain).
    fn bound_chain(&self) -> Option<&JovaChain> {
        match &self.material {
            KeyMaterial::Seed(_) => None,
            KeyMaterial::Secp256k1 { chain, .. } | KeyMaterial::Ed25519 { chain, .. } => Some(chain),
        }
    }

    /// Reject any operation on a chain the imported key is not bound to.
    /// No-op for `Seed` wallets.
    fn ensure_chain_allowed(&self, requested: &JovaChain) -> Result<(), JovaError> {
        if let Some(bound) = self.bound_chain() {
            if bound != requested {
                return Err(JovaError::UnsupportedChain(format!(
                    "wallet bound to {:?}, requested {:?}",
                    bound, requested
                )));
            }
        }
        Ok(())
    }

    fn evm_signer(&self, chain: &JovaChain) -> Result<EvmSigner, JovaError> {
        if chain.evm_chain_id().is_none() {
            return Err(JovaError::UnsupportedChain(format!("{:?}", chain)));
        }
        Ok(EvmSigner {
            chain_label: chain.label(),
        })
    }

    fn derive_for(&self, chain: &JovaChain) -> Result<jova_core_primitives::XPrv, JovaError> {
        match &self.material {
            // Imported secp256k1 leaf key: wrap raw bytes in an XPrv. The chain
            // code is irrelevant for leaf signing — the secp256k1 signers read
            // only private_key_bytes()/public_key_*(). Strict scoping already
            // enforced by ensure_chain_allowed at the public entry points.
            KeyMaterial::Secp256k1 { key, .. } => Ok(
                jova_core_primitives::XPrv::from_raw_key_and_chain_code(*key, [0u8; 32]),
            ),
            KeyMaterial::Ed25519 { .. } => Err(JovaError::UnsupportedChain(format!(
                "ed25519 key cannot derive secp256k1 chain {:?}",
                chain
            ))),
            KeyMaterial::Seed(_) => self.derive_path(chain.derivation_path()),
        }
    }

    fn derive_path(&self, path_str: &str) -> Result<jova_core_primitives::XPrv, JovaError> {
        let seed = match &self.material {
            KeyMaterial::Seed(s) => s,
            _ => {
                return Err(JovaError::Internal {
                    reason: "derive_path_called_on_key_material".into(),
                });
            }
        };
        let path = DerivationPath::parse(path_str).map_err(|_| JovaError::Internal {
            reason: "bad_path".into(),
        })?;
        derive_secp256k1(seed, &path).map_err(|_| JovaError::Internal {
            reason: "derive_failed".into(),
        })
    }

    fn derive_ed25519_for(&self, chain: &JovaChain) -> Result<Ed25519Xprv, JovaError> {
        match &self.material {
            KeyMaterial::Ed25519 { key, .. } => Ok(
                Ed25519Xprv::from_raw_secret_and_chain_code(*key, [0u8; 32]),
            ),
            KeyMaterial::Secp256k1 { .. } => Err(JovaError::UnsupportedChain(format!(
                "secp256k1 key cannot derive ed25519 chain {:?}",
                chain
            ))),
            KeyMaterial::Seed(_) => self.derive_ed25519_path(chain.derivation_path()),
        }
    }

    /// SLIP-10 ed25519 derivation. The `DerivationPath::parse` helper applies
    /// the same hardening syntax used by secp256k1 paths; SLIP-10 ed25519
    /// requires every component to be hardened — `derive_ed25519` enforces
    /// that and returns `HardenedRequired` otherwise. The canonical Solana
    /// path is `m/44'/501'/0'/0'/0'`.
    fn derive_ed25519_path(&self, path_str: &str) -> Result<Ed25519Xprv, JovaError> {
        let seed = match &self.material {
            KeyMaterial::Seed(s) => s,
            _ => {
                return Err(JovaError::Internal {
                    reason: "derive_ed25519_path_called_on_key_material".into(),
                });
            }
        };
        let path = DerivationPath::parse(path_str).map_err(|_| JovaError::Internal {
            reason: "bad_path".into(),
        })?;
        derive_ed25519(seed, &path.indices).map_err(|_| JovaError::Internal {
            reason: "derive_failed".into(),
        })
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

/// Parse a private-key hex string (optional `0x` prefix) into 32 bytes.
fn parse_private_key_hex(hex_str: &str) -> Result<[u8; 32], JovaError> {
    let trimmed = hex_str.trim();
    let body = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    let bytes = hex::decode(body).map_err(|_| JovaError::InvalidPrivateKey {
        reason: "not_hex".into(),
    })?;
    let arr: [u8; 32] = bytes.try_into().map_err(|_| JovaError::InvalidPrivateKey {
        reason: "expected_32_bytes".into(),
    })?;
    Ok(arr)
}

/// The chain a given unsigned tx targets (used for key-material scoping).
fn chain_of_unsigned_tx(tx: &UnsignedTx) -> JovaChain {
    match tx {
        UnsignedTx::Evm(evm) => match evm.chain_id {
            1 => JovaChain::Ethereum,
            137 => JovaChain::Polygon,
            56 => JovaChain::Bsc,
            42161 => JovaChain::Arbitrum,
            10 => JovaChain::Optimism,
            8453 => JovaChain::Base,
            other => JovaChain::CustomEvm { chain_id: other },
        },
        UnsignedTx::Bitcoin { .. } => JovaChain::Bitcoin,
        UnsignedTx::Xrp { .. } => JovaChain::Xrp,
        UnsignedTx::Solana { .. } => JovaChain::Solana,
    }
}

/// The chain a given signable message targets (used for key-material scoping).
fn chain_of_signable_message(msg: &SignableMessage) -> JovaChain {
    match msg {
        SignableMessage::EvmPersonalSign { .. } | SignableMessage::EvmTypedDataV4 { .. } => {
            JovaChain::Ethereum
        }
        SignableMessage::Bitcoin { .. } => JovaChain::Bitcoin,
        SignableMessage::Solana { .. } => JovaChain::Solana,
    }
}

/// Return `true` if `addr` is a valid address for `chain`.
pub fn is_valid_address(addr: &str, chain: &JovaChain) -> bool {
    match chain {
        JovaChain::Bitcoin => jova_core_chains::btc::validate_btc_address(addr),
        JovaChain::Xrp => jova_core_chains::xrp::validate_xrp_address(addr),
        JovaChain::Solana => jova_core_chains::sol::validate_sol_address(addr),
        c if c.evm_chain_id().is_some() => jova_core_chains::evm::validate_address(addr),
        _ => false,
    }
}
