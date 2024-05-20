//! A linear quadrature encoder's movement.

use quadrature_decoder::QuadratureMovement;

use crate::mode::OperationMode;

use super::Movement;

/// The movement detected by a linear quadrature encoder.
#[repr(i8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum LinearMovement {
    /// Forward movement.
    Forward = 1,
    /// Reverse movement.
    Reverse = -1,
}

impl LinearMovement {
    /// Returns the direction of `self`, flipped.
    pub fn flipped(self) -> Self {
        match self {
            Self::Forward => Self::Reverse,
            Self::Reverse => Self::Forward,
        }
    }
}

impl From<QuadratureMovement> for LinearMovement {
    /// Interprets quadrature movement as a linear movement with the following mapping:
    ///
    /// - `QuadratureMovement::AB => LinearMovement::Forward`
    /// - `QuadratureMovement::BA => LinearMovement::Reverse`
    fn from(movement: QuadratureMovement) -> Self {
        match movement {
            QuadratureMovement::AB => Self::Forward,
            QuadratureMovement::BA => Self::Reverse,
        }
    }
}

impl Movement for LinearMovement {
    fn flipped(self) -> Self {
        match self {
            Self::Forward => Self::Reverse,
            Self::Reverse => Self::Forward,
        }
    }
}

/// The mode of a linear quadrature encoder.
pub struct Linear;

impl OperationMode for Linear {
    type Movement = LinearMovement;
}
