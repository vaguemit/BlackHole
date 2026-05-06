//! Bekenstein-Hawking thermodynamics: horizon area, entropy, and the
//! semi-classical mass-loss rate.
//!
//! Bekenstein 1973 *Phys. Rev. D* 7, 2333 introduced the proportionality
//! between horizon area and black-hole entropy. Hawking 1974 *Nature*
//! 248, 30 / 1975 *Commun. Math. Phys.* 43, 199 fixed the constant of
//! proportionality at S_BH = A / 4 in Planck units (A in units of
//! Planck area). Page 1976 *Phys. Rev. D* 13, 198 derived the
//! mass-loss rate dM/dt for a Schwarzschild hole emitting massless
//! particles.
//!
//! Conventions:
//! - Geometric horizon-area formula: A = 4π (r_+² + a²) with r_+ in
//!   units of M, a = a* M, so A is in units of M². Caller multiplies
//!   by (G M / c²)² for SI area in m².
//! - SI mass-loss rate uses ℏ, c, G, k_B from `crate::constants`.
//! - All formulas labelled with their paper references in the doc
//!   comment per rule 05.
//!
//! Out of scope: Greybody factors, charged-hole thermodynamics,
//! second-law tests with infalling matter. These are well-defined
//! follow-ups but each pulls in their own machinery.

use crate::constants::{SI_C, SI_G, SI_HBAR};
use crate::metric::{Kerr, Metric};

/// Outer-horizon area of a Kerr black hole in geometric units (M = 1
/// gives an area in units of M²; multiply by (GM/c²)² for SI m²).
///
/// A = 4π (r_+² + a²)  with  r_+ = M + √(M² − a²).
#[must_use]
pub fn horizon_area_geometric(metric: &Kerr) -> f64 {
    let r_plus = metric.event_horizon();
    let a = metric.a();
    4.0 * std::f64::consts::PI * (r_plus * r_plus + a * a)
}

/// Outer-horizon area in SI units (m²) for a hole of given SI mass.
///
/// Geometric → SI: A_SI = A_geom · (G M / c²)². Callers usually pass
/// the canonical Schwarzschild radius scale rather than mass directly.
#[must_use]
pub fn horizon_area_si(metric: &Kerr, mass_kg: f64) -> f64 {
    let length_scale = SI_G * mass_kg / (SI_C * SI_C);
    horizon_area_geometric(metric) * length_scale * length_scale
}

/// Bekenstein-Hawking entropy in nats (natural log). Multiply by k_B
/// to convert to SI entropy (J/K). The dimensionless form
///
///   S_BH / k_B = A_SI / (4 ℓ_P²)
///
/// holds with ℓ_P² = ℏ G / c³ the squared Planck length. Returns the
/// dimensionless ratio S/k_B.
#[must_use]
pub fn bekenstein_hawking_entropy_per_kb(metric: &Kerr, mass_kg: f64) -> f64 {
    let area_si = horizon_area_si(metric, mass_kg);
    let planck_length_sq = SI_HBAR * SI_G / (SI_C.powi(3));
    area_si / (4.0 * planck_length_sq)
}

/// Page 1976 *Phys. Rev. D* 13, 198 mass-loss rate for a Schwarzschild
/// hole emitting massless particles (photons + 6 neutrino species at
/// the time of the paper; modern accounting raises the constant by
/// a few percent depending on which species are massless above the
/// hole's temperature):
///
///   dM/dt = − ℏ c⁴ / (15360 π G² M²)   (photon-only).
///
/// Returns dM/dt in kg/s (negative; the hole is shedding mass).
#[must_use]
pub fn schwarzschild_mass_loss_rate(mass_kg: f64) -> f64 {
    let numerator = SI_HBAR * SI_C.powi(4);
    let denominator = 15360.0 * std::f64::consts::PI * SI_G * SI_G * mass_kg * mass_kg;
    -numerator / denominator
}

/// Lifetime of a Schwarzschild hole shedding mass at the Page rate
/// (photon-only). Integrating dM/dt = -K/M² gives M(t) = (M₀³ − 3 K t)^(1/3),
/// so the hole evaporates at t_evap = M₀³ / (3 K) where K equals the
/// constant in `schwarzschild_mass_loss_rate`.
///
/// Returns evaporation time in seconds. For a solar-mass hole the
/// answer is around 10^67 years, far longer than the age of the
/// universe; for a primordial hole at 10^11 kg it's of order the
/// universe's age.
#[must_use]
pub fn schwarzschild_evaporation_time(mass_kg: f64) -> f64 {
    let numerator = SI_HBAR * SI_C.powi(4);
    let k = numerator / (15360.0 * std::f64::consts::PI * SI_G * SI_G);
    mass_kg.powi(3) / (3.0 * k)
}
