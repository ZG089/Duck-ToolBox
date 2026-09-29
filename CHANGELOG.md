# Changelog

## Unreleased

### Tricky Store manager
- Complete port of Tricky Addon (Update Target List):
  - Target list with modes, per-app policy, search, bulk selection, DenyList, unnecessary
    apps, system apps and auto-add.
  - Keyboxes from AOSP, "unknown", a local file, the keybox repository and custom providers,
    with import and export.
  - Prop settings, default policy with *Today*, help, keyboard shortcuts and its 23
    translations, including right-to-left languages.
- One manager for Tricky Store (`config.ini` and legacy `target.txt`), TEESimulator
  (`config.json`) and OhMyKeymint (`config.toml` + `injector.toml`).
  - The validation rules follow each module's own sources.
  - Unknown sections, comments and keys are preserved.
- Optional WebUI entry on the keystore module (plus the action button on Magisk), and the
  TSupport-Advance auto-target stop flag.
- Tricky Addon's in-app translation update, translation guide and Telegram link.
- Checked against the real modules on a KernelSU device, which found and fixed:
  - OhMyKeymint v1.2.0 rejected our `injector.toml` (an unknown `version` key) while the
    WebUI reported success. `[scoop.<package>]` tables are now read and written like
    OhMyKeymint's own parser does.
  - The "unknown" keybox now gives each key a two-certificate chain. Tricky Addon's
    single-certificate keybox only loads on Tricky Store: TEESimulator and OhMyKeymint
    refuse it.
  - TEESimulator accepts EC-only keyboxes, such as the RKP workbench exports.
  - Saving tells when OhMyKeymint needs a restart to apply a change.

### WebUI
- Rebuilt on Vue 3.5, Vite 8, Tailwind CSS 4 and shadcn-vue.
  - A black and white theme, light or dark, where color only marks errors, success and
    warnings.
  - Designed for phones along Material 3 Expressive:
    - 48dp touch targets and a visible response to every press.
    - Round buttons that square up while pressed, segmented lists, and spring motion.
    - A large title that collapses into the 64dp app bar, for one-handed reach.
    - Tonal gray surfaces instead of borders and shadows, with contrast checked
      against WCAG.
  - Menus, tabs and button groups mirror in right-to-left languages too.
  - Follows the KernelSU manager's window insets.
  - The Android back gesture closes dialogs and sheets.
- Help: long links and code wrap inside the page, and two slips in upstream translations
  (a link missing its parenthesis, a code span closed with a quote) no longer show raw
  Markdown.
- Every tool is a self-contained feature folder, discovered automatically.
  - Features plug into each other only through extension points.
  - Every backend response is validated against a schema.
- A tool whose backend feature is missing or on another contract version is disabled
  instead of misreading data.
- Unit tests (Vitest) and end-to-end tests (Playwright against a mock KernelSU host) with
  light, dark, right-to-left and Chinese screenshots.
- Contract tests run every API call through `duckctl.sh` and a real `duckd` build and
  parse the results with the WebUI's schemas.
- Device tests (`scripts/device-test.sh`) drive the WebUI inside the real KernelSU manager
  with each keystore module installed.
- Loads as seven files instead of about eighty. The KernelSU manager reads each request
  through a root shell, so this halved the load time on a slow device.
- Type-checked on the TypeScript 7 engine through typescript-native-bridge.

### Backend
- `duckd` builds its CLI from a feature registry. `duckd features` reports the compiled-in
  features, and each feature can be built alone.
- New `system` feature: module and device summary, stable and canary updates with changelog,
  output files, command history, uninstall and reboot.
- RKP profiles read their device defaults from AOSP build properties.
- Updated RustCrypto to its current generation (p256/p384 0.14, ed25519/x25519 3, aes-gcm
  0.11, x509-cert 0.3). Keys derived from saved seeds are pinned by regression tests.

### Module
- Scripts follow the KernelSU module guide:
  - Late-load support (`late-load.sh`).
  - A late prop pass after boot for values the system sets late.
  - The module description goes into KernelSU's temporary config.
- Magisk runs the boot-completed work from `service.sh`.
- The prop handler adds `ro.boot.vbmeta.size`, and applies the boot hash even when the
  handler is disabled, as upstream does.
- Uninstalling removes entry links, flags and saved data.

### Tooling
- CI fails when any source file in any language exceeds 600 lines.
- CI: NDK r30, Node 24, pnpm 12, WebUI lint/format/unit/E2E, per-feature builds.
- Canary artifacts are the installable module directory.
- Dependabot opens one pull request per major update.
