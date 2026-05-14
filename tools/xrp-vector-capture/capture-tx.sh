#!/usr/bin/env bash
# Capture canonical XRPL signed transactions for the BIP-44 leaf key derived
# from the "abandon × 11, about" mnemonic at m/44'/144'/0'/0/0.
#
# Two transactions:
#   1. Payment with DestinationTag — the typical SDK use case.
#   2. OfferCreate — broadens the field-coverage of the canonical serializer.
#
# Capture algorithm (because xrpl-py has no `Wallet.from_private_key`):
#   - Build tx JSON in Python (canonical dict shape).
#   - Inject `SigningPubKey` = compressed pubkey hex (uppercase) from the
#     captured BIP-44 leaf key.
#   - `encode_for_signing(tx)` → hex-encoded canonical signing payload.
#   - `keypairs.sign(payload_bytes, priv_hex_padded_to_66_chars)` → DER sig hex.
#     (xrpl-py's secp256k1 driver does SHA512Half of the input internally
#      and signs with RFC-6979 + canonical-low-S — byte-identical to what
#      our Rust signer must produce.)
#   - Inject `TxnSignature` = DER hex (uppercase).
#   - `encode(tx)` → final signed_hex (uppercase).
#   - tx_hash = SHA512Half("TXN\0" || final_bytes) → hex uppercase.
#
# Outputs (committed, JSON shape):
#   captures/payment_dt.signed.json      — { tx_json, signed_hex, tx_hash }
#   captures/offer_create.signed.json    — { tx_json, signed_hex, tx_hash }

set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CAPTURES="$HERE/captures"
mkdir -p "$CAPTURES"

if [[ ! -s "$CAPTURES/abandon_account0.private_key_hex" ]]; then
    echo "Run capture.sh first" >&2
    exit 1
fi

VENV="${XRP_CAPTURE_VENV:-/tmp/xrp-venv}"
if [[ ! -x "$VENV/bin/python3" ]]; then
    python3 -m venv "$VENV"
    "$VENV/bin/pip" install --quiet xrpl-py bip-utils coincurve
fi

"$VENV/bin/python3" - <<'PY'
import json
import hashlib
from pathlib import Path

from xrpl.core import binarycodec, keypairs

HERE = Path("tools/xrp-vector-capture/captures")
ADDR = (HERE / "abandon_account0.address").read_text().strip()
PRIV_HEX_RAW = (HERE / "abandon_account0.private_key_hex").read_text().strip()
PUB_HEX = (HERE / "abandon_account0.public_key_hex").read_text().strip()

# xrpl-py expects secp256k1 priv keys as 66-char hex (33 bytes = 0x00 prefix +
# 32 bytes). Pad with a leading "00" so the underlying ecpy wrapper consumes
# 32 bytes after stripping its single-byte prefix.
PRIV_HEX_PADDED = ("00" + PRIV_HEX_RAW).upper()


def capture_one(name: str, tx: dict) -> None:
    # Inject signing pubkey BEFORE serializing for signing.
    tx["SigningPubKey"] = PUB_HEX

    # Canonical signing payload — xrpl-rust returns this as a hex string too.
    signing_hex = binarycodec.encode_for_signing(tx)
    signing_bytes = bytes.fromhex(signing_hex)

    # Sign — xrpl-py.keypairs.sign internally does sha512_first_half(signing_bytes)
    # then RFC-6979 + canonical-low-S DER. Returns the DER signature as hex
    # (uppercase) per xrpl-py 4.5 implementation.
    sig_hex = keypairs.sign(signing_bytes, PRIV_HEX_PADDED)

    tx["TxnSignature"] = sig_hex

    signed_hex = binarycodec.encode(tx).upper()
    final_bytes = bytes.fromhex(signed_hex)

    # XRPL transaction-ID hashing: SHA512Half("TXN\0" || final_bytes).
    h = hashlib.sha512(b"TXN\x00" + final_bytes).digest()
    tx_hash = h[:32].hex().upper()

    # Strip the injected fields from the tx_json we persist — the SDK injects
    # them itself. The persisted tx_json is the *unsigned* canonical form.
    persisted_tx = {k: v for k, v in tx.items() if k not in ("SigningPubKey", "TxnSignature")}

    out = {
        "tx_json": persisted_tx,
        "signed_hex": signed_hex,
        "tx_hash": tx_hash,
    }
    (HERE / f"{name}.signed.json").write_text(json.dumps(out, indent=2, separators=(",", ": ")) + "\n")
    print(f"{name}: signed_hex[:48]={signed_hex[:48]}…  tx_hash={tx_hash}")


# 1. Payment with DestinationTag — most-common SDK use case.
payment_tx = {
    "TransactionType": "Payment",
    "Account": ADDR,
    "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
    "DestinationTag": 12345,
    "Amount": "1000000",
    "Fee": "12",
    "Sequence": 1,
    "Flags": 0,
}
capture_one("payment_dt", payment_tx)

# 2. OfferCreate — broaden serializer coverage (different field set).
offer_tx = {
    "TransactionType": "OfferCreate",
    "Account": ADDR,
    "TakerGets": "10000000",        # 10 XRP in drops
    "TakerPays": {
        "currency": "USD",
        "issuer": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
        "value": "1.5",
    },
    "Fee": "12",
    "Sequence": 2,
    "Flags": 0,
}
capture_one("offer_create", offer_tx)
PY
