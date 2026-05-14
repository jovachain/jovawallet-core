#!/usr/bin/env bash
# Capture a single-input P2WPKH PSBT that locks to a DIFFERENT wallet's first
# BIP-84 address (the ozone-drill Trezor test mnemonic at m/84'/0'/0'/0/0,
# bc1qejl9xacvuwn55857qa8jtm6ettgkla6ps0thaq) rather than the abandon-about
# wallet's address. When the SDK is asked to sign this PSBT against the
# abandon-about XPrv (the default Phase 2 BIP-84 test mnemonic), every input
# fails the P2WPKH-for-our-pubkey check in `sign_psbt`, so the signer returns
# `MalformedUnsignedTx("psbt_no_signable_inputs")`. This is the negative
# vector pair to `single_input.sh`.
#
# Outputs:
#   captures/foreign_signer.psbt.b64 — unsigned PSBT, base64 (single input)
#
# Usage: ./foreign_signer.sh
# Requires: python3 venv with `embit==0.8.0` installed (created in BTC_CAPTURE_VENV).

set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CAPTURES="$HERE/captures"
mkdir -p "$CAPTURES"

VENV="${BTC_CAPTURE_VENV:-/tmp/btc-venv}"
if [[ ! -x "$VENV/bin/python3" ]]; then
    python3 -m venv "$VENV"
    "$VENV/bin/pip" install --quiet "embit==0.8.0"
fi

"$VENV/bin/python3" - <<'PY' >"$CAPTURES/.tmp.out"
from embit import bip32, bip39, networks, script
from embit.transaction import Transaction, TransactionInput, TransactionOutput
from embit.psbt import PSBT, DerivationPath
from embit.bip32 import HARDENED_INDEX

# This PSBT spends an output owned by the OZONE-DRILL mnemonic, NOT abandon.
# When the Rust SDK signs it with the abandon-about XPrv at m/84'/0'/0'/0/0,
# the witness-utxo's scriptPubKey is P2WPKH(ozone pubkey), which doesn't
# match hash160(abandon pubkey), so no inputs are signable.
FOREIGN_MNEMONIC = "ozone drill grab fiber curtain grace pudding thank cruise elder eight picnic"

seed = bip39.mnemonic_to_seed(FOREIGN_MNEMONIC)
root = bip32.HDKey.from_seed(seed, version=networks.NETWORKS["main"]["xprv"])
master_fp = root.my_fingerprint

# Foreign spending key at m/84'/0'/0'/0/0 — UTXO locks to its P2WPKH.
child = root.derive("m/84'/0'/0'/0/0")
pubkey = child.key.get_public_key()
foreign_script = script.p2wpkh(pubkey)

# Destination at m/84'/0'/0'/1/0 of the same foreign wallet (arbitrary; the
# Rust SDK never reaches signature generation on this fixture).
dest_child = root.derive("m/84'/0'/0'/1/0")
dest_pk = dest_child.key.get_public_key()
dest_script = script.p2wpkh(dest_pk)

# Synthetic prevout: txid = 0xee..ee, vout = 0, value = 100_000 sats.
prev_txid = bytes.fromhex("ee" * 32)
prev_vout = 0
prev_value = 100_000
fee = 1_000
send_value = prev_value - fee

tx_in = TransactionInput(prev_txid, prev_vout)
tx_out = TransactionOutput(send_value, dest_script)
tx = Transaction(version=2, vin=[tx_in], vout=[tx_out], locktime=0)

p = PSBT(tx)
p.inputs[0].witness_utxo = TransactionOutput(prev_value, foreign_script)
# bip32_derivations point at the foreign wallet's key path — this is
# advisory in PSBT and is intentionally present to make the fixture realistic.
p.inputs[0].bip32_derivations[pubkey] = DerivationPath(
    master_fp,
    [84 + HARDENED_INDEX, 0 + HARDENED_INDEX, 0 + HARDENED_INDEX, 0, 0],
)

print(p.to_base64())
PY

UNSIGNED_B64="$(sed -n '1p' "$CAPTURES/.tmp.out")"
rm "$CAPTURES/.tmp.out"

printf '%s\n' "$UNSIGNED_B64" >"$CAPTURES/foreign_signer.psbt.b64"

echo "Wrote:"
echo "  $CAPTURES/foreign_signer.psbt.b64 ($(wc -c <"$CAPTURES/foreign_signer.psbt.b64") bytes)"
