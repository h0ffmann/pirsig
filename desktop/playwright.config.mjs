// The UI end to end: ztk's API and UI served from the checkout, on a fresh,
// throwaway brain (target/e2e-brain.sqlite), driven in WebKit, the engine of
// the macOS webview Tauri uses. `just desktop-e2e`.
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig, devices } from "@playwright/test";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");

const PORT = 5181;
const DB = "target/e2e-brain.sqlite";

export default defineConfig({
  testDir: "e2e",
  fullyParallel: false,
  workers: 1,
  timeout: 60_000,
  expect: { timeout: 10_000 },
  reporter: process.env.CI ? "github" : "list",
  use: {
    baseURL: `http://127.0.0.1:${PORT}`,
    trace: "retain-on-failure",
  },
  projects: [{ name: "webkit", use: { ...devices["Desktop Safari"] } }],
  webServer: {
    cwd: ROOT,
    stdout: "pipe",
    stderr: "pipe",
    command: `rm -f ${DB} ${DB}-wal ${DB}-shm && cargo run -q --bin zettelkasten-cli -- start`,
    env: { DATABASE_URL: `sqlite://${DB}?mode=rwc`, PORT: String(PORT), BINDING: "127.0.0.1", LOG_LEVEL: "warn" },
    url: `http://127.0.0.1:${PORT}/api/readiness`,
    timeout: 600_000,
    reuseExistingServer: false,
  },
});
