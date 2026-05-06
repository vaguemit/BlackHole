import { defineConfig } from "vitest/config";
import path from "path";

export default defineConfig({
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: ["./vitest.setup.ts"],
    include: [
      "src/**/*.test.ts",
      "src/__tests__/**/*.test.ts",
      "scripts/__tests__/**/*.test.ts",
    ],
    // Visual regression has its own config (vitest.shader.config.ts) so
    // it never runs in the default suite — it boots a dev server and is
    // expensive. shader:check invokes it explicitly.
    exclude: [
      "**/node_modules/**",
      "**/dist/**",
      "**/.next/**",
      "tests/visual-regression/**",
    ],
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
      "blackhole-physics": path.resolve(
        __dirname,
        "./public/wasm/blackhole_physics.js",
      ),
    },
  },
});
