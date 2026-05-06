//! Schwarzschild-degenerate path: at a* = 0 every Kerr-specific code
//! branch collapses to the Schwarzschild form. Closed-form expectations:
//!
//!   r_+ = 2M (event horizon)
//!   r_isco_prograde = 6M (Bardeen 1973 Eq. 38 collapses)
//!   r_isco_retrograde = 6M (both branches coincide at a = 0)
//!   r_ph = 3M (photon sphere)
//!   omega_ZAMO = 0 (no frame dragging at a = 0)
//!
//! A regression that introduces a 1/a division anywhere in the metric
//! tensor, frame-dragging code, or ISCO formula will fail at least one
//! of these assertions.

mod common;

use common::ANALYTIC_TOLERANCE;
use gravitas::metric::{Kerr, Metric, Orbit};

const STRICT_TOLERANCE: f64 = 1e-12;

#[test]
fn event_horizon_is_two_m() {
    let metric = Kerr::new(1.0, 0.0);
    let r_plus = metric.event_horizon();
    assert!(
        (r_plus - 2.0).abs() < STRICT_TOLERANCE,
        "r_+={r_plus}, expected 2M",
    );
}

#[test]
fn cauchy_horizon_collapses_to_zero() {
    let metric = Kerr::new(1.0, 0.0);
    let r_minus = metric.cauchy_horizon();
    assert!(
        r_minus.abs() < STRICT_TOLERANCE,
        "Cauchy horizon at a=0 should be 0, got {r_minus}",
    );
}

#[test]
fn isco_prograde_is_six_m() {
    let metric = Kerr::new(1.0, 0.0);
    let r = metric.isco(Orbit::Prograde);
    assert!(
        (r - 6.0).abs() < ANALYTIC_TOLERANCE,
        "Schwarzschild prograde isco={r}, expected 6M",
    );
}

#[test]
fn isco_retrograde_is_six_m() {
    let metric = Kerr::new(1.0, 0.0);
    let r = metric.isco(Orbit::Retrograde);
    assert!(
        (r - 6.0).abs() < ANALYTIC_TOLERANCE,
        "Schwarzschild retrograde isco={r}, expected 6M",
    );
}

#[test]
fn photon_sphere_is_three_m() {
    let metric = Kerr::new(1.0, 0.0);
    let r = metric.photon_sphere();
    assert!(
        (r - 3.0).abs() < ANALYTIC_TOLERANCE,
        "Schwarzschild photon sphere={r}, expected 3M",
    );
}

#[test]
fn frame_dragging_vanishes_everywhere_at_a_zero() {
    let metric = Kerr::new(1.0, 0.0);
    for &r in &[3.0, 5.0, 10.0, 50.0] {
        let omega = metric.frame_dragging_equator(r);
        assert!(
            omega.abs() < STRICT_TOLERANCE,
            "Schwarzschild frame dragging at r={r} should be 0, got {omega}",
        );
    }
}

#[test]
fn ergosphere_collapses_to_horizon_at_a_zero() {
    // r_ergo = M + sqrt(M^2 - a^2 cos^2 theta) -> 2M for a = 0 at any theta.
    let metric = Kerr::new(1.0, 0.0);
    for theta in [0.0, std::f64::consts::FRAC_PI_2, std::f64::consts::PI] {
        let r_ergo = metric.ergosphere(theta);
        assert!(
            (r_ergo - 2.0).abs() < STRICT_TOLERANCE,
            "Schwarzschild ergosphere at θ={theta} expected 2M, got {r_ergo}",
        );
    }
}
