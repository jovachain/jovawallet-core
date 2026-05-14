#!/usr/bin/env bash
# Capture the canonical XRP classic address (`r…`) derived from the BIP-39
# "abandon × 11, about" mnemonic at BIP-44 path m/44'/144'/0'/0/0.
#
# The reference signer is xrpl-py 4.5 + bip_utils 2.x (Python). Both libraries
# are independent of the Rust SDK and serve as the test-as-contract source.
# xrpl-py does NOT expose `Wallet.from_private_key`, so we derive the BIP-44
# private key with bip_utils, compute the compressed pubkey via coincurve, and
# call xrpl-py's `keypairs.derive_classic_address(pubkey_hex)` directly.
#
# Output (committed):
#   captures/abandon_account0.address          — `r…` classic address
#   captures/abandon_account0.private_key_hex  — 32-byte secp256k1 priv key
#                                                hex (for downstream tx capture)
#   captures/abandon_account0.public_key_hex   — 33-byte compressed pubkey hex
#
# Usage: ./tools/xrp-vector-capture/capture.sh
# Requires: python3 venv with xrpl-py, bip_utils, coincurve installed.

set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CAPTURES="$HERE/captures"
mkdir -p "$CAPTURES"

VENV="${XRP_CAPTURE_VENV:-/tmp/xrp-venv}"
if [[ ! -x "$VENV/bin/python3" ]]; then
    python3 -m venv "$VENV"
    "$VENV/bin/pip" install --quiet xrpl-py bip-utils coincurve
fi

"$VENV/bin/python3" - <<'PY' >"$CAPTURES/.tmp.out"
from bip_utils import Bip39SeedGenerator, Bip44, Bip44Coins, Bip44Changes
from coincurve import PrivateKey as CCKey
from xrpl.core.keypairs import derive_classic_address

MNEMONIC = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"

# bip_utils.Bip44Coins.RIPPLE = BIP-44 coin type 144, secp256k1 — exactly the
# SDK's derivation. This produces the same XPrv leaf the SDK derives from the
# same BIP-39 seed.
seed = Bip39SeedGenerator(MNEMONIC).Generate("")
bip44_xrp = (
    Bip44.FromSeed(seed, Bip44Coins.RIPPLE)
    .Purpose()
    .Coin()
    .Account(0)
    .Change(Bip44Changes.CHAIN_EXT)
    .AddressIndex(0)
)
priv_hex = bip44_xrp.PrivateKey().Raw().ToHex()                   # 64 chars (32 bytes)
sk = CCKey(bytes.fromhex(priv_hex))
pub_compressed_hex = sk.public_key.format(compressed=True).hex().upper()
addr = derive_classic_address(pub_compressed_hex)

print(f"ADDR={addr}")
print(f"PRIV_HEX={priv_hex.upper()}")
print(f"PUB_HEX={pub_compressed_hex}")
PY

# Parse the named output lines.
while IFS='=' read -r key val; do
    case "$key" in
        ADDR) echo -n "$val" > "$CAPTURES/abandon_account0.address" ;;
        PRIV_HEX) echo -n "$val" > "$CAPTURES/abandon_account0.private_key_hex" ;;
        PUB_HEX) echo -n "$val" > "$CAPTURES/abandon_account0.public_key_hex" ;;
    esac
done < "$CAPTURES/.tmp.out"
rm -f "$CAPTURES/.tmp.out"

echo "Wrote:"
echo "  $CAPTURES/abandon_account0.address          $(cat "$CAPTURES/abandon_account0.address")"
echo "  $CAPTURES/abandon_account0.private_key_hex  ($(wc -c <"$CAPTURES/abandon_account0.private_key_hex") chars)"
echo "  $CAPTURES/abandon_account0.public_key_hex   $(cat "$CAPTURES/abandon_account0.public_key_hex")"
