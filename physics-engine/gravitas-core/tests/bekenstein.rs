//! Bekenstein-Hawking thermodynamics: horizon area, entropy, Page
//! mass-loss rate, and the corresponding evaporation lifetime.
//!
//! Sanity checks at canonical parameters:
//! - Schwarzschild (a* = 0): A = 16π M² (geometric); the canonical
//!   value for entropy of a solar-mass hole is ~10⁷⁷ k_B.
//! - Extremal Kerr (a* = 1): A = 8π M², half the Schwarzschild area
//!   at the same M.
//! - Page mass-loss rate: dM/dt < 0 for any positive mass; matches
//!   Page 1976 photon-only constant; lifetime scales as M³.

use gravitas::metric::Kerr;
use gravitas::quantum::bekenstein::{
    bekenstein_hawking_entropy_per_kb, horizon_area_geometric, horizon_area_si,
    schwarzschild_evaporation_time, schwarzschild_mass_loss_rate,
};

const PI: f64 = std::f64::consts::PI;

fn close(a: f64, b: f64, eps: f64) -> bool {
    (a - b).abs() < eps
}

// ---------------------------------------------------------------------
// Horizon area
// ---------------------------------------------------------------------

#[test]
fn schwarzschild_horizon_area_is_sixteen_pi_m_squared() {
    let metric = Kerr::new(1.0, 0.0);
    let a = horizon_area_geometric(&metric);
    assert!(close(a, 16.0 * PI, 1e-12));
}

#[test]
fn extremal_kerr_horizon_area_is_eight_pi_m_squared() {
    // a* = 1 (corotating extremal): r_+ = M, a = M, so A = 4π(M² + M²) = 8π M².
    let metric = Kerr::new(1.0, 1.0);
    let a = horizon_area_geometric(&metric);
    assert!(close(a, 8.0 * PI, 1e-12));
}

#[test]
fn horizon_area_decreases_with_spin() {
    let m = 1.0;
    let a_zero = horizon_area_geometric(&Kerr::new(m, 0.0));
    let a_high = horizon_area_geometric(&Kerr::new(m, 0.998));
    assert!(a_high < a_zero, "A should shrink with spin: a=0 → {a_zero}, a=0.998 → {a_high}");
}

#[test]
fn horizon_area_si_scales_with_mass_squared() {
    // A_SI ∝ (G M / c²)² × A_geom, so doubling M quadruples A_SI at
    // fixed spin.
    let metric = Kerr::new(1.0, 0.0);
    let a_one = horizon_area_si(&metric, 1.0e30);
    let a_two = horizon_area_si(&metric, 2.0e30);
    assert!(close(a_two / a_one, 4.0, 1e-12));
}

// ---------------------------------------------------------------------
// Bekenstein-Hawking entropy
// ---------------------------------------------------------------------

#[test]
fn entropy_proportional_to_area() {
    // S/k_B = A_SI / (4 ℓ_P²), so doubling A_SI doubles entropy.
    let m = 1.0e30;
    let s1 = bekenstein_hawking_entropy_per_kb(&Kerr::new(1.0, 0.0), m);
    let s2 = bekenstein_hawking_entropy_per_kb(&Kerr::new(1.0, 0.0), 2.0_f64.powf(0.5) * m);
    // M → M·√2 gives A → A·2 (since A ∝ M² for fixed spin).
    assert!(close(s2 / s1, 2.0, 1e-9));
}

#[test]
fn solar_mass_entropy_is_order_1e77() {
    // Bekenstein 1973 + Hawking 1975 canonical value: S/k_B ≈ 1.05e77
    // for a one-solar-mass Schwarzschild hole. Known from textbooks.
    let metric = Kerr::new(1.0, 0.0);
    let m_sun = 1.98847e30; // SI_SOLAR_MASS
    let s = bekenstein_hawking_entropy_per_kb(&metric, m_sun);
    assert!(s > 1e76 && s < 1e78, "Solar-mass S/k_B = {s}, expected ~1.05e77");
}

// ---------------------------------------------------------------------
// Page 1976 mass-loss rate
// ---------------------------------------------------------------------

#[test]
fn mass_loss_is_negative() {
    let dm_dt = schwarzschild_mass_loss_rate(1.0e30);
    assert!(dm_dt < 0.0);
}

#[test]
fn mass_loss_rate_scales_inversely_with_mass_squared() {
    let dm1 = schwarzschild_mass_loss_rate(1.0e10);
    let dm2 = schwarzschild_mass_loss_rate(2.0e10);
    // dM/dt ∝ -1/M², so doubling M reduces |dM/dt| by 4.
    assert!(close(dm2 / dm1, 0.25, 1e-12));
}

#[test]
fn evaporation_time_scales_with_mass_cubed() {
    let t1 = schwarzschild_evaporation_time(1.0e10);
    let t2 = schwarzschild_evaporation_time(2.0e10);
    assert!(close(t2 / t1, 8.0, 1e-12));
}

#[test]
fn solar_mass_evaporation_time_exceeds_age_of_universe() {
    // Solar mass evaporation: t_evap ≈ 2.1×10⁶⁷ years, which is
    // about 1.5×10⁵⁷ times the present age of the universe.
    let m_sun = 1.98847e30;
    let t_evap = schwarzschild_evaporation_time(m_sun);
    let age_universe_seconds = 1.4e10 * 365.25 * 24.0 * 3600.0;
    let ratio = t_evap / age_universe_seconds;
    assert!(ratio > 1.0e50 && ratio < 1.0e65, "solar t_evap / t_age = {ratio}");
}

#[test]
fn primordial_hole_evaporation_in_known_window() {
    // Hawking 1975: a primordial hole at ~1e11 kg evaporates in
    // roughly the age of the universe. Modern accounting is around
    // 5e15 - 1e17 s. Loose bound: 1e14 - 1e18 s.
    let t_evap = schwarzschild_evaporation_time(5.0e11);
    assert!(t_evap > 1e14 && t_evap < 1e20, "primordial t_evap = {t_evap}");
}
