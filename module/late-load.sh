#!/system/bin/sh
# KernelSU late-load mode replaces post-fs-data with this stage (KernelSU module guide,
# "Late-load mode"), so the early prop pass runs here instead.
MODDIR=${0%/*}

sh "$MODDIR/prop.sh"
