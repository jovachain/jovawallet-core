# Local Environment Setup (macOS arm64)

This document walks through installing every tool needed to build `jovawallet-core` locally on macOS arm64. Use it for: a fresh Mac, a new team member's machine, or recovery after a `~/.cargo` wipe.

For per-phase reference signers (Foundry / bdk-cli / solana-cli / rippled), see the table in the project root `CLAUDE.md` — those install on demand, not now.

---

## Already-on-the-machine prerequisites

Before running anything in this guide, confirm these exist:

| Thing | Verify command | Expected |
|---|---|---|
| macOS arm64 | `uname -sm` | `Darwin arm64` |
| Xcode + command-line tools | `xcodebuild -version` | `Xcode 16.x` or newer |
| Node.js | `node --version` | `v20.x` or newer |
| pnpm | `pnpm --version` | `10.x` or newer |
| Git + repo cloned | `git -C ~/Documents/Workspace/Jovachain/jovawallet-core status` | `clean working tree on main` |
| Android Studio installed | `ls ~/Library/Android/sdk` | directory listing |

If any are missing: install them first via `xcode-select --install`, the [Node](https://nodejs.org) installer, `npm install -g pnpm`, etc. This guide assumes all six are in place.

---

## Versioning policy for this project

**Use latest stable for all tools and crates.** Decided 2026-05-10 at project start. We use `--locked` on cargo installs (so each install gets its tool's tested transitive-dep tree) but omit `--version` (so cargo picks the current latest tool release). Concrete version numbers below are recorded as of **2026-05-10**; cargo will install whatever's current at run time.

The one hard coupling that survives: **`uniffi-bindgen` (the global binary) and the `uniffi` macro crate (in workspace `Cargo.toml`) must be the same version.** When we install the CLI here, the workspace Cargo.toml's `uniffi` dep must match exactly. Both currently track 0.31.1.

> **Naming heads-up:** older plan files refer to `uniffi-bindgen-cli`. That crate name no longer exists on crates.io — the install crate is now `uniffi-bindgen`, and the installed binary is `uniffi-bindgen` (no `-cli` suffix). When you see `uniffi-bindgen-cli` in the per-phase plans, treat it as the old name for `uniffi-bindgen`.

## What gets installed by this guide

Versions noted are current latest as of 2026-05-10. `cargo install … --locked` (no `--version`) picks whatever is current at the moment you run it.

| # | Tool | Current latest (2026-05-10) | Purpose | Time |
|---|---|---|---|---|
| 1 | `rustup` + Rust stable | **1.95.0** (rustup 1.29.0) | Compiler + cargo + per-project toolchain | 2-3 min |
| 2 | 10 rustup cross-compile targets | (matches active toolchain) | iOS, Android, WASM, embedded pre-built `libstd` | 3-5 min |
| 3 | `uniffi-bindgen` (installed via `uniffi --features cli`) | **0.31.1** | Generates Swift + Kotlin bindings | 3-5 min |
| 4 | `cargo-ndk` | **4.1.2** | Android cross-compile orchestration | 2-3 min |
| 5 | `wasm-pack` | **0.14.0** | Compile + wasm-bindgen + wasm-opt + npm-pack | 3-5 min |
| 6 | Android NDK | **r29 stable (29.0.14206865)** — latest stable per GitHub releases (Oct 2025) | Actual Android cross-compiler/linker | 3-10 min |
| 7 | `ANDROID_HOME` + `ANDROID_NDK_HOME` env vars | — | So `cargo-ndk` and CI scripts find the NDK | <1 min |

**Total wall-clock:** ~30 min sequential, ~15 min if you run steps 2-6 in parallel terminal tabs.

**Order constraint:** Step 1 (rustup) must finish first because everything after uses `cargo`. Steps 2-6 are independent. Step 7 must come after Step 6.

---

## Step 1 — Rust toolchain manager + latest stable Rust

### Why

`rustup` is Rust's toolchain manager. It installs `rustc` (the compiler), `cargo` (build/dependency manager), `rustfmt` (formatter), and `clippy` (linter), and lets you switch toolchain versions per-project via a `rust-toolchain.toml` file. Without it, no Rust code compiles. We install the current stable channel; the project's `rust-toolchain.toml` (created in Phase -1 Task 2) will pin the exact version chosen during the spike, so other Rust projects on this machine remain on whatever they need.

### Command

Fresh install (installs rustup AND the current stable Rust in one step):

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile default
```

Then in the same shell:

```
source "$HOME/.cargo/env"
```

To make `~/.cargo/bin` permanent in new terminals (rustup-init usually does this; this command makes it idempotent):

```
grep -q 'cargo/env' ~/.zshrc 2>/dev/null || echo '. "$HOME/.cargo/env"' >> ~/.zshrc
```

### If you already have an older Rust installed (e.g., 1.95.0 from an earlier env-setup run)

```
rustup install stable
rustup default stable
rustup update
```

### Alternatives

| Method | Pros | Cons |
|---|---|---|
| **Official rustup script (above)** | Standard everywhere; supports per-project pinning via `rust-toolchain.toml`; clean `rustup update` later | Edits shell rc files |
| `brew install rustup` | macOS-native; visible in `brew list` | Brew sometimes lags rustup releases by days/weeks |
| `brew install rust` | One brew command | Installs ONE Rust version only — fights `rust-toolchain.toml`; can't add cross-compile targets cleanly. **Don't use for this project.** |
| Build rustc from source | Full control | Multi-hour compile; no benefit. **Don't.** |

### Recommendation

The official rustup script. It's the only option that handles project-pinned toolchains cleanly.

### Why latest stable, not a specific pin

Per the project's "latest at start" policy (top of this doc): we install the current stable channel now. During Phase -1 Task 2 the spike will record whichever exact `rustc --version` is running and write that into `rust-toolchain.toml`, locking the project to a known-good toolchain from that point on. So "latest" applies to install-time selection; the project becomes pinned to a specific version once the spike chooses one.

**Heads-up as of 2026-05-10:** current latest stable is **1.95.0** (released 2026-04-16). If you ran the rustup install earlier in this session with `--default-toolchain 1.95.0`, you're already on current latest — no upgrade step needed.

### Verify

```
source "$HOME/.cargo/env"
rustup --version
rustc --version
cargo --version
```

`rustc --version` should print `rustc 1.95.0 (...)` or newer.

---

## Step 2 — Cross-compile targets (10 of them)

### Why

A "target triple" identifies an `(architecture, OS, ABI)` combination — e.g., `aarch64-apple-ios` is "Apple Silicon iOS". Each target needs its own pre-compiled standard library (`libstd`, `libcore`, `liballoc`) shipped from rust-lang's CDN. Without these installed, cross-compiling fails at link time. We need 10 of them because we ship to iOS device + simulator, macOS arm64+x86, four Android ABIs, WASM, and ARM Cortex-M for hardware wallet firmware.

### Command

```
rustup target add aarch64-apple-ios aarch64-apple-ios-sim aarch64-apple-darwin x86_64-apple-darwin aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android wasm32-unknown-unknown thumbv7em-none-eabihf
```

### What each target is for

| Triple | Used for |
|---|---|
| `aarch64-apple-ios` | iPhone / iPad (real devices, arm64) |
| `aarch64-apple-ios-sim` | iOS Simulator on Apple Silicon Macs |
| `aarch64-apple-darwin` | macOS arm64 (M-series Macs + Mac slice of XCFramework) |
| `x86_64-apple-darwin` | macOS Intel (universal Mac slice for older hardware) |
| `aarch64-linux-android` | Android `arm64-v8a` (most modern phones) |
| `armv7-linux-androideabi` | Android `armeabi-v7a` (32-bit ARM, ~10% market in some regions) |
| `x86_64-linux-android` | Android x86_64 emulator on Intel hosts |
| `i686-linux-android` | Android x86 (32-bit emulator, low usage but still required by play store) |
| `wasm32-unknown-unknown` | WebAssembly for browsers + Node (Phase 6) |
| `thumbv7em-none-eabihf` | ARM Cortex-M4F/M7 firmware (Phase 7 hardware wallet) |

### Alternatives

| Approach | Pros | Cons |
|---|---|---|
| **All 10 upfront (above)** | One install (~700 MB total); no surprise pauses later | Bigger upfront download |
| Per-phase, just-in-time | Smaller initial footprint | Each phase pause has a "wait, install target..." moment; wastes flow |
| Skip 32-bit Android (`armv7`, `i686`) | Saves ~150 MB | Plan needs them; iOS+Android are v1 launch targets. **Don't skip.** |

### Recommendation

All 10 upfront. They're needed across the whole project lifetime; the install is cached; per-phase friction is worse than one-time download.

### Verify

```
rustup target list --installed
```

Should list 11 entries: 10 added above plus your host (`aarch64-apple-darwin` shown twice if it's also your host, which it is on Apple Silicon — that's fine).

---

## Step 3 — `uniffi-bindgen` (current latest 0.31.1) — installed via `uniffi --features cli`

> **Install pattern change in modern uniffi:** the plan files refer to `cargo install uniffi-bindgen-cli` — that crate doesn't exist on crates.io. In uniffi 0.30+, the CLI binary ships inside the `uniffi` umbrella crate, gated behind the `cli` feature. So the install crate is `uniffi`, not `uniffi-bindgen-cli`; the installed binaries are `uniffi-bindgen` and `uniffi-bindgen-swift`.

### Why

`uniffi-rs` is Mozilla's tool for generating language bindings from Rust. We annotate Rust functions and types with `#[uniffi::export]`, build the crate as a dynamic library, and then run `uniffi-bindgen` to produce idiomatic Swift and Kotlin source files that call into our Rust dylib. Without it, no Swift package and no Kotlin AAR — i.e., no iOS or Android binding. The CLI binary lives inside the same `uniffi` crate that the project depends on as a macro/build dep, which means **macro and CLI version are the same by construction**, eliminating the historical mismatch footgun.

### Command

```
cargo install uniffi --features cli --locked
```

Installs current latest of the `uniffi` crate (0.31.1 as of 2026-05-10) with the `cli` feature enabled. Drops two binaries into `~/.cargo/bin/`:
- `uniffi-bindgen` — the unified CLI for Swift, Kotlin, Python, Ruby bindings
- `uniffi-bindgen-swift` — a Swift-optimized variant

The `--locked` flag uses the exact `Cargo.lock` shipped with that release — reproducible build of the CLI's own dep tree.

After install, **confirm the version** (`uniffi-bindgen --version`). Workspace `Cargo.toml` (created in Phase -1 Task 2) will reference the same `uniffi` crate at the same version — the macro and CLI are now literally the same crate.

### Alternatives

| Approach | Pros | Cons |
|---|---|---|
| **`cargo install uniffi --features cli --locked` (above)** | Macro/CLI version match by construction; one crate to track | Pulls clap + camino as transitive build deps |
| `cargo install --git https://github.com/mozilla/uniffi-rs uniffi-bindgen-cli --locked` (legacy) | Works on older uniffi versions | Git source is slower to install; no version pinning; deprecated for 0.30+ |
| Hand-written Swift/Kotlin bindings | Full control over ergonomics | Drift = production bugs. The whole point of uniffi is one source of truth. **Don't.** |
| Trust Wallet Core's binding generator | Mature ecosystem | We're explicitly NOT using TWC (`docs/decisions.md` D1) |

### Recommendation

`cargo install uniffi --features cli --locked`. Use the same `uniffi` version in workspace Cargo.toml (matches automatically since it's the same crate).

### Verify

```
uniffi-bindgen --version
which uniffi-bindgen-swift
```

`uniffi-bindgen --version` should print `uniffi-bindgen 0.31.1` (or whatever's current latest). The second line confirms the Swift variant binary also installed. Note the version — workspace `Cargo.toml`'s `uniffi` dep must match it.

---

## Step 4 — `cargo-ndk` (current latest 4.1.2)

### Why

Cross-compiling Rust to Android requires the NDK's `clang` and `lld` linker, plus the right sysroot for each ABI. `cargo-ndk` is a `cargo` subcommand that wraps `cargo build`, injecting the right `CC`, `AR`, `CARGO_TARGET_*_LINKER` env vars and sysroot for each target. One command builds all 4 ABIs and writes them into the `jniLibs/<abi>/` layout that Android expects. Without it, you'd hand-write 4 sets of NDK paths per build — brittle and easy to break.

> **Major-version note:** plan files reference `cargo-ndk 3.5`. Current latest is `4.1.2` — major-version jump. The CLI surface is largely the same; if Phase -1 Task 4 commands break, check the cargo-ndk 4.x changelog.

### Command

```
cargo install cargo-ndk --locked
```

Installs current latest (4.1.2 as of 2026-05-10), transitive deps frozen.

### Alternatives

| Approach | Pros | Cons |
|---|---|---|
| **`cargo-ndk` (above)** | Standard; one command builds all 4 ABIs; widely used in Android-Rust projects | One more cargo plugin |
| Hand-set env vars in `.cargo/config.toml` per target | No external dep | Brittle; NDK path moves break it; 4 ABIs × N env vars = lots of YAML |
| `cross` (Docker-based) | Reproducible across hosts; isolates host tooling | Docker required; slower per build; NDK still has to be in the container |
| `gradle-rust-plugin` | Drives Rust from Gradle | Wrong direction — we publish a Maven AAR, not a Gradle-driven Android app |

### Recommendation

`cargo-ndk` latest with `--locked`.

### Verify

```
cargo ndk --version
```

Should print `cargo-ndk 4.1.2` or newer.

---

## Step 5 — `wasm-pack` (current latest 0.14.0)

### Why

Building a WebAssembly package that's loadable from npm requires four steps: (1) compile to `wasm32-unknown-unknown`; (2) run `wasm-bindgen` to generate JavaScript glue code; (3) run `wasm-opt` (binaryen) to shrink the binary; (4) write a `package.json` with the correct entry points. `wasm-pack` does all four in one command and ensures version compatibility between the `wasm-bindgen` macro (in our Cargo.toml) and the `wasm-bindgen-cli` tool (a separate install). Without it, you'd manually chain four tools and hit version-mismatch footguns regularly.

### Command

```
cargo install wasm-pack --locked
```

Installs current latest (0.14.0 as of 2026-05-10), transitive deps frozen.

### Alternatives

| Approach | Pros | Cons |
|---|---|---|
| **`wasm-pack` (above)** | Standard; one-command build; auto-installs matching `wasm-bindgen-cli` and `wasm-opt` | Pulls binaryen behind your back |
| Hand-chain `cargo + wasm-bindgen + wasm-opt + npm pack` | Full control | Version skew between `wasm-bindgen` macro and `wasm-bindgen` CLI is a persistent footgun |
| `trunk` | Better for full-app builds (HTML+JS+WASM) | Wrong tool — we're building a library, not an app |
| `cargo-component` (WASI components) | Future-facing | Not the target — we ship classic wasm-bindgen for browsers |

### Recommendation

`wasm-pack` latest with `--locked`.

### Verify

```
wasm-pack --version
```

Should print `wasm-pack 0.14.0` or newer.

---

## Step 6 — Android NDK r29 stable (current latest 29.0.14206865)

### Why

Android Rust libraries are loaded into the JVM as `.so` files. To produce `.so` for each ABI (`arm64-v8a`, `armeabi-v7a`, `x86_64`, `x86`), you need the corresponding clang cross-compiler, linker, sysroot headers, and runtime libs. Those all live in the **Android NDK** (Native Development Kit). `cargo-ndk` (Step 4) is just a wrapper — the NDK itself does the actual compilation. Without it, no Android binding.

### Which version to install

**Latest stable: r29 (NDK version `29.0.14206865`, released October 2025).** Verified via GitHub's release API as of 2026-05-10 — there is no stable r30 yet. r30 is in beta (`30.0.14904198-beta1` and similar builds) and Studio's SDK Manager surfaces both stable and beta entries in one sorted list, so it's easy to accidentally pick the beta. **Tick the row WITHOUT a `-beta` or `-rc` suffix.**

For a signing SDK that goes through external audit and CI parity, beta NDKs are a no-go — they can shift codegen between builds and CI typically uses stable.

### Three install paths

You have Android Studio installed, so two of the three options are easy.

#### First: check whether Studio's `cmdline-tools` are present

```
ls "$HOME/Library/Android/sdk/cmdline-tools/latest/bin/sdkmanager" 2>/dev/null && echo PRESENT || echo MISSING
```

If `PRESENT`, prefer **Option B** below (CLI install, faster).
If `MISSING`, prefer **Option A** below (Studio GUI), or install cmdline-tools from Studio first.

#### Option A — Android Studio SDK Manager (GUI)

1. Open Android Studio.
2. **More Actions / Welcome → SDK Manager** (or **Settings → Languages & Frameworks → Android SDK** if a project is open).
3. Click the **SDK Tools** tab.
4. Check **Show Package Details** at the bottom-right.
5. Expand **NDK (Side by side)**.
6. Tick `29.0.14206865` (the r29 stable line, **no `-beta` or `-rc` suffix**). Note the exact version string — you'll need it for Step 7. **Do not** tick `30.0.x-beta1` or any `-betaN` / `-rcN` row even if it has a higher number.
7. Click **Apply**, accept the license, wait for the ~1 GB download.
8. While there, also check **Android SDK Command-line Tools (latest)** so the CLI path works in future.

| | |
|---|---|
| **Pros** | Visual confirmation; same SDK Studio already manages; license UI is clear; updates surface in Studio's notifications; if you ever build a real Android app, no extra setup |
| **Cons** | Slowest path (Studio launch + UI navigation); have to read the version path string by eye; locked to whatever NDK versions Studio offers (usually fine) |

#### Option B — `sdkmanager` CLI

First, list available NDK versions and pick the highest:

```
export ANDROID_HOME="$HOME/Library/Android/sdk"
"$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" --list | grep -E "^\s*ndk;" | sort -V | tail -10
```

Then install the highest version shown (substitute the version string after `ndk;`):

```
"$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" --install "ndk;<HIGHEST_VERSION>"
```

Accept any pending licenses (one-time):

```
yes | "$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" --licenses
```

| | |
|---|---|
| **Pros** | Scriptable — paste 3 lines, done; same approach CI uses; ~1-3 min install (no UI); easy to redo on a new machine |
| **Cons** | Needs `cmdline-tools/latest` present (one-time GUI step in Studio if missing); license accept can hang if you forget the second command |

#### Option C — Direct download (no SDK Manager)

Pick the latest NDK from <https://developer.android.com/ndk/downloads>, then substitute the release filename below. Example with r27c (replace with the actual current release):

```
mkdir -p "$HOME/Library/Android/ndk" && cd "$HOME/Library/Android/ndk"
curl -L -O https://dl.google.com/android/repository/android-ndk-<RELEASE>-darwin.dmg
hdiutil attach android-ndk-<RELEASE>-darwin.dmg
cp -R "/Volumes/Android NDK <RELEASE>/AndroidNDK"*.app/Contents/NDK ./<RELEASE>
hdiutil detach "/Volumes/Android NDK <RELEASE>"
```

| | |
|---|---|
| **Pros** | Fully self-contained; no SDK Manager dependency; smallest footprint; easy to delete and reinstall |
| **Cons** | Manual download + extract; no automatic update path; doesn't integrate with Studio (Studio will want its own copy if you ever use it for real Android dev); DMG layout occasionally changes between releases |

### Recommendation

**Option B** if the cmdline-tools check above said `PRESENT`. Reasons:
- Matches what CI does (the GitHub Actions workflow uses `nttld/setup-ndk@v2`, which is `sdkmanager` semantics).
- Reuses the SDK Studio already manages — one Android SDK on the machine, not two.
- Scriptable and fast — 5 minutes total including license accept.

Fallback to **Option A** if the check said `MISSING` (one-time GUI step in Studio is fine since you already have it open). While in Studio's SDK Manager, also tick the **Android SDK Command-line Tools (latest)** box so future Mac setups can use Option B.

Avoid **Option C** unless you specifically don't want Studio to know about this NDK.

### Verify

```
ls "$HOME/Library/Android/sdk/ndk/"
```

Should show one or more version directories. Note the exact directory name of the version you installed — Step 7 needs that string.

---

## Step 7 — Set Android env vars permanently

`cargo-ndk` and the CI scripts read `ANDROID_NDK_HOME` to find the NDK, and `ANDROID_HOME` to find the rest of the SDK. Set both in `~/.zshrc` so they persist across terminal sessions.

First, confirm your installed NDK version directory matches `29.0.14206865`:

```
ls ~/Library/Android/sdk/ndk/
```

Then run (uses r29 stable directory name as recorded 2026-05-10; substitute if you installed a newer stable later):

```
echo 'export ANDROID_HOME="$HOME/Library/Android/sdk"' >> ~/.zshrc
echo 'export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/29.0.14206865"' >> ~/.zshrc
source ~/.zshrc
```

### Verify

```
echo "ANDROID_HOME=$ANDROID_HOME"
echo "ANDROID_NDK_HOME=$ANDROID_NDK_HOME"
ls "$ANDROID_NDK_HOME/build/cmake/android.toolchain.cmake"
```

The last command should print the path of the toolchain file (proving the NDK is reachable through the env var). If it prints "No such file", the env var path is wrong — recheck Step 7's version string.

---

## Final one-shot verification

Once all 7 steps are done, run this block. It prints the version of every tool we installed and confirms the env vars are set:

```
source ~/.cargo/env
echo "=== Rust ===" ; rustc --version ; cargo --version
echo "=== Targets ===" ; rustup target list --installed
echo "=== uniffi-bindgen-cli ===" ; uniffi-bindgen-cli --version
echo "=== cargo-ndk ===" ; cargo ndk --version
echo "=== wasm-pack ===" ; wasm-pack --version
echo "=== Android ===" ; echo "ANDROID_HOME=$ANDROID_HOME" ; echo "ANDROID_NDK_HOME=$ANDROID_NDK_HOME" ; ls "$ANDROID_NDK_HOME" 2>/dev/null | head -5
echo "=== Already-on-machine ===" ; node --version ; pnpm --version ; xcodebuild -version | head -1
```

Expected output (versions current as of 2026-05-10; yours may be newer):

```
=== Rust ===
rustc 1.95.0 (59807616e 2026-04-14)
cargo 1.95.0 (...)
=== Targets ===
aarch64-apple-darwin
aarch64-apple-ios
aarch64-apple-ios-sim
aarch64-linux-android
armv7-linux-androideabi
i686-linux-android
thumbv7em-none-eabihf
wasm32-unknown-unknown
x86_64-apple-darwin
x86_64-linux-android
=== uniffi-bindgen ===
uniffi-bindgen 0.31.1
=== cargo-ndk ===
cargo-ndk 4.1.2
=== wasm-pack ===
wasm-pack 0.14.0
=== Android ===
ANDROID_HOME=/Users/satoshi/Library/Android/sdk
ANDROID_NDK_HOME=/Users/satoshi/Library/Android/sdk/ndk/<your-version>
<your-version>
=== Already-on-machine ===
v25.9.0
10.33.2
Xcode 26.4.1
```

Record the four tool version numbers (Rust, uniffi-bindgen, cargo-ndk, wasm-pack) and the NDK version directory name — they get plugged into `rust-toolchain.toml`, the workspace `Cargo.toml`, and the GitHub Actions workflow during Phase -1 Task 2.

If anything is missing or the version is wrong, jump back to that step.

---

## Troubleshooting

### `cargo: command not found` after Step 1

You haven't sourced `~/.cargo/env` in this terminal session. Run:

```
source "$HOME/.cargo/env"
```

For permanent fix, see Step 1's "make permanent" line.

### `error: failed to compile uniffi-bindgen-cli` (or any cargo install)

Usually a transient network or rate-limit issue. Retry. If repeated, check `~/.cargo/registry/index/` permissions and that `crates.io` is reachable.

### `cargo-ndk` errors with "ANDROID_NDK_HOME not set"

You haven't completed Step 7, or the env var points to a path that doesn't exist. Verify with `ls "$ANDROID_NDK_HOME"`.

### iOS targets fail with linker errors

Make sure Xcode and Xcode command-line tools are both installed:

```
xcode-select -p
xcodebuild -version
```

If `xcode-select -p` prints `/Library/Developer/CommandLineTools` (instead of `/Applications/Xcode.app/...`), point it at the full Xcode:

```
sudo xcode-select -s /Applications/Xcode.app/Contents/Developer
```

### `armv7-linux-androideabi` build fails on a specific crate

This was historically common with the monolithic `solana-sdk` on 32-bit ARM. We use the Anza split crates, which should be cleaner. If you hit it during Phase -1 Task 4, it's a feasibility-spike finding to record in the report — that's the spike's whole point.

### NDK installed via Studio but `cargo-ndk` can't find it

The Studio path uses the version directory under `~/Library/Android/sdk/ndk/<VERSION>`. Verify what's there:

```
ls ~/Library/Android/sdk/ndk/
```

Use that exact directory name in `ANDROID_NDK_HOME`, not a "latest" symlink (which Studio doesn't create).

---

## What we deliberately do NOT install yet

These are required by later phases. The plan installs each one as a precondition to the task that needs it. Don't install them now — install with `--locked` (no `--version`) when each phase begins.

| Tool | Required by | When |
|---|---|---|
| `just` (task runner) | Phase 0 task running | Phase 0 |
| `cargo-fuzz` (requires nightly Rust) | Phase 1 fuzz harnesses | Phase 1 |
| Foundry / `cast` (EVM signer) | Phase 1 EVM vector capture | Phase 1 |
| `bdk-cli` (Bitcoin signer) | Phase 2 BTC vector capture | Phase 2 |
| `solana-cli` (Anza release) | Phase 3 SOL vector capture | Phase 3 |
| `rippled` or `xrpl-py` | Phase 3 XRP vector capture | Phase 3 |

---

## When you're done with this guide

- Tell the agent the output of the "Final one-shot verification" block.
- The agent marks env-readiness complete and either stops the session or proceeds to Phase -1 Task 1 (creating the `spike/feasibility` branch). Per the project's no-autonomous-execution preference, the agent will confirm before doing the latter.
