#!/system/bin/sh
# Action button entry point. KernelSU and APatch open the WebUI directly; on Magisk (which
# has no built-in WebUI) this opens the module in KSUWebUIStandalone if it is installed.

MODULE_ID="duck-toolbox"

if pm path io.github.a13e300.ksuwebui >/dev/null 2>&1; then
  am start -n io.github.a13e300.ksuwebui/.WebUIActivity -e id "$MODULE_ID" >/dev/null 2>&1
elif pm path com.dergoogler.mmrl.wx >/dev/null 2>&1; then
  am start -n com.dergoogler.mmrl.wx/.ui.activity.webui.WebUIActivity -e MOD_ID "$MODULE_ID" >/dev/null 2>&1
else
  echo "- No standalone WebUI app found."
  echo "- Install KSUWebUIStandalone or open Duck ToolBox from your root manager."
fi
