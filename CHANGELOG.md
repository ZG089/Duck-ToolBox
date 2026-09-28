## Duck ToolBox

A KernelSU module for modular Android security tooling.

---

# Changelog

## Unreleased

### Architecture
- Reorganized the Rust backend into a Cargo workspace of focused crates
  (`duck-core`, `duck-platform`, `duck-rkp`, `duck-device-ids`, `duck-tricky-store`, `duckd`)
  so features are decoupled and depend only on the shared runtime.
- Versioned the JSON envelope (`api` field) so the WebUI detects backend drift instead of
  misreading data.
- Rebuilt the WebUI around a feature registry, a typed API client split into small modules,
  and a schema-driven policy editor.
- Added a CI gate (`cargo xtask line-limit`) that fails when any source file exceeds 600 lines.

### Tricky Store Manager
- Ported KOWX712's Tricky Addon (Update Target List) as a first-party feature with support for
  multiple keystore backends behind one adapter trait:
  - Tricky Store (`config.ini`) and the legacy `target.txt` + `security_patch.txt` layout.
  - TEESimulator (`config.json` profiles).
  - OhMyKeymint (`config.toml` + `injector.toml`).
- Target list with per-app modes and per-app / default attestation policy.
- Keybox installation: bundled AOSP software key, generated "unknown" self-signed key
  (EC + RSA), local file, and URL / custom providers with a safe declarative decode pipeline.
- Sensitive-prop handler and Verified Boot hash management.
- Bulk selection: Xposed modules, Magisk DenyList, and the unnecessary-apps list.
- Auto-add newly installed apps at boot.

### Dependencies & tooling
- Updated Rust crates, WebUI packages, GitHub Actions and the NDK toolchain to current stable.
- Added Dependabot for cargo, npm and GitHub Actions.
- Module scripts follow the KernelSU module guide (post-fs-data prop handler,
  boot-completed auto-apply, action.sh) and pass ShellCheck.
