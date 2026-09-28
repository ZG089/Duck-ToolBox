import { defineConfig } from "@playwright/test"

// Real-device run: see scripts/device-test.sh. One device, so tests run in order.
export default defineConfig({
  testDir: "e2e-device",
  testMatch: "**/*.device.ts",
  outputDir: "test-results/device",
  fullyParallel: false,
  workers: 1,
  // Real devices are slow right after boot and vary a lot.
  timeout: 300_000,
  expect: { timeout: 45_000 },
  reporter: "list",
})
