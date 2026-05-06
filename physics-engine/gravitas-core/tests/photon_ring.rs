//! Photon-sphere regression: the engine's photon-sphere radius must agree
//! with the analytic Bardeen 1973 form across the spin grid, prograde
//! branch only (the public API exposes a single prograde photon sphere).
//!
//! The Bardeen 1973 formula reproduced in `common::bardeen_b_c_prograde`
//! returns the critical impact parameter at infinity, not the radius;
//! the on-grid radius itself comes from the closed form
//!
//!   r_ph = 2M [ 1 + cos( (2/3) arccos(-a*) ) ]
//!
//! which is what `Kerr::photon_sphere` implements. This test pins that
//! formula against an independent recomputation, so a future refactor
//! can't silently invert the cosine branch.

mod common;

use common::REGRESSION_SPIN_GRID;
use gravitas::metric::Kerr;

const PHOTON_SPHERE_TOLERANCE: f64 = 1e-9;

fn analytic_photon_sphere_prograde(a_star: f64) -> f64 {
    2.0 * (1.0 + ((2.0 / 3.0) * (-a_star).acos()).cos())
}

#[test]
fn photon_sphere_matches_bardeen_across_spin_grid() {
    for &a_star in REGRESSION_SPIN_GRID.iter() {
        let metric = Kerr::new(1.0, a_star);
        let predicted = metric.photon_sphere();
        let expected = analytic_photon_sphere_prograde(a_star);
        let err = (predicted - expected).abs();
        assert!(
            err < PHOTON_SPHERE_TOLERANCE,
            "a*={a_star}: r_ph engine={predicted}, Bardeen={expected}, |Δ|={err}",
        );
    }
}

#[test]
fn photon_sphere_collapses_to_three_m_at_a_zero() {
    // Schwarzschild: r_ph = 3M.
    let metric = Kerr::new(1.0, 0.0);
    let r_ph = metric.photon_sphere();
    let err = (r_ph - 3.0).abs();
    assert!(err < PHOTON_SPHERE_TOLERANCE, "Schwarzschild r_ph={r_ph}, expected 3M");
}

#[test]
fn photon_sphere_approaches_m_at_extremal() {
    // Extremal prograde: r_ph -> M as a* -> 1.
    let metric = Kerr::new(1.0, 0.998);
    let r_ph = metric.photon_sphere();
    assert!(r_ph > 1.0 && r_ph < 1.5, "near-extremal r_ph={r_ph} outside (M, 1.5M)");
}
