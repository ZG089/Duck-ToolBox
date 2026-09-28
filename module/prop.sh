#!/system/bin/sh
# Sensitive-prop handler. Runs in post-fs-data so `resetprop -n` takes effect before Zygote
# starts (the KernelSU guide forbids `setprop` here). Skipped when the user disables it.

DISABLE_FLAG="/data/adb/disable_prop_handler"
BOOT_HASH_FILE="/data/adb/boot_hash"

[ -f "$DISABLE_FLAG" ] && exit 0

resetprop_bin=""
for candidate in /data/adb/ksu/bin/resetprop /data/adb/ap/bin/resetprop /data/adb/magisk/resetprop; do
  if [ -x "$candidate" ]; then
    resetprop_bin="$candidate"
    break
  fi
done
[ -n "$resetprop_bin" ] || exit 0

# Replace VALUE only when the prop exists and differs from EXPECTED.
reset_expected() {
  current="$(getprop "$1")"
  [ -z "$current" ] || [ "$current" = "$2" ] || "$resetprop_bin" -n "$1" "$2"
}

# Fill VALUE only when the prop is currently empty.
reset_if_empty() {
  [ -z "$(getprop "$1")" ] && "$resetprop_bin" -n "$1" "$2"
}

# Replace the value when it contains a marker (e.g. a recovery boot mode).
reset_if_contains() {
  case "$(getprop "$1")" in
    *"$2"*) "$resetprop_bin" -n "$1" "$3" ;;
  esac
}

reset_expected ro.boot.vbmeta.device_state locked
reset_expected ro.boot.verifiedbootstate green
reset_expected ro.boot.flash.locked 1
reset_expected ro.boot.veritymode enforcing
reset_expected ro.boot.warranty_bit 0
reset_expected ro.warranty_bit 0
reset_expected ro.debuggable 0
reset_expected ro.force.debuggable 0
reset_expected ro.secure 1
reset_expected ro.adb.secure 1
reset_expected ro.build.type user
reset_expected ro.build.tags release-keys
reset_expected ro.vendor.boot.warranty_bit 0
reset_expected ro.vendor.warranty_bit 0
reset_expected vendor.boot.vbmeta.device_state locked
reset_expected vendor.boot.verifiedbootstate green
reset_expected sys.oem_unlock_allowed 0
reset_expected ro.secureboot.lockstate locked
reset_expected ro.boot.realmebootstate green
reset_expected ro.boot.realme.lockstate 1

reset_if_contains ro.bootmode recovery unknown
reset_if_contains ro.boot.bootmode recovery unknown
reset_if_contains vendor.boot.bootmode recovery unknown

reset_if_empty ro.boot.vbmeta.device_state locked
reset_if_empty ro.boot.vbmeta.invalidate_on_error yes
reset_if_empty ro.boot.vbmeta.avb_version 1.0
reset_if_empty ro.boot.vbmeta.hash_alg sha256

# Restore a user-provided verified boot hash captured from KeyAttestation.
if [ -f "$BOOT_HASH_FILE" ]; then
  hash_value="$(grep -v '^#' "$BOOT_HASH_FILE" | tr -d '[:space:]' | tr 'A-F' 'a-f')"
  [ -n "$hash_value" ] && "$resetprop_bin" -n ro.boot.vbmeta.digest "$hash_value"
fi
