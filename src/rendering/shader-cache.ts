interface CachedModule {
  module: GPUShaderModule;
  compiledAt: number;
}

const cache = new Map<string, CachedModule>();

export type HudWarn = (message: string) => void;

// Compile a WGSL module with a last-good fallback.
//
// On success the module is cached under `pipelineId`; subsequent compile
// failures (after a hot-reload, parameter sweep, or constant override that
// produced invalid WGSL) reuse the previous good module instead of leaving
// the user with an unrenderable canvas. The error surfaces through the
// supplied `hudWarn` so the operator sees what went wrong.
export async function compileWithFallback(
  device: GPUDevice,
  pipelineId: string,
  source: string,
  hudWarn: HudWarn,
): Promise<GPUShaderModule> {
  const candidate = device.createShaderModule({ code: source });
  const info = await candidate.getCompilationInfo();
  const errors = info.messages.filter((m) => m.type === "error");
  if (errors.length === 0) {
    cache.set(pipelineId, { module: candidate, compiledAt: Date.now() });
    return candidate;
  }

  const detail = errors
    .map((e) => `[${e.lineNum}:${e.linePos}] ${e.message}`)
    .join("\n");
  const lastGood = cache.get(pipelineId);
  if (lastGood) {
    hudWarn(
      `Shader '${pipelineId}' compile failed; using last-good from ${new Date(
        lastGood.compiledAt,
      ).toLocaleTimeString()}.\n${detail}`,
    );
    return lastGood.module;
  }
  hudWarn(`Shader '${pipelineId}' compile failed; no fallback.\n${detail}`);
  throw new Error(`Shader '${pipelineId}' compile failed: ${detail}`);
}
