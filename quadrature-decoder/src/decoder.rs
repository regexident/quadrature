//! Quadrature-based decoder.

mod incremental;
mod indexed;

pub use self::{incremental::IncrementalDecoder, indexed::IndexedIncrementalDecoder};
