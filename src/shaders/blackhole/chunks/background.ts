export const BACKGROUND_CHUNK = `
  // === GARGANTUA AESTHETIC: Pure void — zero stars, zero nebula ===
  // PRD §2.1: "No stars. Pure void. Gargantua lives alone in the dark."
  // Background clears to vec3(0.0, 0.0, 0.0) — actual zero, not near-black.
  vec3 starfield(vec3 dir) {
    // Intentionally empty — function kept as stub so call-sites compile.
    return vec3(0.0);
  }
`;
