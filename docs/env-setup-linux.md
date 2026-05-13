# Local Environment Setup (Linux x86_64 — Ubuntu 22.04+/24.04 LTS)

This document walks through installing every tool needed to build `jovawallet-core` on Linux. It is the **recommended dev path for Phases 2-7** because every chain (BTC, SOL, XRP), the Kotlin/Android binding, the WASM binding, the no_std primitives crate, and all reference signers (`cast`, `bdk-cli`, `solana-cli`, `xrpl-py`) run natively on Linux. The only work that requires macOS is the **Swift / iOS XCFramework** — that is validated by GitHub Actions on `macos-latest` runners and is reserved for Phase 4 (app integration) and Phase 5 (release).

For macOS arm64, see [`env-setup.md`](env-setup.md).

For per-phase reference-signer installs (Foundry / bdk-cli / solana-cli / xrpl-py), see the table in the project root [`CLAUDE.md`](../CLAUDE.md) — those install on demand, not now.

---

## Already-on-the-machine prerequisites

Before running this guide, confirm these exist:

| Thing | Verify command | Expected |
|---|---|---|
| Linux x86_64 | `uname -sm` | `Linux x86_64` |
| Distro | `lsb_release -a` | `Ubuntu 22.04` or `24.04` (other distros work but apt commands need adapting) |
| Git | `git --version` | `git version 2.40+` |
| curl | `curl --version` | any recent |
| Build essentials | `gcc --version` | `gcc 11+` or `gcc 13+` |

If anything is missing:

```bash
sudo apt-get update
sudo apt-get install -y build-essential curl git pkg-config libssl-dev unzip ca-certificates
```

---

## Versioning policy

Same as the macOS doc: **latest stable for all tools**. Use `--locked` on cargo installs (so the tool's tested transitive deps are pinned) but omit `--version` (so cargo picks the current latest). One hard coupling: **`uniffi-bindgen` (the binary) and the `uniffi` crate in workspace `Cargo.toml` must be the exact same version** (currently 0.31.1).

> **Naming heads-up:** the install crate is `uniffi` (with `--features cli`), and the installed binary is `uniffi-bindgen`. Older plan text says `uniffi-bindgen-cli` — that crate name no longer exists; treat it as the old name for `uniffi-bindgen`.

---

## What gets installed

Versions noted are current latest as of 2026-05-13. `cargo install … --locked` (no `--version`) picks whatever is current at the moment you run it.

| # | Tool | Current latest (2026-05-13) | Purpose | Time |
|---|---|---|---|---|
| 1 | `rustup` + Rust stable + nightly | **1.95.0** (rustup 1.29.0) | Compiler + cargo + per-project toolchain. Nightly needed for `cargo-fuzz` and `miri`. | 3-5 min |
| 2 | 9 rustup cross-compile targets | (matches active toolchain) | Android (4), WASM, embedded (1), Linux host (1) + the two Apple targets for CI parity if you also want them locally | 3-5 min |
| 3 | `uniffi-bindgen` (via `uniffi --features cli`) | **0.31.1** | Generates Kotlin (and Swift, for CI test parity) bindings | 3-5 min |
| 4 | `cargo-ndk` | **4.1.2** | Android cross-compile orchestration | 2-3 min |
| 5 | `wasm-pack` | **0.14.0** | Compile + wasm-bindgen + wasm-opt + npm-pack | 3-5 min |
| 6 | `cargo-deny` + `cargo-audit` | latest | Security and licence audits | 2-3 min |
| 7 | `cargo-fuzz` | **0.13.1** | Fuzz harnesses (needs nightly to run) | 1-2 min |
| 8 | `just` | **1.51.0** | Project task runner | 1 min |
| 9 | Java 21 (Temurin or OpenJDK) | **21.x LTS** | Required by AGP for Kotlin / Android binding tests | 1 min |
| 10 | Android cmdline-tools + SDK platform 36 + NDK r29 | **NDK 29.0.14206865** | Android cross-compile + Gradle test | 10-15 min |
| 11 | `ANDROID_HOME` + `ANDROID_NDK_HOME` env vars | — | So `cargo-ndk` and Gradle find the SDK/NDK | <1 min |
| 12 | Node 22+ + pnpm 10+ | Node **22.x LTS**, pnpm **10.x** | Required by the WASM binding test | 2-3 min |
| 13 | `gh` (GitHub CLI) | latest | Auth + PR / tag operations | 1 min |
| 14 | `gcc-arm-none-eabi` | apt package | C cross-compiler for `secp256k1-sys` on `thumbv7em-none-eabihf` (Phase 7) | 1 min |

**Total wall-clock:** ~45 min sequential, ~20 min if you run independent steps in parallel terminal tabs.

**Order constraint:** Step 1 (rustup) must finish first because everything after uses `cargo`. Steps 2-8, 10-14 are independent. Step 11 must come after Step 10.

---

## Step 1 — `rustup` + Rust stable + nightly

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
. "$HOME/.cargo/env"
rustc --version             # rustc 1.95.0
cargo --version             # cargo 1.95.0
rustup toolchain install nightly
rustup toolchain list       # confirm both stable and nightly are listed
```

Add cargo to PATH permanently (rustup usually adds this to `~/.bashrc` / `~/.profile`, verify):

```bash
grep -F '. "$HOME/.cargo/env"' ~/.bashrc ~/.profile 2>/dev/null || echo '. "$HOME/.cargo/env"' >> ~/.bashrc
```

If your shell is zsh, also append to `~/.zshenv` so non-interactive shells pick it up (important for CI and shell scripts).

---

## Step 2 — Cross-compile targets

```bash
rustup target add \
  aarch64-linux-android \
  armv7-linux-androideabi \
  x86_64-linux-android \
  i686-linux-android \
  wasm32-unknown-unknown \
  thumbv7em-none-eabihf
rustup target list --installed
```

The two Apple targets (`aarch64-apple-ios`, `aarch64-apple-darwin`) are not added here — they cannot link without Xcode's SDK, which is macOS-only. CI runs them on `macos-latest`.

---

## Step 3 — `uniffi-bindgen`

```bash
cargo install uniffi --features cli --locked
uniffi-bindgen --version    # 0.31.1
```

Two binaries get installed: `uniffi-bindgen` (the unified CLI) and `uniffi-bindgen-swift` (Swift-optimised). The Kotlin / WASM build path uses the unified `uniffi-bindgen`.

---

## Step 4 — `cargo-ndk`

```bash
cargo install cargo-ndk --locked
cargo-ndk --version         # 4.1.2 or newer
```

---

## Step 5 — `wasm-pack`

```bash
cargo install wasm-pack --locked
wasm-pack --version         # 0.14.0 or newer
```

Linux clang supports `wasm32-unknown-unknown` natively, so unlike macOS no Homebrew LLVM is needed. The `.cargo/config.toml` defines a `CFLAGS_wasm32_unknown_unknown=-Dmemmove=__builtin_memmove` to work around the bundled secp256k1-sys sysroot's missing `memmove` declaration; this applies on both platforms.

---

## Step 6 — `cargo-deny` + `cargo-audit`

```bash
cargo install cargo-deny --locked
cargo install cargo-audit --locked
```

These are exercised by `.github/workflows/audit.yml`.

---

## Step 7 — `cargo-fuzz`

```bash
cargo install cargo-fuzz --locked
cargo-fuzz --version        # 0.13.1
```

`cargo-fuzz` builds on stable Rust but `cargo fuzz run` commands require nightly (installed in Step 1).

---

## Step 8 — `just`

```bash
cargo install just --locked
just --list                 # lists the project's recipes
```

---

## Step 9 — Java 21

```bash
sudo apt-get install -y openjdk-21-jdk
java -version              # openjdk version "21.x"
javac -version             # javac 21.x
```

If `update-alternatives` doesn't pick OpenJDK 21 as default:

```bash
sudo update-alternatives --config java
sudo update-alternatives --config javac
```

The Kotlin scaffold pins `jvmToolchain(21)` in `bindings/kotlin/jova-core/build.gradle.kts`.

---

## Step 10 — Android cmdline-tools + SDK 36 + NDK r29

There is no SDK Manager GUI on a server VM, so install via Google's cmdline-tools.

```bash
# Pick an install root. Standard convention is $HOME/Android/sdk.
export ANDROID_HOME="$HOME/Android/sdk"
mkdir -p "$ANDROID_HOME/cmdline-tools"

# Download the latest cmdline-tools zip (URL changes; check developer.android.com for current)
CMDLINE_ZIP=commandlinetools-linux-12266719_latest.zip
curl -o /tmp/$CMDLINE_ZIP "https://dl.google.com/android/repository/$CMDLINE_ZIP"
unzip -q /tmp/$CMDLINE_ZIP -d "$ANDROID_HOME/cmdline-tools"
mv "$ANDROID_HOME/cmdline-tools/cmdline-tools" "$ANDROID_HOME/cmdline-tools/latest"

# Accept licences and install platform 36 + build tools + NDK r29
yes | "$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" --licenses
"$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" \
  "platform-tools" \
  "platforms;android-36" \
  "build-tools;36.0.0" \
  "ndk;29.0.14206865"
```

**NDK version is fixed at `29.0.14206865`** (r29 stable). Do not install `30.x` betas — Phase -1 spike confirmed r29 stable; betas can shift codegen between builds.

---

## Step 11 — Export `ANDROID_HOME` + `ANDROID_NDK_HOME`

Append to `~/.bashrc` (and `~/.zshenv` if you use zsh — the harness's non-interactive bash invocations read `.zshenv` but not `.zshrc`, so prefer `.zshenv` / `.bashrc` for env exports):

```bash
cat >> ~/.bashrc <<'EOF'

# Jovawallet-core Android build environment
export ANDROID_HOME="$HOME/Android/sdk"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/29.0.14206865"
export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$PATH"
EOF
source ~/.bashrc
echo "$ANDROID_HOME / $ANDROID_NDK_HOME"
ls "$ANDROID_NDK_HOME/source.properties"   # confirms r29 stable is installed
```

---

## Step 12 — Node 22+ + pnpm 10+

NodeSource is the simplest path on Ubuntu:

```bash
curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
sudo apt-get install -y nodejs
node --version             # v22.x or newer

# pnpm via corepack (ships with Node 16+):
sudo corepack enable
corepack prepare pnpm@latest --activate
pnpm --version             # 10.x or newer
```

Required by `bindings/wasm/` to run the Node hello-world / parity tests.

---

## Step 13 — `gh` CLI

```bash
curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo dd of=/etc/apt/keyrings/githubcli-archive-keyring.gpg
sudo chmod a+r /etc/apt/keyrings/githubcli-archive-keyring.gpg
echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null
sudo apt-get update
sudo apt-get install -y gh

# Authenticate against github.com — interactive on first run:
gh auth login
```

`gh auth login` prompts for HTTPS-vs-SSH and a one-time browser auth; both work. SSH is recommended so `git push` over SSH uses the same auth.

---

## Step 14 — `gcc-arm-none-eabi`

Needed by `secp256k1-sys` when building `jova-core-primitives` for `thumbv7em-none-eabihf` (Phase 7 / `ci-no-std.yml`).

```bash
sudo apt-get install -y gcc-arm-none-eabi
arm-none-eabi-gcc --version
```

Ubuntu's `gcc-arm-none-eabi` ships with newlib, unlike Homebrew core on macOS. No extra setup needed.

---

## Per-phase reference signers (install on demand, not now)

| Phase | Tool | Install |
|---|---|---|
| 1 (EVM) | Foundry `cast` | `curl -L https://foundry.paradigm.xyz \| bash && ~/.foundry/bin/foundryup` |
| 2 (BTC) | `bdk-cli` | `cargo install bdk-cli --locked` |
| 3 (SOL) | `solana-cli` (Anza) | `sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"` |
| 3 (XRP) | `xrpl-py` or `rippled` | `pipx install xrpl-py` (simpler than running a `rippled` node) |

---

## Verifying everything

```bash
cd ~/jovawallet-core   # or wherever you cloned the repo
. "$HOME/.cargo/env"
export ANDROID_HOME="$HOME/Android/sdk"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/29.0.14206865"

# Rust host build
cargo build --workspace

# Rust tests
cargo test --workspace

# no_std crate compiles for thumbv7em (requires gcc-arm-none-eabi)
cargo build -p jova-core-primitives --target thumbv7em-none-eabihf --release --no-default-features

# Kotlin / Android: cross-compile + build AAR + run JVM unit tests
./bindings/kotlin/scripts/build-aar.sh
(cd bindings/kotlin && ./gradlew :jova-core:test --console=plain)

# WASM: build and run Node smoke
./bindings/wasm/scripts/build-wasm.sh
(cd bindings/wasm && pnpm install && pnpm test)

# Spec consistency
cargo run --release -p jova-verify-spec
```

If all of the above succeed, the Linux dev VM is fully ready for Phase 2+ work. The Swift parity step is the only one missing locally; CI (`ci-bindings-swift.yml`) handles that on `macos-latest`.
