import fs from "node:fs/promises";
import { PNG } from "pngjs";
import pixelmatch from "pixelmatch";
import { ssim } from "ssim.js";

export interface DiffResult {
  ssim: number;
  maxPixelDelta: number;
  diffPixels: number;
  totalPixels: number;
  width: number;
  height: number;
}

export async function diffPng(
  expectedPath: string,
  actualPath: string,
  diffOutPath: string | null,
): Promise<DiffResult> {
  const [expectedBuf, actualBuf] = await Promise.all([
    fs.readFile(expectedPath),
    fs.readFile(actualPath),
  ]);
  const expected = PNG.sync.read(expectedBuf);
  const actual = PNG.sync.read(actualBuf);

  if (expected.width !== actual.width || expected.height !== actual.height) {
    throw new Error(
      `Dimension mismatch: expected ${expected.width}x${expected.height}, got ${actual.width}x${actual.height}`,
    );
  }

  const diff = new PNG({ width: expected.width, height: expected.height });
  const diffPixels = pixelmatch(
    expected.data,
    actual.data,
    diff.data,
    expected.width,
    expected.height,
    { threshold: 0.1, includeAA: false },
  );

  if (diffOutPath) {
    await fs.mkdir(diffOutPath.replace(/[^/\\]+$/, ""), { recursive: true });
    await fs.writeFile(diffOutPath, PNG.sync.write(diff));
  }

  const { mssim } = ssim(
    {
      data: new Uint8ClampedArray(expected.data),
      width: expected.width,
      height: expected.height,
    },
    {
      data: new Uint8ClampedArray(actual.data),
      width: actual.width,
      height: actual.height,
    },
  );

  let maxPixelDelta = 0;
  for (let i = 0; i < expected.data.length; i += 4) {
    const dr = Math.abs((expected.data[i] ?? 0) - (actual.data[i] ?? 0));
    const dg = Math.abs(
      (expected.data[i + 1] ?? 0) - (actual.data[i + 1] ?? 0),
    );
    const db = Math.abs(
      (expected.data[i + 2] ?? 0) - (actual.data[i + 2] ?? 0),
    );
    if (dr > maxPixelDelta) maxPixelDelta = dr;
    if (dg > maxPixelDelta) maxPixelDelta = dg;
    if (db > maxPixelDelta) maxPixelDelta = db;
  }

  return {
    ssim: mssim,
    maxPixelDelta,
    diffPixels,
    totalPixels: expected.width * expected.height,
    width: expected.width,
    height: expected.height,
  };
}
