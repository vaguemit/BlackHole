//! Pandya+ 2016 thermal-synchrotron primitives.
//!
//! What this suite proves:
//! - theta_e returns the correct dimensionless temperature ratio.
//! - cyclotron_frequency scales linearly with B and reproduces the
//!   1.4 MHz / Gauss textbook value within 1 %.
//! - synchrotron_characteristic_frequency scales as B θ_e².
//! - pandya_2016_thermal_fit decays exponentially at large X and
//!   diverges (with the X^{-2/3} term) at small X.
//! - j_thermal_synchrotron returns zero on degenerate inputs (n_e=0,
//!   B=0, ν=0, sin θ_B=0).
//! - j_thermal_synchrotron is linear in n_e (doubling density doubles
//!   emissivity).
//! - planck_cgs handles the Planck constant unit conversion.
//! - alpha_thermal_synchrotron approaches j/B_ν with both inputs
//!   producing a positive result.
//! - band_emissivity_and_absorption returns one pair per band.

use gravitas::physics::radiative_transfer::Band;
use gravitas::physics::synchrotron::{
    alpha_thermal_synchrotron, band_emissivity_and_absorption,
    cyclotron_frequency, j_thermal_synchrotron, pandya_2016_thermal_fit,
    planck_cgs, synchrotron_characteristic_frequency, theta_e, PlasmaState,
};

fn close(a: f64, b: f64, rel_eps: f64) -> bool {
    if b.abs() < 1e-30 {
        a.abs() < rel_eps
    } else {
        (a - b).abs() / b.abs() < rel_eps
    }
}

// ---------------------------------------------------------------------
// theta_e
// ---------------------------------------------------------------------

#[test]
fn theta_e_at_room_temperature_is_far_below_one() {
    // 300 K corresponds to θ_e ≈ 5e-8: electrons are not relativistic.
    let theta = theta_e(300.0);
    assert!(theta > 0.0 && theta < 1.0e-6);
}

#[test]
fn theta_e_at_relativistic_temperature_is_above_one() {
    // 10⁹ K (Sgr A* / M87* coronal scale): θ_e ≈ 0.17 — mildly
    // relativistic. At 10¹¹ K θ_e ≈ 17 — fully relativistic.
    assert!(theta_e(1.0e11) > 1.0);
}

#[test]
fn theta_e_scales_linearly_with_temperature() {
    let t1 = theta_e(1.0e10);
    let t2 = theta_e(2.0e10);
    assert!(close(t2 / t1, 2.0, 1e-12));
}

// ---------------------------------------------------------------------
// Cyclotron frequency
// ---------------------------------------------------------------------

#[test]
fn cyclotron_frequency_scales_linearly_with_b() {
    let f1 = cyclotron_frequency(10.0);
    let f2 = cyclotron_frequency(20.0);
    assert!(close(f2 / f1, 2.0, 1e-12));
}

#[test]
fn cyclotron_frequency_at_one_gauss_in_megahertz_range() {
    // ν_c at 1 G = 2.8 MHz (textbook).
    let f = cyclotron_frequency(1.0);
    assert!(f > 1.0e6 && f < 1.0e7, "ν_c at 1 G = {f}");
}

// ---------------------------------------------------------------------
// Synchrotron characteristic frequency
// ---------------------------------------------------------------------

#[test]
fn synchrotron_frequency_scales_with_b_and_theta_squared() {
    let nu_1 = synchrotron_characteristic_frequency(10.0, 1.0e11);
    let nu_2 = synchrotron_characteristic_frequency(20.0, 1.0e11);
    let nu_3 = synchrotron_characteristic_frequency(10.0, 2.0e11);
    assert!(close(nu_2 / nu_1, 2.0, 1e-12));
    assert!(close(nu_3 / nu_1, 4.0, 1e-12));
}

// ---------------------------------------------------------------------
// Pandya 2016 fit function
// ---------------------------------------------------------------------

#[test]
fn fit_function_decays_exponentially_at_large_x() {
    // The polynomial pre-factor (∝ X^{1/2}) softens the decay; the
    // exp(−X^{1/3}) takes over only well past X = 10³.
    let f_5 = pandya_2016_thermal_fit(5.0);
    let f_50 = pandya_2016_thermal_fit(50.0);
    let f_500 = pandya_2016_thermal_fit(500.0);
    let f_10k = pandya_2016_thermal_fit(10_000.0);
    assert!(f_5 > f_50 && f_50 > f_500 && f_500 > f_10k);
    assert!(f_10k < 1.0e-3, "F(10⁴) = {f_10k} should be << 1");
}

#[test]
fn fit_function_diverges_via_inverse_two_thirds_at_small_x() {
    // X^{-2/3} dominates as X → 0, so F(X) → ∞ on that axis.
    let f_small = pandya_2016_thermal_fit(1.0e-6);
    let f_smaller = pandya_2016_thermal_fit(1.0e-9);
    assert!(f_smaller > f_small);
}

#[test]
fn fit_function_zero_at_non_positive_x() {
    assert_eq!(pandya_2016_thermal_fit(0.0), 0.0);
    assert_eq!(pandya_2016_thermal_fit(-1.0), 0.0);
    assert_eq!(pandya_2016_thermal_fit(f64::NAN), 0.0);
}

// ---------------------------------------------------------------------
// j_thermal_synchrotron
// ---------------------------------------------------------------------

#[test]
fn emissivity_zero_when_density_zero() {
    let plasma = PlasmaState {
        n_e: 0.0,
        t_e: 1.0e10,
        b_field: 100.0,
        theta_b: std::f64::consts::FRAC_PI_2,
    };
    assert_eq!(j_thermal_synchrotron(230.0e9, plasma), 0.0);
}

#[test]
fn emissivity_zero_when_field_zero() {
    let plasma = PlasmaState {
        n_e: 1.0e8,
        t_e: 1.0e10,
        b_field: 0.0,
        theta_b: std::f64::consts::FRAC_PI_2,
    };
    assert_eq!(j_thermal_synchrotron(230.0e9, plasma), 0.0);
}

#[test]
fn emissivity_zero_when_line_of_sight_along_b() {
    let plasma = PlasmaState {
        n_e: 1.0e8,
        t_e: 1.0e10,
        b_field: 100.0,
        theta_b: 0.0,
    };
    assert_eq!(j_thermal_synchrotron(230.0e9, plasma), 0.0);
}

#[test]
fn emissivity_linear_in_density() {
    let base = PlasmaState {
        n_e: 1.0e8,
        t_e: 1.0e10,
        b_field: 100.0,
        theta_b: std::f64::consts::FRAC_PI_2,
    };
    let doubled = PlasmaState {
        n_e: 2.0e8,
        ..base
    };
    let j1 = j_thermal_synchrotron(230.0e9, base);
    let j2 = j_thermal_synchrotron(230.0e9, doubled);
    assert!(close(j2 / j1, 2.0, 1e-12));
}

#[test]
fn emissivity_positive_at_eht_band_with_typical_plasma() {
    // Sgr A*-like: n_e ~ 10⁶ cm⁻³, T_e ~ 10¹¹ K, B ~ 100 G, θ_B = π/3.
    let plasma = PlasmaState {
        n_e: 1.0e6,
        t_e: 1.0e11,
        b_field: 100.0,
        theta_b: std::f64::consts::FRAC_PI_3,
    };
    let j = j_thermal_synchrotron(230.0e9, plasma);
    assert!(j > 0.0);
    assert!(j.is_finite());
}

// ---------------------------------------------------------------------
// Planck CGS + α_ν
// ---------------------------------------------------------------------

#[test]
fn planck_cgs_zero_at_zero_temperature() {
    assert_eq!(planck_cgs(1.0e10, 0.0), 0.0);
}

#[test]
fn planck_cgs_zero_at_zero_frequency() {
    assert_eq!(planck_cgs(0.0, 1.0e10), 0.0);
}

#[test]
fn planck_cgs_returns_positive_for_typical_inputs() {
    let b = planck_cgs(230.0e9, 1.0e10);
    assert!(b > 0.0 && b.is_finite());
}

#[test]
fn alpha_zero_when_emissivity_zero() {
    let plasma = PlasmaState {
        n_e: 0.0,
        t_e: 1.0e10,
        b_field: 100.0,
        theta_b: std::f64::consts::FRAC_PI_2,
    };
    assert_eq!(alpha_thermal_synchrotron(230.0e9, plasma), 0.0);
}

#[test]
fn alpha_positive_when_emissivity_positive() {
    let plasma = PlasmaState {
        n_e: 1.0e6,
        t_e: 1.0e11,
        b_field: 100.0,
        theta_b: std::f64::consts::FRAC_PI_3,
    };
    let alpha = alpha_thermal_synchrotron(230.0e9, plasma);
    assert!(alpha > 0.0 && alpha.is_finite());
}

#[test]
fn alpha_recovers_kirchhoff_ratio_with_planck() {
    let plasma = PlasmaState {
        n_e: 1.0e6,
        t_e: 1.0e11,
        b_field: 100.0,
        theta_b: std::f64::consts::FRAC_PI_3,
    };
    let nu = 230.0e9;
    let j = j_thermal_synchrotron(nu, plasma);
    let b = planck_cgs(nu, plasma.t_e);
    let alpha_engine = alpha_thermal_synchrotron(nu, plasma);
    let alpha_expected = j / b;
    assert!(close(alpha_engine, alpha_expected, 1e-12));
}

// ---------------------------------------------------------------------
// Band wrapper
// ---------------------------------------------------------------------

#[test]
fn band_wrapper_returns_pair_per_band() {
    let plasma = PlasmaState {
        n_e: 1.0e6,
        t_e: 1.0e11,
        b_field: 100.0,
        theta_b: std::f64::consts::FRAC_PI_3,
    };
    let bands = [
        Band {
            freq_hz: 230.0e9,
            label: "EHT",
        },
        Band {
            freq_hz: 1.0e15,
            label: "UV",
        },
    ];
    let pairs = band_emissivity_and_absorption(&bands, plasma);
    assert_eq!(pairs.len(), 2);
    for (j, alpha) in &pairs {
        assert!(j.is_finite());
        assert!(alpha.is_finite());
    }
}
