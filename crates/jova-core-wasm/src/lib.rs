//! jova-core-wasm — wasm-bindgen bindings for the public JovaWallet API.
//!
//! Phase 6 surface: full `JovaWallet` for EVM + SOL. Bitcoin and XRP variants
//! are rejected at runtime with `unsupportedChain` per the 2026-05-11 user
//! decision; see `bindings/wasm/COVERAGE.md`.

#![forbid(unsafe_code)]

use jova_core::{
    JovaChain, JovaError, JovaWallet as InnerWallet, Mnemonic as CoreMnemonic, SignableMessage,
    Signature, SignedTx, Strength, UnsignedTx,
};
use serde::Serialize;
use serde_wasm_bindgen as swb;
use wasm_bindgen::prelude::*;

// The `getrandom_02` import exists only to force Cargo to feature-unify the
// `js` feature on the 0.2 line of getrandom (pulled in transitively by alloy
// and the Solana split crates).  The crate is otherwise unused; suppress the
// unused-crate-dependencies warning if the linter ever surfaces it.
#[allow(unused_imports)]
use getrandom_02 as _;

#[wasm_bindgen(start)]
pub fn _start() {
    // Hook for browser-side panic-to-console mapping; useful for debug builds.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
}

// ---------- Free functions ----------

#[wasm_bindgen(js_name = createMnemonic)]
pub fn create_mnemonic(bits256: bool) -> Result<JsValue, JsValue> {
    let s = if bits256 {
        Strength::Bits256
    } else {
        Strength::Bits128
    };
    let m = jova_core::create_mnemonic(s);
    // Project the zeroize-protected Mnemonic to a plain `{ words, passphrase }`
    // shape that serde-wasm-bindgen can round-trip through JS.  Mnemonic
    // derives Serialize itself in jova-core-primitives, but we re-pack to keep
    // the JS surface defined in this crate.
    let payload = MnemonicJs {
        words: m.words.clone(),
        passphrase: m.passphrase.clone(),
    };
    swb::to_value(&payload).map_err(jserr)
}

#[wasm_bindgen(js_name = isValidMnemonic)]
pub fn is_valid_mnemonic(words: &str, passphrase: &str) -> bool {
    jova_core::is_valid_mnemonic(words, passphrase)
}

#[wasm_bindgen(js_name = isValidAddress)]
pub fn is_valid_address(addr: &str, chain_js: JsValue) -> Result<bool, JsValue> {
    let chain: JovaChain = swb::from_value(chain_js).map_err(jserr)?;
    Ok(jova_core::is_valid_address(addr, &chain))
}

// ---------- Wallet object ----------

#[wasm_bindgen]
pub struct JovaWallet {
    inner: Option<InnerWallet>, // Some until destroy(); None after.
}

#[wasm_bindgen]
impl JovaWallet {
    #[wasm_bindgen(constructor)]
    pub fn new(mnemonic_js: JsValue) -> Result<JovaWallet, JsValue> {
        let m: MnemonicJs = swb::from_value(mnemonic_js).map_err(jserr)?;
        let inner = InnerWallet::from_mnemonic(&m.words, &m.passphrase).map_err(jova_err)?;
        Ok(JovaWallet { inner: Some(inner) })
    }

    #[wasm_bindgen(js_name = destroy)]
    pub fn destroy(&mut self) {
        // Dropping the inner wallet zeroizes via Drop on Seed.
        self.inner = None;
    }

    #[wasm_bindgen(js_name = address)]
    pub fn address(&self, chain_js: JsValue, account: u32) -> Result<JsValue, JsValue> {
        let inner = self
            .inner
            .as_ref()
            .ok_or_else(|| jova_err_payload(JsErrorPayload::wallet_destroyed()))?;
        let chain: JovaChain = swb::from_value(chain_js).map_err(jserr)?;
        // Bitcoin / XRP are deferred in WASM; address derivation also routes
        // through engine crates we have not validated in this build.  Match
        // the signing policy: reject at the WASM boundary.
        if matches!(chain, JovaChain::Bitcoin) {
            return Err(jova_err_payload(JsErrorPayload::unsupported_chain(
                "bitcoin",
            )));
        }
        if matches!(chain, JovaChain::Xrp) {
            return Err(jova_err_payload(JsErrorPayload::unsupported_chain("xrp")));
        }
        let a = inner.address(&chain, account).map_err(jova_err)?;
        swb::to_value(&a).map_err(jserr)
    }

    #[wasm_bindgen(js_name = signTx)]
    pub fn sign_tx(&self, unsigned_js: JsValue, account: u32) -> Result<JsValue, JsValue> {
        let inner = self
            .inner
            .as_ref()
            .ok_or_else(|| jova_err_payload(JsErrorPayload::wallet_destroyed()))?;
        let unsigned: UnsignedTx = swb::from_value(unsigned_js).map_err(jserr)?;
        // Reject BTC / XRP before the inner wallet touches them.
        match &unsigned {
            UnsignedTx::Bitcoin { .. } => {
                return Err(jova_err_payload(JsErrorPayload::unsupported_chain(
                    "bitcoin",
                )));
            }
            UnsignedTx::Xrp { .. } => {
                return Err(jova_err_payload(JsErrorPayload::unsupported_chain("xrp")));
            }
            _ => {}
        }
        let signed: SignedTx = inner.sign_tx(&unsigned, account).map_err(jova_err)?;
        swb::to_value(&signed).map_err(jserr)
    }

    #[wasm_bindgen(js_name = signMessage)]
    pub fn sign_message(&self, msg_js: JsValue, account: u32) -> Result<JsValue, JsValue> {
        let inner = self
            .inner
            .as_ref()
            .ok_or_else(|| jova_err_payload(JsErrorPayload::wallet_destroyed()))?;
        let msg: SignableMessage = swb::from_value(msg_js).map_err(jserr)?;
        if matches!(msg, SignableMessage::Bitcoin { .. }) {
            return Err(jova_err_payload(JsErrorPayload::unsupported_chain(
                "bitcoin",
            )));
        }
        let sig: Signature = inner.sign_message(&msg, account).map_err(jova_err)?;
        swb::to_value(&sig).map_err(jserr)
    }
}

// ---------- Local DTOs ----------

#[derive(serde::Deserialize, Serialize)]
struct MnemonicJs {
    words: String,
    #[serde(default)]
    passphrase: String,
}

// Round-trip via CoreMnemonic to ensure `MnemonicJs` and the Rust core
// `Mnemonic` stay structurally aligned (compile-time check).
const _MNEMONIC_FIELDS_MATCH: fn() = || {
    let _ = |m: CoreMnemonic| MnemonicJs {
        words: m.words.clone(),
        passphrase: m.passphrase.clone(),
    };
};

// ---------- Error mapping ----------

fn jserr<E: std::fmt::Display>(e: E) -> JsValue {
    JsValue::from_str(&format!("{}", e))
}

#[derive(Serialize)]
struct JsErrorPayload<'a> {
    kind: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    chain: Option<String>,
}

impl<'a> JsErrorPayload<'a> {
    fn unsupported_chain(chain: &str) -> JsErrorPayload<'static> {
        JsErrorPayload {
            kind: "unsupportedChain",
            reason: None,
            chain: Some(chain.to_string()),
        }
    }
    fn wallet_destroyed() -> JsErrorPayload<'static> {
        JsErrorPayload {
            kind: "internal",
            reason: Some("wallet destroyed".to_string()),
            chain: None,
        }
    }
}

fn jova_err_payload(payload: JsErrorPayload<'_>) -> JsValue {
    swb::to_value(&payload).unwrap_or_else(|_| JsValue::from_str(payload.kind))
}

fn jova_err(e: JovaError) -> JsValue {
    let payload = match &e {
        JovaError::InvalidMnemonic => JsErrorPayload {
            kind: "invalidMnemonic",
            reason: None,
            chain: None,
        },
        JovaError::InvalidPassphrase => JsErrorPayload {
            kind: "invalidPassphrase",
            reason: None,
            chain: None,
        },
        JovaError::InvalidAddress { chain } => JsErrorPayload {
            kind: "invalidAddress",
            reason: None,
            chain: Some(chain.clone()),
        },
        JovaError::UnsupportedChain(s) => JsErrorPayload {
            kind: "unsupportedChain",
            reason: None,
            chain: Some(s.clone()),
        },
        JovaError::MalformedUnsignedTx { reason } => JsErrorPayload {
            kind: "malformedUnsignedTx",
            reason: Some(reason.clone()),
            chain: None,
        },
        JovaError::MalformedSignableMessage { reason } => JsErrorPayload {
            kind: "malformedSignableMessage",
            reason: Some(reason.clone()),
            chain: None,
        },
        JovaError::SigningFailed { reason } => JsErrorPayload {
            kind: "signingFailed",
            reason: Some(reason.clone()),
            chain: None,
        },
        JovaError::Internal { reason } => JsErrorPayload {
            kind: "internal",
            reason: Some(reason.clone()),
            chain: None,
        },
        JovaError::InvalidPrivateKey { reason } => JsErrorPayload {
            kind: "invalidPrivateKey",
            reason: Some(reason.clone()),
            chain: None,
        },
    };
    swb::to_value(&payload).unwrap_or_else(|_| JsValue::from_str(&e.to_string()))
}
