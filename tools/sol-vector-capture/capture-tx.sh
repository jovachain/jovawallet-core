#!/usr/bin/env bash
# Capture canonical Solana VersionedTransaction (v0) signing vectors for the
# SLIP-10 ed25519 leaf derived from the "abandon × 11, about" mnemonic at
# m/44'/501'/0'/0'/0' (all hardened).
#
# Two transactions:
#   1. System Program transfer (1_000_000 lamports → 11111…12 burn address).
#   2. (Currently the same shape; ALT capture is deferred to follow-up if
#      complex. The plan accepts an 8-vector ship if ALT can't be captured
#      cleanly. See report.)
#
# Capture algorithm:
#   - Build a MessageV0 with solders for a System Program transfer.
#   - Get the wire-form message bytes via solders: 0x80 || message_body
#     (Solana's VersionedMessage::V0 serialization prepends 0x80 to the body
#     to disambiguate from legacy messages).
#   - Sign those wire-form bytes with PyNaCl SigningKey(seed).sign(...).
#   - Build a VersionedTransaction(msg, [kp]) via solders and serialize via
#     `bytes(vt)`; that's the bincode-encoded signed_hex.
#
# The blockhash is a fixed deterministic value so the capture is reproducible
# offline (no live RPC needed).
#
# Outputs (committed, JSON shape):
#   captures/system_transfer_v0.signed.json
#       { message_base64, recent_blockhash, signed_hex, signature_b58 }
#   captures/sign_message.signed.json
#       { message_base64, signature_b58 }      — raw ed25519 over arbitrary bytes
#
# Usage: ./tools/sol-vector-capture/capture-tx.sh
# Requires: python3 venv with solders + bip-utils + PyNaCl installed.

set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CAPTURES="$HERE/captures"
mkdir -p "$CAPTURES"

if [[ ! -s "$CAPTURES/abandon_account_0.private_key_hex" ]]; then
    echo "Run capture.sh first" >&2
    exit 1
fi

VENV="${SOL_CAPTURE_VENV:-/tmp/xrp-venv}"
if [[ ! -x "$VENV/bin/python3" ]]; then
    python3 -m venv "$VENV"
    "$VENV/bin/pip" install --quiet bip-utils solders PyNaCl
fi
"$VENV/bin/python3" -c "import nacl, solders" 2>/dev/null || \
    "$VENV/bin/pip" install --quiet solders PyNaCl

"$VENV/bin/python3" - <<'PY'
import base64
import json
from pathlib import Path

import base58
import nacl.signing
from solders.hash import Hash
from solders.keypair import Keypair
from solders.message import MessageV0
from solders.pubkey import Pubkey
from solders.system_program import TransferParams, transfer
from solders.transaction import VersionedTransaction

HERE = Path("tools/sol-vector-capture/captures")
PRIV_HEX = (HERE / "abandon_account_0.private_key_hex").read_text().strip()
ADDR = (HERE / "abandon_account_0.pubkey_b58").read_text().strip()

PRIV_BYTES = bytes.fromhex(PRIV_HEX)
kp = Keypair.from_seed(PRIV_BYTES)
assert str(kp.pubkey()) == ADDR, "keypair mismatch"

# Use a fixed deterministic blockhash so captures are reproducible. Any valid
# 32-byte base58 hash works for offline parity testing.
FAKE_BLOCKHASH = Hash.from_string("4nHfMbJDp1HJk4SbnzKKM9qhSqMzM6XnxNvY2KrA9HEt")


def emit(name: str, msg: MessageV0) -> None:
    body = bytes(msg)
    wire = b"\x80" + body  # VersionedMessage::V0 wire form

    # Sign the wire bytes directly with ed25519.
    sk = nacl.signing.SigningKey(PRIV_BYTES)
    sig = sk.sign(wire).signature  # 64 bytes

    # Cross-check against solders' VersionedTransaction signing.
    vt = VersionedTransaction(msg, [kp])
    vt_bytes = bytes(vt)
    assert vt_bytes[1:65] == sig, "signature mismatch vs solders"

    payload = {
        "message_base64": base64.b64encode(wire).decode(),
        "recent_blockhash": str(msg.recent_blockhash),
        "signed_hex": vt_bytes.hex(),
        "signature_b58": base58.b58encode(sig).decode(),
    }
    out = HERE / f"{name}.signed.json"
    out.write_text(json.dumps(payload, indent=2) + "\n")
    print(f"{name}: signed_hex[:48]={vt_bytes.hex()[:48]}…  sig_b58[:20]={payload['signature_b58'][:20]}…")


# 1. System Program transfer (no ALT).
sender = kp.pubkey()
receiver = Pubkey.from_string("11111111111111111111111111111112")
ixns = [
    transfer(
        TransferParams(from_pubkey=sender, to_pubkey=receiver, lamports=1_000_000)
    )
]
msg1 = MessageV0.try_compile(sender, ixns, [], FAKE_BLOCKHASH)
emit("system_transfer_v0", msg1)

# 2. VersionedTransaction with Address Lookup Table — broadens v0 coverage.
# The LUT is fabricated (its key is derived from a deterministic seed) but
# byte-equality at signing time only cares about the on-wire encoding, not
# whether the LUT account actually exists on-chain.
from solders.address_lookup_table_account import AddressLookupTableAccount

LUT_SEED = bytes.fromhex("11" * 32)
lut_key = Keypair.from_seed(LUT_SEED).pubkey()
alt_receiver = Pubkey.from_string("So11111111111111111111111111111111111111112")
alt_filler = Pubkey.from_string("11111111111111111111111111111112")
lut = AddressLookupTableAccount(key=lut_key, addresses=[alt_receiver, alt_filler])

ixns_alt = [
    transfer(
        TransferParams(from_pubkey=sender, to_pubkey=alt_receiver, lamports=2_000_000)
    )
]
msg_alt = MessageV0.try_compile(sender, ixns_alt, [lut], FAKE_BLOCKHASH)
emit("with_alt_v0", msg_alt)

# 2. Raw ed25519 message signing — sign arbitrary bytes (not a Solana
# message wire-form). Output is just the base58 signature.
MSG_BYTES = b"hello solana"
msg_b64 = base64.b64encode(MSG_BYTES).decode()
sk = nacl.signing.SigningKey(PRIV_BYTES)
sig = sk.sign(MSG_BYTES).signature
sig_b58 = base58.b58encode(sig).decode()
(HERE / "sign_message.signed.json").write_text(
    json.dumps(
        {"message_base64": msg_b64, "signature_b58": sig_b58},
        indent=2,
    )
    + "\n"
)
print(f"sign_message: msg_b64={msg_b64}  sig_b58[:20]={sig_b58[:20]}…")
PY
