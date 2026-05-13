use jova_core_primitives::{Mnemonic, MnemonicError, Strength};

#[test]
fn validates_official_bip39_test_vector() {
    // BIP-39 official: 12 words of "abandon" + "about" — known-valid.
    let words = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    assert!(Mnemonic::validate(words, "").is_ok());
}

#[test]
fn rejects_invalid_checksum() {
    let words = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";
    // 12x abandon = wrong checksum
    assert!(matches!(
        Mnemonic::validate(words, ""),
        Err(MnemonicError::InvalidChecksum)
    ));
}

#[test]
fn rejects_unknown_word() {
    let words = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon zzzzz";
    assert!(matches!(
        Mnemonic::validate(words, ""),
        Err(MnemonicError::InvalidWord(_))
    ));
}

#[test]
fn generates_24_word_mnemonic_at_bits256() {
    let m = Mnemonic::generate(Strength::Bits256);
    let count = m.words.split_whitespace().count();
    assert_eq!(count, 24);
    assert!(Mnemonic::validate(&m.words, "").is_ok());
}

#[test]
fn to_seed_matches_bip39_official_vector() {
    // BIP-39 official vector (from python-mnemonic reference implementation):
    // entropy = 00000000000000000000000000000000 (16 zero bytes),
    // mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
    // passphrase = "" (empty) → seed = 5eb00b...
    // The plan's snippet listed "TREZOR" but the expected_hex corresponds to the empty-passphrase
    // vector; the empty-passphrase value is the correct BIP-39 spec value to assert here.
    let words = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let seed = Mnemonic::to_seed(words, "").expect("valid");
    let expected_hex = "5eb00bbddcf069084889a8ab9155568165f5c453ccb85e70811aaed6f6da5fc19a5ac40b389cd370d086206dec8aa6c43daea6690f20ad3d8d48b2d2ce9e38e4";
    assert_eq!(hex::encode(seed.as_bytes()), expected_hex);
}
