//! Polarization primitives: Stokes vector algebra, Walker-Penrose
//! constant invariants, and the EVPA rotation operator.
//!
//! What this suite proves:
//! - Stokes invariants: I and V untouched by EVPA rotation, Q² + U²
//!   preserved exactly (rotation is a 2D orthogonal map on (Q, U)).
//! - rotate_evpa is unitary across the spin grid of test cases.
//! - rotate_evpa(0) is the identity; rotate_evpa(π) is the identity
//!   on physical Stokes (the (Q, U) double cover means a π rotation
//!   in EVPA is a 2π rotation in (Q, U) space).
//! - linear_polarisation_degree behaves as expected at boundaries.
//! - Walker-Penrose κ has the correct linearity in f^μ and the
//!   correct dependence on r and θ.
//! - evpa_rotation returns Δarg between two κ values, with
//!   degenerate inputs yielding 0.
//! - The orthogonality predicate accepts orthogonal pairs and
//!   rejects non-orthogonal pairs by tolerance.
//! - Disk-emission init produces a vector with the requested
//!   polarisation degree, perpendicular to the supplied B-field
//!   azimuth (Schnittman+Krolik 2009 Eq. 5 sign convention).
//!
//! What it does *not* prove (out of scope this PR):
//! - κ_WP conservation along an integrated null geodesic. That
//!   requires extending the integrator to parallel-transport f^μ
//!   alongside p^μ, which is a future change. The conservation is
//!   theorem-level; the per-point evaluation is what this suite
//!   exercises.

use gravitas::physics::polarization::{
    evpa_rotation, initial_disk_stokes, is_polarisation_orthogonal_to_momentum,
    walker_penrose_kappa, StokesVector,
};
use num_complex::Complex64;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};

const TIGHT: f64 = 1e-12;

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < TIGHT
}

// ---------------------------------------------------------------------
// StokesVector algebra
// ---------------------------------------------------------------------

#[test]
fn unpolarised_unit_has_zero_polarisation_degree() {
    let s = StokesVector::UNPOLARISED_UNIT;
    assert!(close(s.linear_polarisation_degree(), 0.0));
    assert!(close(s.total_polarisation_degree(), 0.0));
    assert!(s.is_valid());
}

#[test]
fn fully_linear_polarised_has_degree_one() {
    let s = StokesVector::new(1.0, 1.0, 0.0, 0.0);
    assert!(close(s.linear_polarisation_degree(), 1.0));
    assert!(s.is_valid());
}

#[test]
fn evpa_zero_for_q_axis_polarised() {
    let s = StokesVector::new(1.0, 0.5, 0.0, 0.0);
    assert!(close(s.evpa(), 0.0));
}

#[test]
fn evpa_quarter_pi_for_u_axis_polarised() {
    let s = StokesVector::new(1.0, 0.0, 0.5, 0.0);
    assert!(close(s.evpa(), FRAC_PI_4));
}

#[test]
fn rotate_evpa_preserves_intensity_and_circular() {
    let s = StokesVector::new(2.0, 0.4, 0.3, -0.1);
    for &chi in &[0.0, 0.1, FRAC_PI_4, FRAC_PI_2, PI, -0.7] {
        let r = s.rotate_evpa(chi);
        assert!(close(r.i, s.i), "I changed after EVPA rotation by {chi}");
        assert!(close(r.v, s.v), "V changed after EVPA rotation by {chi}");
    }
}

#[test]
fn rotate_evpa_preserves_linear_polarisation_magnitude() {
    let s = StokesVector::new(1.0, 0.6, 0.2, 0.0);
    let lin_before = s.q * s.q + s.u * s.u;
    for &chi in &[0.05, 0.5, 1.3, 2.7, -0.4] {
        let r = s.rotate_evpa(chi);
        let lin_after = r.q * r.q + r.u * r.u;
        assert!(
            (lin_after - lin_before).abs() < 1e-13,
            "Q² + U² drifted under rotation: {lin_before} → {lin_after}",
        );
    }
}

#[test]
fn rotate_evpa_zero_is_identity() {
    let s = StokesVector::new(1.5, 0.4, -0.1, 0.05);
    let r = s.rotate_evpa(0.0);
    assert!(close(r.i, s.i));
    assert!(close(r.q, s.q));
    assert!(close(r.u, s.u));
    assert!(close(r.v, s.v));
}

#[test]
fn rotate_evpa_pi_is_identity_on_qu() {
    // π rotation in EVPA is a 2π rotation in (Q, U) space, so the
    // physical state is identical to the input.
    let s = StokesVector::new(1.0, 0.4, -0.2, 0.0);
    let r = s.rotate_evpa(PI);
    assert!((r.q - s.q).abs() < 1e-12);
    assert!((r.u - s.u).abs() < 1e-12);
}

#[test]
fn rotate_evpa_quarter_pi_swaps_q_into_minus_u() {
    // EVPA rotation by π/4 corresponds to a (Q, U) rotation by π/2:
    //   Q' = Q cos(π/2) - U sin(π/2) = -U
    //   U' = Q sin(π/2) + U cos(π/2) = +Q
    let s = StokesVector::new(1.0, 1.0, 0.0, 0.0);
    let r = s.rotate_evpa(FRAC_PI_4);
    assert!(r.q.abs() < 1e-12, "Q after π/4 EVPA rotation = {}", r.q);
    assert!((r.u - 1.0).abs() < 1e-12, "U after π/4 EVPA rotation = {}", r.u);
}

#[test]
fn linear_degree_is_zero_for_zero_intensity() {
    let s = StokesVector::new(0.0, 0.0, 0.0, 0.0);
    assert_eq!(s.linear_polarisation_degree(), 0.0);
    assert_eq!(s.total_polarisation_degree(), 0.0);
}

#[test]
fn negative_intensity_fails_validity() {
    let s = StokesVector::new(-0.1, 0.0, 0.0, 0.0);
    assert!(!s.is_valid());
}

#[test]
fn over_polarised_state_fails_validity() {
    // Q² + U² + V² > I² is unphysical.
    let s = StokesVector::new(1.0, 1.0, 1.0, 1.0);
    assert!(!s.is_valid());
}

// ---------------------------------------------------------------------
// Walker-Penrose constant
// ---------------------------------------------------------------------

#[test]
fn walker_penrose_is_linear_in_polarisation_vector() {
    // κ_WP is bilinear in p^μ and f^μ; doubling f doubles κ.
    let position = [0.0, 8.0, FRAC_PI_2, 0.0];
    let momentum = [1.0, 0.05, 0.0, 0.04];
    let f1 = [0.0, 0.1, 0.05, 0.02];
    let f2 = [0.0, 0.2, 0.10, 0.04];
    let spin = 0.5;

    let k1 = walker_penrose_kappa(position, momentum, f1, spin);
    let k2 = walker_penrose_kappa(position, momentum, f2, spin);
    assert!((k2.re - 2.0 * k1.re).abs() < 1e-12);
    assert!((k2.im - 2.0 * k1.im).abs() < 1e-12);
}

#[test]
fn walker_penrose_vanishes_when_p_and_f_are_parallel() {
    // If f^μ ∝ p^μ then every (p, f) pair in the formula is zero.
    let position = [0.0, 6.0, FRAC_PI_2 - 0.1, 0.3];
    let momentum = [1.0, 0.1, 0.05, 0.07];
    let f_parallel = momentum;
    let kappa = walker_penrose_kappa(position, momentum, f_parallel, 0.7);
    assert!(kappa.norm() < 1e-12, "κ_WP should vanish for f ∝ p, got {kappa:?}");
}

#[test]
fn walker_penrose_picks_up_imaginary_component_off_equator() {
    // The (r − i a cos θ) factor produces a non-zero imaginary part
    // whenever a ≠ 0 and θ ≠ π/2, even for a real-valued (A − i B).
    let position_off = [0.0, 5.0, 0.7, 0.0];
    let momentum = [1.0, 0.0, 0.1, 0.05];
    let f_vec = [0.0, 0.2, 0.0, 0.0];
    let kappa = walker_penrose_kappa(position_off, momentum, f_vec, 0.9);
    assert!(kappa.im.abs() > 1e-9, "expected non-zero Im(κ) off equator");
}

#[test]
fn walker_penrose_real_at_schwarzschild_equator_for_pr_only() {
    // At a = 0, the (r − i a cos θ) factor is real (= r). Pick a
    // momentum and f-vector so that the (A − i B) factor is also
    // purely real (B = 0): set p^θ = f^θ = 0 and p^φ = f^φ = 0 so
    // every cross term in B drops out.
    let position = [0.0, 7.0, FRAC_PI_2, 0.0];
    let momentum = [1.0, 0.1, 0.0, 0.0];
    let f_vec = [0.5, 0.2, 0.0, 0.0];
    let kappa = walker_penrose_kappa(position, momentum, f_vec, 0.0);
    assert!(kappa.im.abs() < 1e-12, "expected real κ_WP, got {kappa:?}");
}

// ---------------------------------------------------------------------
// EVPA rotation operator
// ---------------------------------------------------------------------

#[test]
fn evpa_rotation_zero_when_kappa_unchanged() {
    let kappa = Complex64::new(0.4, 0.7);
    let chi = evpa_rotation(kappa, kappa);
    assert!(close(chi, 0.0));
}

#[test]
fn evpa_rotation_returns_phase_difference() {
    let kappa_emit = Complex64::from_polar(2.0, 0.3);
    let kappa_obs = Complex64::from_polar(1.5, 0.8);
    let chi = evpa_rotation(kappa_emit, kappa_obs);
    assert!(close(chi, 0.5));
}

#[test]
fn evpa_rotation_handles_zero_magnitude_inputs() {
    let kappa_emit = Complex64::new(0.0, 0.0);
    let kappa_obs = Complex64::new(1.0, 0.5);
    assert_eq!(evpa_rotation(kappa_emit, kappa_obs), 0.0);
    assert_eq!(evpa_rotation(kappa_obs, kappa_emit), 0.0);
}

#[test]
fn evpa_rotation_then_stokes_rotation_round_trip() {
    // Composing the two primitives: feed the EVPA rotation back into
    // the Stokes rotator; the result should be a unitary EVPA rotation
    // by exactly that angle, leaving I and V untouched and preserving
    // Q² + U².
    let kappa_emit = Complex64::from_polar(1.0, 0.0);
    let kappa_obs = Complex64::from_polar(1.0, 0.6);
    let chi = evpa_rotation(kappa_emit, kappa_obs);
    let s_emit = StokesVector::new(1.0, 0.5, 0.0, 0.05);
    let s_obs = s_emit.rotate_evpa(chi);
    let lin_in = s_emit.q.powi(2) + s_emit.u.powi(2);
    let lin_out = s_obs.q.powi(2) + s_obs.u.powi(2);
    assert!((lin_in - lin_out).abs() < 1e-13);
    assert!(close(s_obs.i, s_emit.i));
    assert!(close(s_obs.v, s_emit.v));
}

// ---------------------------------------------------------------------
// Orthogonality predicate
// ---------------------------------------------------------------------

#[test]
fn orthogonality_accepts_perpendicular_vectors() {
    let p_down = [1.0, 0.0, 0.0, 0.0];
    let f_up = [0.0, 1.0, 0.0, 0.0];
    assert!(is_polarisation_orthogonal_to_momentum(p_down, f_up, 1e-9));
}

#[test]
fn orthogonality_rejects_aligned_vectors() {
    let p_down = [1.0, 0.5, 0.2, 0.1];
    let f_up = [1.0, 0.5, 0.2, 0.1];
    assert!(!is_polarisation_orthogonal_to_momentum(p_down, f_up, 1e-9));
}

#[test]
fn orthogonality_respects_tolerance_for_small_inner_product() {
    let p_down = [1.0, 0.0, 0.0, 0.0];
    let f_up = [1e-12, 1.0, 0.0, 0.0];
    assert!(is_polarisation_orthogonal_to_momentum(p_down, f_up, 1e-9));
    assert!(!is_polarisation_orthogonal_to_momentum(p_down, f_up, 1e-15));
}

// ---------------------------------------------------------------------
// Disk emission initialiser
// ---------------------------------------------------------------------

#[test]
fn disk_emission_zero_intensity_yields_zero_stokes() {
    let s = initial_disk_stokes(0.0, 0.5, 0.0);
    assert_eq!(s.i, 0.0);
    assert_eq!(s.q, 0.0);
    assert_eq!(s.u, 0.0);
    assert_eq!(s.v, 0.0);
}

#[test]
fn disk_emission_zero_polarisation_yields_unpolarised_at_intensity() {
    let s = initial_disk_stokes(2.5, 0.0, 1.234);
    assert!(close(s.i, 2.5));
    assert!(close(s.q, 0.0));
    assert!(close(s.u, 0.0));
    assert!(close(s.v, 0.0));
}

#[test]
fn disk_emission_polarisation_degree_matches_input() {
    let i = 1.0;
    let p = 0.117; // Chandrasekhar 1960 thermal limit at edge-on
    let s = initial_disk_stokes(i, p, 0.3);
    assert!((s.linear_polarisation_degree() - p).abs() < 1e-12);
    assert!(close(s.v, 0.0));
}

#[test]
fn disk_emission_clamps_polarisation_to_unit_interval() {
    // Inputs above 1.0 or below 0.0 are clamped silently; the output
    // is still a valid Stokes vector.
    let s_high = initial_disk_stokes(1.0, 1.5, 0.0);
    let s_low = initial_disk_stokes(1.0, -0.1, 0.0);
    assert!(s_high.is_valid());
    assert!(s_low.is_valid());
    assert!(close(s_high.linear_polarisation_degree(), 1.0));
    assert!(close(s_low.linear_polarisation_degree(), 0.0));
}

#[test]
fn disk_emission_evpa_perpendicular_to_b_field() {
    // Schnittman+Krolik convention: the EVPA is at +π/2 to the local
    // B-field azimuth. Setting B along φ = 0 should put the EVPA at
    // π/2, which is the U axis (cos(π) = -1, sin(π) = 0; i.e., Q
    // negative, U near zero).
    let s = initial_disk_stokes(1.0, 0.5, 0.0);
    // 2χ = π → cos(2χ) = -1, sin(2χ) = 0 → Q = -i*p, U = 0
    assert!((s.q + 0.5).abs() < 1e-12, "Q should be -p, got {}", s.q);
    assert!(s.u.abs() < 1e-12, "U should be 0, got {}", s.u);
}
