import type { MockModule } from "@/core/mock"
import { MockFailure } from "@/core/mock"

const genericProfile = () => ({
  key_source: { kind: "unset" },
  curve: "ed25519",
  device: {
    brand: "generic",
    model: "default",
    device: "default",
    product: "default",
    manufacturer: "generic",
    fused: 1,
    vb_state: "green",
    os_version: "13",
    security_level: "tee",
    bootloader_state: "locked",
    boot_patch_level: 20250101,
    system_patch_level: 202501,
    vendor_patch_level: 20250101,
    vbmeta_digest: null,
    dice_issuer: "Android",
    dice_subject: "KeyMint",
  },
  fingerprint: { value: "generic/default/default:13/TP1A.220624.014/0:user/release-keys" },
  server_url: "https://remoteprovisioning.googleapis.com/v1",
  num_keys: 1,
  output_path: "var/outputs/keybox.xml",
})

let profile: ReturnType<typeof genericProfile> = genericProfile()
const paths = {
  profile_path: "/data/adb/duck-toolbox/var/profile.toml",
  profile_secrets_path: "/data/adb/duck-toolbox/var/profile.secrets.toml",
  outputs_dir: "/data/adb/duck-toolbox/var/outputs",
}
const report = {
  version: 3,
  dice_entries: 2,
  uds_pub_hex: "d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c9778737",
  signature_valid: true,
  csr_version: 3,
  cert_type: "keymint",
  brand: "google",
  keys_to_sign: 1,
}

function requireKey() {
  if (profile.key_source.kind === "unset") {
    throw new MockFailure("missing_key_source", "profile has no seed or hardware key")
  }
}

const mock: MockModule = {
  feature: { id: "rkp", contract: 1 },
  handlers: {
    "rkp.profile.show": () => ({ profile, paths }),
    "rkp.profile.save": (_args, _flags, input) => {
      profile = input as typeof profile
      return { profile, paths }
    },
    "rkp.profile.clear": () => {
      profile = genericProfile()
      return { cleared: true, paths }
    },
    "rkp.profile.detect": () => ({
      profile: {
        ...genericProfile(),
        device: {
          ...genericProfile().device,
          brand: "google",
          model: "Pixel 9 Pro",
          device: "caiman",
          product: "caiman",
          manufacturer: "Google",
          os_version: "16",
          system_patch_level: 202609,
          vendor_patch_level: 20260905,
          boot_patch_level: 20260905,
          vbmeta_digest: "a1".repeat(32),
        },
        fingerprint: { value: "google/caiman/caiman:16/BP3A.250905.014/1:user/release-keys" },
      },
    }),
    "rkp.info": () => {
      requireKey()
      return {
        mode: profile.key_source.kind === "seed" ? "direct-seed" : "hw-kdf",
        curve: profile.curve,
        seed_hex: "11".repeat(32),
        public_key_hex: report.uds_pub_hex,
        output_path: paths.outputs_dir,
      }
    },
    "rkp.provision": () => {
      requireKey()
      return {
        curve: profile.curve,
        challenge_hex: "22".repeat(32),
        csr_path: `${paths.outputs_dir}/rkp-provision-20260928/csr.cbor`,
        csr_len: 1536,
        protected_data_len: 0,
        local_verify: report,
        cert_chains: [
          {
            index: 0,
            path: `${paths.outputs_dir}/rkp-provision-20260928/cert_chain_0.der`,
            summary: {
              certificates: 3,
              subjects: ["CN=Droid CA3", "CN=Droid CA2", "CN=Droid Root"],
            },
          },
        ],
        local_test_mode: false,
        fetch_eek_error: null,
        server_submission_error: null,
      }
    },
    "rkp.keybox": () => {
      requireKey()
      return {
        csr_path: `${paths.outputs_dir}/keybox_20260928.cbor`,
        keybox_path: `${paths.outputs_dir}/keybox_20260928.xml`,
        keybox_xml:
          '<?xml version="1.0"?>\n<AndroidAttestation>\n  <NumberOfKeyboxes>1</NumberOfKeyboxes>\n</AndroidAttestation>\n',
        device_id: "duck-caiman",
        chain_summary: { certificates: 3, subjects: ["CN=Droid CA3"] },
      }
    },
    "rkp.verify": (args) => ({ path: args[0], report }),
  },
}

export default mock
