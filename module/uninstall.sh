#!/system/bin/sh
# Runs when KernelSU removes the module. Runtime data under $DATA_ROOT is left in place so a
# reinstall keeps saved profiles; only the disable flag for the prop handler is cleaned up.

rm -f /data/adb/disable_prop_handler
