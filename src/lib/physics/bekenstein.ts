/**
 * Bekenstein-Hawking thermodynamics in JS, mirroring the Rust
 * gravitas::quantum::bekenstein surface for use in React readouts.
 *
 * References:
 * - Bekenstein 1973, Phys. Rev. D 7, 2333.
 * - Hawking 1975, Commun. Math. Phys. 43, 199.
 * - Page 1976, Phys. Rev. D 13, 198 (photon-only mass-loss constant).
 */

const SI_C = 299_792_458;
const SI_G = 6.6743e-11;
const SI_HBAR = 1.054_571_817e-34;

/**
 * Outer-horizon area in geometric units (M = 1).
 *
 * A = 4π (r_+² + a²),  r_+ = M + √(M² − a²),  a = a* M.
 */
export function horizonAreaGeometric(spinStar: number): number {
  const a = Math.max(-1, Math.min(1, spinStar));
  const disc = Math.sqrt(Math.max(0, 1 - a * a));
  const rPlus = 1 + disc;
  return 4 * Math.PI * (rPlus * rPlus + a * a);
}

/**
 * Outer-horizon area in SI m² for a hole of given SI mass and spin.
 *
 *   A_SI = (G M / c²)² · A_geom.
 */
export function horizonAreaSi(massKg: number, spinStar: number): number {
  const lengthScale = (SI_G * massKg) / (SI_C * SI_C);
  return horizonAreaGeometric(spinStar) * lengthScale * lengthScale;
}

/**
 * Dimensionless Bekenstein-Hawking entropy S/k_B = A_SI / (4 ℓ_P²)
 * with ℓ_P² = ℏ G / c³.
 */
export function bekensteinHawkingEntropyPerKb(
  massKg: number,
  spinStar: number,
): number {
  const aSi = horizonAreaSi(massKg, spinStar);
  const planckLengthSq = (SI_HBAR * SI_G) / SI_C ** 3;
  return aSi / (4 * planckLengthSq);
}

/**
 * Page 1976 photon-only Schwarzschild mass-loss rate dM/dt (kg/s).
 * The result is negative (mass shedding).
 */
export function schwarzschildMassLossRate(massKg: number): number {
  if (massKg <= 0) return 0;
  const numerator = SI_HBAR * SI_C ** 4;
  const denominator = 15360 * Math.PI * SI_G * SI_G * massKg * massKg;
  return -numerator / denominator;
}

/**
 * Schwarzschild evaporation lifetime in seconds. Integrating
 * dM/dt = −K/M² gives t_evap = M³ / (3 K).
 */
export function schwarzschildEvaporationTimeSeconds(massKg: number): number {
  if (massKg <= 0) return Infinity;
  const k = (SI_HBAR * SI_C ** 4) / (15360 * Math.PI * SI_G * SI_G);
  return massKg ** 3 / (3 * k);
}

/**
 * Format a very-large or very-small number for HUD display: keeps
 * three significant figures, switches between fixed decimal and
 * scientific notation at |log10|=4.
 */
export function formatScientific(value: number): string {
  if (!Number.isFinite(value)) return "—";
  if (value === 0) return "0";
  const abs = Math.abs(value);
  const log = Math.log10(abs);
  if (log >= -3 && log < 4) {
    return value.toFixed(3);
  }
  return value.toExponential(2);
}
