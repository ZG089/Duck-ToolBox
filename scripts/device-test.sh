#!/usr/bin/env bash
# Prepares a disposable userdebug emulator (or rooted-adb device) for the real-device WebUI
# tests, then runs them:
#   1. KernelSU late-loaded with `ksud late-load` (KernelSU's LKM "jailbreak" mode), so no
#      boot image is patched.
#   2. The Duck ToolBox module zip and the Tricky Store release installed with ksud.
#   3. WebView debugging and a deep-link token set in the manager's preferences, which
#      the manager reads in ui/webui/WebViewHelper.kt and navigation3/IntentDispatcher.kt.
#
#   ANDROID_SERIAL=emulator-5554 scripts/device-test.sh dist/duck-toolbox.zip
#
# Needs adb, gh (to download releases) and a module zip built by CI or package-module.ps1.
set -euo pipefail

MODULE_ZIP="${1:?usage: device-test.sh <duck-toolbox module zip>}"
: "${ANDROID_SERIAL:?set ANDROID_SERIAL to the target device}"
KSU_TAG="${KSU_TAG:-v3.3.0}"
TS_TAG="${TS_TAG:-1.4.1}"
MANAGER="me.weishu.kernelsu"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

device_shell() { adb shell "$@"; }

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
adb root >/dev/null
adb wait-for-device
[ "$(device_shell id -u | tr -d '\r')" = 0 ] || { echo "adbd is not root" >&2; exit 1; }

echo "- KernelSU $KSU_TAG"
download tiann/KernelSU "$KSU_TAG" 'KernelSU_*-release.apk'
download tiann/KernelSU "$KSU_TAG" 'ksud-aarch64-linux-android'
adb install -r "$WORK"/KernelSU_*-release.apk >/dev/null
adb push "$WORK/ksud-aarch64-linux-android" /data/local/tmp/ksud >/dev/null
device_shell chmod 755 /data/local/tmp/ksud
device_shell /data/local/tmp/ksud late-load >/dev/null 2>&1 || true
sleep 10
device_shell /data/adb/ksud -V

echo "- Installing modules"
adb push "$MODULE_ZIP" /data/local/tmp/duck-toolbox.zip >/dev/null
device_shell /data/adb/ksud module install /data/local/tmp/duck-toolbox.zip | grep -v inflating
download 5ec1cff/TrickyStore "$TS_TAG" 'Tricky-Store-*-release.zip'
adb push "$WORK"/Tricky-Store-*-release.zip /data/local/tmp/tricky-store.zip >/dev/null
device_shell /data/adb/ksud module install /data/local/tmp/tricky-store.zip | grep -v inflating
# Late-load again: activates the new modules and runs every boot stage, as a reboot would.
device_shell /data/adb/ksud late-load >/dev/null 2>&1 || true
sleep 15

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

echo "- Running the device tests"
cd "$(dirname "$0")/../ui"
DUCK_KSU_TOKEN="$token" pnpm test:device
