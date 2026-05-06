/**
 * Oblivion Post-Process Shader — GLSL ES 3.0
 * PRD §6
 *
 * Full-screen pass applied last in the pipeline (after TAA + Bloom).
 * Active whenever OblivionPhase !== 'IDLE'.
 *
 * Implements:
 *   - Spaghettification UV distortion
 *   - Chromatic aberration (separate R/G/B channels)
 *   - Gravitational redshift tint
 *   - Vignette
 *   - Singularity white flash
 */

export const OBLIVION_VERT_SOURCE = `#version 300 es
in vec2 a_position;
out vec2 v_uv;
void main() {
  v_uv = a_position * 0.5 + 0.5;
  gl_Position = vec4(a_position, 0.0, 1.0);
}
`;

export const OBLIVION_FRAG_SOURCE = `#version 300 es
precision highp float;

uniform sampler2D u_scene_tex;
uniform float u_spaghetti_strength;
uniform float u_redshift_amount;
uniform float u_vignette_radius;
uniform float u_flash_intensity;
uniform float u_chromatic_spread;  // 0.0–8.0 pixels
uniform float u_void_fade;

in vec2 v_uv;
out vec4 fragColor;

// Chromatic aberration: separate R/G/B along radial direction
vec3 sampleChromatic(sampler2D tex, vec2 uv, float spread) {
  vec2 dir = normalize(uv - vec2(0.5));
  vec2 res = vec2(textureSize(tex, 0));
  vec2 offset = dir * spread / res;
  float r = texture(tex, uv + offset).r;
  float g = texture(tex, uv).g;
  float b = texture(tex, uv - offset).b;
  return vec3(r, g, b);
}

// Radial vignette — returns multiplier 1.0 (center) → 0.0 (edge)
float vignette(vec2 uv, float radius) {
  float dist = distance(uv, vec2(0.5));
  return smoothstep(radius, radius * 0.3, dist);
}

void main() {
  // --- Spaghettification UV distortion ---
  vec2 uv = v_uv;
  float dy = uv.y - 0.5;
  float sign_dy = sign(dy);
  float abs_dy = abs(dy) * 2.0;
  // pow(abs_dy, 0.25) at full strength = extreme vertical compression of
  // middle → the center band balloons to fill the screen
  float stretched = pow(max(abs_dy, 0.00001), mix(1.0, 0.25, u_spaghetti_strength));
  uv.y = mix(uv.y, sign_dy * stretched * 0.5 + 0.5, u_spaghetti_strength);

  // Clamp so we don't sample outside [0,1]
  uv = clamp(uv, vec2(0.001), vec2(0.999));

  // --- Sample scene (with chromatic aberration) ---
  vec3 color = sampleChromatic(u_scene_tex, uv, u_chromatic_spread);

  // --- Gravitational redshift tint ---
  // At u_redshift_amount = 1.0: keep only dim red channel, crush green/blue
  vec3 redColor = vec3(color.r * 0.85, color.g * 0.05, color.b * 0.02);
  color = mix(color, redColor, u_redshift_amount);

  // --- Vignette ---
  // When u_vignette_radius = 0: full blackout. radius = 1: no vignette.
  float vig = (u_vignette_radius <= 0.001)
      ? 0.0
      : vignette(v_uv, u_vignette_radius);
  color *= vig;

  // --- Singularity white flash ---
  color = mix(color, vec3(1.0), u_flash_intensity);

  // --- Void black overlay (RESET fade-in + VOID exit) ---
  color = mix(color, vec3(0.0), u_void_fade);

  fragColor = vec4(color, 1.0);
}
`;
