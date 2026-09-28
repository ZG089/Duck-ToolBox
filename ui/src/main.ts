import "./styles/main.css"

import { createApp } from "vue"

import App from "./App.vue"
import { installApp } from "./app"

declare global {
  interface Window {
    /** Set by the WebView version check in index.html. */
    __duckBlocked?: boolean
  }
}

async function start() {
  if (window.__duckBlocked) return
  if (import.meta.env.DEV || import.meta.env.MODE === "e2e") {
    // A fake `window.ksu` so the WebUI runs in a desktop browser and in E2E tests.
    const { installMockHost } = await import("./dev/mock-host")
    installMockHost()
  }

  const app = createApp(App)
  await installApp(app)
  app.mount("#app")
}

void start()
