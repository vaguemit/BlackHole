//! Shared regression-test fixtures.
//!
//! Bardeen, J. M. (1973), "Timelike and null geodesics in the Kerr metric",
//! in *Black Holes* (eds. DeWitt & DeWitt), is the canonical reference for
//! analytic Kerr orbits. The closed-form expressions for the photon-sphere
//! critical impact parameter (Eq. 28 in the cited prograde-equatorial form)
//! and the ISCO (Eq. 38, sometimes credited Bardeen-Press-Teukolsky 1972)
//! are reproduced here as the regression source of truth, cross-checked
//! against MTW *Gravitation* Table 33.1 and the EHT Sgr A* / M87* spin
//! papers (which use the same formulas to bracket inferred spin).
//!
//! Drift threshold for E, L_z, Carter Q over 10^5 steps: 1e-9 absolute,
//! per the project's physics regression rule.

#![allow(dead_code)] // each integration test compiles its own copy

/// Solar-mass black hole in geometric units throughout (M = 1).
pub const MASS_M: f64 = 1.0;

/// Drift threshold for conservation invariants over `STEP_COUNT` steps.
pub const DRIFT_THRESHOLD: f64 = 1e-9;

/// Step budget for conservation runs. 10^5 matches the rule and keeps
/// release-mode wall time under a few seconds on a modern laptop.
pub const STEP_COUNT: usize = 100_000;

/// Tolerance for closed-form analytic predictions vs the engine's outputs.
pub const ANALYTIC_TOLERANCE: f64 = 1e-6;

/// Photon-sphere critical impact parameter for a prograde equatorial null
/// circular orbit (Bardeen 1973, Eq. 28, prograde branch):
///
///   b_c = -a + 6M cos( arccos(-a/M) / 3 )
///
/// Returns the analytic prediction for spin `a_star = a/M` with `M = 1`.
pub fn bardeen_b_c_prograde(a_star: f64) -> f64 {
    let a = a_star * MASS_M;
    -a + 6.0 * MASS_M * ((-a / MASS_M).acos() / 3.0).cos()
}

/// Retrograde equatorial photon-sphere critical impact parameter (Bardeen
/// 1973, Eq. 28, retrograde branch):
///
///   b_c = +a + 6M cos( arccos(+a/M) / 3 )
pub fn bardeen_b_c_retrograde(a_star: f64) -> f64 {
    let a = a_star * MASS_M;
    a + 6.0 * MASS_M * ((a / MASS_M).acos() / 3.0).cos()
}

/// Bardeen-Press-Teukolsky ISCO radius for prograde equatorial circular
/// orbits (Bardeen 1973 Eq. 38, prograde branch):
///
///   z_1 = 1 + (1 - a*^2)^(1/3) [ (1+a*)^(1/3) + (1-a*)^(1/3) ]
///   z_2 = sqrt( 3 a*^2 + z_1^2 )
///   r_isco = M ( 3 + z_2 - sqrt( (3 - z_1)(3 + z_1 + 2 z_2) ) )
pub fn bardeen_r_isco_prograde(a_star: f64) -> f64 {
    let a2 = a_star * a_star;
    let z1 =
        1.0 + (1.0 - a2).cbrt() * ((1.0 + a_star).cbrt() + (1.0 - a_star).cbrt());
    let z2 = (3.0 * a2 + z1 * z1).sqrt();
    MASS_M * (3.0 + z2 - ((3.0 - z1) * (3.0 + z1 + 2.0 * z2)).sqrt())
}

/// Retrograde branch of the Bardeen-Press-Teukolsky ISCO formula.
pub fn bardeen_r_isco_retrograde(a_star: f64) -> f64 {
    let a2 = a_star * a_star;
    let z1 =
        1.0 + (1.0 - a2).cbrt() * ((1.0 + a_star).cbrt() + (1.0 - a_star).cbrt());
    let z2 = (3.0 * a2 + z1 * z1).sqrt();
    MASS_M * (3.0 + z2 + ((3.0 - z1) * (3.0 + z1 + 2.0 * z2)).sqrt())
}

/// Outer event-horizon radius for Kerr: r_+ = M + sqrt(M^2 - a^2).
pub fn kerr_event_horizon(a_star: f64) -> f64 {
    let a = a_star * MASS_M;
    MASS_M + (MASS_M * MASS_M - a * a).sqrt()
}

/// Spin grid used across regression suites. Covers Schwarzschild (0),
/// moderate (0.5), high (0.9), near-extremal (0.99), and the EHT-cited
/// upper-bracket (0.998).
pub const REGRESSION_SPIN_GRID: [f64; 5] = [0.0, 0.5, 0.9, 0.99, 0.998];
