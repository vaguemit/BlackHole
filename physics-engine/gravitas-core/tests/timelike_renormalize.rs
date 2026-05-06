//! Timelike renormalization: H = −1/2 shell, with the same
//! discriminant-band semantics as the null branch.

use gravitas::geodesic::GeodesicState;
use gravitas::invariants::{
    hamiltonian, renormalize_null, renormalize_timelike, NormalizationError,
};
use gravitas::metric::Kerr;
use std::f64::consts::FRAC_PI_2;

const TIGHT: f64 = 1e-10;

/// Build a radial-infalling timelike state at radius r with energy E
/// and no angular momentum, then let renormalize_timelike pin p_r so
/// the state lies exactly on the H = −1/2 shell. Radial-only states
/// always admit a real p_r outside the horizon; that makes them the
/// stable test fixture for the renormaliser.
fn build_radial_timelike_state(metric: &Kerr, r: f64, e: f64) -> GeodesicState {
    let mut state = GeodesicState {
        x: [0.0, r, FRAC_PI_2, 0.0],
        p: [-e, 0.0, 0.0, 0.0],
    };
    renormalize_timelike(&mut state, metric).expect("radial timelike state should normalize");
    state
}

#[test]
fn timelike_renormalize_pins_state_on_h_minus_half_shell() {
    // Schwarzschild at r=10M, E = 1: radial-infall marginal-binding
    // case, p_r solved by the renormaliser.
    let metric = Kerr::new(1.0, 0.0);
    let state = build_radial_timelike_state(&metric, 10.0, 1.0);
    let h = hamiltonian(&state, &metric);
    assert!((h + 0.5).abs() < TIGHT, "H should be -1/2 after timelike renormalization, got {h}");
}

#[test]
fn timelike_renormalize_works_at_kerr_moderate_spin() {
    let metric = Kerr::new(1.0, 0.5);
    let state = build_radial_timelike_state(&metric, 12.0, 1.0);
    let h = hamiltonian(&state, &metric);
    assert!((h + 0.5).abs() < TIGHT, "Kerr a=0.5 H = {h}");
}

#[test]
fn null_renormalize_pins_state_on_h_zero_shell() {
    // Sanity check the existing null path still does what it claims.
    let metric = Kerr::new(1.0, 0.5);
    let mut state = GeodesicState {
        x: [0.0, 10.0, FRAC_PI_2, 0.0],
        p: [-1.0, 0.0, 0.0, 3.0],
    };
    renormalize_null(&mut state, &metric).expect("initial null state should normalize");
    let h = hamiltonian(&state, &metric);
    assert!(h.abs() < TIGHT, "H should be 0 after null renormalization, got {h}");
}

#[test]
fn timelike_and_null_produce_distinct_p_r_at_same_state() {
    // Same (x, p_t, p_φ, p_θ); the renormaliser picks p_r differently
    // because the two shells differ by exactly 1 in C.
    let metric = Kerr::new(1.0, 0.0);
    let mut s_null = GeodesicState {
        x: [0.0, 10.0, FRAC_PI_2, 0.0],
        p: [-1.0, 0.0, 0.0, 0.0],
    };
    let mut s_time = s_null;
    renormalize_null(&mut s_null, &metric).unwrap();
    renormalize_timelike(&mut s_time, &metric).unwrap();
    assert!(
        (s_null.p[1] - s_time.p[1]).abs() > 1e-3,
        "expected distinct p_r between null ({}) and timelike ({}) shells",
        s_null.p[1],
        s_time.p[1],
    );
}

#[test]
fn timelike_renormalize_errors_on_off_shell_state() {
    // p_φ = 100 at r = 5 with negligible energy: angular momentum
    // dominates the C term and the discriminant goes severely
    // negative because no real radial momentum can balance the
    // shell.
    let metric = Kerr::new(1.0, 0.0);
    let mut state = GeodesicState {
        x: [0.0, 5.0, FRAC_PI_2, 0.0],
        p: [-0.01, 0.0, 0.0, 100.0],
    };
    let result = renormalize_timelike(&mut state, &metric);
    assert!(
        matches!(result, Err(NormalizationError::NegativeDiscriminant { .. })),
        "expected NegativeDiscriminant, got {result:?}",
    );
}
