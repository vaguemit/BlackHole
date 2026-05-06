import fs from "node:fs/promises";
import path from "node:path";
import type { FrameConfig } from "./capture";

export interface ManifestEntry {
  name: string;
  config: FrameConfig;
  ssim_threshold: number;
  max_pixel_delta: number;
  captured_at: string | null;
  captured_commit: string | null;
}

export interface Manifest {
  version: number;
  viewport: { width: number; height: number };
  stabilization_frames: number;
  frames: ManifestEntry[];
}

export const MANIFEST_PATH = path.resolve(
  process.cwd(),
  "tests/golden/manifest.json",
);

export async function readManifest(): Promise<Manifest> {
  const raw = await fs.readFile(MANIFEST_PATH, "utf8");
  return JSON.parse(raw) as Manifest;
}

export async function writeManifest(manifest: Manifest): Promise<void> {
  const serialized = JSON.stringify(manifest, null, 2) + "\n";
  await fs.writeFile(MANIFEST_PATH, serialized);
}

export function goldenPath(name: string): string {
  return path.resolve(process.cwd(), `tests/golden/${name}.png`);
}

export function actualPath(name: string): string {
  return path.resolve(process.cwd(), `tests/golden/diff/${name}-actual.png`);
}

export function diffPath(name: string): string {
  return path.resolve(process.cwd(), `tests/golden/diff/${name}-diff.png`);
}
