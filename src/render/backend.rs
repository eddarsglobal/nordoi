use super::core::RenderBatch;

/// Backends implement the platform-specific final step.
/// The Atomic Render Core itself remains platform-independent.
pub trait RenderBackend {
    type Error;

    fn apply(&mut self, batch: &RenderBatch) -> Result<(), Self::Error>;
}
