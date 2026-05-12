//! verify-spec — fails CI if docs/* and spec/* drift, or if test-vectors.json is malformed.
//!
//! Checks performed:
//! 1. Frozen-copy invariant: docs/api.md == spec/api.md, docs/chains.md == spec/chains.md.
//! 2. test-vectors.json must parse and contain a "vectors" array.
//! 3. Every vector must have required fields: id, kind, input, expected.
//! 4. No "expected" field value may contain the placeholder tokens TODO, <capture, or REPLACE.

use std::fs;
use std::process::ExitCode;

const PLACEHOLDER_TOKENS: &[&str] = &["TODO", "<capture", "REPLACE"];

/// Recursively scan a JSON value for forbidden placeholder strings.
/// Returns a list of field paths where a placeholder was found.
fn find_placeholders(value: &serde_json::Value, path: &str) -> Vec<String> {
    let mut hits = Vec::new();
    match value {
        serde_json::Value::String(s) => {
            for token in PLACEHOLDER_TOKENS {
                if s.contains(token) {
                    hits.push(format!("{}: contains \"{}\"", path, token));
                    break; // only report first match per leaf
                }
            }
        }
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                let child_path = format!("{}.{}", path, k);
                hits.extend(find_placeholders(v, &child_path));
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let child_path = format!("{}[{}]", path, i);
                hits.extend(find_placeholders(v, &child_path));
            }
        }
        _ => {}
    }
    hits
}

fn main() -> ExitCode {
    let mut errors: Vec<String> = Vec::new();

    // ── 1. Frozen-copy invariant: docs/api.md == spec/api.md, docs/chains.md == spec/chains.md ──
    for (a, b) in [
        ("docs/api.md", "spec/api.md"),
        ("docs/chains.md", "spec/chains.md"),
    ] {
        let da = fs::read_to_string(a).unwrap_or_else(|e| {
            errors.push(format!("read {}: {}", a, e));
            String::new()
        });
        let db = fs::read_to_string(b).unwrap_or_else(|e| {
            errors.push(format!("read {}: {}", b, e));
            String::new()
        });
        if !da.is_empty() && !db.is_empty() && da != db {
            errors.push(format!("DRIFT: {} != {}", a, b));
        }
    }

    // ── 2. test-vectors.json must parse ──
    let vectors_raw = fs::read_to_string("spec/test-vectors.json").unwrap_or_else(|e| {
        errors.push(format!("read spec/test-vectors.json: {}", e));
        String::new()
    });

    if !vectors_raw.is_empty() {
        match serde_json::from_str::<serde_json::Value>(&vectors_raw) {
            Err(e) => errors.push(format!("test-vectors.json parse: {}", e)),
            Ok(doc) => {
                // ── 3. "vectors" array must exist ──
                match doc.get("vectors").and_then(|x| x.as_array()) {
                    None => errors.push("test-vectors.json: missing 'vectors' array".into()),
                    Some(arr) => {
                        for (i, vector) in arr.iter().enumerate() {
                            let id = vector
                                .get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("<unknown>");

                            // ── 3a. Required fields per vector ──
                            for field in &["id", "kind", "input", "expected"] {
                                if vector.get(field).is_none() {
                                    errors.push(format!(
                                        "test-vectors.json vectors[{}]: missing required field '{}'",
                                        i, field
                                    ));
                                }
                            }

                            // ── 4. No placeholder tokens allowed in "expected" ──
                            if let Some(expected) = vector.get("expected") {
                                let path = format!("vectors[{}](id={}).expected", i, id);
                                let hits = find_placeholders(expected, &path);
                                for hit in hits {
                                    errors.push(format!("PLACEHOLDER: {}", hit));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // ── Report ──
    if errors.is_empty() {
        println!("verify-spec: OK");
        ExitCode::SUCCESS
    } else {
        for e in &errors {
            eprintln!("verify-spec ERROR: {}", e);
        }
        ExitCode::FAILURE
    }
}
