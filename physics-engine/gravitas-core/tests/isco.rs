//! ISCO regression: the engine's `Kerr::isco(orbit)` must match the
//! Bardeen-Press-Teukolsky formula (Bardeen 1973 Eq. 38) across the
//! standard spin grid, both prograde and retrograde.
//!
//! Schwarzschild limit: r_isco -> 6M for both branches at a* = 0.
//! Extremal prograde: r_isco -> M as a* -> 1.
//! Extremal retrograde: r_isco -> 9M as a* -> 1.

mod common;

use common::{
    bardeen_r_isco_prograde, bardeen_r_isco_retrograde, REGRESSION_SPIN_GRID,
};
use gravitas::metric::{Kerr, Orbit};

const ISCO_TOLERANCE: f64 = 1e-6;

#[test]
fn isco_prograde_matches_bardeen_across_spin_grid() {
    for &a_star in REGRESSION_SPIN_GRID.iter() {
        let metric = Kerr::new(1.0, a_star);
        let predicted = metric.isco(Orbit::Prograde);
        let expected = bardeen_r_isco_prograde(a_star);
        let err = (predicted - expected).abs();
        assert!(
            err < ISCO_TOLERANCE,
            "a*={a_star} prograde: engine={predicted}, Bardeen={expected}, |Δ|={err}",
        );
    }
}

#[test]
fn isco_retrograde_matches_bardeen_across_spin_grid() {
    // Skip a* = 0 because the engine returns 6M as a fast path; both
    // branches collapse there and the Bardeen formula numerically agrees.
    for &a_star in REGRESSION_SPIN_GRID.iter().filter(|&&a| a > 0.0) {
        let metric = Kerr::new(1.0, a_star);
        let predicted = metric.isco(Orbit::Retrograde);
        let expected = bardeen_r_isco_retrograde(a_star);
        let err = (predicted - expected).abs();
        assert!(
            err < ISCO_TOLERANCE,
            "a*={a_star} retrograde: engine={predicted}, Bardeen={expected}, |Δ|={err}",
        );
    }
}

#[test]
fn isco_schwarzschild_limit_is_six_m_both_branches() {
    let metric = Kerr::new(1.0, 0.0);
    assert!((metric.isco(Orbit::Prograde) - 6.0).abs() < ISCO_TOLERANCE);
    assert!((metric.isco(Orbit::Retrograde) - 6.0).abs() < ISCO_TOLERANCE);
}

#[test]
fn isco_near_extremal_prograde_under_two_m() {
    // Near-extremal prograde collapses toward r_+ = M; at a* = 0.998 the
    // analytic value is around 1.24M.
    let metric = Kerr::new(1.0, 0.998);
    let r = metric.isco(Orbit::Prograde);
    assert!(r > 1.0 && r < 2.0, "a*=0.998 prograde isco={r} outside (M, 2M)");
}

#[test]
fn isco_near_extremal_retrograde_above_eight_m() {
    // Near-extremal retrograde grows toward 9M; at a* = 0.998 it sits near 8.97M.
    let metric = Kerr::new(1.0, 0.998);
    let r = metric.isco(Orbit::Retrograde);
    assert!(r > 8.5 && r < 9.0, "a*=0.998 retrograde isco={r} outside (8.5M, 9M)");
}
