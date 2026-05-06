//! Polarization primitives for null geodesics in Kerr spacetime.
//!
//! Three pieces. The [`StokesVector`] type carries the (I, Q, U, V)
//! Stokes parameters along with helpers that preserve their geometric
//! invariants. [`walker_penrose_kappa`] evaluates the Walker-Penrose
//! constant κ_WP that Walker & Penrose (1970, *Commun. Math. Phys.* 18,
//! 265, Eq. 2.3) showed is conserved along null geodesics in any
//! type-D spacetime, including Kerr. [`evpa_rotation`] produces the
//! electric-vector position-angle rotation Δχ between two evaluated
//! κ_WP values; combined with [`StokesVector::rotate_evpa`] it is the
//! transport law from emission to observation.
//!
//! What this module does *not* yet do: parallel-transport the
//! polarization 4-vector f^μ along the geodesic. That requires
//! extending the integrator to step (x, p, f) jointly using the
//! parallel-transport equation Df^μ/dλ = 0. Until that lands, callers
//! supply f^μ at both ends and this module composes the rotation; the
//! invariant is the κ_WP value, not the raw f^μ.
//!
//! References:
//! - Walker, M. & Penrose, R. (1970), "On Quadratic First Integrals
//!   of the Geodesic Equations for Type [22] Spacetimes", *Commun.
//!   Math. Phys.* 18, 265, Eq. 2.3.
//! - Connors, P. A. & Stark, R. F. (1977), "Observable gravitational
//!   effects on polarised radiation coming from near a black hole",
//!   *Nature* 269, 128, Eq. 4-6.
//! - Schnittman, J. D. & Krolik, J. H. (2009), "X-ray polarization
//!   from accreting black holes: the thermal state", *ApJ* 701, 1175,
//!   §2 for disk-emission Stokes initialization.

use num_complex::Complex64;

/// Stokes parameters describing a partially-polarised electromagnetic
/// signal. Conventions follow IAU 1974: I is total intensity, Q and U
/// describe linear polarisation (Q along the reference axis, U at 45°
/// to it), and V describes circular polarisation (positive = right-
/// handed when seen from the observer).
///
/// The physical constraints are I ≥ 0 and I² ≥ Q² + U² + V² (the
/// degree of polarisation cannot exceed unity). Values that violate
/// these are not rejected at construction; [`StokesVector::is_valid`]
/// reports the constraint state explicitly.
#[derive(Clone, Copy, Debug)]
pub struct StokesVector {
    /// Total intensity. Must be non-negative for physical signals.
    pub i: f64,
    /// Linear polarisation along the reference axis (Q = I_x - I_y).
    pub q: f64,
    /// Linear polarisation at 45° to the reference axis (U = I_a - I_b).
    pub u: f64,
    /// Circular polarisation (V = I_R - I_L).
    pub v: f64,
}

impl StokesVector {
    /// Unpolarised reference signal of unit intensity.
    pub const UNPOLARISED_UNIT: Self = Self {
        i: 1.0,
        q: 0.0,
        u: 0.0,
        v: 0.0,
    };

    /// Construct from raw Stokes components.
    #[must_use]
    pub const fn new(i: f64, q: f64, u: f64, v: f64) -> Self {
        Self { i, q, u, v }
    }

    /// Linear polarisation degree p_lin = sqrt(Q² + U²) / I.
    /// Returns 0.0 when I is zero (no signal).
    #[must_use]
    pub fn linear_polarisation_degree(&self) -> f64 {
        if self.i.abs() < f64::EPSILON {
            0.0
        } else {
            self.q.hypot(self.u) / self.i
        }
    }

    /// Total polarisation degree p = sqrt(Q² + U² + V²) / I.
    #[must_use]
    pub fn total_polarisation_degree(&self) -> f64 {
        if self.i.abs() < f64::EPSILON {
            0.0
        } else {
            (self.q * self.q + self.u * self.u + self.v * self.v).sqrt() / self.i
        }
    }

    /// Electric-vector position angle (EVPA) χ = (1/2) atan2(U, Q),
    /// measured in radians. Convention matches Connors+Stark 1977
    /// Eq. 4: χ = 0 when the polarisation aligns with the reference
    /// axis (positive Q). The factor 1/2 reflects that the Stokes Q,U
    /// representation is double-valued in the polarisation plane.
    #[must_use]
    pub fn evpa(&self) -> f64 {
        0.5 * self.u.atan2(self.q)
    }

    /// True iff I ≥ 0 and I² ≥ Q² + U² + V².
    /// Strict equality holds for fully-polarised radiation.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        if self.i.is_sign_negative() {
            return false;
        }
        let p2 = self.q * self.q + self.u * self.u + self.v * self.v;
        // Allow a small numerical slack (1e-12) so values produced by
        // unitary rotations of fully-polarised input still pass.
        p2 <= self.i * self.i * (1.0 + 1e-12)
    }

    /// Rotate the linear-polarisation plane by Δχ radians. Q and U
    /// transform as a spin-2 quantity:
    ///
    ///   Q' = Q cos(2 Δχ) - U sin(2 Δχ)
    ///   U' = Q sin(2 Δχ) + U cos(2 Δχ)
    ///
    /// I and V are invariant under this rotation. Returns a new
    /// vector; the original is unchanged.
    #[must_use]
    pub fn rotate_evpa(self, delta_chi: f64) -> Self {
        let two_chi = 2.0 * delta_chi;
        let (sin2, cos2) = two_chi.sin_cos();
        Self {
            i: self.i,
            q: self.q * cos2 - self.u * sin2,
            u: self.q * sin2 + self.u * cos2,
            v: self.v,
        }
    }
}

/// Walker-Penrose constant κ_WP for a null geodesic in Kerr spacetime.
///
/// The Walker-Penrose theorem (Walker & Penrose 1970, Eq. 2.3) states
/// that the complex scalar
///
///   κ_WP = (A − i B)(r − i a cos θ)
///
/// with
///
///   A = (p^t f^r − p^r f^t) + a sin²θ (p^r f^φ − p^φ f^r)
///   B = (r² + a²) (p^φ f^θ − p^θ f^φ) − a (p^t f^θ − p^θ f^t)
///
/// is invariant along any null geodesic with parallel-transported
/// polarisation 4-vector f^μ. The expression is given here in the
/// covariant Boyer-Lindquist tetrad and operates on the contravariant
/// momentum components.
///
/// Inputs are
/// - `position`: the four-position (t, r, θ, φ) in Boyer-Lindquist;
///   only r and θ enter the formula.
/// - `momentum_up`: the contravariant photon four-momentum p^μ.
/// - `f_up`: the contravariant polarisation four-vector f^μ. The
///   theorem assumes f is null and orthogonal to p; the function
///   does not enforce that — see [`is_polarisation_orthogonal_to_momentum`].
/// - `spin_a`: the geometric spin parameter a = a* M.
///
/// The return value is the unmodified κ_WP. Callers are expected to
/// take its argument (`Complex64::arg`) when computing the EVPA
/// rotation between two points along the geodesic.
#[must_use]
pub fn walker_penrose_kappa(
    position: [f64; 4],
    momentum_up: [f64; 4],
    f_up: [f64; 4],
    spin_a: f64,
) -> Complex64 {
    let r = position[1];
    let theta = position[2];
    let cos_theta = theta.cos();
    let sin_theta = theta.sin();
    let sin2_theta = sin_theta * sin_theta;
    let a = spin_a;

    let pt = momentum_up[0];
    let pr = momentum_up[1];
    let pth = momentum_up[2];
    let pphi = momentum_up[3];

    let ft = f_up[0];
    let fr = f_up[1];
    let fth = f_up[2];
    let fphi = f_up[3];

    let a_part = (pt * fr - pr * ft) + a * sin2_theta * (pr * fphi - pphi * fr);
    let b_part = (r * r + a * a) * (pphi * fth - pth * fphi) - a * (pt * fth - pth * ft);

    let scalar = Complex64::new(a_part, -b_part);
    let rho = Complex64::new(r, -a * cos_theta);
    scalar * rho
}

/// EVPA rotation Δχ induced by parallel transport from `kappa_emit`
/// at the emission point to `kappa_obs` at the observation point.
/// Walker & Penrose 1970 give κ_WP as conserved, so the difference in
/// arg(κ_WP) between the two points is the rotation of the linear-
/// polarisation plane in the chosen reference frame.
///
/// Returns 0.0 when either input has near-zero magnitude (treats the
/// rotation as undefined; the caller should report I → 0 separately).
#[must_use]
pub fn evpa_rotation(kappa_emit: Complex64, kappa_obs: Complex64) -> f64 {
    let mag_emit = kappa_emit.norm();
    let mag_obs = kappa_obs.norm();
    if mag_emit < f64::EPSILON || mag_obs < f64::EPSILON {
        return 0.0;
    }
    (kappa_obs / kappa_emit).arg()
}

/// True iff the polarisation 4-vector f^μ is orthogonal to the
/// photon 4-momentum p_μ (in the Lorentz-invariant inner product
/// f^μ p_μ = 0). This is a precondition of the Walker-Penrose
/// theorem; supplying a non-orthogonal pair gives a κ_WP value with
/// no physical meaning.
///
/// `momentum_down` is the covariant momentum p_μ. The function does
/// not require the metric to evaluate the inner product because we
/// already have the index-lowered form.
#[must_use]
pub fn is_polarisation_orthogonal_to_momentum(
    momentum_down: [f64; 4],
    f_up: [f64; 4],
    tolerance: f64,
) -> bool {
    let mut dot = 0.0;
    for i in 0..4 {
        dot += momentum_down[i] * f_up[i];
    }
    dot.abs() < tolerance
}

/// Initialise a Stokes vector at a thermal-disk emission point.
/// Schnittman & Krolik 2009 §2 model: optically-thick thermal
/// synchrotron from a magnetised disk produces linear polarisation
/// with degree p_lin set by the local viewing geometry (Chandrasekhar
/// 1960 limit ≈ 11.7 % at edge-on for pure scattering, lower for
/// thermal). Circular polarisation V is negligible for thermal
/// synchrotron and is set to zero.
///
/// The emitted EVPA is taken perpendicular to the local magnetic-
/// field projection in the disk plane (`b_field_phi_angle` is the
/// azimuth of B in radians). The caller supplies the local intensity
/// `i_local` (already including g-factor scaling and emissivity) and
/// the polarisation degree `p_lin` (the model parameter; values in
/// [0, 0.117] for thermal, up to ~0.7 for power-law synchrotron).
///
/// The convention matches Schnittman+Krolik 2009 Eq. 5: positive Q
/// means the EVPA aligns with the disk's azimuthal direction.
#[must_use]
pub fn initial_disk_stokes(
    i_local: f64,
    p_lin: f64,
    b_field_phi_angle: f64,
) -> StokesVector {
    let p = p_lin.clamp(0.0, 1.0);
    let chi = b_field_phi_angle + std::f64::consts::FRAC_PI_2;
    let two_chi = 2.0 * chi;
    let (sin2, cos2) = two_chi.sin_cos();
    StokesVector {
        i: i_local,
        q: i_local * p * cos2,
        u: i_local * p * sin2,
        v: 0.0,
    }
}
