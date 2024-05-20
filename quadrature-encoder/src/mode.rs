mod linear;
mod rotary;

use quadrature_decoder::Change;

pub use self::{
    linear::{Linear, LinearMovement},
    rotary::{Rotary, RotaryMovement},
};

pub trait Movement: From<Change> + Eq {
    /// Returns the direction of `self`, flipped.
    fn flipped(self) -> Self;
}

/// The mode of physical operation of a quadrature encoder.
pub trait OperationMode {
    /// The mode's type of movement.
    type Movement: Movement;
}
