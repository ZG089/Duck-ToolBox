#!/system/bin/sh
# Run by a hot-install capable metamodule right after installation (see customize.sh): does
# what a reboot would, so the module works immediately.
MODPATH=${0%/*}

sh "$MODPATH/post-fs-data.sh" >/dev/null 2>&1
sh "$MODPATH/service.sh" >/dev/null 2>&1
sh "$MODPATH/boot-completed.sh" >/dev/null 2>&1 &
