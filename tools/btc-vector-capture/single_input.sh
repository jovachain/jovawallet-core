#!/usr/bin/env bash
# Capture a known-good single-input P2WPKH PSBT (unsigned, base64) and its
# corresponding signed transaction (hex), against the BIP-84 test mnemonic
# at path m/84'/0'/0'/0/0. The signer used here is `embit` (Python), an
# INDEPENDENT implementation of PSBT signing — this is what makes it a true
# reference for the Rust SDK's `sign_psbt`.
#
# Path B from the task plan: pure-Python independent signer, no bitcoind
# required. We synthesize a single P2WPKH prevout (txid = 0xaa..aa, vout = 0,
# value = 100_000 sats, scriptPubKey = P2WPKH of our derived pubkey), build the
# unsigned PSBT, sign with `embit.psbt.PSBT.sign_with(root)`, then manually
# finalize the P2WPKH witness.
#
# Outputs (committed to the repo):
#   captures/single_input.psbt.b64   — unsigned PSBT, base64
#   captures/single_input.signed_hex — signed tx, lowercase hex
#
# Usage: ./single_input.sh
# Requires: python3 venv with `embit` installed.

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
from embit import psbt, bip32, bip39, networks, script
from embit.transaction import Transaction, TransactionInput, TransactionOutput, Witness
from embit.psbt import PSBT, DerivationPath
from embit.bip32 import HARDENED_INDEX

MNEMONIC = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"

seed = bip39.mnemonic_to_seed(MNEMONIC)
root = bip32.HDKey.from_seed(seed, version=networks.NETWORKS["main"]["xprv"])
master_fp = root.my_fingerprint

# Our spending key at m/84'/0'/0'/0/0 (the BIP-84 first external address).
child = root.derive("m/84'/0'/0'/0/0")
pubkey = child.key.get_public_key()
my_script = script.p2wpkh(pubkey)

# Destination at m/84'/0'/0'/1/0 (BIP-84 first change address).
dest_child = root.derive("m/84'/0'/0'/1/0")
dest_pk = dest_child.key.get_public_key()
dest_script = script.p2wpkh(dest_pk)

# Synthetic prevout: txid = 0xaa..aa, vout = 0, value = 100_000 sats.
prev_txid = bytes.fromhex("aa" * 32)
prev_vout = 0
prev_value = 100_000
fee = 1_000
send_value = prev_value - fee

tx_in = TransactionInput(prev_txid, prev_vout)
tx_out = TransactionOutput(send_value, dest_script)
tx = Transaction(version=2, vin=[tx_in], vout=[tx_out], locktime=0)

p = PSBT(tx)
p.inputs[0].witness_utxo = TransactionOutput(prev_value, my_script)
# bip32_derivations needed so embit's sign_with can match the input to root.
p.inputs[0].bip32_derivations[pubkey] = DerivationPath(
    master_fp,
    [84 + HARDENED_INDEX, 0 + HARDENED_INDEX, 0 + HARDENED_INDEX, 0, 0],
)

unsigned_b64 = p.to_base64()

# Sign with the root HDKey. embit returns the number of signatures added; we
# expect exactly 1 for a single-input P2WPKH that we own.
n = p.sign_with(root)
assert n == 1, f"expected 1 signature added, got {n}"

# Manual P2WPKH finalization: witness = [sig_der_with_sighash, compressed_pk].
sig = list(p.inputs[0].partial_sigs.values())[0]
pk_bytes = list(p.inputs[0].partial_sigs.keys())[0].sec()

signed_tx = Transaction(version=tx.version, vin=tx.vin, vout=tx.vout, locktime=tx.locktime)
signed_tx.vin[0].witness = Witness(items=[sig, pk_bytes])
signed_hex = signed_tx.serialize().hex()

print(unsigned_b64)
print(signed_hex)
PY

UNSIGNED_B64="$(sed -n '1p' "$CAPTURES/.tmp.out")"
SIGNED_HEX="$(sed -n '2p' "$CAPTURES/.tmp.out")"
rm "$CAPTURES/.tmp.out"

printf '%s\n' "$UNSIGNED_B64" >"$CAPTURES/single_input.psbt.b64"
printf '%s\n' "$SIGNED_HEX"   >"$CAPTURES/single_input.signed_hex"

echo "Wrote:"
echo "  $CAPTURES/single_input.psbt.b64   ($(wc -c <"$CAPTURES/single_input.psbt.b64") bytes)"
echo "  $CAPTURES/single_input.signed_hex ($(wc -c <"$CAPTURES/single_input.signed_hex") bytes)"
