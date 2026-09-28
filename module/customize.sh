#!/system/bin/sh
# Sourced by the KernelSU/Magisk installer. Only shell built-ins and the installer helpers
# (ui_print, abort, set_perm*) are guaranteed here.

MODULE_ID="duck-toolbox"
DATA_ROOT="${DUCK_TOOLBOX_DATA_ROOT:-/data/adb/$MODULE_ID}"
VAR_DIR="$DATA_ROOT/var"

ui_print " "
ui_print "- Duck ToolBox installer"

if [ "$KSU" = "true" ]; then
  ui_print "- Root manager: KernelSU ($KSU_VER_CODE)"
elif [ "$APATCH" = "true" ]; then
  ui_print "- Root manager: APatch ($APATCH_VER_CODE)"
elif [ -n "$MAGISK_VER_CODE" ]; then
  ui_print "- Root manager: Magisk ($MAGISK_VER_CODE)"
else
  abort "! Unsupported environment; install from KernelSU, APatch or Magisk."
fi

case "$ARCH" in
  arm64 | x64) ui_print "- Device platform: $ARCH" ;;
  *) abort "! Unsupported platform: $ARCH (arm64 or x64 required)" ;;
esac

if [ -z "$API" ] || [ "$API" -lt 26 ]; then
  abort "! Android 8.0 (API 26) or newer is required."
fi

if [ ! -d "/data/adb/modules/tricky_store" ] &&
  [ ! -d "/data/adb/modules/oh_my_keymint" ] &&
  [ ! -d "/data/adb/modules/teesim" ]; then
  ui_print "! No keystore backend found (Tricky Store, OhMyKeymint or TEESimulator)."
  ui_print "  The Tricky Store manager stays idle until one is installed."
fi

. "$MODPATH/util_functions.sh"

ui_print "- Preparing runtime directory at $DATA_ROOT"
umask 077
repair_runtime

ui_print "- Installation complete."
ui_print " "
