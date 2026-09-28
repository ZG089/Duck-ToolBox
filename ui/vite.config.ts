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
    // Few large chunks on purpose (see codeSplitting below); they are read from local storage.
    chunkSizeWarningLimit: 1024,
    rolldownOptions: {
      output: {
        // The KernelSU manager reads every requested file through a root shell
        // (webui/SuFilePathHandler.java), so a few larger files load much faster than many
        // small ones: libraries, the app and the messages each get one chunk. The i18n
        // plugin turns each locale file into an anonymous virtual module, so languages cannot
        // be told apart here.
        codeSplitting: {
          groups: [
            {
              name: "locales",
              test: /^virtual:intlify-i18n|[\\/]locales[\\/][\w-]+\.json$/,
              priority: 3,
            },
            { name: "vendor", test: /node_modules/, priority: 2 },
            { name: "app", test: /[\\/]src[\\/]/, priority: 1 },
          ],
        },
      },
    },
  },
})
