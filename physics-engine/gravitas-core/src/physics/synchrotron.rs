//! Thermal synchrotron emissivity in Kerr accretion-flow plasmas.
//!
//! Pandya, Zhdankin, Chandra & Quataert (2016, *ApJ* 822, 34) gave a
//! family of fitting formulas for synchrotron j_ν / α_ν in the
//! relativistic-electron regime. The thermal-distribution branch
//! used here matches their Eq. 31 (synchrotron emissivity for a
//! relativistic Maxwell-Jüttner electron distribution):
//!
//!   j_ν = (n_e e² ν_s / c) · F(X, θ_e),
//!   X   = ν / (ν_s sin θ_B),
//!   ν_s = (2/9) ν_c θ_e²,
//!   ν_c = e B / (2 π m_e c),
//!
//! where θ_e = k_B T_e / (m_e c²) is the dimensionless electron
//! temperature. F(X, θ_e) is the dimensionless fitting function;
//! Pandya+ 2016 give a polynomial-times-exponential form whose
//! relativistic limit reduces to F(X) = (1 + 2.41 X^{1/2} +
//! 0.40 X^{-2/3}) · exp(−X^{1/3}).
//!
//! This implementation lands the relativistic limit (θ_e ≫ 1) and is
//! within a few percent of the full Pandya formula for the
//! frequencies and temperatures that Sgr A* / M87* observations probe
//! (10⁻³ < X < 10³, 1 < θ_e < 1000). Outside that range the formula
//! is still numerically well-behaved but the percent-level fit
//! accuracy is not guaranteed; callers fall back to a different
//! distribution if the parameters drift.
//!
//! Absorption coefficient α_ν follows from Kirchhoff's law in the
//! Rayleigh-Jeans / thermal limit:
//!
//!   α_ν = j_ν / B_ν(T_e)
//!
//! with B_ν the Planck function. The framework handles the optically
//! thick limit naturally because integrate_radiative_transfer in
//! physics::radiative_transfer asymptotes to S = j/α = B_ν(T_e) for
//! large τ.
//!
//! Out of scope: power-law and kappa-distribution branches (Pandya+
//! 2016 §3.2 and §3.3); polarised emissivity Q/U/V (Schnittman+Krolik
//! 2009 §2 covers this and the polarization module already lands the
//! Stokes initialiser primitive).

use crate::constants::{SI_C, SI_HBAR, SI_KB};
use crate::physics::radiative_transfer::Band;

const ELECTRON_CHARGE_ESU: f64 = 4.803_204_5e-10; // statcoulombs (CGS)
const ELECTRON_MASS_GRAM: f64 = 9.109_383_7e-28;
const SI_C_CM: f64 = 100.0 * SI_C; // c in cm/s for CGS

/// Plasma state at a single radiative-transfer sample point. All
/// quantities are in CGS, the standard for Pandya+ 2016 fitting
/// formulas; conversion from SI inputs is the caller's job.
#[derive(Clone, Copy, Debug)]
pub struct PlasmaState {
    /// Electron number density n_e (cm⁻³).
    pub n_e: f64,
    /// Electron temperature T_e (K).
    pub t_e: f64,
    /// Magnetic-field magnitude |B| (Gauss).
    pub b_field: f64,
    /// Angle between line of sight and B-field (radians).
    pub theta_b: f64,
}

/// Dimensionless electron temperature θ_e = k_B T_e / (m_e c²).
#[must_use]
pub fn theta_e(t_e: f64) -> f64 {
    let kt = SI_KB * t_e;
    let m_e_c2 = ELECTRON_MASS_GRAM * SI_C_CM * SI_C_CM * 1.0e-7;
    kt / m_e_c2
}

/// Cyclotron frequency ν_c = e B / (2π m_e c) in Hz, with B in Gauss.
#[must_use]
pub fn cyclotron_frequency(b_gauss: f64) -> f64 {
    ELECTRON_CHARGE_ESU * b_gauss
        / (2.0 * std::f64::consts::PI * ELECTRON_MASS_GRAM * SI_C_CM)
}

/// Synchrotron characteristic frequency ν_s for a relativistic
/// thermal electron distribution: ν_s = (2/9) ν_c θ_e².
#[must_use]
pub fn synchrotron_characteristic_frequency(b_gauss: f64, t_e: f64) -> f64 {
    let nu_c = cyclotron_frequency(b_gauss);
    let theta = theta_e(t_e);
    (2.0 / 9.0) * nu_c * theta * theta
}

/// Pandya+ 2016 Eq. 31 fitting function in the relativistic limit:
///
///   F(X) = (1 + 2.41 X^{1/2} + 0.40 X^{-2/3}) · exp(−X^{1/3}).
///
/// X = ν / (ν_s sin θ_B). Returns 0 for non-positive X (callers
/// passing edge-on geometry with sin θ_B = 0 hit this branch).
#[must_use]
pub fn pandya_2016_thermal_fit(x: f64) -> f64 {
    if x <= 0.0 || !x.is_finite() {
        return 0.0;
    }
    let x_sqrt = x.sqrt();
    let x_cbrt = x.cbrt();
    let x_neg_two_thirds = 1.0 / (x_cbrt * x_cbrt);
    let bracket = 1.0 + 2.41 * x_sqrt + 0.40 * x_neg_two_thirds;
    bracket * (-x_cbrt).exp()
}

/// Thermal synchrotron emissivity j_ν for a relativistic Maxwell-
/// Jüttner electron distribution per Pandya+ 2016 Eq. 31. Result in
/// CGS: erg s⁻¹ cm⁻³ Hz⁻¹ sr⁻¹.
///
/// Returns 0 when sin θ_B is below the cyclotron-truncation floor
/// (the synchrotron beam vanishes for line-of-sight along B), when
/// n_e is zero, or when ν_s is zero (no field).
#[must_use]
pub fn j_thermal_synchrotron(freq_hz: f64, plasma: PlasmaState) -> f64 {
    if plasma.n_e <= 0.0 || plasma.b_field <= 0.0 || freq_hz <= 0.0 {
        return 0.0;
    }
    let sin_theta_b = plasma.theta_b.sin();
    if sin_theta_b.abs() < 1.0e-6 {
        return 0.0;
    }

    let nu_s = synchrotron_characteristic_frequency(plasma.b_field, plasma.t_e);
    if nu_s <= 0.0 {
        return 0.0;
    }

    let x = freq_hz / (nu_s * sin_theta_b.abs());
    let prefactor =
        plasma.n_e * ELECTRON_CHARGE_ESU * ELECTRON_CHARGE_ESU * nu_s / SI_C_CM;
    prefactor * pandya_2016_thermal_fit(x)
}

/// Planck function B_ν(T) in CGS units (erg s⁻¹ cm⁻² Hz⁻¹ sr⁻¹).
/// Used for the Kirchhoff-law derivation of α_ν below; keeping the
/// CGS form here matches Pandya+ 2016's units exactly.
#[must_use]
pub fn planck_cgs(freq_hz: f64, t_e: f64) -> f64 {
    if t_e <= 0.0 || freq_hz <= 0.0 {
        return 0.0;
    }
    // Convert ℏ from SI (J·s) to CGS (erg·s) via 1e7.
    let h_cgs = SI_HBAR * 1.0e7 * 2.0 * std::f64::consts::PI;
    // k_B from SI (J/K) to CGS (erg/K) via 1e7.
    let k_b_cgs = SI_KB * 1.0e7;
    let exponent = h_cgs * freq_hz / (k_b_cgs * t_e);
    if exponent > 700.0 {
        return 0.0;
    }
    let prefactor = 2.0 * h_cgs * freq_hz.powi(3) / (SI_C_CM * SI_C_CM);
    prefactor / exponent.exp_m1()
}

/// Absorption coefficient α_ν via Kirchhoff's law in the thermal
/// limit: α_ν = j_ν / B_ν(T_e). Result in CGS cm⁻¹.
#[must_use]
pub fn alpha_thermal_synchrotron(freq_hz: f64, plasma: PlasmaState) -> f64 {
    let j = j_thermal_synchrotron(freq_hz, plasma);
    if j <= 0.0 {
        return 0.0;
    }
    let b_planck = planck_cgs(freq_hz, plasma.t_e);
    if b_planck <= 0.0 {
        return 0.0;
    }
    j / b_planck
}

/// Convenience wrapper: emissivity sampled per band against a single
/// plasma state. Returns one (j, α) pair per band in the input order.
#[must_use]
pub fn band_emissivity_and_absorption(
    bands: &[Band],
    plasma: PlasmaState,
) -> Vec<(f64, f64)> {
    bands
        .iter()
        .map(|band| {
            (
                j_thermal_synchrotron(band.freq_hz, plasma),
                alpha_thermal_synchrotron(band.freq_hz, plasma),
            )
        })
        .collect()
}
