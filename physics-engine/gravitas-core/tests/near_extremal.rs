//! Near-extremal Kerr stability: at a* = 0.99 the outer horizon collapses
//! toward M and the integrator is most likely to step into the horizon
//! or hit the renormalization band defined in `invariants::renormalization`.
//! Property-based test asserts that for a wide spread of equatorial-ish
//! initial conditions the integrator either escapes, terminates cleanly,
//! or returns NormalizationFailure: never NaN, never panic, never silent
//! divergence.

mod common;

use gravitas::geodesic::{
    integrate, GeodesicKind, GeodesicState, IntegrationMethod, IntegrationOptions, TerminationReason,
};
use gravitas::metric::Kerr;
use proptest::prelude::*;

/// Deliberately tight step budget to keep the proptest run under a few
/// seconds; the property under test is "no NaN within budget", not
/// "all rays escape".
const NEAR_EXTREMAL_STEP_BUDGET: usize = 2000;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 200,
        max_shrink_iters: 64,
        ..ProptestConfig::default()
    })]

    #[test]
    fn near_extremal_no_nan_for_random_inputs(
        r0 in 6.0_f64..30.0,
        theta0 in 0.2_f64..(std::f64::consts::PI - 0.2),
        p_r in -0.5_f64..0.5,
        p_theta in -0.3_f64..0.3,
        l_z in 1.0_f64..6.0,
    ) {
        let metric = Kerr::new(1.0, 0.99);
        let initial = GeodesicState {
            x: [0.0, r0, theta0, 0.0],
            p: [-1.0, p_r, p_theta, l_z],
        };

        let options = IntegrationOptions {
            method: IntegrationMethod::AdaptiveRKF45,
            tolerance: 1e-9,
            initial_step: 1e-3,
            max_steps: NEAR_EXTREMAL_STEP_BUDGET,
            escape_radius: 1.0e6,
            renormalize_interval: 5,
            record_path: false,
            geodesic_kind: GeodesicKind::Null,
        };

        let traj = integrate(&initial, &metric, &options);

        // Whatever termination reason fired, the final state must be finite.
        let s = &traj.final_state;
        prop_assert!(s.x.iter().all(|v| v.is_finite()), "x had non-finite: {:?}", s.x);
        prop_assert!(s.p.iter().all(|v| v.is_finite()), "p had non-finite: {:?}", s.p);

        // NormalizationFailure / Horizon / Escape / MaxSteps are all OK; the
        // contract is just "terminated cleanly without garbage state".
        prop_assert!(
            traj.termination != TerminationReason::None,
            "integrator returned None termination",
        );
    }
}
