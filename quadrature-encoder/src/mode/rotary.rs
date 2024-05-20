//! A rotary quadrature encoder's movement.

use quadrature_decoder::QuadratureMovement;

use crate::mode::OperationMode;

use super::Movement;

/// The movement detected by a rotary quadrature encoder.
#[repr(i8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum RotaryMovement {
    /// Clockwise movement.
    Clockwise = 1,
    /// Counter-clockwise movement.
    CounterClockwise = -1,
}

impl From<QuadratureMovement> for RotaryMovement {
    /// Interprets quadrature movement as a rotary movement with the following mapping:
    ///
    /// - `QuadratureMovement::AB => RotaryMovement::Clockwise`
    /// - `QuadratureMovement::BA => RotaryMovement::CounterClockwise`
    fn from(movement: QuadratureMovement) -> Self {
        match movement {
            QuadratureMovement::AB => Self::Clockwise,
            QuadratureMovement::BA => Self::CounterClockwise,
        }
    }
}

impl Movement for RotaryMovement {
    fn flipped(self) -> Self {
        match self {
            Self::Clockwise => Self::CounterClockwise,
            Self::CounterClockwise => Self::Clockwise,
        }
    }
}

/// The mode of a rotary quadrature encoder.
pub struct Rotary;

impl OperationMode for Rotary {
    type Movement = RotaryMovement;
}
