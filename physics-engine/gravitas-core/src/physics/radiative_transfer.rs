//! Spectrally-resolved radiative transfer along a null geodesic.
//!
//! The radiative-transfer equation in the comoving frame is
//!
//!   dI_ν / dλ = j_ν − α_ν I_ν
//!
//! per Younsi, Zhidenko & Rezzolla (2016, *Phys. Rev. D* 94, 084025,
//! Eq. 14). Integrating it along a discretised geodesic path gives the
//! observed specific intensity at the chosen band. This module ships
//! the canonical 5-band frequency grid (matching the EHT 230 GHz
//! observation point and bracketing it with radio, sub-mm, optical,
//! and UV anchors) plus a per-band integrator that uses the
//! analytic step solution
//!
//!   I_{n+1} = I_n e^{−α dλ} + j ⋅ (1 − e^{−α dλ}) / α
//!
//! which is exact for piecewise-constant (j, α) over each segment and
//! degenerates correctly when α → 0.
//!
//! Out of scope for this module: the per-frame plasma model (n_e,
//! T_e, B, θ_B) that feeds j_ν and α_ν. Synchrotron emissivity per
//! Pandya, Zhdankin, Chandra & Quataert (2016, *ApJ* 822, 34) is the
//! standard fit and is wired in a follow-up because it pulls in
//! modified Bessel function approximations that deserve their own
//! review. Callers of [`integrate_radiative_transfer`] supply
//! whatever (j_ν, α_ν) source they want; the integrator's
//! correctness is independent of the choice.

/// A spectral band identified by its central frequency.
///
/// Frequencies are in Hz so callers can chain ν → λ → photon energy
/// without unit confusion. `label` is operator-facing.
#[derive(Clone, Copy, Debug)]
pub struct Band {
    pub freq_hz: f64,
    pub label: &'static str,
}

/// Five canonical bands bracketing the EHT observation point.
///
/// Anchors:
/// - 1.4 GHz: radio continuum (low frequency, optically thick limit).
/// - 230 GHz: Event Horizon Telescope band.
/// - 100 THz: sub-millimetre / mid-infrared.
/// - 500 THz: optical (≈ 600 nm).
/// - 1 PHz: ultraviolet.
pub const BANDS_5: [Band; 5] = [
    Band {
        freq_hz: 1.4e9,
        label: "radio_1.4GHz",
    },
    Band {
        freq_hz: 230.0e9,
        label: "EHT_230GHz",
    },
    Band {
        freq_hz: 100.0e12,
        label: "submm_IR_100THz",
    },
    Band {
        freq_hz: 500.0e12,
        label: "optical_500THz",
    },
    Band {
        freq_hz: 1.0e15,
        label: "UV_1PHz",
    },
];

/// Three-band reduced grid for tier-2 hardware: keeps radio, the EHT
/// observational anchor, and an optical sample. Indices into [`BANDS_5`].
pub const BANDS_3_INDICES: [usize; 3] = [0, 1, 3];

/// Single broadband fallback for tier-1 hardware. The chosen frequency
/// (230 GHz) matches the EHT primary band so the broadband renderer
/// stays comparable to the observational baseline.
pub const BAND_BROADBAND: Band = Band {
    freq_hz: 230.0e9,
    label: "broadband_230GHz",
};

/// One step of the radiative-transfer integration: a piecewise-
/// constant emissivity / absorption pair and the affine-parameter
/// step length over which they apply.
///
/// `j_nu` is the emissivity in the comoving frame (units: erg s⁻¹
/// cm⁻³ Hz⁻¹ sr⁻¹ in CGS, or whatever consistent system the caller
/// uses). `alpha_nu` is the absorption coefficient (cm⁻¹). `d_lambda`
/// is the affine-parameter step (cm in Younsi+ 2016's normalisation).
///
/// Negative or non-finite values are accepted by the integrator and
/// will produce NaN downstream; callers are responsible for sanitising
/// their plasma model.
#[derive(Clone, Copy, Debug)]
pub struct RadiativeTransferSample {
    pub j_nu: f64,
    pub alpha_nu: f64,
    pub d_lambda: f64,
}

/// Threshold below which we treat α dλ as effectively zero and use
/// the optically-thin limit I → I + j dλ. This bound keeps the
/// exponential expansion accurate to better than one part in 10¹².
const OPTICAL_DEPTH_FLOOR: f64 = 1.0e-12;

/// Integrate dI/dλ = j − α I along a sampled path.
///
/// Per-step solution: I_{n+1} = I_n e^{−τ} + S (1 − e^{−τ}), where
/// τ = α_n dλ_n is the optical depth across the segment and
/// S = j_n / α_n is the source function. The function falls back to
/// I + j dλ when τ < `OPTICAL_DEPTH_FLOOR`, the small-τ Taylor
/// expansion of the same expression.
///
/// Initial intensity defaults to zero (no background); callers that
/// want a background source provide it via `initial_intensity`.
#[must_use]
pub fn integrate_radiative_transfer(
    samples: &[RadiativeTransferSample],
    initial_intensity: f64,
) -> f64 {
    let mut intensity = initial_intensity;
    for sample in samples {
        let tau = sample.alpha_nu * sample.d_lambda;
        if tau.abs() < OPTICAL_DEPTH_FLOOR {
            intensity += sample.j_nu * sample.d_lambda;
        } else {
            let exp_neg_tau = (-tau).exp();
            let source = sample.j_nu / sample.alpha_nu;
            intensity = intensity * exp_neg_tau + source * (1.0 - exp_neg_tau);
        }
    }
    intensity
}

/// Convenience wrapper: integrate every band in `bands` against the
/// same path of samples, where the caller has produced one sample
/// stream per band. Returns one intensity per band in the input order.
///
/// Panics in debug builds if the lengths don't match; releases let
/// the iterator zip silently truncate so unbalanced inputs degrade to
/// the shorter length rather than panicking in production.
#[must_use]
pub fn integrate_bands(
    bands: &[Band],
    per_band_samples: &[Vec<RadiativeTransferSample>],
    initial_intensity: f64,
) -> Vec<f64> {
    debug_assert_eq!(
        bands.len(),
        per_band_samples.len(),
        "band count must match per-band sample stream count",
    );
    bands
        .iter()
        .zip(per_band_samples.iter())
        .map(|(_, samples)| integrate_radiative_transfer(samples, initial_intensity))
        .collect()
}

/// Photon wavelength in metres for a band's frequency, computed via
/// λ = c / ν with c the SI speed of light in vacuum.
#[must_use]
pub fn wavelength_metres(band: Band) -> f64 {
    crate::constants::SI_C / band.freq_hz
}
