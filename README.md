# Duck ToolBox

Duck ToolBox is a root module (KernelSU, APatch or Magisk) that puts several Android
attestation tools behind one WebUI. The backend is a Cargo workspace of single-purpose crates
compiled into one JSON CLI (`duckd`); the WebUI is a set of feature folders that the app
discovers at build time. Features only meet through versioned contracts, so each one can be
changed, removed or added on its own.

## Tools

- **Tricky Store manager**: a complete port of KOWX712's
  [Tricky Addon: Update Target List](https://github.com/KOWX712/Tricky-Addon-Update-Target-List),
  for whichever keystore module is active:
  - **Tricky Store**: `config.ini` with per-app modes (`pkg`, `pkg!`, `pkg?`) and per-app
    policy, and the legacy `target.txt` + `security_patch.txt` layout.
  - **TEESimulator**: `config.json` profiles (mode, patch levels, identity, auto-include).
  - **OhMyKeymint**: `injector.toml` scoop and every `config.toml` `[trust]` key.

  It has the same features as upstream:
  - Target list with search, select all, deselect all and refresh.
  - Mode sheet on long press, and per-app policy.
  - Actions: select from DenyList (Magisk), deselect unnecessary apps (online list plus
    Xposed modules), add system apps, auto-add new apps.
  - Keyboxes from AOSP, generated "unknown", a local file, the keybox repository (the same
    iframe protocol) or custom providers (add, edit, remove, reset, import, export).
  - Prop handler and Verified Boot hash, schema-driven default policy with *Today*, help.
  - Keyboard shortcuts (Ctrl+A/D/F/S, Esc), the back gesture closing dialogs, and upstream's
    23 translations, including right-to-left languages.
  - Optionally, a WebUI entry on the keystore module itself, and the TSupport-Advance
    auto-target stop flag.
- **RKP Workbench**: Remote Key Provisioning.
  - Builds `AuthenticatedRequest` CSRs (DICE + COSE, per the AOSP
    `IRemotelyProvisionedComponent` HAL) and talks to the RKP server.
  - Verifies CSRs offline and exports `keybox.xml`, which the Tricky Store manager can
    install directly.
  - Device values are read from AOSP build properties.
- **Device ID Provisioner**: writes attestation device IDs into the Qualcomm Keymaster
  trusted app through `libQSEEComAPI.so`, with a dry-run mode.
- **Module & updates**: version and device summary, stable and canary updates with
  changelog, output files, command history, uninstall and reboot.

Every tool is translated into 14 languages: English, Simplified and Traditional Chinese,
Japanese, Korean, Spanish, French, German, Italian, Brazilian Portuguese, Russian, Arabic,
Vietnamese and Indonesian. Tricky Addon's other translations cover the strings the Tricky
Store manager shares with it.

## Architecture

```txt
crates/
  duck-core/          Shared runtime: paths, JSON envelope, Feature trait, command log,
                      and a Sysroot so every feature is testable off-device.
  duck-platform/      Device adapters: props/resetprop, pm, modules, root managers, HTTP.
  duck-rkp/           RKP workbench.            duck-device-ids/  Device ID provisioning.
  duck-tricky-store/  Keystore module manager.  duck-system/      Module lifecycle, updates.
  duckd/              The binary: builds its CLI from the compiled-in features.
xtask/                Repository checks (`cargo xtask line-limit`).
ui/src/
  app/                Shell: router, app bar, home and settings; discovers features.
  core/               Bridge to the root manager, validated duckd client, i18n, theme,
                      extension points and shared components.
  features/<id>/      One folder per tool: index.ts, api.ts (zod), locales/, pages, mock.ts.
  components/ui/      shadcn-vue components, restyled for Material 3 Expressive on phones.
module/               Module payload: scripts, then the built WebUI and backend binary.
```

What keeps it modular and resistant to breaking changes:

- **Features never import each other.** Backend feature crates depend only on `duck-core`
  and `duck-platform`. WebUI features depend only on `core/`; ESLint (`eslint-plugin-boundaries`)
  rejects anything else, and only `core/bridge` may touch the KernelSU JavaScript API.
- **Extension points instead of imports.** A feature contributes to others through
  `contributes`: `keyboxTargets` (RKP installs its keybox through the Tricky Store manager),
  `settingsSections` and `homeWidgets`.
- **Versioned contracts.** Every command prints `{ ok, api, command, data, error, ts }`.
  `duckd features` lists the compiled-in features and their contract versions; the WebUI
  disables a tool whose backend is missing or on another contract. It validates every
  response with zod, so a changed shape fails loudly as `contract_mismatch`.
- **Keystore modules are adapters.** Each backend implements `ConfigAdapter` and describes
  its editable policy as a schema; the WebUI renders whatever the schema declares.
- **Mature tools, no home-grown frameworks.**
  - Backend: clap, serde, tokio, reqwest, the RustCrypto crates, rcgen and `toml_edit`
    (which keeps comments).
  - WebUI: Vue 3, vue-router, Pinia, TanStack Query, vue-i18n, zod, Tailwind CSS 4 and
    shadcn-vue on reka-ui, with VueUse.
- **No source file over 600 lines.** `cargo xtask line-limit` fails CI for any source file
  in any language, including scripts without an extension; ESLint's `max-lines` flags it
  while editing.
- **Dependency updates are reviewable.** Dependabot groups minor and patch updates; every
  major update gets its own pull request and a full CI run.

### Adding a tool

1. **Backend:** create `crates/duck-<name>` with a `clap` subcommand enum and a
   `pub static FEATURE: ClapFeature<Command>` (see `duck-system`). Add it to `duckd` as an
   optional dependency and cargo feature, and one entry in the `FEATURES` array in
   `crates/duckd/src/main.rs`.
2. **WebUI:** create `ui/src/features/<id>/` with an `index.ts` exporting
   `defineFeature({ id, namespace, icon, order, routes, messages })`, an `api.ts` that calls
   `duckd()` with zod schemas, a `locales/<language>.json` for each of the 14 languages
   (`src/locales.test.ts` fails on a missing key), and optionally a `mock.ts`. The home page,
   router, i18n, availability check and dev mock pick it up automatically.

## Module lifecycle

| Stage | What runs |
| --- | --- |
| `customize.sh` | Checks the root manager, ABI and API level; prepares `/data/adb/duck-toolbox`; normalizes the boot hash; drops `action.sh` where the manager has a WebUI button; requests hot install from metamodules that support it. |
| `post-fs-data.sh` / `late-load.sh` | Early pass of the sensitive-prop handler (`resetprop -n`, never `setprop`). KernelSU's late-load mode runs `late-load.sh` instead of `post-fs-data.sh`. |
| `service.sh` | Repairs the runtime directory. On Magisk, which has no boot-completed stage, it waits for boot and runs `boot-completed.sh`. |
| `boot-completed.sh` | Late prop pass, keystore-module entry links, auto-add of new apps, the module description (KernelSU temporary `override.description`), and the TSupport-Advance flag. |
| `action.sh` (Magisk) | Opens the WebUI in KSUWebUIStandalone or WebUI X, installing KSUWebUIStandalone if neither is present. |
| `uninstall.sh` | Removes entry links, flags, the boot hash and `/data/adb/duck-toolbox`. |

Saved data lives in `/data/adb/duck-toolbox/var`, outside the module directory, so updates
keep it.

**Updates.** The stable channel follows `updateJson` in `module.prop`. The canary channel
reads the newest CI artifact through nightly.link: CI uploads the module directory as
`duck-toolbox-<version>-<commit count>-canary`, and the commit count is its `versionCode`.

## JSON CLI

```txt
duckd features | describe
duckd rkp profile show|save|clear|detect
duckd rkp info | provision | keybox | verify <file>
duckd device-ids defaults | provision
duckd tricky-store status | save | props | auto-apply
duckd tricky-store keybox install <file> | import | set-aosp | generate | fetch --url <u> [--decode <steps>]
duckd tricky-store keybox providers list | save | reset | import <file> | import-content | export [--path <file>]
duckd tricky-store apps xposed | denylist | unnecessary [--refresh]
duckd tricky-store entry status | enable | disable | apply | remove
duckd system info | update check|install [--channel stable|canary] | artifacts | files --path <dir> --extension <ext>
duckd system log [--limit <n>] | open-url <url> | uninstall | reboot
```

Commands that take input read JSON from stdin with `--stdin-json`.

## Building and testing

Requirements:
- Rust stable with the `aarch64-linux-android` target (`rust-toolchain.toml` installs it).
- Android NDK r30.
- Node.js 24 and pnpm 12 (`corepack enable` picks the version from `ui/package.json`).
  Type checking runs the TypeScript 7 engine through typescript-native-bridge, because
  stock TypeScript 7 no longer has the API vue-tsc needs.

```bash
cargo xtask line-limit
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# Device binary, e.g. with the NDK's clang as linker
cargo build --release --package duckd --target aarch64-linux-android

cd ui
pnpm install --frozen-lockfile
pnpm lint && pnpm format:check && pnpm test
pnpm build                 # type-check, then bundle into module/webroot
pnpm test:contract         # every API call through duckctl.sh and a host duckd build
pnpm test:e2e              # Playwright against a mock KernelSU host, with screenshots
pnpm dev                   # the same mock host in a desktop browser
pnpm locales:import <Tricky-Addon>/webui/public/locales/strings   # refresh translations
```

On Windows, `pwsh ./scripts/build.ps1 -PackageModule` builds everything with `cargo-ndk`
and writes the module zip to `dist/`.

**On a real KernelSU.** `scripts/device-test.sh <module zip>` prepares a disposable
userdebug emulator (or any device with root adbd) and runs `ui/e2e-device` there:
- KernelSU is late-loaded with `ksud late-load`, so no boot image is patched.
- The module and the Tricky Store, TEESimulator and OhMyKeymint releases are installed
  with `ksud module install`.
- For each keystore module, only that one is enabled and the device reboots.
- The tests drive the WebUI inside the KernelSU manager through Playwright's Android
  support. They read the files back over adb, and check each daemon's own log to confirm
  it accepted what the WebUI wrote.

## Acknowledgements

- The Tricky Store manager, its translations and the keybox repository protocol come from
  KOWX712's
  [Tricky Addon: Update Target List](https://github.com/KOWX712/Tricky-Addon-Update-Target-List)
  (Apache-2.0).
- The bundled software keybox is AOSP's reference attestation key
  (`system/keymaster/contexts/soft_attestation_cert.cpp`).
