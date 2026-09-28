#!/system/bin/sh
# Runs in post-fs-data mode (blocking, before Zygote). Keep it fast and side-effect free
# beyond the sensitive-prop handler.
MODDIR=${0%/*}

sh "$MODDIR/prop.sh"
