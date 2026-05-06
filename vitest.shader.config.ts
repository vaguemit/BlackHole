import { defineConfig } from "vitest/config";
import path from "path";

// Visual-regression suite. Distinct from the default vitest run so that
// shader:check stays opt-in (it boots a Next dev server, drives Playwright,
// and can take 60-120 s per frame). The default vitest run skips this
// directory entirely.
export default defineConfig({
  test: {
    globals: true,
    environment: "node",
    include: ["tests/visual-regression/**/*.test.ts"],
    testTimeout: 180_000,
    hookTimeout: 180_000,
    pool: "forks",
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
});
