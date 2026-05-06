//! Hawking spectrum primitives: blackbody Planck shape at the
//! Hawking temperature, plus the Wien peak frequency.

use gravitas::quantum::hawking::{
    hawking_spectrum_planck, hawking_temperature, wien_peak_frequency,
};

fn close(a: f64, b: f64, rel_eps: f64) -> bool {
    if b.abs() < 1e-30 {
        a.abs() < rel_eps
    } else {
        (a - b).abs() / b.abs() < rel_eps
    }
}

// ---------------------------------------------------------------------
// Planck spectrum sanity
// ---------------------------------------------------------------------

#[test]
fn planck_returns_zero_at_zero_temperature() {
    let b = hawking_spectrum_planck(1.0e10, 0.0);
    assert_eq!(b, 0.0);
}

#[test]
fn planck_returns_zero_at_negative_temperature() {
    let b = hawking_spectrum_planck(1.0e10, -1.0);
    assert_eq!(b, 0.0);
}

#[test]
fn planck_returns_zero_at_zero_frequency() {
    let b = hawking_spectrum_planck(0.0, 100.0);
    assert_eq!(b, 0.0);
}

#[test]
fn planck_handles_large_exponent_without_nan() {
    // h ν / k T = 1000 → exp underflow if not guarded.
    let b = hawking_spectrum_planck(1.0e16, 1.0);
    assert!(b.is_finite());
    assert!(b >= 0.0);
}

#[test]
fn planck_peaks_near_wien_frequency() {
    // Sample around Wien peak; should be the maximum across the
    // sample band.
    let t = 100.0;
    let nu_peak = wien_peak_frequency(t);
    let b_peak = hawking_spectrum_planck(nu_peak, t);
    let b_low = hawking_spectrum_planck(nu_peak * 0.1, t);
    let b_high = hawking_spectrum_planck(nu_peak * 10.0, t);
    assert!(b_peak > b_low, "Planck low-side: {b_low} >= peak {b_peak}");
    assert!(b_peak > b_high, "Planck high-side: {b_high} >= peak {b_peak}");
}

#[test]
fn planck_low_frequency_rayleigh_jeans_limit() {
    // Rayleigh-Jeans: B_ν ≈ 2 ν² k_B T / c² when h ν ≪ k_B T.
    let t = 100.0;
    let nu = wien_peak_frequency(t) * 1.0e-3; // well into RJ regime
    let b = hawking_spectrum_planck(nu, t);
    let _h = 6.62607015e-34; // kept for derivation clarity
    let kb = 1.380649e-23;
    let c = 299_792_458.0;
    let rj = 2.0 * nu * nu * kb * t / (c * c);
    // 1/(e^x - 1) ≈ 1/x − 1/2 + ... at small x; the leading term
    // matches RJ to within a few percent.
    assert!(close(b, rj, 0.05), "Planck B = {b}, RJ = {rj}");
}

// ---------------------------------------------------------------------
// Wien peak
// ---------------------------------------------------------------------

#[test]
fn wien_peak_zero_at_zero_temperature() {
    assert_eq!(wien_peak_frequency(0.0), 0.0);
}

#[test]
fn wien_peak_clamps_negative_temperature_to_zero() {
    assert_eq!(wien_peak_frequency(-100.0), 0.0);
}

#[test]
fn wien_peak_proportional_to_temperature() {
    let nu1 = wien_peak_frequency(100.0);
    let nu2 = wien_peak_frequency(200.0);
    assert!(close(nu2 / nu1, 2.0, 1e-12));
}

// ---------------------------------------------------------------------
// Hawking temperature integration
// ---------------------------------------------------------------------

#[test]
fn solar_mass_hawking_temperature_is_picokelvin_scale() {
    // T_H for one solar mass Schwarzschild ≈ 6.2e-8 K.
    let t = hawking_temperature(1.98847e30, 0.0);
    assert!(t > 1.0e-9 && t < 1.0e-6, "solar mass T_H = {t}");
}

#[test]
fn primordial_hole_hawking_temperature_in_known_window() {
    // 1e12 kg primordial hole has T_H ≈ 1e11 K (Hawking 1975).
    let t = hawking_temperature(1.0e12, 0.0);
    assert!(t > 1.0e10 && t < 1.0e12, "primordial T_H = {t}");
}

#[test]
fn extremal_kerr_temperature_is_zero() {
    // T_H = 0 at extremal: r_+ = r_- so surface gravity vanishes.
    let t = hawking_temperature(1.0e30, 1.0);
    assert!(t.abs() < 1.0e-30, "extremal T_H should be 0, got {t}");
}
