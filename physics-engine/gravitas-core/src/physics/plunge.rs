//! ISCO plunging-stream entry state.
//!
//! Inside the innermost stable circular orbit no bound circular orbit
//! exists; matter that loses an infinitesimal amount of angular
//! momentum at the ISCO falls inward with the conserved energy
//! E_ISCO and axial angular momentum L_z_ISCO that it had at the
//! marginally-stable orbit. Bardeen, Press & Teukolsky (1972, *ApJ*
//! 178, 347) gave the closed-form values; Cunningham (1975, *ApJ*
//! 202, 788) wrote the radiative spectrum of the plunging stream;
//! Mummery & Balbus (2024, *MNRAS* 531, 1163) extended the time-domain
//! treatment.
//!
//! What this module ships: the entry-state primitives — E_ISCO,
//! L_z_ISCO, Ω_ISCO, plus the canonical binding-energy efficiency
//! η = 1 − E_ISCO that quantifies how much rest-mass energy a thin
//! disk can radiate. An analytic falloff for the plunge emissivity
//! is provided for renderer use.
//!
//! What it does NOT ship: the integrated timelike geodesic from ISCO
//! to the horizon. That requires extending the integrator to
//! propagate timelike orbits with H = −1/2 (the existing code path
//! enforces null normalisation H = 0). The trajectory primitive is
//! its own change.

use crate::geodesic::{
    integrate, GeodesicKind, GeodesicState, IntegrationMethod, IntegrationOptions, Trajectory,
};
use crate::metric::{Kerr, Metric, Orbit};

/// Bardeen-Press-Teukolsky 1972 specific energy of an equatorial
/// circular geodesic at radius r in Kerr spacetime, in the dimensionless
/// Bardeen 1973 form using v ≡ √(M/r):
///
///   E/μ = (1 − 2 v² ± a* v³) / √(1 − 3 v² ± 2 a* v³)
///
/// where a* = a/M and the upper sign is prograde, lower retrograde.
/// Outside the photon sphere the denominator under the square root
/// is positive; inside, no circular geodesic exists and the function
/// returns 1.0 (rest-mass energy with no orbital binding).
#[must_use]
pub fn circular_specific_energy(r: f64, m: f64, a: f64, orbit: Orbit) -> f64 {
    let rm = r / m;
    let v = (m / r).sqrt();
    let v3 = v * v * v;
    let a_star = a / m;
    let sign = match orbit {
        Orbit::Prograde => 1.0,
        Orbit::Retrograde => -1.0,
    };

    let num = 1.0 - 2.0 / rm + sign * a_star * v3;
    let den_sq = 1.0 - 3.0 / rm + sign * 2.0 * a_star * v3;

    if den_sq <= 0.0 {
        return 1.0;
    }
    num / den_sq.sqrt()
}

/// BPT 1972 specific axial angular momentum of an equatorial circular
/// geodesic at radius r in the Bardeen 1973 form:
///
///   L_z/(μM) = ±(1 ∓ 2 a* v³ + a*² v⁴) / [v · √(1 − 3 v² ± 2 a* v³)]
///
/// where v = √(M/r) and a* = a/M. Upper sign prograde, lower retrograde.
/// Returns 0.0 when no circular orbit exists at r.
#[must_use]
pub fn circular_specific_angular_momentum(
    r: f64,
    m: f64,
    a: f64,
    orbit: Orbit,
) -> f64 {
    let rm = r / m;
    let v = (m / r).sqrt();
    let v3 = v * v * v;
    let v4 = v3 * v;
    let a_star = a / m;
    let a_star_sq = a_star * a_star;
    let sign = match orbit {
        Orbit::Prograde => 1.0,
        Orbit::Retrograde => -1.0,
    };

    // L_z = sign · M · (1 - sign·2·a*·v³ + a*²·v⁴) / [v · √(1 - 3v² + sign·2·a*·v³)].
    // The √m · √r prefactor in equivalent forms equals M / v, which is what we use here.
    let num = sign * m * (1.0 - sign * 2.0 * a_star * v3 + a_star_sq * v4);
    let den_sq = 1.0 - 3.0 / rm + sign * 2.0 * a_star * v3;

    if den_sq <= 0.0 {
        return 0.0;
    }
    num / (v * den_sq.sqrt())
}

/// Keplerian angular velocity of an equatorial circular orbit:
///
///   Ω = ±√M / (r^{3/2} ± a √M)
#[must_use]
pub fn circular_angular_velocity(r: f64, m: f64, a: f64, orbit: Orbit) -> f64 {
    let sign = match orbit {
        Orbit::Prograde => 1.0,
        Orbit::Retrograde => -1.0,
    };
    let denom = r.powf(1.5) + sign * a * m.sqrt();
    if denom.abs() < f64::EPSILON {
        return 0.0;
    }
    sign * m.sqrt() / denom
}

/// Conserved (E, L_z, Ω) at the ISCO for the chosen orbit branch.
/// These are the integrals of motion that the plunging stream carries
/// inward from r_ISCO.
#[derive(Clone, Copy, Debug)]
pub struct PlungeEntryState {
    pub r_isco: f64,
    pub energy: f64,
    pub angular_momentum: f64,
    pub angular_velocity: f64,
}

#[must_use]
pub fn plunge_entry_state(metric: &Kerr, orbit: Orbit) -> PlungeEntryState {
    let r_isco = metric.isco(orbit);
    let m = metric.mass();
    let a = metric.a();
    PlungeEntryState {
        r_isco,
        energy: circular_specific_energy(r_isco, m, a, orbit),
        angular_momentum: circular_specific_angular_momentum(r_isco, m, a, orbit),
        angular_velocity: circular_angular_velocity(r_isco, m, a, orbit),
    }
}

/// Cunningham 1975 radiative efficiency η = 1 − E_ISCO: the fraction
/// of rest-mass energy a thin disk converts to radiation before the
/// gas plunges into the hole. Limits:
/// - Schwarzschild prograde: η = 1 − √(8/9) ≈ 5.72 %.
/// - Extremal prograde (a* → 1): η = 1 − √(1/3) ≈ 42.26 % (the
///   Thorne 1974 maximum-spin limit for matter accreted from a
///   thin disk; spin caps near 0.998 in practice because of photon
///   capture).
/// - Schwarzschild retrograde: η = 5.72 % (same as prograde at a=0).
/// - Extremal retrograde: η = 1 − √(25/27) ≈ 3.78 %.
#[must_use]
pub fn radiative_efficiency(metric: &Kerr, orbit: Orbit) -> f64 {
    let state = plunge_entry_state(metric, orbit);
    1.0 - state.energy
}

/// Integrate the timelike plunge trajectory from r_ISCO inward.
///
/// The infalling stream carries the conserved (E_ISCO, L_z_ISCO)
/// from the marginally stable orbit. The initial state is built by
/// dropping a small inward radial perturbation `dpr_seed` onto the
/// circular orbit at r_ISCO so the orbit is no longer stable and the
/// integrator can step inward.
///
/// The integrator is the existing adaptive RKF45 in timelike mode
/// (renormalises to H = −1/2 every `renormalize_interval` steps).
/// Termination reasons: horizon crossing (clean stop), max-steps
/// budget (caller's `max_steps`), or normalization failure (the
/// near-extremal case where r_ISCO is very close to r_+).
pub fn plunge_trajectory(
    metric: &Kerr,
    orbit: Orbit,
    dpr_seed: f64,
    max_steps: usize,
    record_path: bool,
) -> Trajectory {
    let entry = plunge_entry_state(metric, orbit);
    let theta_eq = std::f64::consts::FRAC_PI_2;

    // Equatorial circular initial state with conserved (E, L_z) from
    // the ISCO. p_t = -E by sign convention; p_θ = 0; p_r seeded
    // with a tiny inward perturbation so the orbit is not exactly
    // marginal.
    let initial = GeodesicState {
        x: [0.0, entry.r_isco, theta_eq, 0.0],
        p: [-entry.energy, -dpr_seed.abs(), 0.0, entry.angular_momentum],
    };

    let options = IntegrationOptions {
        method: IntegrationMethod::AdaptiveRKF45,
        tolerance: 1e-10,
        initial_step: 1e-3,
        max_steps,
        escape_radius: 1.0e6,
        renormalize_interval: 10,
        record_path,
        geodesic_kind: GeodesicKind::Timelike,
    };

    integrate(&initial, metric, &options)
}

/// Renderer-friendly emissivity envelope along the plunge path.
/// Returns 0 outside [r_+, r_ISCO], rises to 1 at r = r_ISCO, and
/// falls off with characteristic scale `falloff_scale_m` as r drops
/// toward the horizon. The shape is an exponential decay from the
/// ISCO inward; `falloff_scale_m` is in units of the BH mass M.
///
/// This is a visual envelope, not a derived radiative-transfer
/// solution; the actual plunging-stream spectrum needs the timelike
/// integration that is out of scope this PR. Callers that want a
/// physical model multiply this envelope by their own emissivity.
#[must_use]
pub fn plunge_emissivity_envelope(
    metric: &Kerr,
    r: f64,
    orbit: Orbit,
    falloff_scale_m: f64,
) -> f64 {
    let r_plus = metric.event_horizon();
    let r_isco = metric.isco(orbit);
    if r < r_plus || r > r_isco {
        return 0.0;
    }
    if falloff_scale_m <= 0.0 {
        return 0.0;
    }
    let m = metric.mass();
    let depth_below_isco = (r_isco - r) / m;
    (-depth_below_isco / falloff_scale_m).exp()
}
