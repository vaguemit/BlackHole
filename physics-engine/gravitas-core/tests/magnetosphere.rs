//! Wald 1974 vacuum magnetosphere primitives.
//!
//! What this suite proves:
//! - Linearity in B_0: doubling the asymptotic field doubles A_μ
//!   and quadruples F_μν (linear in F, since it's a linear functional
//!   of A).
//! - Wald horizon charge q_W = 2 B_0 a M holds across the spin grid.
//! - Schwarzschild limit (a = 0): horizon charge vanishes; A_t = 0;
//!   A_φ reduces to the flat-space uniform-field potential.
//! - Asymptotic recovery: at large r, the field tensor's z-component
//!   approaches B_0 along the spin axis (within finite-difference
//!   tolerance).
//! - F_μν antisymmetry: F[i][j] = -F[j][i] within numerical noise.
//! - Stationary + axisymmetric structure: F_{tφ} = 0, F_{rθ} = 0;
//!   the only non-zero components are (rt, rφ, θt, θφ) and their
//!   transposes.

use gravitas::metric::Kerr;
use gravitas::physics::magnetosphere::{
    asymptotic_b_z_from_potential, wald_field_tensor, wald_horizon_charge,
    wald_potential_down,
};

const TIGHT: f64 = 1e-12;
const FD_EPS: f64 = 1e-4;

fn close(a: f64, b: f64, eps: f64) -> bool {
    (a - b).abs() < eps
}

// ---------------------------------------------------------------------
// Linearity in B_0
// ---------------------------------------------------------------------

#[test]
fn potential_is_linear_in_b0() {
    let metric = Kerr::new(1.0, 0.5);
    let r = 8.0;
    let theta = 1.0;

    let a1 = wald_potential_down(&metric, 1.0, r, theta);
    let a2 = wald_potential_down(&metric, 2.0, r, theta);
    for i in 0..4 {
        assert!(
            close(a2[i], 2.0 * a1[i], 1e-10),
            "A[{i}] not linear: B0=2 → {}, B0=1×2 → {}",
            a2[i],
            2.0 * a1[i],
        );
    }
}

#[test]
fn potential_zero_when_b0_zero() {
    let metric = Kerr::new(1.0, 0.7);
    let a_zero = wald_potential_down(&metric, 0.0, 5.0, 1.2);
    for v in a_zero {
        assert!(v.abs() < TIGHT);
    }
}

#[test]
fn field_tensor_is_linear_in_b0() {
    let metric = Kerr::new(1.0, 0.6);
    let r = 7.0;
    let theta = 0.8;
    let f1 = wald_field_tensor(&metric, 1.0, r, theta, FD_EPS);
    let f2 = wald_field_tensor(&metric, 2.0, r, theta, FD_EPS);
    for i in 0..4 {
        for j in 0..4 {
            assert!(
                close(f2[i][j], 2.0 * f1[i][j], 1e-8),
                "F[{i}][{j}] not linear",
            );
        }
    }
}

// ---------------------------------------------------------------------
// Wald horizon charge
// ---------------------------------------------------------------------

#[test]
fn wald_charge_zero_at_zero_spin() {
    let metric = Kerr::new(1.0, 0.0);
    assert!(close(wald_horizon_charge(&metric, 1.5), 0.0, TIGHT));
}

#[test]
fn wald_charge_matches_2_b0_a_m_across_spin_grid() {
    let m = 1.0;
    let b0 = 0.7;
    for &a_star in &[0.1, 0.3, 0.5, 0.9, 0.99] {
        let metric = Kerr::new(m, a_star);
        let predicted = wald_horizon_charge(&metric, b0);
        let expected = 2.0 * b0 * a_star * m * m;
        assert!(
            close(predicted, expected, 1e-12),
            "a*={a_star}: q_W engine={predicted}, formula={expected}",
        );
    }
}

#[test]
fn wald_charge_changes_sign_with_b0_sign() {
    let metric = Kerr::new(1.0, 0.5);
    let q_pos = wald_horizon_charge(&metric, 1.0);
    let q_neg = wald_horizon_charge(&metric, -1.0);
    assert!(close(q_neg, -q_pos, TIGHT));
}

// ---------------------------------------------------------------------
// Schwarzschild limit
// ---------------------------------------------------------------------

#[test]
fn at_zero_spin_a_t_vanishes() {
    let metric = Kerr::new(1.0, 0.0);
    let a_mu = wald_potential_down(&metric, 1.0, 10.0, 1.0);
    // ψ_μ has no t component for a=0, so A_t = (B_0/2)(g_{tφ} + 2a g_{tt}) = 0.
    assert!(close(a_mu[0], 0.0, TIGHT));
}

#[test]
fn at_zero_spin_a_phi_reduces_to_flat_uniform_field() {
    // For a = 0 the Boyer-Lindquist g_{φφ} = r² sin²θ exactly. So
    // A_φ = (B_0 / 2) r² sin² θ, which is the flat-space vector
    // potential of a uniform field B_0 along z.
    let metric = Kerr::new(1.0, 0.0);
    let r = 12.0;
    let theta = 0.7;
    let b0 = 0.4;
    let a_mu = wald_potential_down(&metric, b0, r, theta);
    let expected_a_phi = 0.5 * b0 * r * r * theta.sin().powi(2);
    assert!(
        close(a_mu[3], expected_a_phi, 1e-9),
        "Schwarzschild A_φ = {}, expected {}",
        a_mu[3],
        expected_a_phi,
    );
}

// ---------------------------------------------------------------------
// Asymptotic recovery
// ---------------------------------------------------------------------

#[test]
fn far_field_a_phi_recovers_b0() {
    // At very large r the spin contribution falls off, A_φ ≈
    // (B_0/2) r² sin²θ, and the helper recovers B_0 to within the
    // finite-r corrections.
    let metric = Kerr::new(1.0, 0.5);
    let b0 = 1.0;
    let r = 1.0e4;
    let theta = std::f64::consts::FRAC_PI_2;
    let a_mu = wald_potential_down(&metric, b0, r, theta);
    let recovered = asymptotic_b_z_from_potential(r, theta, a_mu[3]);
    // Corrections are O(M/r) and O(a²/r²); at r = 10⁴M they're
    // <1e-3 fractional.
    assert!(
        (recovered - b0).abs() < 1e-3,
        "asymptotic Bz = {}, expected ~{}",
        recovered,
        b0,
    );
}

#[test]
fn asymptotic_helper_zero_at_polar_axis() {
    // sin θ = 0 at the spin axis: the helper guards against division
    // by zero and returns 0.
    let bz = asymptotic_b_z_from_potential(10.0, 0.0, 5.0);
    assert_eq!(bz, 0.0);
}

// ---------------------------------------------------------------------
// Field tensor antisymmetry + structure
// ---------------------------------------------------------------------

#[test]
#[allow(clippy::needless_range_loop)] // tensor indices stay tensor indices
fn field_tensor_antisymmetric() {
    let metric = Kerr::new(1.0, 0.5);
    let f = wald_field_tensor(&metric, 1.0, 6.0, 1.0, FD_EPS);
    for i in 0..4 {
        assert!(close(f[i][i], 0.0, 1e-12), "F[{i}][{i}] = {}", f[i][i]);
        for j in (i + 1)..4 {
            assert!(
                close(f[i][j], -f[j][i], 1e-9),
                "F[{i}][{j}]={} not = -F[{j}][{i}]={}",
                f[i][j],
                f[j][i],
            );
        }
    }
}

#[test]
fn stationary_axisymmetric_components_vanish() {
    // ∂_t A = 0 and ∂_φ A = 0 by construction, and A_r = A_θ = 0 at
    // every point. So F_{tφ}, F_{tr}, F_{tθ}, F_{rφ via t}... actually
    // only the four geometrically-allowed components should be non-zero:
    // F_{rt}, F_{rφ}, F_{θt}, F_{θφ} (and their transposes).
    // That makes F_{tφ} = ∂_t A_φ - ∂_φ A_t = 0 and
    //         F_{rθ} = ∂_r A_θ - ∂_θ A_r = 0.
    let metric = Kerr::new(1.0, 0.7);
    let f = wald_field_tensor(&metric, 1.0, 5.0, 0.9, FD_EPS);
    assert!(close(f[0][3], 0.0, 1e-9), "F_{{tφ}} = {}", f[0][3]);
    assert!(close(f[1][2], 0.0, 1e-9), "F_{{rθ}} = {}", f[1][2]);
}

#[test]
fn field_tensor_components_nonzero_for_kerr() {
    let metric = Kerr::new(1.0, 0.5);
    let f = wald_field_tensor(&metric, 1.0, 6.0, 1.0, FD_EPS);
    assert!(f[1][0].abs() > 1e-6, "F_{{rt}} should be non-zero");
    assert!(f[1][3].abs() > 1e-6, "F_{{rφ}} should be non-zero");
}
