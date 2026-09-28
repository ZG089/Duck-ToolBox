import { defineConfig } from "@playwright/test"

// Real-device run: see scripts/device-test.sh. One device, so tests run in order.
export default defineConfig({
  testDir: "e2e-device",
  testMatch: "**/*.device.ts",
  outputDir: "test-results/device",
  fullyParallel: false,
  workers: 1,
  timeout: 120_000,
  expect: { timeout: 15_000 },
  reporter: "list",
})
