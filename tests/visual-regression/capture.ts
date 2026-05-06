import { chromium } from "@playwright/test";
import fs from "node:fs/promises";
import path from "node:path";

export interface FrameConfig {
  mass: number;
  spin: number;
  zoom: number;
  diskTemp?: number;
  diskDensity?: number;
  diskSize?: number;
  lensing?: number;
  autoSpin?: number;
}

export interface CaptureOptions {
  config: FrameConfig;
  viewport: { width: number; height: number };
  outPath: string;
  baseUrl: string;
  stabilizationFrames: number;
}

function encodeHash(config: FrameConfig): string {
  const parts = Object.entries(config).map(([k, v]) => `${k}=${v}`);
  return parts.length > 0 ? `#${parts.join("&")}` : "";
}

export async function captureFrame(opts: CaptureOptions): Promise<void> {
  const browser = await chromium.launch({
    headless: true,
    args: [
      "--enable-unsafe-webgpu",
      "--use-gl=angle",
      "--use-angle=vulkan",
      "--ignore-gpu-blocklist",
    ],
  });
  try {
    const ctx = await browser.newContext({
      viewport: opts.viewport,
      deviceScaleFactor: 1,
    });
    const page = await ctx.newPage();

    const targetUrl = `${opts.baseUrl}/${encodeHash(opts.config)}`;
    await page.goto(targetUrl, { waitUntil: "domcontentloaded" });

    // Wait for the canvas to mount and the renderer to start producing frames.
    await page.waitForSelector("canvas", {
      state: "attached",
      timeout: 30_000,
    });

    // The renderer is asynchronous (WASM load + WebGL init). Wait for the
    // canvas to have a non-zero device-pixel size, which indicates init
    // completed at least one frame.
    await page.waitForFunction(
      () => {
        const c = document.querySelector("canvas") as HTMLCanvasElement | null;
        return !!c && c.width > 0 && c.height > 0;
      },
      { timeout: 30_000 },
    );

    // TAA accumulates over a handful of frames; let it settle before capture.
    await page.evaluate(async (n: number) => {
      for (let i = 0; i < n; i++) {
        await new Promise<void>((r) => requestAnimationFrame(() => r()));
      }
    }, opts.stabilizationFrames);

    await fs.mkdir(path.dirname(opts.outPath), { recursive: true });
    const buf = await page.screenshot({
      type: "png",
      omitBackground: false,
      clip: { x: 0, y: 0, ...opts.viewport },
    });
    await fs.writeFile(opts.outPath, buf);
  } finally {
    await browser.close();
  }
}
