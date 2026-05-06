//! Younsi+ 2016 radiative-transfer integrator: optically-thin and
//! optically-thick limits, multi-step composition, and degenerate
//! input handling.
//!
//! What this suite proves:
//! - Empty path returns the initial intensity unchanged.
//! - Optically thin (α → 0): final intensity equals the discrete
//!   sum of j_ν · dλ over all segments, exactly.
//! - Optically thick (α dλ ≫ 1, j and α positive): output asymptotes
//!   to the source function S = j/α (Kirchhoff's law in LTE).
//! - Optical depth across a segment composes additively: a single
//!   step of length 2L gives the same I as two steps of length L.
//! - Background intensity attenuates by exp(−τ_total) when emissivity
//!   is zero: pure absorption recovers Beer-Lambert.
//! - Multi-band integration runs each band independently; results
//!   match calling the scalar integrator per band.
//! - Band frequencies match the published canonical anchors and the
//!   λ = c/ν conversion is consistent with SI_C.

use gravitas::physics::radiative_transfer::{
    integrate_bands, integrate_radiative_transfer, wavelength_metres, Band,
    RadiativeTransferSample, BANDS_3_INDICES, BANDS_5, BAND_BROADBAND,
};

const TIGHT: f64 = 1e-12;

fn close(a: f64, b: f64, eps: f64) -> bool {
    (a - b).abs() < eps
}

// ---------------------------------------------------------------------
// Empty / degenerate inputs
// ---------------------------------------------------------------------

#[test]
fn empty_path_returns_initial_intensity() {
    assert!(close(integrate_radiative_transfer(&[], 0.0), 0.0, TIGHT));
    assert!(close(integrate_radiative_transfer(&[], 3.7), 3.7, TIGHT));
}

#[test]
fn zero_step_is_identity() {
    let samples = vec![RadiativeTransferSample {
        j_nu: 5.0,
        alpha_nu: 2.0,
        d_lambda: 0.0,
    }];
    assert!(close(
        integrate_radiative_transfer(&samples, 1.5),
        1.5,
        TIGHT,
    ));
}

// ---------------------------------------------------------------------
// Optically-thin limit
// ---------------------------------------------------------------------

#[test]
fn optically_thin_recovers_emissivity_sum() {
    // α = 0 → exactly I_n+1 = I_n + j dλ. Three segments with mixed
    // j values, no background.
    let samples = vec![
        RadiativeTransferSample {
            j_nu: 0.5,
            alpha_nu: 0.0,
            d_lambda: 1.0,
        },
        RadiativeTransferSample {
            j_nu: 1.5,
            alpha_nu: 0.0,
            d_lambda: 2.0,
        },
        RadiativeTransferSample {
            j_nu: 0.25,
            alpha_nu: 0.0,
            d_lambda: 4.0,
        },
    ];
    let expected = 0.5 * 1.0 + 1.5 * 2.0 + 0.25 * 4.0;
    let actual = integrate_radiative_transfer(&samples, 0.0);
    assert!(close(actual, expected, TIGHT));
}

#[test]
fn optically_thin_preserves_initial_intensity() {
    // No emissivity, no absorption: I stays put.
    let samples = vec![
        RadiativeTransferSample {
            j_nu: 0.0,
            alpha_nu: 0.0,
            d_lambda: 5.0,
        };
        4
    ];
    assert!(close(
        integrate_radiative_transfer(&samples, 7.5),
        7.5,
        TIGHT,
    ));
}

#[test]
fn optically_thin_floor_kicks_in_for_tiny_optical_depth() {
    // α dλ = 1e-15 should fall through the floor branch and behave
    // as α = 0 within tolerance.
    let samples = vec![RadiativeTransferSample {
        j_nu: 1.0,
        alpha_nu: 1.0e-15,
        d_lambda: 1.0,
    }];
    let actual = integrate_radiative_transfer(&samples, 0.0);
    assert!(close(actual, 1.0, 1e-10));
}

// ---------------------------------------------------------------------
// Optically-thick limit
// ---------------------------------------------------------------------

#[test]
fn optically_thick_approaches_source_function() {
    // Single very-thick segment: I → S = j/α regardless of initial.
    let j = 3.0;
    let alpha = 1.0e3;
    let d_lambda = 0.1; // τ = 100, e^{-100} ≈ 0
    let samples = vec![RadiativeTransferSample {
        j_nu: j,
        alpha_nu: alpha,
        d_lambda,
    }];
    let s = j / alpha;
    let actual = integrate_radiative_transfer(&samples, 999.0);
    assert!(close(actual, s, 1e-9));
}

#[test]
fn optically_thick_isothermal_path_stays_at_source() {
    // After the first thick segment we're at S; subsequent thick
    // segments with the same (j, α) leave us there.
    let sample = RadiativeTransferSample {
        j_nu: 2.0,
        alpha_nu: 100.0,
        d_lambda: 0.5, // τ = 50
    };
    let s = sample.j_nu / sample.alpha_nu;
    let samples = vec![sample; 10];
    let actual = integrate_radiative_transfer(&samples, 0.0);
    assert!(close(actual, s, 1e-9));
}

// ---------------------------------------------------------------------
// Beer-Lambert (pure absorption)
// ---------------------------------------------------------------------

#[test]
fn pure_absorption_attenuates_by_exp_neg_tau() {
    // j = 0, α > 0: I → I_0 e^{-α dλ}.
    let alpha = 0.5;
    let d_lambda = 2.0;
    let samples = vec![RadiativeTransferSample {
        j_nu: 0.0,
        alpha_nu: alpha,
        d_lambda,
    }];
    let initial = 4.0;
    let expected = initial * (-(alpha * d_lambda)).exp();
    let actual = integrate_radiative_transfer(&samples, initial);
    assert!(close(actual, expected, 1e-12));
}

#[test]
fn beer_lambert_composes_across_steps() {
    // Two segments of (α, dλ) compose to total τ = α(dλ_1 + dλ_2).
    let alpha = 0.7;
    let initial = 2.0;
    let samples_split = vec![
        RadiativeTransferSample {
            j_nu: 0.0,
            alpha_nu: alpha,
            d_lambda: 1.0,
        },
        RadiativeTransferSample {
            j_nu: 0.0,
            alpha_nu: alpha,
            d_lambda: 1.5,
        },
    ];
    let samples_single = vec![RadiativeTransferSample {
        j_nu: 0.0,
        alpha_nu: alpha,
        d_lambda: 2.5,
    }];
    let split = integrate_radiative_transfer(&samples_split, initial);
    let single = integrate_radiative_transfer(&samples_single, initial);
    assert!(close(split, single, 1e-12));
}

// ---------------------------------------------------------------------
// Step composition
// ---------------------------------------------------------------------

#[test]
fn one_long_step_equals_two_short_steps_when_uniform() {
    // For piecewise-constant (j, α) the analytic per-step solution is
    // exact, so subdividing should not change the result.
    let j = 1.5;
    let alpha = 0.4;
    let total = 3.0;

    let one = integrate_radiative_transfer(
        &[RadiativeTransferSample {
            j_nu: j,
            alpha_nu: alpha,
            d_lambda: total,
        }],
        0.5,
    );
    let split = integrate_radiative_transfer(
        &[
            RadiativeTransferSample {
                j_nu: j,
                alpha_nu: alpha,
                d_lambda: total / 2.0,
            },
            RadiativeTransferSample {
                j_nu: j,
                alpha_nu: alpha,
                d_lambda: total / 2.0,
            },
        ],
        0.5,
    );
    assert!(close(one, split, 1e-12));
}

#[test]
fn analytic_step_solution_matches_closed_form() {
    // Exact closed form: I = I_0 e^{-τ} + S (1 - e^{-τ}).
    let j = 2.5;
    let alpha = 1.2;
    let d_lambda = 0.8;
    let initial = 0.6;
    let tau: f64 = alpha * d_lambda;
    let s = j / alpha;
    let expected = initial * (-tau).exp() + s * (1.0 - (-tau).exp());
    let actual = integrate_radiative_transfer(
        &[RadiativeTransferSample {
            j_nu: j,
            alpha_nu: alpha,
            d_lambda,
        }],
        initial,
    );
    assert!(close(actual, expected, 1e-13));
}

// ---------------------------------------------------------------------
// Multi-band wrapper
// ---------------------------------------------------------------------

#[test]
fn integrate_bands_returns_one_intensity_per_band() {
    let bands = [
        Band {
            freq_hz: 1.0e9,
            label: "low",
        },
        Band {
            freq_hz: 1.0e15,
            label: "high",
        },
    ];
    let per_band: Vec<Vec<RadiativeTransferSample>> = vec![
        vec![RadiativeTransferSample {
            j_nu: 1.0,
            alpha_nu: 0.0,
            d_lambda: 1.0,
        }],
        vec![RadiativeTransferSample {
            j_nu: 4.0,
            alpha_nu: 0.0,
            d_lambda: 0.5,
        }],
    ];
    let result = integrate_bands(&bands, &per_band, 0.0);
    assert_eq!(result.len(), 2);
    assert!(close(result[0], 1.0, TIGHT));
    assert!(close(result[1], 2.0, TIGHT));
}

#[test]
fn integrate_bands_matches_per_band_scalar_call() {
    let bands = BANDS_5;
    let per_band: Vec<Vec<RadiativeTransferSample>> = (0..5)
        .map(|i| {
            vec![RadiativeTransferSample {
                j_nu: 0.5 * (i as f64 + 1.0),
                alpha_nu: 0.1,
                d_lambda: 1.0,
            }]
        })
        .collect();
    let bulk = integrate_bands(&bands, &per_band, 0.0);
    for i in 0..5 {
        let scalar = integrate_radiative_transfer(&per_band[i], 0.0);
        assert!(
            close(bulk[i], scalar, 1e-12),
            "band {} mismatch: bulk={}, scalar={}",
            BANDS_5[i].label,
            bulk[i],
            scalar,
        );
    }
}

// ---------------------------------------------------------------------
// Band table
// ---------------------------------------------------------------------

#[test]
fn bands_5_anchors_match_canonical_frequencies() {
    // Floating-point literals may round; pin to a tight relative band.
    assert!((BANDS_5[0].freq_hz / 1.4e9 - 1.0).abs() < 1e-12);
    assert!((BANDS_5[1].freq_hz / 230.0e9 - 1.0).abs() < 1e-12);
    assert!((BANDS_5[2].freq_hz / 100.0e12 - 1.0).abs() < 1e-12);
    assert!((BANDS_5[3].freq_hz / 500.0e12 - 1.0).abs() < 1e-12);
    assert!((BANDS_5[4].freq_hz / 1.0e15 - 1.0).abs() < 1e-12);
}

#[test]
fn bands_5_strictly_increasing_in_frequency() {
    for w in BANDS_5.windows(2) {
        assert!(w[0].freq_hz < w[1].freq_hz);
    }
}

#[test]
fn bands_3_indices_select_radio_eht_optical() {
    assert_eq!(BANDS_3_INDICES, [0, 1, 3]);
    assert_eq!(BANDS_5[BANDS_3_INDICES[0]].label, "radio_1.4GHz");
    assert_eq!(BANDS_5[BANDS_3_INDICES[1]].label, "EHT_230GHz");
    assert_eq!(BANDS_5[BANDS_3_INDICES[2]].label, "optical_500THz");
}

#[test]
fn broadband_uses_eht_anchor_frequency() {
    assert!((BAND_BROADBAND.freq_hz / 230.0e9 - 1.0).abs() < 1e-12);
}

#[test]
fn wavelength_eht_band_in_millimetre_range() {
    // 230 GHz → λ ≈ 1.30 mm.
    let lambda = wavelength_metres(BANDS_5[1]);
    assert!(close(lambda, 0.001_303, 5.0e-6));
}

#[test]
fn wavelength_optical_band_near_600_nm() {
    // 500 THz → λ ≈ 599.6 nm.
    let lambda = wavelength_metres(BANDS_5[3]);
    assert!(close(lambda, 5.996e-7, 1.0e-9));
}
