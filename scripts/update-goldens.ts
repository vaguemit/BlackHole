#!/usr/bin/env bun
/**
 * One-shot golden re-capture. Walks tests/golden/manifest.json, drives the
 * dev server through Playwright for each frame entry, writes the PNG, and
 * stamps captured_at + captured_commit on the manifest entry.
 *
 * Requires --confirm because overwriting goldens silently shifts the
 * regression baseline. Reviewers should diff the resulting PNGs visually
 * before committing.
 */

import { spawn, type ChildProcess, execSync } from "node:child_process";
import process from "node:process";
import { captureFrame } from "../tests/visual-regression/capture";
import {
  readManifest,
  writeManifest,
  goldenPath,
  type Manifest,
} from "../tests/visual-regression/manifest";

const CONFIRM_FLAG = "--confirm";
const BASE_URL = process.env.SHADER_CHECK_BASE_URL ?? "http://localhost:3000";
const READY_TIMEOUT_MS = 90_000;

async function waitForServerReady(timeoutMs: number): Promise<boolean> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      const res = await fetch(BASE_URL);
      if (res.ok) return true;
    } catch {
      // Connection refused; still booting.
    }
    await new Promise<void>((r) => setTimeout(r, 1000));
  }
  return false;
}

function gitHeadSha(): string {
  try {
    return execSync("git rev-parse HEAD", { encoding: "utf8" }).trim();
  } catch {
    return "unknown";
  }
}

async function recapture(manifest: Manifest): Promise<void> {
  const sha = gitHeadSha();
  const now = new Date().toISOString();
  for (const entry of manifest.frames) {
    process.stdout.write(`Capturing ${entry.name}... `);
    await captureFrame({
      config: entry.config,
      viewport: manifest.viewport,
      outPath: goldenPath(entry.name),
      baseUrl: BASE_URL,
      stabilizationFrames: manifest.stabilization_frames,
    });
    entry.captured_at = now;
    entry.captured_commit = sha;
    process.stdout.write("done\n");
  }
}

async function main(): Promise<void> {
  if (!process.argv.includes(CONFIRM_FLAG)) {
    process.stderr.write(
      `update-goldens: this overwrites every PNG in tests/golden/.\n` +
        `Pass ${CONFIRM_FLAG} to proceed; review the diffs before committing.\n`,
    );
    process.exit(1);
  }

  const manifest = await readManifest();

  // Boot the dev server unless one is already up at BASE_URL.
  let dev: ChildProcess | null = null;
  const alreadyUp = await waitForServerReady(2000);
  if (!alreadyUp) {
    process.stdout.write(`Starting dev server at ${BASE_URL}...\n`);
    dev = spawn("bun", ["run", "dev"], {
      stdio: "inherit",
      shell: true,
      detached: false,
    });
    const ready = await waitForServerReady(READY_TIMEOUT_MS);
    if (!ready) {
      if (dev?.pid) dev.kill("SIGTERM");
      throw new Error(
        `Dev server did not become ready within ${READY_TIMEOUT_MS / 1000}s`,
      );
    }
  } else {
    process.stdout.write(`Reusing dev server already up at ${BASE_URL}\n`);
  }

  try {
    await recapture(manifest);
    await writeManifest(manifest);
    process.stdout.write(
      `Goldens updated. git diff tests/golden/ before committing.\n`,
    );
  } finally {
    if (dev?.pid && !dev.killed) {
      dev.kill("SIGTERM");
    }
  }
}

main().catch((err: unknown) => {
  const msg = err instanceof Error ? err.message : String(err);
  process.stderr.write(`update-goldens failed: ${msg}\n`);
  process.exit(1);
});
