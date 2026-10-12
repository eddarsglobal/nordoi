#![forbid(unsafe_code)]
//! P0.5C closed build-graph boundary for the explicitly selected fenced host profile.
//!
//! This crate intentionally exposes only the P0.5A `FencedProductionEffects` facade.
//! It does not re-export the raw P0.3/P0.4 backend types, the kernel `EffectBackend`
//! trait, or P0.1 trust primitives. A consumer that declares only this crate as a
//! direct Cargo dependency therefore cannot name those transitive crates directly.
//!
//! This is a build-graph/import boundary, not a kernel-wide removal of historical
//! APIs and not production deployment authorization.

pub use nordoi_native_effect_p03::enforced::FencedProductionEffects;

pub const PROFILE_ID: &str = "NORDOI:P0.5C:FENCED-PRODUCTION-PROFILE:V1";

#[must_use]
pub const fn profile_id() -> &'static str {
    PROFILE_ID
}
