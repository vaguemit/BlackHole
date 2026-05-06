#![warn(clippy::pedantic, clippy::nursery)]
// Allows below carry justification per rule 03 §6. Each preserves the
// canonical mathematical form of formulas as they appear in published
// references (Bardeen 1973, Carter 1968, MTW Gravitation, Page 1976).
#![allow(
    // Single-char names match GR notation (a, M, r, theta, phi, g_tt, etc.).
    clippy::similar_names,
    clippy::many_single_char_names,
    // f64 casts match the paper notation; spurious in numerical code.
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    // Module names mirror physics subdomains (metric::kerr, geodesic::geodesic).
    clippy::module_name_repetitions,
    // Long-form physics doc comments lose meaning when reflowed.
    clippy::doc_markdown,
    clippy::too_long_first_doc_paragraph,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    // Float arithmetic in geodesic integrators: comparisons and FMA-equivalent
    // forms are intentional; fused-mul-add is not always available cross-platform.
    clippy::suboptimal_flops,
    clippy::float_cmp,
    // Quotient rule (f'g - fg')/g^2 trips this; metric discriminants too.
    clippy::suspicious_operation_groupings,
    // Physics constants kept in published form preserves traceability to source.
    clippy::unreadable_literal,
    // const-fn promotion is nursery noise on math kernels.
    clippy::missing_const_for_fn,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    // Stylistic; bit-exact equality on test fixtures.
    clippy::derive_partial_eq_without_eq,
    clippy::cloned_instead_of_copied,
    // Format-string inlining: physics code prefers explicit args for clarity.
    clippy::uninlined_format_args,
    // Control-flow style preferences.
    clippy::useless_let_if_seq,
    clippy::redundant_else
)]

//! # Gravitas -- General Relativity Computation Engine
//!
//! A high-fidelity physics library for computing geodesics, spacetime geometry,
//! and relativistic observables in Kerr (rotating) black hole spacetimes.
//!
//! ## Quick Start
//!
//! ```rust
//! use gravitas::prelude::*;
//! use gravitas::metric::Metric;
//!
//! // Create a Kerr black hole (mass = 1.0, spin = 0.9)
//! let bh = Kerr::new(1.0, 0.9);
//!
//! // Key radii
//! let r_h = bh.event_horizon();
//! let r_isco = bh.isco(Orbit::Prograde);
//! let r_ph = bh.photon_sphere();
//!
//! // Get the metric tensor at a point
//! let g = bh.covariant(5.0, std::f64::consts::FRAC_PI_2);
//! ```
//!
//! ## Architecture
//!
//! The library is organized into the following modules:
//!
//! - [`metric`] -- Spacetime geometry: Metric trait, Kerr, Schwarzschild, Minkowski
//! - [`geodesic`] -- Ray state, Hamiltonian derivatives, integrators (RKF45, RK4, Symplectic)
//! - [`invariants`] -- Constants of motion (E, Lz, Q, H), momentum renormalization
//! - [`physics`] -- Physical observables: photon tracing, accretion disk, redshift, spectrum
//! - [`spacetime`] -- Visualization helpers: embedding diagrams, light cones, curvature
//! - [`tensor`] -- Tensor algebra: MetricTensor4, Christoffel symbols
//! - [`quantum`] -- Semi-classical effects: Hawking temperature, Planck-scale fluctuations
//! - [`constants`] -- Physical constants in SI and geometric units

pub mod constants;
pub mod geodesic;
pub mod invariants;
pub mod metric;
pub mod physics;
pub mod quantum;
pub mod spacetime;
pub mod tensor;

/// Convenience re-exports for common usage.
pub mod prelude {
    pub use crate::constants::*;
    pub use crate::geodesic::{
        GeodesicState, IntegrationMethod, IntegrationOptions, TerminationReason, Trajectory,
    };
    pub use crate::invariants::ConstantsOfMotion;
    pub use crate::metric::{Kerr, Metric, Orbit};
    pub use crate::tensor::MetricTensor4;
}
