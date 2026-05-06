import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { spawn, type ChildProcess } from "node:child_process";
import fs from "node:fs/promises";
import { captureFrame } from "./capture";
import { diffPng } from "./diff";
import { readManifest, goldenPath, actualPath, diffPath } from "./manifest";

const BASE_URL = process.env.SHADER_CHECK_BASE_URL ?? "http://localhost:3000";
const SKIP_DEV_BOOT = process.env.SHADER_CHECK_SKIP_DEV === "1";
const READY_TIMEOUT_MS = 90_000;

let devProc: ChildProcess | null = null;

async function fileExists(p: string): Promise<boolean> {
  try {
    await fs.access(p);
    return true;
  } catch {
    return false;
  }
}

async function waitForServerReady(timeoutMs: number): Promise<boolean> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      const res = await fetch(BASE_URL);
      if (res.ok) return true;
    } catch {
      // Connection refused; server still booting.
    }
    await new Promise<void>((r) => setTimeout(r, 1000));
  }
  return false;
}

beforeAll(async () => {
  if (SKIP_DEV_BOOT) return;
  // shell:true so the bun-script form (which involves cd into physics-engine
  // for the wasm build step) inherits the shell that resolves it.
  devProc = spawn("bun", ["run", "dev"], {
    stdio: "ignore",
    shell: true,
    detached: false,
  });
  const ready = await waitForServerReady(READY_TIMEOUT_MS);
  if (!ready) {
    throw new Error(
      `Dev server did not become ready within ${READY_TIMEOUT_MS / 1000}s`,
    );
  }
}, READY_TIMEOUT_MS + 30_000);

afterAll(() => {
  if (devProc?.pid && !devProc.killed) {
    try {
      devProc.kill("SIGTERM");
    } catch {
      // Process already exited.
    }
  }
});

describe("shader-visual regression", () => {
  it("manifest loads and is well-formed", async () => {
    const m = await readManifest();
    expect(m.frames.length).toBeGreaterThan(0);
    expect(m.viewport.width).toBeGreaterThan(0);
    expect(m.viewport.height).toBeGreaterThan(0);
  });
});

describe.concurrent("shader-visual frames", async () => {
  const manifest = await readManifest();

  for (const entry of manifest.frames) {
    it(`shader-visual ${entry.name}`, async () => {
      const golden = goldenPath(entry.name);
      const goldenExists = await fileExists(golden);
      if (!goldenExists) {
        // Goldens are committed once captured. Until then, the test skips
        // with a clear instruction so the suite still runs green.
        console.warn(
          `Golden missing for ${entry.name}. Run \`bun run shader:update-goldens --confirm\` to capture, then commit ${golden}.`,
        );
        return;
      }

      const actual = actualPath(entry.name);
      const diff = diffPath(entry.name);

      await captureFrame({
        config: entry.config,
        viewport: manifest.viewport,
        outPath: actual,
        baseUrl: BASE_URL,
        stabilizationFrames: manifest.stabilization_frames,
      });

      const result = await diffPng(golden, actual, diff);

      expect(
        result.ssim,
        `${entry.name} SSIM ${result.ssim} below threshold ${entry.ssim_threshold}`,
      ).toBeGreaterThanOrEqual(entry.ssim_threshold);
      expect(
        result.maxPixelDelta,
        `${entry.name} max pixel delta ${result.maxPixelDelta} above ${entry.max_pixel_delta}`,
      ).toBeLessThanOrEqual(entry.max_pixel_delta);
    }, 120_000);
  }
});
