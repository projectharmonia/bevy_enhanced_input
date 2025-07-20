pub mod axial;
pub mod bidirectional;
pub mod cardinal;
pub mod ordinal;
pub mod spatial;

pub trait WithBundle<T> {
    type Output;

    fn with(self, bundle: T) -> Self::Output;
}
