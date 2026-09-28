# Duck ToolBox

Duck ToolBox is a KernelSU module that hosts several Android security-tooling utilities
behind one WebUI. It is built for modularity: the backend is a Cargo workspace of small,
single-purpose crates, the WebUI is a feature registry, and every feature is decoupled from
the others through a shared JSON contract.

## Tools

- **RKP Workbench** — Remote Key Provisioning. Persists a device profile, builds an
  `AuthenticatedRequest` CSR (DICE + COSE per the AOSP `IRemotelyProvisionedComponent` HAL),
  talks to the RKP server, verifies the CSR offline, and exports a `keybox.xml`.
- **Tricky Store Manager** — a full port of
  [Tricky Addon: Update Target List](https://github.com/KOWX712/Tricky-Addon-Update-Target-List).
  Detects and manages whichever keystore-spoofing backend is installed:
  - **Tricky Store** (`config.ini`, with per-app modes and policy) and the legacy
    `target.txt` + `security_patch.txt` layout.
  - **TEESimulator** (`config.json` profiles).
  - **OhMyKeymint** (`config.toml` + `injector.toml`).

  It edits the target list and attestation policy, installs keyboxes (bundled AOSP software
  key, a generated "unknown" self-signed key, a local file, or a URL/custom provider),
  manages the sensitive-prop handler and Verified Boot hash, adds system apps, auto-adds
  newly installed apps, and offers bulk selection (Xposed modules, Magisk DenyList, and the
  "unnecessary apps" list).
- **Device ID Provisioner** — provisions Qualcomm Keymaster attestation device IDs through
  `libQSEEComAPI.so`.

## Architecture

```txt
crates/
  duck-core/          Runtime shared by every feature: paths, JSON envelope, command result,
                      file helpers, and a Sysroot abstraction so features are testable off-device.
  duck-platform/      Android/root-manager adapters: getprop/resetprop, pm, module directory,
                      HTTP client, root-manager detection.
  duck-rkp/           RKP workbench (CBOR/COSE/DICE, RKP client, keybox export, CSR verify).
  duck-device-ids/    Qualcomm Keymaster device ID provisioning.
  duck-tricky-store/  Multi-backend keystore manager (the Tricky Addon port).
  duckd/              The binary: wires features into one JSON CLI.
xtask/                Repository automation (the 600-line source gate).
ui/                   Vue 3 + Vite WebUI (a feature registry + typed API client).
module/               KernelSU module payload (scripts + built WebUI + backend binary).
```

Design rules that keep the project modular and resistant to breaking changes:

- **Features never depend on each other.** They depend only on `duck-core` and
  `duck-platform`, and communicate through the versioned JSON envelope.
- **One JSON envelope, one version.** Every command prints `{ ok, api, command, data, error, ts }`.
  The WebUI checks `api` and refuses to misread a newer backend.
- **New keystore backends are one file.** Add an adapter implementing `ConfigAdapter` and one
  line in `adapters::for_backend`; the shared logic, WebUI and policy editor need no changes.
- **The WebUI renders policies from a schema.** Each backend describes its editable fields, so
  a new field or backend appears in the UI automatically.
- **No source file exceeds 600 lines** — enforced by `cargo xtask line-limit` in CI.

### Adding a tool

1. Create a crate under `crates/` that depends on `duck-core` (and `duck-platform` if it
   touches the device). Expose a `clap` subcommand and a `run` entry point returning
   `duck_core::CommandResult`.
2. Add one match arm in `crates/duckd/src/cli.rs` and `main.rs`.
3. Add a feature entry in `ui/src/lib/features.ts` and a workbench component. The launcher,
   lazy loading and command log pick it up automatically.

## JSON CLI

```txt
duckd rkp profile show|save|clear
duckd rkp info|provision|keybox|verify <file>
duckd device-ids defaults|provision
duckd tricky-store status|save|auto-apply
duckd tricky-store keybox install <file>|set-aosp|generate|fetch --url <u> --decode <steps>
duckd tricky-store keybox providers list|save|reset|import <file>|export
duckd tricky-store apps xposed|denylist|unnecessary [--refresh]
duckd tricky-store props        # prop handler + boot hash (stdin JSON)
duckd tricky-store files --path <dir> --extension xml
duckd artifacts list
```

Commands that take input read JSON from stdin with `--stdin-json`. Every command prints one
JSON envelope on stdout.

## Runtime layout

- Module root: `/data/adb/modules/duck-toolbox/`
- Shared data: `/data/adb/duck-toolbox/var/` (profiles, outputs, logs, feature state)

Saved data lives outside the module directory, so module updates never wipe it.

## Building

Requirements: Rust (stable), the `aarch64-linux-android` target, Android NDK r28+, Node 22+,
and pnpm 10+.

```bash
# Backend (host checks)
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo xtask line-limit

# Backend (device binary)
export ANDROID_NDK_HOME=/path/to/ndk
cargo build --release --package duckd --target aarch64-linux-android   # via cargo-ndk or NDK env

# WebUI (into module/webroot)
cd ui && pnpm install --frozen-lockfile && pnpm build
```

On Windows, `pwsh ./scripts/build.ps1 [-PackageModule]` runs the whole flow with `cargo-ndk`.

## Acknowledgements

The Tricky Store Manager is a port of KOWX712's
[Tricky Addon: Update Target List](https://github.com/KOWX712/Tricky-Addon-Update-Target-List)
(Apache-2.0). The bundled AOSP software keybox is the reference attestation key from
`system/keymaster` in AOSP.
