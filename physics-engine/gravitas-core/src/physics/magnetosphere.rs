//! Wald's analytic vacuum magnetosphere for a Kerr black hole
//! immersed in a uniform asymptotic magnetic field.
//!
//! Wald (1974, *Phys. Rev. D* 10, 1680) showed that the spacetime
//! Killing vectors ξ = ∂_t and ψ = ∂_φ both produce divergence-free
//! 1-forms whose curl is a vacuum Maxwell solution; the linear
//! combination
//!
//!   A_μ = (B_0 / 2) (ψ_μ + 2 a ξ_μ)
//!
//! describes a uniform asymptotic field B_0 along the spin axis with
//! the spin-dragging correction that lets the field thread the
//! horizon without singularity. The horizon picks up a net charge of
//! magnitude 2 B_0 a M because of the dragging; this is the Wald
//! charge.
//!
//! The expressions used here are the Boyer-Lindquist forms with
//! signature (−,+,+,+) and metric components matching the existing
//! `Kerr` covariant tensor:
//!
//!   ψ_μ = g_{μ φ}              (the φ Killing vector, lowered)
//!   ξ_μ = g_{μ t}              (the t Killing vector, lowered)
//!
//!   A_t   = (B_0 / 2) (g_{t φ} + 2 a g_{t t})
//!   A_r   = 0                  (Killing vectors have no r component)
//!   A_θ   = 0                  (no θ component for axisymmetric vacuum)
//!   A_φ   = (B_0 / 2) (g_{φ φ} + 2 a g_{φ t})
//!
//! Out of scope here: the Blandford-Znajek 1977 split-monopole and
//! paraboloidal solutions for jet launching, the field-line
//! visualisation, and the field-tensor F_μν derivatives. Those land
//! in follow-up changes once a downstream consumer needs them.

use crate::metric::{Kerr, Metric};

/// Lowered-index electromagnetic 4-potential in Boyer-Lindquist
/// coordinates for the Wald solution. Indices in the returned array
/// are [A_t, A_r, A_θ, A_φ].
#[must_use]
pub fn wald_potential_down(metric: &Kerr, b0: f64, r: f64, theta: f64) -> [f64; 4] {
    let g = metric.covariant(r, theta);
    let g_tt = g.get(0, 0);
    let g_tphi = g.get(0, 3);
    let g_phiphi = g.get(3, 3);
    let a = metric.a();
    let half_b0 = 0.5 * b0;

    let a_t = half_b0 * (g_tphi + 2.0 * a * g_tt);
    let a_phi = half_b0 * (g_phiphi + 2.0 * a * g_tphi);

    [a_t, 0.0, 0.0, a_phi]
}

/// Asymptotic magnetic-field strength (Cartesian-z component) in the
/// flat region. At r → ∞, the Wald potential reduces to A_φ ≈
/// (B_0 / 2) r² sin²θ, the vector potential of a uniform field B_0
/// along the spin axis. This function returns the asymptotic Bz
/// recovered from a finite-radius A_φ sample, for sanity tests and
/// rendering normalisation.
#[must_use]
pub fn asymptotic_b_z_from_potential(r: f64, theta: f64, a_phi: f64) -> f64 {
    let sin_theta = theta.sin();
    let denom = r * r * sin_theta * sin_theta;
    if denom < f64::EPSILON {
        return 0.0;
    }
    2.0 * a_phi / denom
}

/// Wald horizon charge q_W = 2 B_0 a M induced by spin-dragging of
/// field lines through the horizon. Wald 1974 §III shows that the
/// vacuum solution is the *uniqueness* solution for a neutral hole;
/// astrophysical holes screen this charge through plasma but the
/// vacuum value is still the right reference scale.
#[must_use]
pub fn wald_horizon_charge(metric: &Kerr, b0: f64) -> f64 {
    2.0 * b0 * metric.a() * metric.mass()
}

/// Numerical electromagnetic field tensor F_μν = ∂_μ A_ν − ∂_ν A_μ
/// at (r, θ). Computes the partials by central differences on
/// [`wald_potential_down`] with the supplied step `eps`. The result
/// is antisymmetric: F[i][i] = 0 and F[i][j] = -F[j][i] within
/// numerical tolerance.
///
/// Because A_t and A_φ are the only non-zero components and the
/// solution is stationary + axisymmetric, only F_{tr}, F_{tθ},
/// F_{rφ}, and F_{θφ} are non-zero by construction; the time and
/// φ derivatives both return zero exactly without any difference
/// step.
#[must_use]
pub fn wald_field_tensor(
    metric: &Kerr,
    b0: f64,
    r: f64,
    theta: f64,
    eps: f64,
) -> [[f64; 4]; 4] {
    let a_plus_r = wald_potential_down(metric, b0, r + eps, theta);
    let a_minus_r = wald_potential_down(metric, b0, r - eps, theta);
    let a_plus_th = wald_potential_down(metric, b0, r, theta + eps);
    let a_minus_th = wald_potential_down(metric, b0, r, theta - eps);

    let mut f = [[0.0_f64; 4]; 4];

    // ∂_r A_μ = (A_μ(r+ε) - A_μ(r-ε)) / (2ε); μ ∈ {t, φ}.
    let d_a_dr_t = (a_plus_r[0] - a_minus_r[0]) / (2.0 * eps);
    let d_a_dr_phi = (a_plus_r[3] - a_minus_r[3]) / (2.0 * eps);
    let d_a_dth_t = (a_plus_th[0] - a_minus_th[0]) / (2.0 * eps);
    let d_a_dth_phi = (a_plus_th[3] - a_minus_th[3]) / (2.0 * eps);

    // F_μν = ∂_μ A_ν - ∂_ν A_μ. Time and φ partials are zero
    // because the solution is stationary + axisymmetric and A_r = A_θ
    // = 0.
    f[1][0] = d_a_dr_t; // F_{rt} = ∂_r A_t
    f[0][1] = -f[1][0];
    f[1][3] = d_a_dr_phi; // F_{rφ} = ∂_r A_φ
    f[3][1] = -f[1][3];
    f[2][0] = d_a_dth_t; // F_{θt}
    f[0][2] = -f[2][0];
    f[2][3] = d_a_dth_phi; // F_{θφ}
    f[3][2] = -f[2][3];

    f
}
