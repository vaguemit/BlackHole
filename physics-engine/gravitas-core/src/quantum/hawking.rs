//! Hawking radiation temperature.

use crate::constants::{SI_C, SI_HBAR, SI_KB};

/// Hawking temperature for a Kerr black hole.
///
/// T_H = (hbar * c^3) / (8 pi G M k_B) * (r+ - r-) / (r+^2 + a^2)
///
/// For the Schwarzschild limit (a=0): T_H = hbar c^3 / (8 pi G M k_B)
///
/// # Arguments
/// - `mass_kg` -- Mass in kilograms (SI)
/// - `spin` -- Dimensionless spin a* in [-1, 1]
pub fn hawking_temperature(mass_kg: f64, spin: f64) -> f64 {
    let g_si = 6.674_30e-11;
    let a_star = spin.clamp(-1.0, 1.0);

    // In geometric units: M = 1, a = a*
    // r_+ = 1 + sqrt(1 - a*^2),  r_- = 1 - sqrt(1 - a*^2)
    let disc = (1.0 - a_star * a_star).max(0.0).sqrt();
    let r_plus = 1.0 + disc;
    let r_minus = 1.0 - disc;

    // Surface gravity kappa = (r+ - r-) / (2 * (r+^2 + a*^2))
    let kappa = (r_plus - r_minus) / (2.0 * (r_plus * r_plus + a_star * a_star));

    // T_H = hbar * kappa_SI / (2 pi k_B)
    // kappa_SI = kappa * c^3 / (G * M)
    let kappa_si = kappa * SI_C * SI_C * SI_C / (g_si * mass_kg);

    SI_HBAR * kappa_si / (2.0 * std::f64::consts::PI * SI_KB)
}

/// Spectral radiance of a Hawking-emitting hole at temperature T_H,
/// modelled as a blackbody (no greybody factor; rule 12 §"no false
/// claims" applies — the visualization panel uses this as a
/// Planck-shape illustrative spectrum, not the full grey-body
/// transmission coefficient).
///
/// Returns B_ν(T) in W·m⁻²·Hz⁻¹·sr⁻¹ per the SI Planck law:
///
///   B_ν(T) = (2 h ν³ / c²) / (exp(h ν / k_B T) − 1).
///
/// Caller passes frequency in Hz and the Hawking temperature in K
/// (computed via `hawking_temperature` for a given M, a*).
#[must_use]
pub fn hawking_spectrum_planck(freq_hz: f64, temperature_k: f64) -> f64 {
    if temperature_k <= 0.0 || freq_hz <= 0.0 {
        return 0.0;
    }
    let h = SI_HBAR * 2.0 * std::f64::consts::PI;
    let exponent = h * freq_hz / (SI_KB * temperature_k);
    if exponent > 700.0 {
        // exp would overflow; the Planck law is exponentially small,
        // so return zero rather than a NaN from inf-inf.
        return 0.0;
    }
    let prefactor = 2.0 * h * freq_hz.powi(3) / (SI_C * SI_C);
    prefactor / exponent.exp_m1()
}

/// Wien displacement law for a Hawking-emitting hole: peak frequency
/// of the Planck spectrum at a given temperature.
///
///   ν_peak / T = 5.879 × 10¹⁰ Hz/K   (Wien 1893, modern value).
///
/// Useful for the visualization panel's x-axis tick: the peak
/// frequency tells the operator which band the hole shines in.
#[must_use]
pub fn wien_peak_frequency(temperature_k: f64) -> f64 {
    const WIEN_FREQUENCY_CONSTANT_HZ_PER_K: f64 = 5.878_925_757e10;
    WIEN_FREQUENCY_CONSTANT_HZ_PER_K * temperature_k.max(0.0)
}
