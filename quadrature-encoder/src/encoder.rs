//! Quadrature-based encoder drivers.

mod incremental;
mod indexed;

pub use self::{
    incremental::{IncrementalEncoder, LinearEncoder, RotaryEncoder},
    indexed::{IndexedIncrementalEncoder, IndexedLinearEncoder, IndexedRotaryEncoder},
};
