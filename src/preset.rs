pub mod axial;
pub mod bidirectional;
pub mod cardinal;
pub mod ordinal;
pub mod spatial;

/// Helper trait for attaching a bundle to a preset.
pub trait WithBundle<T> {
    type Output;

    /// Returns a new instance where the given bundle is added to each preset bundle.
    fn with(self, bundle: T) -> Self::Output;
}
