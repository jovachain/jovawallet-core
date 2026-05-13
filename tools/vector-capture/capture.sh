#!/usr/bin/env bash
# capture.sh — reproduce Phase 1 EVM test vectors using cast as the reference signer.
#
# Prerequisites: cast 1.7.1+ on PATH (from Foundry).
# Usage: bash tools/vector-capture/capture.sh
# Output: prints the captured vector data to stdout (for review/audit).
#
# The ACTUAL committed values live in spec/test-vectors.json.
# This script allows re-verification that cast still produces the same output.

set -euo pipefail

CAST=${CAST:-cast}

ABANDON_MNEMONIC="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
TREZOR_MNEMONIC="ozone drill grab fiber curtain grace pudding thank cruise elder eight picnic"
PATH_EVM="m/44'/60'/0'/0/0"

ABANDON_PRIV=$($CAST wallet derive-private-key "$ABANDON_MNEMONIC" --mnemonic-derivation-path "$PATH_EVM")
TREZOR_PRIV=$($CAST wallet derive-private-key "$TREZOR_MNEMONIC" --mnemonic-derivation-path "$PATH_EVM")

ABANDON_ADDR=$($CAST wallet address --private-key "$ABANDON_PRIV")
TREZOR_ADDR=$($CAST wallet address --private-key "$TREZOR_PRIV")

echo "=== Address vectors ==="
echo "abandon (ethereum/polygon/bsc): $ABANDON_ADDR"
echo "trezor  (ethereum/polygon/bsc): $TREZOR_ADDR"

echo ""
echo "=== Tx: simple_transfer (chainId=1, nonce=0, to=0x0000, value=1eth, gasLimit=21000, maxFee=30gwei, maxPri=2gwei) ==="
ST=$(  $CAST mktx --private-key "$ABANDON_PRIV" --chain 1   --nonce 0 --gas-limit 21000  --gas-price 30000000000 --priority-gas-price 2000000000  --value 1000000000000000000 0x0000000000000000000000000000000000000000)
STH=$( $CAST keccak "$ST")
echo "signed_hex: $ST"
echo "tx_hash:    $STH"

echo ""
echo "=== Tx: erc20_transfer (chainId=1, nonce=7, to=USDC, value=0, gasLimit=65000, maxFee=30gwei, maxPri=2gwei) ==="
ERC=$(  $CAST mktx --private-key "$ABANDON_PRIV" --chain 1   --nonce 7 --gas-limit 65000  --gas-price 30000000000 --priority-gas-price 2000000000  --value 0 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 0xa9059cbb000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa960450000000000000000000000000000000000000000000000000de0b6b3a7640000)
ERCH=$( $CAST keccak "$ERC")
echo "signed_hex: $ERC"
echo "tx_hash:    $ERCH"

echo ""
echo "=== Tx: with_access_list (chainId=1, nonce=1, to=0x1234, value=0, gasLimit=100000, accessList=[USDC,slot0]) ==="
ACL_JSON='[{"address":"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48","storageKeys":["0x0000000000000000000000000000000000000000000000000000000000000000"]}]'
ACL=$(  $CAST mktx --private-key "$ABANDON_PRIV" --chain 1   --nonce 1 --gas-limit 100000 --gas-price 30000000000 --priority-gas-price 2000000000  --value 0 --access-list "$ACL_JSON" 0x0000000000000000000000000000000000001234)
ACLH=$( $CAST keccak "$ACL")
echo "signed_hex: $ACL"
echo "tx_hash:    $ACLH"

echo ""
echo "=== Tx: polygon_transfer (chainId=137, nonce=0, to=0x0000, value=1eth, gasLimit=21000, maxFee=60gwei, maxPri=30gwei) ==="
POL=$(  $CAST mktx --private-key "$ABANDON_PRIV" --chain 137 --nonce 0 --gas-limit 21000  --gas-price 60000000000 --priority-gas-price 30000000000 --value 1000000000000000000 0x0000000000000000000000000000000000000000)
POLH=$( $CAST keccak "$POL")
echo "signed_hex: $POL"
echo "tx_hash:    $POLH"

echo ""
echo "=== Message: eip191_hello 'Hello, Jova' ==="
EIP191=$( $CAST wallet sign --private-key "$ABANDON_PRIV" "Hello, Jova")
echo "signature: $EIP191"

echo ""
echo "=== Message: eip712_permit (ERC-2612 Permit, USDC chainId=1) ==="
PERMIT_TMP=$(mktemp /tmp/jova_permit_XXXXXX.json)
cat > "$PERMIT_TMP" <<'EOJSON'
{"types":{"EIP712Domain":[{"name":"name","type":"string"},{"name":"version","type":"string"},{"name":"chainId","type":"uint256"},{"name":"verifyingContract","type":"address"}],"Permit":[{"name":"owner","type":"address"},{"name":"spender","type":"address"},{"name":"value","type":"uint256"},{"name":"nonce","type":"uint256"},{"name":"deadline","type":"uint256"}]},"primaryType":"Permit","domain":{"name":"USD Coin","version":"2","chainId":1,"verifyingContract":"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"},"message":{"owner":"0x9858EfFD232B4033E47d90003D41EC34EcaEda94","spender":"0x0000000000000000000000000000000000000001","value":"1000000000000000000","nonce":0,"deadline":1893456000}}
EOJSON
EIP712=$( $CAST wallet sign --private-key "$ABANDON_PRIV" --data --from-file "$PERMIT_TMP")
rm -f "$PERMIT_TMP"
echo "signature: $EIP712"
