use jova_core_chains::btc::{derive_p2wpkh, validate_btc_address};
use jova_core_primitives::{DerivationPath, Mnemonic, derive_secp256k1};

const BIP84_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fn xprv_at(path_str: &str) -> jova_core_primitives::XPrv {
    let seed = Mnemonic::to_seed(BIP84_MNEMONIC, "").expect("seed");
    let path = DerivationPath::parse(path_str).expect("path");
    derive_secp256k1(&seed, &path).expect("derive")
}

#[test]
fn bip84_first_external_address_matches_official_vector() {
    let addr = derive_p2wpkh(&xprv_at("m/84'/0'/0'/0/0")).expect("derive");
    assert_eq!(addr, "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu");
}

#[test]
fn bip84_second_external_address_matches_official_vector() {
    let addr = derive_p2wpkh(&xprv_at("m/84'/0'/0'/0/1")).expect("derive");
    assert_eq!(addr, "bc1qnjg0jd8228aq7egyzacy8cys3knf9xvrerkf9g");
}

#[test]
fn bip84_first_change_address_matches_official_vector() {
    let addr = derive_p2wpkh(&xprv_at("m/84'/0'/0'/1/0")).expect("derive");
    assert_eq!(addr, "bc1q8c6fshw2dlwun7ekn9qwf37cu2rn755upcp6el");
}

#[test]
fn validates_known_bech32_mainnet_addresses() {
    assert!(validate_btc_address(
        "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu"
    ));
    assert!(validate_btc_address(
        "bc1qnjg0jd8228aq7egyzacy8cys3knf9xvrerkf9g"
    ));
    assert!(validate_btc_address(
        "bc1q8c6fshw2dlwun7ekn9qwf37cu2rn755upcp6el"
    ));
}

#[test]
fn rejects_malformed_addresses() {
    // Bad checksum (last char flipped).
    assert!(!validate_btc_address(
        "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyx"
    ));
    // Way too short.
    assert!(!validate_btc_address("bc1q"));
    // Random garbage.
    assert!(!validate_btc_address("not-an-address"));
    // Empty string.
    assert!(!validate_btc_address(""));
}

#[test]
fn rejects_legacy_p2pkh_addresses() {
    // P2PKH (`1…`) — outside Phase 2 scope.
    assert!(!validate_btc_address("1NhMV5x8d4owZ3p3HHo2tQrEoWpJK7tnvm"));
}

#[test]
fn rejects_p2sh_addresses() {
    // P2SH (`3…`).
    assert!(!validate_btc_address("3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy"));
}

#[test]
fn rejects_taproot_addresses() {
    // P2TR (`bc1p…`) — outside Phase 2 scope; v1.x adds Taproot later if needed.
    assert!(!validate_btc_address(
        "bc1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4z9hcz6jq4xx6rj"
    ));
}

#[test]
fn rejects_testnet_addresses() {
    // tb1q… — testnet must not validate on mainnet.
    assert!(!validate_btc_address(
        "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx"
    ));
}
