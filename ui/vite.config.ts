import { fileURLToPath, URL } from "node:url"

import VueI18nPlugin from "@intlify/unplugin-vue-i18n/vite"
import tailwindcss from "@tailwindcss/vite"
import vue from "@vitejs/plugin-vue"
import { defineConfig } from "vite"

export default defineConfig({
  // KernelSU serves webroot/ through WebViewAssetLoader, so assets must resolve relative
  // to index.html.
  base: "./",
  plugins: [
    vue(),
    tailwindcss(),
    VueI18nPlugin({
      include: [fileURLToPath(new URL("./src/**/locales/*.json", import.meta.url))],
      strictMessage: false,
      escapeHtml: false,
      // Keep the message compiler: translations downloaded at runtime arrive as strings.
      runtimeOnly: false,
    }),
  ],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
  build: {
    outDir: "../module/webroot",
    emptyOutDir: true,
    // Tailwind CSS v4 needs Chrome 111+; index.html blocks older WebViews with a notice.
    target: "chrome111",
  },
})
