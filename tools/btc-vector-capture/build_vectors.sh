#!/usr/bin/env bash
# Emit the 12 Phase 2 BTC vectors as a JSON array on stdout, reading the
# raw capture files in `captures/` and embedding their contents as JSON
# string values (with trailing newlines stripped).
#
# This script is the documented capture-to-vector pipeline so future phases
# (SOL, XRP) can mimic the same structure. It does NOT mutate
# `spec/test-vectors.json` directly; the resulting JSON is hand-merged into
# the `vectors` array of that file (the file already contains Phase 0 + 1
# vectors that must not be reformatted).
#
# Reproduce the captures from scratch by running, in order:
#   ./single_input.sh
#   ./multi_input.sh
#   ./multi_party.sh
#   ./messages.sh
#   ./foreign_signer.sh
# All five drop output into `captures/` and are byte-deterministic against
# embit 0.8.0 + bip322 (PyPI).
#
# Usage: ./build_vectors.sh > /tmp/btc_vectors.json
# Requires: jq (used to safely escape capture strings into JSON).

set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CAPTURES="$HERE/captures"

read_capture() {
    # Read a capture file, strip a single trailing newline (the capture
    # scripts emit `printf '%s\n'`), and emit on stdout — never edit the
    # captured bytes themselves.
    local f="$CAPTURES/$1"
    if [[ ! -f "$f" ]]; then
        echo "missing capture: $f" >&2
        exit 1
    fi
    # `awk` keeps any internal newlines; we only strip the last one. In
    # practice every capture is single-line, but be defensive.
    awk 'BEGIN{ORS=""} {if (NR>1) printf "\n"; printf "%s", $0}' "$f"
}

SINGLE_PSBT="$(read_capture single_input.psbt.b64)"
SINGLE_HEX="$(read_capture single_input.signed_hex)"
MULTI_PSBT="$(read_capture multi_input.psbt.b64)"
MULTI_HEX="$(read_capture multi_input.signed_hex)"
TWO_PSBT="$(read_capture two_party.psbt.b64)"
TWO_AFTER_A="$(read_capture two_party.after_a.psbt.b64)"
BIP322_SIG="$(read_capture bip322_sig.txt)"
LEGACY_SIG="$(read_capture legacy_sig.txt)"
FOREIGN_PSBT="$(read_capture foreign_signer.psbt.b64)"

# `psbt:` prefix mirrors the multi-party convention from Task 6: BtcSigner
# returns the unfinalized PSBT as `raw_hex = "psbt:" + base64`.
TWO_AFTER_A_TAGGED="psbt:${TWO_AFTER_A}"

jq -n \
    --arg single_psbt   "$SINGLE_PSBT" \
    --arg single_hex    "$SINGLE_HEX" \
    --arg multi_psbt    "$MULTI_PSBT" \
    --arg multi_hex     "$MULTI_HEX" \
    --arg two_psbt      "$TWO_PSBT" \
    --arg two_after_a   "$TWO_AFTER_A_TAGGED" \
    --arg bip322_sig    "$BIP322_SIG" \
    --arg legacy_sig    "$LEGACY_SIG" \
    --arg foreign_psbt  "$FOREIGN_PSBT" \
    '[
  {
    "id": "btc.address.bip84_abandon_account0_index0",
    "kind": "address",
    "input": {
      "mnemonic": "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
      "passphrase": "",
      "chain": { "kind": "bitcoin" }
    },
    "expected": { "address": "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu" },
    "source": "BIP-84 official test vector (Account 0, External chain, address 0) cross-checked by embit 0.8.0",
    "source_url": "https://github.com/bitcoin/bips/blob/master/bip-0084.mediawiki",
    "capture_cmd": "tools/btc-vector-capture/single_input.sh derives the same address as a side effect"
  },
  {
    "id": "btc.address.bip84_abandon_account0_index1",
    "kind": "address",
    "input": {
      "mnemonic": "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
      "passphrase": "",
      "chain": { "kind": "bitcoin" }
    },
    "expected": { "address": "bc1qnjg0jd8228aq7egyzacy8cys3knf9xvrerkf9g" },
    "source": "BIP-84 official test vector (Account 0, External chain, address 1) cross-checked by embit 0.8.0",
    "source_url": "https://github.com/bitcoin/bips/blob/master/bip-0084.mediawiki",
    "capture_cmd": "embit: root.derive(\"m/84'\''/0'\''/0'\''/0/1\").key.get_public_key() -> p2wpkh address"
  },
  {
    "id": "btc.address.bip84_ozone_account0_index0",
    "kind": "address",
    "input": {
      "mnemonic": "ozone drill grab fiber curtain grace pudding thank cruise elder eight picnic",
      "passphrase": "",
      "chain": { "kind": "bitcoin" }
    },
    "expected": { "address": "bc1qejl9xacvuwn55857qa8jtm6ettgkla6ps0thaq" },
    "source": "embit 0.8.0 BIP-84 derivation from the Trezor test mnemonic (replaces a planned account-5 vector — the JovaWallet::address API does not yet accept an account index, so the vector is at account 0)",
    "source_url": "https://github.com/trezor/python-mnemonic/blob/master/vectors.json",
    "capture_cmd": "embit: root.derive(\"m/84'\''/0'\''/0'\''/0/0\").key.get_public_key() -> p2wpkh address"
  },
  {
    "id": "btc.address.bip84_trezor_account0_index0",
    "kind": "address",
    "input": {
      "mnemonic": "ozone drill grab fiber curtain grace pudding thank cruise elder eight picnic",
      "passphrase": "",
      "chain": { "kind": "bitcoin" }
    },
    "expected": { "address": "bc1qejl9xacvuwn55857qa8jtm6ettgkla6ps0thaq" },
    "source": "embit 0.8.0 BIP-84 derivation from the Trezor test mnemonic — duplicate of bip84_ozone_account0_index0 under the alternative naming convention used by the Phase 2 task spec",
    "source_url": "https://github.com/trezor/python-mnemonic/blob/master/vectors.json",
    "capture_cmd": "embit: root.derive(\"m/84'\''/0'\''/0'\''/0/0\").key.get_public_key() -> p2wpkh address"
  },
  {
    "id": "btc.sign_tx.psbt_single_input",
    "kind": "sign_tx",
    "input": {
      "mnemonic": "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
      "passphrase": "",
      "unsigned_tx": {
        "kind": "bitcoin",
        "psbt_base64": $single_psbt
      }
    },
    "expected": { "signed_hex": $single_hex },
    "source": "embit 0.8.0 with low-R-grinded ECDSA (matches Bitcoin Core >= 0.17 default)",
    "source_url": "tools/btc-vector-capture/single_input.sh",
    "capture_cmd": "tools/btc-vector-capture/single_input.sh"
  },
  {
    "id": "btc.sign_tx.psbt_multi_input_owned",
    "kind": "sign_tx",
    "input": {
      "mnemonic": "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
      "passphrase": "",
      "unsigned_tx": {
        "kind": "bitcoin",
        "psbt_base64": $multi_psbt
      }
    },
    "expected": { "signed_hex": $multi_hex },
    "source": "embit 0.8.0 with low-R-grinded ECDSA (two P2WPKH inputs, both locking to the wallet'\''s m/84'\''/0'\''/0'\''/0/0 address, fully finalized)",
    "source_url": "tools/btc-vector-capture/multi_input.sh",
    "capture_cmd": "tools/btc-vector-capture/multi_input.sh"
  },
  {
    "id": "btc.sign_tx.psbt_multi_party_partial",
    "kind": "sign_tx",
    "input": {
      "mnemonic": "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
      "passphrase": "",
      "unsigned_tx": {
        "kind": "bitcoin",
        "psbt_base64": $two_psbt
      }
    },
    "expected": { "signed_hex": $two_after_a },
    "source": "embit 0.8.0: 2-party PSBT (input 0 owned by abandon-about, input 1 owned by ozone-drill); the abandon wallet signs input 0 only and returns the updated (unfinalized) PSBT as \"psbt:<base64>\" per the Task 6 convention",
    "source_url": "tools/btc-vector-capture/multi_party.sh",
    "capture_cmd": "tools/btc-vector-capture/multi_party.sh"
  },
  {
    "id": "btc.sign_message.bip322_simple",
    "kind": "sign_message",
    "input": {
      "mnemonic": "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
      "passphrase": "",
      "message": {
        "kind": "bitcoin",
        "message": "Hello, Jova",
        "address": "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu",
        "scheme": "bip322"
      }
    },
    "expected": { "signature_hex": $bip322_sig },
    "source": "embit 0.8.0 BIP-322 simple (low-R-grinded ECDSA), cross-verified by the bip322 PyPI verifier (signature_hex holds a base64 string per the Phase 1 EVM convention of using signature_hex regardless of encoding)",
    "source_url": "https://github.com/bitcoin/bips/blob/master/bip-0322.mediawiki",
    "capture_cmd": "tools/btc-vector-capture/messages.sh"
  },
  {
    "id": "btc.sign_message.legacy_bitcoin_core",
    "kind": "sign_message",
    "input": {
      "mnemonic": "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
      "passphrase": "",
      "message": {
        "kind": "bitcoin",
        "message": "Hello, Jova",
        "address": "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu",
        "scheme": "legacy"
      }
    },
    "expected": { "signature_hex": $legacy_sig },
    "source": "Bitcoin Core legacy signmessage scheme (sha256d over \"\\x18Bitcoin Signed Message:\\n\" prefix, recoverable ECDSA without low-R grinding, header_byte = recid + 27 + 4 for compressed pubkeys); captured via embit + libsecp256k1",
    "source_url": "tools/btc-vector-capture/messages.sh",
    "capture_cmd": "tools/btc-vector-capture/messages.sh"
  },
  {
    "id": "btc.error.psbt_invalid_base64",
    "kind": "error",
    "input": {
      "mnemonic": "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
      "passphrase": "",
      "unsigned_tx": {
        "kind": "bitcoin",
        "psbt_base64": "not-base64!"
      }
    },
    "expected": {
      "error_variant": "MalformedUnsignedTx",
      "reason": "psbt_invalid_base64"
    },
    "source": "Phase 2 error-path definition",
    "source_url": "spec/errors.md#bitcoin",
    "capture_cmd": "n/a"
  },
  {
    "id": "btc.error.psbt_no_signable_inputs",
    "kind": "error",
    "input": {
      "mnemonic": "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
      "passphrase": "",
      "unsigned_tx": {
        "kind": "bitcoin",
        "psbt_base64": $foreign_psbt
      }
    },
    "expected": {
      "error_variant": "MalformedUnsignedTx",
      "reason": "psbt_no_signable_inputs"
    },
    "source": "embit 0.8.0: a valid single-input P2WPKH PSBT whose witness-utxo locks to the ozone-drill wallet (m/84'\''/0'\''/0'\''/0/0) — signing with the abandon-about XPrv finds zero signable inputs",
    "source_url": "tools/btc-vector-capture/foreign_signer.sh",
    "capture_cmd": "tools/btc-vector-capture/foreign_signer.sh"
  },
  {
    "id": "btc.error.btc_message_address_mismatch",
    "kind": "error",
    "input": {
      "mnemonic": "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
      "passphrase": "",
      "message": {
        "kind": "bitcoin",
        "message": "Hello, Jova",
        "address": "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4",
        "scheme": "bip322"
      }
    },
    "expected": {
      "error_variant": "MalformedSignableMessage",
      "reason": "btc_message_address_mismatch"
    },
    "source": "BIP-173 example mainnet bech32 P2WPKH address (NOT owned by the abandon-about wallet), supplied to sign_btc_message which compares against derived address and rejects",
    "source_url": "https://github.com/bitcoin/bips/blob/master/bip-0173.mediawiki",
    "capture_cmd": "n/a"
  }
]'
