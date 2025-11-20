import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  fullyParallel: false,
  workers: 1,
  retries: 0,
  reporter: [["list"]],
  use: {
    baseURL: "http://localhost:3030",
    trace: "on-first-retry",
  },
  projects: [
    {
      name: "chromium",
      use: { },
    },
  ],
  webServer: {
    command: "cargo run",
    url: "http://localhost:3030",
    reuseExistingServer: false,
    timeout: 30000,
  },
});
