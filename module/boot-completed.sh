#!/system/bin/sh
# Runs after boot completes, when `pm` is available. Applies the Tricky Store auto-target
# additions if the user enabled them; a failure here never blocks anything.

MODPATH=${0%/*}

if [ -z "${DUCK_TOOLBOX_BUSYBOX_REEXEC:-}" ] && [ -x /data/adb/ksu/bin/busybox ]; then
  export DUCK_TOOLBOX_BUSYBOX_REEXEC=1
  export ASH_STANDALONE=1
  exec /data/adb/ksu/bin/busybox sh "$0" "$@"
fi

"$MODPATH/bin/duckctl.sh" tricky-store auto-apply --json >/dev/null 2>&1 || true
