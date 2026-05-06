//! Conservation invariants for null geodesics in Kerr spacetime.
//!
//! For an unperturbed null geodesic the energy E = -p_t, axial angular
//! momentum L_z = p_phi, and Carter constant Q are all integrals of
//! motion. This suite asserts that the engine's integrator preserves
//! all three within `DRIFT_THRESHOLD` over `STEP_COUNT` integration steps.
//!
//! Reference geodesic: an equatorial null ray launched at r = 20M with a
//! tangential momentum tuned so the orbit is bound but does not cross
//! the horizon over the test budget. The same setup is run at three spin
//! values across the regression spin grid (Schwarzschild, moderate Kerr,
//! near-extremal) so that a regression in the metric, the Hamiltonian
//! derivative, the integrator, or the renormalization step shows up as
//! drift in at least one configuration.

mod common;

use common::{DRIFT_THRESHOLD, STEP_COUNT};
use gravitas::geodesic::{
    integrate, GeodesicKind, GeodesicState, IntegrationMethod, IntegrationOptions, TerminationReason,
};
use gravitas::invariants::compute_constants;
use gravitas::metric::Kerr;

/// Build an outward-launched null ray in the equatorial plane with the
/// given axial angular momentum. The integrator's renormalization step
/// pins p_r so the state is genuinely null (H = 0) after construction.
fn equatorial_null_ray(r: f64, l_z: f64) -> GeodesicState {
    GeodesicState {
        x: [0.0, r, std::f64::consts::FRAC_PI_2, 0.0],
        p: [-1.0, 0.0, 0.0, l_z],
    }
}

fn run_long_integration(spin: f64, l_z: f64) -> (f64, f64, f64) {
    let metric = Kerr::new(1.0, spin);
    let initial = equatorial_null_ray(20.0, l_z);
    let options = IntegrationOptions {
        method: IntegrationMethod::AdaptiveRKF45,
        tolerance: 1e-10,
        initial_step: 0.01,
        max_steps: STEP_COUNT,
        escape_radius: 1.0e6,
        renormalize_interval: 10,
        record_path: false,
        geodesic_kind: GeodesicKind::Null,
    };

    let traj = integrate(&initial, &metric, &options);
    let initial_constants = compute_constants(&initial, &metric);
    let final_constants = compute_constants(&traj.final_state, &metric);

    if matches!(traj.termination, TerminationReason::NormalizationFailure) {
        panic!("integrator hit NormalizationFailure at step {}", traj.steps_taken);
    }

    let drift_e = (final_constants.energy - initial_constants.energy).abs();
    let drift_lz =
        (final_constants.angular_momentum - initial_constants.angular_momentum).abs();
    let drift_q =
        (final_constants.carter_constant - initial_constants.carter_constant).abs();
    (drift_e, drift_lz, drift_q)
}

#[test]
fn schwarzschild_equatorial_conserves_invariants() {
    let (de, dlz, dq) = run_long_integration(0.0, 4.0);
    assert!(de < DRIFT_THRESHOLD, "Schwarzschild ΔE={de} exceeds {DRIFT_THRESHOLD}");
    assert!(dlz < DRIFT_THRESHOLD, "Schwarzschild ΔL_z={dlz}");
    assert!(dq < DRIFT_THRESHOLD, "Schwarzschild ΔQ={dq}");
}

#[test]
fn moderate_kerr_equatorial_conserves_invariants() {
    let (de, dlz, dq) = run_long_integration(0.5, 4.0);
    assert!(de < DRIFT_THRESHOLD, "a*=0.5 ΔE={de} exceeds {DRIFT_THRESHOLD}");
    assert!(dlz < DRIFT_THRESHOLD, "a*=0.5 ΔL_z={dlz}");
    assert!(dq < DRIFT_THRESHOLD, "a*=0.5 ΔQ={dq}");
}

#[test]
fn near_extremal_kerr_equatorial_conserves_invariants() {
    // Near-extremal needs a slightly higher angular momentum to stay
    // outside the rapidly-shrinking ergosphere over the long step budget.
    let (de, dlz, dq) = run_long_integration(0.99, 5.0);
    assert!(de < DRIFT_THRESHOLD, "a*=0.99 ΔE={de} exceeds {DRIFT_THRESHOLD}");
    assert!(dlz < DRIFT_THRESHOLD, "a*=0.99 ΔL_z={dlz}");
    assert!(dq < DRIFT_THRESHOLD, "a*=0.99 ΔQ={dq}");
}
