#!/usr/bin/env bash
# Capture reference Bitcoin message signatures for the BIP-84 test mnemonic at
# m/84'/0'/0'/0/0 (address bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu).
#
# Two schemes are captured:
#   1. BIP-322 simple (witness-based, modern). Built in-script from embit
#      0.8.0 primitives (tagged hash, transaction structures, BIP-143 sighash,
#      low-R-grinded ECDSA, witness consensus serialization). The resulting
#      base64 is cross-verified by the `bip322` PyPI package's
#      `verify_simple_encoded`, which is an INDEPENDENT Rust-backed verifier.
#   2. Legacy signMessage (Bitcoin Core's pre-BIP-322 scheme). Recoverable
#      ECDSA over the double-SHA256 of the `"\x18Bitcoin Signed Message:\n"`
#      prefixed varint-length message, header_byte = recid + 27 + 4 for
#      compressed pubkeys.
#
# Outputs (committed to the repo):
#   captures/bip322_sig.txt  — BIP-322 simple sig, base64 (single line)
#   captures/legacy_sig.txt  — Legacy signMessage sig, base64 (single line)
#
# Usage: ./messages.sh
# Requires: python3 venv with `embit==0.8.0` and `bip322` installed.

set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CAPTURES="$HERE/captures"
mkdir -p "$CAPTURES"

VENV="${BTC_CAPTURE_VENV:-/tmp/btc-venv}"
if [[ ! -x "$VENV/bin/python3" ]]; then
    python3 -m venv "$VENV"
fi
# Install both deps idempotently (cheap if already present).
"$VENV/bin/pip" install --quiet "embit==0.8.0" "bip322"

"$VENV/bin/python3" - <<'PY' >"$CAPTURES/.tmp.out"
import base64
import hashlib
import io

import bip322
from embit import bip32, bip39, networks, script
from embit.script import Script, Witness
from embit.compact import to_bytes as varint_to_bytes
from embit.transaction import (
    SIGHASH,
    Transaction,
    TransactionInput,
    TransactionOutput,
)
from embit.util import secp256k1

MNEMONIC = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
ADDRESS = "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu"
MESSAGE = "Hello, Jova"

seed = bip39.mnemonic_to_seed(MNEMONIC)
root = bip32.HDKey.from_seed(seed, version=networks.NETWORKS["main"]["xprv"])
child = root.derive("m/84'/0'/0'/0/0")
prv = child.key
pubkey = prv.get_public_key()

derived = script.p2wpkh(pubkey).address()
assert derived == ADDRESS, f"derived address {derived} != expected {ADDRESS}"


# --- BIP-322 simple ---------------------------------------------------------

def tagged_hash(tag: bytes, data: bytes) -> bytes:
    t = hashlib.sha256(tag).digest()
    return hashlib.sha256(t + t + data).digest()


message_hash = tagged_hash(b"BIP0322-signed-message", MESSAGE.encode())

# to_spend: version=0, locktime=0
# input[0]: prevout = (0x00..00, 0xFFFFFFFF), sequence=0,
#           scriptSig = OP_0 PUSH32 <message_hash>
# output[0]: value=0, scriptPubKey = address P2WPKH
addr_spk = script.p2wpkh(pubkey)
script_sig = Script(bytes([0x00, 0x20]) + message_hash)
to_spend = Transaction(
    version=0,
    vin=[TransactionInput(b"\x00" * 32, 0xFFFFFFFF, script_sig=script_sig, sequence=0)],
    vout=[TransactionOutput(0, addr_spk)],
    locktime=0,
)

# to_sign: version=0, locktime=0
# input[0]: prevout = (to_spend_txid, 0), sequence=0, scriptSig empty
# output[0]: value=0, scriptPubKey = OP_RETURN
op_return_spk = Script(b"\x6a")
to_sign = Transaction(
    version=0,
    vin=[TransactionInput(to_spend.txid(), 0, sequence=0)],
    vout=[TransactionOutput(0, op_return_spk)],
    locktime=0,
)

# BIP-143 sighash. witness-utxo = to_spend.vout[0] (value=0, spk=address P2WPKH).
# For P2WPKH the BIP-143 scriptCode is the equivalent P2PKH script.
script_for_sighash = script.p2pkh_from_p2wpkh(addr_spk)
sighash = to_sign.sighash_segwit(0, script_for_sighash, 0, sighash=SIGHASH.ALL)

# Low-R-grinded ECDSA (embit grinds by default).
sig = prv.sign(sighash)
sig_der_with_sighash = sig.serialize() + bytes([SIGHASH.ALL])

# Witness = [sig_der_with_sighash, compressed_pk]
witness = Witness(items=[sig_der_with_sighash, pubkey.sec()])
buf = io.BytesIO()
witness.write_to(buf)
bip322_b64 = base64.b64encode(buf.getvalue()).decode()

# Cross-verify with the bip322 PyPI package (independent Rust-backed verifier).
# verify_simple_encoded returns None on success, raises on failure.
bip322.verify_simple_encoded(ADDRESS, MESSAGE, bip322_b64)


# --- Legacy signMessage -----------------------------------------------------

prefix = b"\x18Bitcoin Signed Message:\n"
msg_bytes = MESSAGE.encode()
payload = prefix + varint_to_bytes(len(msg_bytes)) + msg_bytes
digest = hashlib.sha256(hashlib.sha256(payload).digest()).digest()

# Recoverable ECDSA (no low-R grinding; Bitcoin Core's signmessage uses the
# plain recoverable signing path).
sig_rec = secp256k1.ecdsa_sign_recoverable(digest, prv._secret)
sig_compact, recid = secp256k1.ecdsa_recoverable_signature_serialize_compact(sig_rec)
header = bytes([recid + 27 + 4])  # +4 for compressed pubkey
legacy_b64 = base64.b64encode(header + sig_compact).decode()

# Self-check by recovering the pubkey and comparing.
parsed = secp256k1.ecdsa_recoverable_signature_parse_compact(sig_compact, recid)
recovered = secp256k1.ec_pubkey_serialize(
    secp256k1.ecdsa_recover(parsed, digest),
    secp256k1.EC_COMPRESSED,
)
assert recovered == pubkey.sec(), "legacy sig recovery mismatch"

print(bip322_b64)
print(legacy_b64)
PY

BIP322_B64="$(sed -n '1p' "$CAPTURES/.tmp.out")"
LEGACY_B64="$(sed -n '2p' "$CAPTURES/.tmp.out")"
rm "$CAPTURES/.tmp.out"

printf '%s\n' "$BIP322_B64" >"$CAPTURES/bip322_sig.txt"
printf '%s\n' "$LEGACY_B64" >"$CAPTURES/legacy_sig.txt"

echo "Wrote:"
echo "  $CAPTURES/bip322_sig.txt  ($(wc -c <"$CAPTURES/bip322_sig.txt") bytes)"
echo "  $CAPTURES/legacy_sig.txt  ($(wc -c <"$CAPTURES/legacy_sig.txt") bytes)"
