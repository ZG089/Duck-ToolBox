#!/usr/bin/env bash
# Prepares a disposable userdebug emulator (or rooted-adb device) for the real-device WebUI
# tests, then runs them:
#   1. KernelSU late-loaded with `ksud late-load` (KernelSU's LKM "jailbreak" mode), so no
#      boot image is patched.
#   2. The Duck ToolBox module zip and the Tricky Store, TEESimulator and OhMyKeymint
#      releases installed with ksud.
#   3. WebView debugging and a deep-link token set in the manager's preferences, which
#      the manager reads in ui/webui/WebViewHelper.kt and navigation3/IntentDispatcher.kt.
#   4. For each keystore module: only it enabled, a reboot and one late-load (late-load
#      is meant to run once per boot), then the tests tagged with its module id.
#
#   ANDROID_SERIAL=emulator-5554 scripts/device-test.sh dist/duck-toolbox.zip
#
# Needs adb, gh (to download releases) and a module zip built by CI or package-module.ps1.
set -euo pipefail

MODULE_ZIP="${1:?usage: device-test.sh <duck-toolbox module zip>}"
: "${ANDROID_SERIAL:?set ANDROID_SERIAL to the target device}"
KSU_TAG="${KSU_TAG:-v3.3.0}"
TS_TAG="${TS_TAG:-1.4.1}"
TEES_TAG="${TEES_TAG:-v4.0}"
OMK_TAG="${OMK_TAG:-v1.2.0-67dc5e7}"
MANAGER="me.weishu.kernelsu"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

device_shell() { adb shell "$@"; }

# Every `ksud install` deletes and rewrites /data/adb/ksud, and both late-load and each
# start of the manager (ui/MainActivity.kt) run one. A call in that window finds the file
# missing (127) or still being written (126, "Text file busy").
ksud() {
  local attempt status
  for attempt in $(seq 1 15); do
    status=0
    device_shell /data/adb/ksud "$@" || status=$?
    case "$status" in
      126 | 127) sleep 2 ;;
      *) return "$status" ;;
    esac
  done
  return "$status"
}

# `ksud late-load` daemonizes and returns at once. The daemon reinstalls ksud, applies
# pending module updates, runs the boot stages and restarts the manager
# (userspace/ksud/src/late_load.rs).
wait_for_late_load() {
  local tries
  for tries in $(seq 1 90); do
    # The brackets keep the pattern from matching the shell that runs it.
    device_shell "ps -A -o ARGS | grep -q '[k]sud late-load'" || return 0
    sleep 2
  done
  echo "late-load still running after three minutes" >&2
  return 1
}

# adbd restarts to become root and may drop the first request right after boot.
become_root() {
  local attempt
  for attempt in 1 2 3 4 5 6 7 8 9 10; do
    adb root >/dev/null 2>&1 || true
    adb wait-for-device
    [ "$(device_shell id -u | tr -d '\r')" = 0 ] && return 0
    echo "  adbd is not root yet (attempt $attempt)" >&2
    sleep 3
  done
  echo "adbd did not become root" >&2
  return 1
}

# Downloads release assets matching a glob and checks them against GitHub's SHA-256 digests.
download() {
  local repo="$1" tag="$2" glob="$3"
  gh release download "$tag" -R "$repo" -p "$glob" -D "$WORK" --clobber
  gh api "repos/$repo/releases/tags/$tag" \
    --jq '.assets[] | (.digest | sub("sha256:"; "")) + "  " + .name' >"$WORK/sums"
  (
    cd "$WORK"
    for file in $glob; do
      grep -F "  $file" sums | shasum -a 256 -c -
    done
  )
}

echo "- Restarting adbd as root"
become_root

echo "- KernelSU $KSU_TAG"
download tiann/KernelSU "$KSU_TAG" 'KernelSU_*-release.apk'
download tiann/KernelSU "$KSU_TAG" 'ksud-aarch64-linux-android'
adb install -r "$WORK"/KernelSU_*-release.apk >/dev/null
adb push "$WORK/ksud-aarch64-linux-android" /data/local/tmp/ksud >/dev/null
device_shell chmod 755 /data/local/tmp/ksud
device_shell /data/local/tmp/ksud late-load >/dev/null 2>&1 || true
wait_for_late_load
ksud -V

echo "- Installing modules"
adb push "$MODULE_ZIP" /data/local/tmp/duck-toolbox.zip >/dev/null
ksud module install /data/local/tmp/duck-toolbox.zip | grep -v inflating
install_release() {
  local repo="$1" tag="$2" glob="$3"
  download "$repo" "$tag" "$glob"
  for zip in "$WORK"/$glob; do
    adb push "$zip" /data/local/tmp/keystore-module.zip >/dev/null
    ksud module install /data/local/tmp/keystore-module.zip | grep -v inflating
  done
}
install_release 5ec1cff/TrickyStore "$TS_TAG" 'Tricky-Store-*-release.zip'
install_release JingMatrix/TEESimulator "$TEES_TAG" 'TEESimulator-*-Release.zip'
install_release qwq233/OhMyKeymint "$OMK_TAG" 'OhMyKeymint-release-arm64-v8a-*.zip'

echo "- Manager preferences"
token="$(openssl rand -hex 32)"
prefs="/data/data/$MANAGER/shared_prefs"
owner="$(device_shell stat -c %u:%g "/data/data/$MANAGER" | tr -d '\r')"
device_shell am force-stop "$MANAGER"
device_shell "mkdir -p $prefs && cat > $prefs/settings.xml" <<XML
<?xml version='1.0' encoding='utf-8' standalone='yes' ?>
<map>
    <boolean name="enable_web_debugging" value="true" />
    <string name="intent_token">$token</string>
</map>
XML
device_shell "chown $owner $prefs $prefs/settings.xml && chmod 660 $prefs/settings.xml && restorecon -R $prefs" >/dev/null 2>&1

# Waits until the 1-minute load average drops below 4, or five minutes pass: right after
# boot an emulator is busy (dexopt, the keystore daemons) and ANRs its own UI.
settle() {
  local tries load
  for tries in $(seq 1 60); do
    load="$(device_shell cut -d. -f1 /proc/loadavg | tr -d '\r')"
    [ "${load:-99}" -lt 4 ] && return 0
    sleep 5
  done
  echo "  device still busy after $tries checks; testing anyway" >&2
}

# Reboots and late-loads KernelSU once: pending installs and enable/disable flags apply,
# and every boot stage of every enabled module runs, as on a normal boot.
reboot_and_load() {
  adb reboot
  # Without this, the old boot's sys.boot_completed can be read before it goes down.
  adb wait-for-disconnect
  adb wait-for-device
  until [ "$(device_shell getprop sys.boot_completed | tr -d '\r')" = 1 ]; do sleep 2; done
  become_root
  ksud late-load >/dev/null 2>&1 || true
  wait_for_late_load
  # An "isn't responding" dialog would cover the WebUI on a slow emulator.
  device_shell settings put global hide_error_dialogs 1
  settle
}

# Only one keystore module may hook keystore at a time. Flags must be set on applied
# modules: a pending update would replace the directory, flag and all.
activate() {
  local wanted="$1" module
  for module in tricky_store teesim oh_my_keymint; do
    if [ "$module" = "$wanted" ]; then
      ksud module enable "$module" >/dev/null
    else
      ksud module disable "$module" >/dev/null
    fi
  done
  reboot_and_load
  device_shell "[ ! -e /data/adb/modules/$wanted/disable ]" ||
    { echo "$wanted is still disabled" >&2; return 1; }
}

echo "- Applying the installs"
reboot_and_load

cd "$(dirname "$0")/../ui"
for module in tricky_store teesim oh_my_keymint; do
  echo "- Testing with $module"
  activate "$module"
  DUCK_KSU_TOKEN="$token" pnpm test:device --grep "@$module"
done
