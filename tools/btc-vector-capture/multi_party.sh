#!/usr/bin/env bash
# Capture a 2-party PSBT (input 0 owned by wallet A, input 1 owned by wallet B)
# and the after-A-signs partial PSBT. Wallet A is the abandon-about BIP-84
# mnemonic at m/84'/0'/0'/0/0; wallet B is the ozone-drill mnemonic at the
# same path.
#
# Two outputs:
#   captures/two_party.psbt.b64          — unsigned PSBT (2 inputs, A + B)
#   captures/two_party.after_a.psbt.b64  — PSBT after A signs input 0 ONLY
#
# Both follow the Path B independent-signer approach (synthetic prevouts via
# embit, no bitcoind).
#
# Usage: ./multi_party.sh
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

MNEMONIC_A = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
MNEMONIC_B = "ozone drill grab fiber curtain grace pudding thank cruise elder eight picnic"

def derive(mnemonic, path):
    seed = bip39.mnemonic_to_seed(mnemonic)
    root = bip32.HDKey.from_seed(seed, version=networks.NETWORKS["main"]["xprv"])
    child = root.derive(path)
    return root, child, root.my_fingerprint

root_a, child_a, fp_a = derive(MNEMONIC_A, "m/84'/0'/0'/0/0")
root_b, child_b, fp_b = derive(MNEMONIC_B, "m/84'/0'/0'/0/0")

pk_a = child_a.key.get_public_key()
pk_b = child_b.key.get_public_key()
spk_a = script.p2wpkh(pk_a)
spk_b = script.p2wpkh(pk_b)

# Destination: m/84'/0'/0'/1/0 of A (same as Task 3 / multi_input — sane reuse).
dest_child = root_a.derive("m/84'/0'/0'/1/0")
dest_pk = dest_child.key.get_public_key()
dest_script = script.p2wpkh(dest_pk)

# Synthetic prevouts.
prev_a_txid = bytes.fromhex("cc" * 32)
prev_b_txid = bytes.fromhex("dd" * 32)
prev_value = 100_000  # each
total_in = prev_value * 2  # 200_000
fee = 2_000
send_value = total_in - fee  # 198_000

tx_in_a = TransactionInput(prev_a_txid, 0)
tx_in_b = TransactionInput(prev_b_txid, 0)
tx_out = TransactionOutput(send_value, dest_script)
tx = Transaction(version=2, vin=[tx_in_a, tx_in_b], vout=[tx_out], locktime=0)

p = PSBT(tx)
# Input 0 owned by A.
p.inputs[0].witness_utxo = TransactionOutput(prev_value, spk_a)
p.inputs[0].bip32_derivations[pk_a] = DerivationPath(
    fp_a,
    [84 + HARDENED_INDEX, 0 + HARDENED_INDEX, 0 + HARDENED_INDEX, 0, 0],
)
# Input 1 owned by B.
p.inputs[1].witness_utxo = TransactionOutput(prev_value, spk_b)
p.inputs[1].bip32_derivations[pk_b] = DerivationPath(
    fp_b,
    [84 + HARDENED_INDEX, 0 + HARDENED_INDEX, 0 + HARDENED_INDEX, 0, 0],
)

unsigned_b64 = p.to_base64()

# Sign ONLY with A's root: input 0 gets a partial sig, input 1 stays untouched.
n = p.sign_with(root_a)
assert n == 1, f"expected 1 signature added (A on input 0), got {n}"

after_a_b64 = p.to_base64()

print(unsigned_b64)
print(after_a_b64)
PY

UNSIGNED_B64="$(sed -n '1p' "$CAPTURES/.tmp.out")"
AFTER_A_B64="$(sed -n '2p' "$CAPTURES/.tmp.out")"
rm "$CAPTURES/.tmp.out"

printf '%s\n' "$UNSIGNED_B64" >"$CAPTURES/two_party.psbt.b64"
printf '%s\n' "$AFTER_A_B64"  >"$CAPTURES/two_party.after_a.psbt.b64"

echo "Wrote:"
echo "  $CAPTURES/two_party.psbt.b64         ($(wc -c <"$CAPTURES/two_party.psbt.b64") bytes)"
echo "  $CAPTURES/two_party.after_a.psbt.b64 ($(wc -c <"$CAPTURES/two_party.after_a.psbt.b64") bytes)"
