#!/system/bin/sh
# post-fs-data mode: blocking, before Zygote (KernelSU module guide). Only the early prop
# pass runs here.
MODDIR=${0%/*}

sh "$MODDIR/prop.sh"
