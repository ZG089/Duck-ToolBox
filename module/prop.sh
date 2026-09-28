#!/system/bin/sh
# Sensitive-prop handler (ported from Tricky Addon). Runs early (post-fs-data, or late-load
# on a late-loaded KernelSU) so values are in place before apps start, and again after boot
# for props the system writes late, such as sys.oem_unlock_allowed. `setprop` would
# deadlock post-fs-data (KernelSU module guide); only `resetprop -n` is used.

DISABLE_FLAG="/data/adb/disable_prop_handler"
BOOT_HASH_FILE="/data/adb/boot_hash"

resetprop_bin=""
for candidate in /data/adb/ksu/bin/resetprop /data/adb/ap/bin/resetprop /data/adb/magisk/resetprop; do
  if [ -x "$candidate" ]; then
    resetprop_bin="$candidate"
    break
  fi
done
[ -n "$resetprop_bin" ] || exit 0

# The Verified Boot hash from Key Attestation applies even with the handler disabled.
if [ -f "$BOOT_HASH_FILE" ]; then
  hash_value="$(grep -v '^#' "$BOOT_HASH_FILE" | tr -d '[:space:]' | tr '[:upper:]' '[:lower:]')"
  if [ -n "$hash_value" ]; then
    "$resetprop_bin" -n ro.boot.vbmeta.digest "$hash_value"
  else
    rm -f "$BOOT_HASH_FILE"
  fi
fi

[ -f "$DISABLE_FLAG" ] && exit 0

# Replace the value when the prop exists and differs from the expected one.
reset_expected() {
  current="$("$resetprop_bin" "$1")"
  [ -z "$current" ] || [ "$current" = "$2" ] || "$resetprop_bin" -n "$1" "$2"
}

# Replace the value when it contains a marker (e.g. a recovery boot mode).
reset_if_contains() {
  case "$("$resetprop_bin" "$1")" in
    *"$2"*) "$resetprop_bin" -n "$1" "$3" ;;
  esac
}

# Fill in the value when the prop is empty.
reset_if_empty() {
  [ -n "$("$resetprop_bin" "$1")" ] || "$resetprop_bin" -n "$1" "$2"
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

# MIUI
reset_expected ro.secureboot.lockstate locked

# Realme
reset_expected ro.boot.realmebootstate green
reset_expected ro.boot.realme.lockstate 1

# Hide a boot from recovery (Magisk in recovery mode).
reset_if_contains ro.bootmode recovery unknown
reset_if_contains ro.boot.bootmode recovery unknown
reset_if_contains vendor.boot.bootmode recovery unknown

# vbmeta props an AVB-verified boot always has.
reset_if_empty ro.boot.vbmeta.device_state locked
reset_if_empty ro.boot.vbmeta.invalidate_on_error yes
reset_if_empty ro.boot.vbmeta.avb_version 1.0
reset_if_empty ro.boot.vbmeta.hash_alg sha256
reset_if_empty ro.boot.vbmeta.size 4096
