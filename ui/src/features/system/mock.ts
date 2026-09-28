import type { MockModule } from "@/core/mock"
import { MockFailure } from "@/core/mock"

const now = () => Math.floor(Date.now() / 1000)
let uninstalled = false

const mock: MockModule = {
  feature: { id: "system", contract: 1 },
  handlers: {
    "system.info": () => ({
      module: {
        id: "duck-toolbox",
        name: "Duck ToolBox",
        version: "v0.1.0",
        version_code: 10,
        author: "Eltavine & KOW & MhmRdd",
        update_json: "https://raw.githubusercontent.com/eltavine/Duck-ToolBox/main/update.json",
        binary_version: "0.1.0",
      },
      root_manager: { kind: "kernel-su", version: "ksud 3.1.0" },
      device: {
        brand: "google",
        model: "Pixel 9 Pro",
        device: "caiman",
        android_release: "16",
        sdk: "36",
        security_patch: "2026-09-05",
        fingerprint: "google/caiman/caiman:16/BP3A.250905.014/1:user/release-keys",
        abi: "arm64-v8a",
        kernel_release: "6.1.134-android14-11-gd2b",
        selinux: "enforcing",
      },
    }),
    "system.update.check": (_args, flags) => ({
      channel: flags.channel === "canary" ? "canary" : "stable",
      current_version: "v0.1.0",
      current_version_code: 10,
      available: true,
      version: flags.channel === "canary" ? "v0.1.1" : "v0.2.0",
      version_code: flags.channel === "canary" ? 57 : 20,
      changelog:
        "- Tricky Store manager with TEESimulator and OhMyKeymint support\n- New WebUI following the KernelSU theme\n- [Full changelog](https://github.com/eltavine/Duck-ToolBox)",
    }),
    "system.update.install": () => ({
      version: "v0.2.0",
      version_code: 20,
      output: "- Installing module\n- Done",
      reboot_required: true,
    }),
    "system.artifacts": () => ({
      outputs: [
        {
          name: "keybox-20260928-101500.xml",
          path: "/data/adb/duck-toolbox/var/outputs/keybox-20260928-101500.xml",
          size: 4211,
          modified_unix: now() - 3600,
        },
        {
          name: "rkp-provision-20260928/csr.cbor",
          path: "/data/adb/duck-toolbox/var/outputs/rkp-provision-20260928/csr.cbor",
          size: 1536,
          modified_unix: now() - 7200,
        },
      ],
      outputs_dir: "/data/adb/duck-toolbox/var/outputs",
      profile_path: "/data/adb/duck-toolbox/var/profile.toml",
      profile_secrets_path: "/data/adb/duck-toolbox/var/profile.secrets.toml",
      log_path: "/data/adb/duck-toolbox/var/logs/duckd.log",
    }),
    "system.log": () => ({
      entries: [
        { ts: now() - 30, command: "tricky-store.save", ok: true },
        { ts: now() - 120, command: "tricky-store.keybox.set-aosp", ok: true },
        {
          ts: now() - 600,
          command: "rkp.keybox",
          ok: false,
          code: "missing_device_field",
          message: "device field `brand` must not be empty",
        },
      ],
    }),
    "system.files": (_args, flags) => {
      const path = String(flags.path ?? "/storage/emulated/0/Download")
      const extension = String(flags.extension ?? "")
      const files = [
        "keybox.xml",
        "keybox-backup.xml",
        "TA-keybox_config_20260901.json",
        "notes.txt",
      ]
      return {
        path,
        parent: path.split("/").slice(0, -1).join("/") || null,
        entries: [
          {
            name: "Keyboxes",
            path: `${path}/Keyboxes`,
            directory: true,
            size: 0,
            modified_unix: now(),
          },
          ...files
            .filter((name) => !extension || name.endsWith(`.${extension}`))
            .map((name) => ({
              name,
              path: `${path}/${name}`,
              directory: false,
              size: 4096,
              modified_unix: now() - 86_400,
            })),
        ],
      }
    },
    "system.open-url": (args) => {
      window.open(args[0], "_blank")
      return { url: args[0] }
    },
    "system.uninstall": () => {
      if (uninstalled) throw new MockFailure("internal_error", "already marked for removal")
      uninstalled = true
      return { module_id: "duck-toolbox", reboot_required: true }
    },
    "system.reboot": () => ({ rebooting: true }),
  },
}

export default mock
