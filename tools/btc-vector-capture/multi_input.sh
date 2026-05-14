#!/usr/bin/env bash
# Capture a multi-input PSBT that the wallet fully owns: two P2WPKH UTXOs both
# locking to the SAME address (m/84'/0'/0'/0/0 of the BIP-84 abandon-about
# mnemonic). Sign with embit, finalize manually, and emit the corresponding
# signed transaction hex.
#
# This mirrors the Path B approach from `single_input.sh` (synthetic prevouts,
# no bitcoind required). Using two UTXOs at the SAME scriptPubKey keeps the
# `sign_psbt(xprv, ...)` API surface from Task 3 unchanged: a single XPrv at
# m/84'/0'/0'/0/0 is sufficient to sign both inputs.
#
# Outputs:
#   captures/multi_input.psbt.b64   — unsigned PSBT, base64 (2 inputs)
#   captures/multi_input.signed_hex — signed tx, lowercase hex (finalized)
#
# Usage: ./multi_input.sh
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
from embit.transaction import Transaction, TransactionInput, TransactionOutput, Witness
from embit.psbt import PSBT, DerivationPath
from embit.bip32 import HARDENED_INDEX

MNEMONIC = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"

seed = bip39.mnemonic_to_seed(MNEMONIC)
root = bip32.HDKey.from_seed(seed, version=networks.NETWORKS["main"]["xprv"])
master_fp = root.my_fingerprint

# Single spending key at m/84'/0'/0'/0/0 — both UTXOs lock to its P2WPKH.
child = root.derive("m/84'/0'/0'/0/0")
pubkey = child.key.get_public_key()
my_script = script.p2wpkh(pubkey)

# Destination at m/84'/0'/0'/1/0 (BIP-84 first change address).
dest_child = root.derive("m/84'/0'/0'/1/0")
dest_pk = dest_child.key.get_public_key()
dest_script = script.p2wpkh(dest_pk)

# Two synthetic prevouts, both locking to the same P2WPKH (our address).
prev_a_txid = bytes.fromhex("aa" * 32)
prev_b_txid = bytes.fromhex("ab" * 32)
prev_value = 100_000  # each
total_in = prev_value * 2  # 200_000
fee = 2_000
send_value = total_in - fee  # 198_000

tx_in_a = TransactionInput(prev_a_txid, 0)
tx_in_b = TransactionInput(prev_b_txid, 0)
tx_out = TransactionOutput(send_value, dest_script)
tx = Transaction(version=2, vin=[tx_in_a, tx_in_b], vout=[tx_out], locktime=0)

p = PSBT(tx)
for i in range(2):
    p.inputs[i].witness_utxo = TransactionOutput(prev_value, my_script)
    p.inputs[i].bip32_derivations[pubkey] = DerivationPath(
        master_fp,
        [84 + HARDENED_INDEX, 0 + HARDENED_INDEX, 0 + HARDENED_INDEX, 0, 0],
    )

unsigned_b64 = p.to_base64()

# Sign with the root HDKey. Expect exactly 2 signatures (one per input).
n = p.sign_with(root)
assert n == 2, f"expected 2 signatures added, got {n}"

# Manual P2WPKH finalization for each input.
signed_tx = Transaction(
    version=tx.version, vin=[TransactionInput(i.txid, i.vout) for i in tx.vin],
    vout=tx.vout, locktime=tx.locktime,
)
for i in range(2):
    sig = list(p.inputs[i].partial_sigs.values())[0]
    pk_bytes = list(p.inputs[i].partial_sigs.keys())[0].sec()
    signed_tx.vin[i].witness = Witness(items=[sig, pk_bytes])

signed_hex = signed_tx.serialize().hex()

print(unsigned_b64)
print(signed_hex)
PY

UNSIGNED_B64="$(sed -n '1p' "$CAPTURES/.tmp.out")"
SIGNED_HEX="$(sed -n '2p' "$CAPTURES/.tmp.out")"
rm "$CAPTURES/.tmp.out"

printf '%s\n' "$UNSIGNED_B64" >"$CAPTURES/multi_input.psbt.b64"
printf '%s\n' "$SIGNED_HEX"   >"$CAPTURES/multi_input.signed_hex"

echo "Wrote:"
echo "  $CAPTURES/multi_input.psbt.b64   ($(wc -c <"$CAPTURES/multi_input.psbt.b64") bytes)"
echo "  $CAPTURES/multi_input.signed_hex ($(wc -c <"$CAPTURES/multi_input.signed_hex") bytes)"
