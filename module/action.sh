#!/system/bin/sh
# Action button, kept on Magisk only (customize.sh removes it elsewhere): Magisk has no
# WebUI, so this opens it in a standalone WebUI app and installs KSUWebUIStandalone when
# none is present, as Tricky Addon does. Linked into a keystore module, it opens that
# module's WebUI, which is ours and lands on the Tricky Store manager.

MODULE_DIR="$(cd "$(dirname "$0")" && pwd)"
ID="$(basename "$MODULE_DIR")"
[ -f "$MODULE_DIR/module.prop" ] || ID="duck-toolbox"

KSUWEBUI="io.github.a13e300.ksuwebui"
WEBUIX="com.dergoogler.mmrl.wx"
RELEASES="KOWX712/KsuWebUIStandalone"

open_releases() {
  echo "$1"
  echo "- Opening the KSUWebUIStandalone releases page..."
  sleep 2
  am start -a android.intent.action.VIEW -d "https://github.com/$RELEASES/releases" >/dev/null 2>&1
  exit 1
}

fetch() {
  if command -v curl >/dev/null 2>&1; then
    curl --connect-timeout 10 -fsSL "$1"
  else
    wget -T 10 -qO- "$1"
  fi
}

install_ksuwebui() {
  echo "- Downloading KSUWebUIStandalone..."
  url="$(fetch "https://api.github.com/repos/$RELEASES/releases/latest" |
    grep -o '"browser_download_url": *"[^"]*\.apk"' | head -n 1 | cut -d '"' -f 4)"
  [ -n "$url" ] || open_releases "! Could not find the latest release."
  apk="/data/local/tmp/ksuwebui-$$.apk"
  fetch "$url" >"$apk" || {
    rm -f "$apk"
    open_releases "! Download failed."
  }
  echo "- Installing..."
  if ! pm install -r "$apk" >/dev/null 2>&1; then
    rm -f "$apk"
    open_releases "! Installation failed."
  fi
  rm -f "$apk"
}

if pm path "$KSUWEBUI" >/dev/null 2>&1; then
  echo "- Opening $ID in KSUWebUIStandalone..."
  am start -n "$KSUWEBUI/.WebUIActivity" -e id "$ID" >/dev/null 2>&1
elif pm path "$WEBUIX" >/dev/null 2>&1; then
  echo "- Opening $ID in WebUI X..."
  am start -n "$WEBUIX/.ui.activity.webui.WebUIActivity" -e MOD_ID "$ID" >/dev/null 2>&1
else
  echo "! No WebUI app found."
  install_ksuwebui
  echo "- Opening $ID in KSUWebUIStandalone..."
  am start -n "$KSUWEBUI/.WebUIActivity" -e id "$ID" >/dev/null 2>&1
fi
