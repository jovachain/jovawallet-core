//! BTC migration spot-check.
//!
//! Reads `tools/btc-migration-check/known-android-mappings.csv` — a 2-column
//! `mnemonic,address` file exported from the production Android wallet — and
//! verifies that the SDK's BIP-84 derivation reproduces every stored address
//! byte-for-byte.
//!
//! The CSV contains user mnemonics and is excluded from git via
//! `tools/btc-migration-check/.gitignore`. The tool is run on a trusted
//! engineering machine, not in CI.
//!
//! Exit codes:
//! - `0` — every row matched.
//! - `1` — at least one row mismatched; first-two-words preview of the offending
//!   mnemonic is printed to stderr (the full mnemonic is never logged).
//! - `2` — CSV missing or unreadable.

#![forbid(unsafe_code)]

use std::process::ExitCode;

use jova_core::{JovaChain, JovaWallet};

const CSV_PATH: &str = "tools/btc-migration-check/known-android-mappings.csv";

fn main() -> ExitCode {
    let mut rdr = match csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(CSV_PATH)
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: cannot read {CSV_PATH}: {e}");
            eprintln!();
            eprintln!("The Android team must export ≥100 rows of `mnemonic,address` pairs");
            eprintln!("from production storage to this path (do NOT commit the file). See");
            eprintln!("docs/btc-migration-check.md and the open GitHub issue.");
            return ExitCode::from(2);
        }
    };

    let mut total = 0usize;
    let mut matches = 0usize;
    let mut mismatches: Vec<(String, String, String)> = Vec::new();

    for record in rdr.records() {
        let r = match record {
            Ok(r) => r,
            Err(e) => {
                eprintln!("error: malformed CSV row near record #{total}: {e}");
                return ExitCode::from(2);
            }
        };
        if r.len() < 2 {
            eprintln!(
                "error: CSV row #{total} has fewer than 2 columns; expected mnemonic,address"
            );
            return ExitCode::from(2);
        }
        let mnemonic = r[0].trim();
        let expected_addr = r[1].trim();

        let wallet = match JovaWallet::from_mnemonic(mnemonic, "") {
            Ok(w) => w,
            Err(e) => {
                eprintln!("error: row #{total}: rejected by from_mnemonic: {e}");
                return ExitCode::from(2);
            }
        };
        let derived = match wallet.address(&JovaChain::Bitcoin, 0) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("error: row #{total}: BIP-84 derivation failed: {e}");
                return ExitCode::from(2);
            }
        };

        total += 1;
        if derived.value == expected_addr {
            matches += 1;
        } else {
            mismatches.push((
                mnemonic.to_string(),
                expected_addr.to_string(),
                derived.value,
            ));
        }
    }

    println!("BTC migration check: {matches}/{total} match");

    if !mismatches.is_empty() {
        for (m, expected, got) in &mismatches {
            // First two words only — the full mnemonic is never logged.
            let preview: String = m.split_whitespace().take(2).collect::<Vec<_>>().join(" ");
            eprintln!("MISMATCH: '{preview}...' expected={expected} got={got}");
        }
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
