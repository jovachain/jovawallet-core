#!/usr/bin/env bash
# Capture the canonical Solana ed25519 public key (and base58 address) derived
# from the BIP-39 "abandon × 11, about" mnemonic at the Phantom/Solflare BIP-44
# path m/44'/501'/0'/0'/0' (all hardened — SLIP-10 ed25519 requirement).
#
# The reference signer is bip_utils 2.x (Python), which implements SLIP-10
# ed25519 for `Bip44Coins.SOLANA`. bip_utils expands the BIP-44 spec for
# Solana to a 5-component all-hardened path, matching what mainstream Solana
# wallets (Phantom, Solflare) use as the default.
#
# Output (committed):
#   captures/abandon_account_0.pubkey_b58       — base58 address (32-44 chars)
#   captures/abandon_account_0.pubkey_hex       — 32-byte ed25519 pub key hex
#   captures/abandon_account_0.private_key_hex  — 32-byte ed25519 priv key hex
#                                                 (for downstream tx capture)
#
# Usage: ./tools/sol-vector-capture/capture.sh
# Requires: python3 venv with bip_utils installed. Reuses the existing
# /tmp/xrp-venv if present.

set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CAPTURES="$HERE/captures"
mkdir -p "$CAPTURES"

VENV="${SOL_CAPTURE_VENV:-/tmp/xrp-venv}"
if [[ ! -x "$VENV/bin/python3" ]]; then
    python3 -m venv "$VENV"
    "$VENV/bin/pip" install --quiet bip-utils solders
fi
# Ensure solders is available too (used by capture-tx.sh).
"$VENV/bin/python3" -c "import solders" 2>/dev/null || \
    "$VENV/bin/pip" install --quiet solders

"$VENV/bin/python3" - <<'PY' >"$CAPTURES/.tmp.out"
from bip_utils import Bip39SeedGenerator, Bip44, Bip44Coins, Bip44Changes

MNEMONIC = (
    "abandon abandon abandon abandon abandon abandon "
    "abandon abandon abandon abandon abandon about"
)

# Bip44Coins.SOLANA expands to m/44'/501'/0'/0'/0' (all hardened) — the
# Phantom / Solflare default derivation. SLIP-10 ed25519 requires every
# component to be hardened.
seed = Bip39SeedGenerator(MNEMONIC).Generate("")
node = (
    Bip44.FromSeed(seed, Bip44Coins.SOLANA)
    .Purpose().Coin().Account(0)
    .Change(Bip44Changes.CHAIN_EXT)
    .AddressIndex(0)
)
priv_hex = node.PrivateKey().Raw().ToHex()
# .RawCompressed() returns the ed25519 pubkey with the 0x00 prefix byte;
# strip it for the 32-byte raw form expected by ed25519_dalek and bs58.
pub_raw_hex = node.PublicKey().RawCompressed().ToHex()
assert pub_raw_hex.startswith("00"), pub_raw_hex
pub_hex = pub_raw_hex[2:]
addr = node.PublicKey().ToAddress()

print(f"ADDR={addr}")
print(f"PRIV_HEX={priv_hex}")
print(f"PUB_HEX={pub_hex}")
PY

while IFS='=' read -r key val; do
    case "$key" in
        ADDR) echo -n "$val" > "$CAPTURES/abandon_account_0.pubkey_b58" ;;
        PRIV_HEX) echo -n "$val" > "$CAPTURES/abandon_account_0.private_key_hex" ;;
        PUB_HEX) echo -n "$val" > "$CAPTURES/abandon_account_0.pubkey_hex" ;;
    esac
done < "$CAPTURES/.tmp.out"
rm -f "$CAPTURES/.tmp.out"

echo "Wrote:"
echo "  $CAPTURES/abandon_account_0.pubkey_b58      $(cat "$CAPTURES/abandon_account_0.pubkey_b58")"
echo "  $CAPTURES/abandon_account_0.pubkey_hex      $(cat "$CAPTURES/abandon_account_0.pubkey_hex")"
echo "  $CAPTURES/abandon_account_0.private_key_hex ($(wc -c <"$CAPTURES/abandon_account_0.private_key_hex") chars)"
