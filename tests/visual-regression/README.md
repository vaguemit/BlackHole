# Shader visual regression

Headless capture + SSIM/pixelmatch diff against committed PNG goldens.
Runs out-of-band from the default vitest suite because it boots the
Next.js dev server and drives Playwright per frame; expensive on CI,
fine on a laptop.

## Run the suite

```
bun run shader:check
```

The suite boots a dev server at `http://localhost:3000`, walks every
entry in `tests/golden/manifest.json`, captures the canvas at the
manifest viewport, and asserts SSIM + max-pixel-delta against the
committed PNG. If a golden is missing the entry skip-warns rather
than fails, so the suite stays green until goldens land.

If a dev server is already running, point the suite at it and skip
the boot step:

```
SHADER_CHECK_BASE_URL=http://localhost:3003 SHADER_CHECK_SKIP_DEV=1 bun run shader:check
```

## Capture / refresh the goldens

```
bun run shader:update-goldens --confirm
```

Walks the manifest, captures each frame, and stamps `captured_at` +
`captured_commit` on the manifest entry. Always review the resulting
PNGs visually before committing; a wrong golden silently becomes the
new "truth".

## Manifest schema

`tests/golden/manifest.json`:

- `version`: schema revision.
- `viewport`: capture dimensions, applied to every frame.
- `stabilization_frames`: number of `requestAnimationFrame` ticks to
  wait after canvas init before capture. TAA needs a handful of frames
  to converge.
- `frames[]`: each entry has a `name`, a `config` (URL-hash params:
  `mass`, `spin`, `zoom`, `diskTemp`, `diskDensity`, `diskSize`,
  `lensing`, `autoSpin`), a per-frame `ssim_threshold` and
  `max_pixel_delta`, plus stamps for the last capture.

The threshold ladder (0.998 / 0.997 / 0.996) reflects increasing
visual variance with spin: face-on Schwarzschild is deterministic,
near-extremal edge-on has the most temporal noise from the disk.
