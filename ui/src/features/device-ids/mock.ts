import type { MockModule } from "@/core/mock"

const mock: MockModule = {
  feature: { id: "device-ids", contract: 1 },
  handlers: {
    "device-ids.defaults": () => ({
      brand: "OnePlus",
      device: "OP5D55L1",
      product: "PJZ110",
      serial: "a1b2c3d4",
      manufacturer: "OnePlus",
      model: "PJZ110",
      imei: "",
      imei2: "",
      meid: "",
      meid2: "",
      ta_name: "keymaster64",
      ta_path: "/vendor/firmware_mnt/image",
      dry_run: false,
    }),
    "device-ids.provision": (_args, _flags, input) => {
      const profile = input as Record<string, string | boolean>
      const ids = ["brand", "device", "product", "serial", "manufacturer", "model"]
        .filter((key) => profile[key])
        .map((key) => ({ label: key.toUpperCase(), value: String(profile[key]) }))
      return {
        count: ids.length,
        ids,
        dry_run: Boolean(profile.dry_run),
        ta_name: profile.ta_name,
        ta_path: profile.ta_path,
        loaded_library: profile.dry_run ? null : "/vendor/lib64/libQSEEComAPI.so",
        ta_api_version: profile.dry_run ? null : "4.0",
        ta_version: profile.dry_run ? null : "400",
        report_path: "/data/adb/duck-toolbox/var/outputs/device-ids-20260928.json",
      }
    },
  },
}

export default mock
