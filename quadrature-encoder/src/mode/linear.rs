//! A linear quadrature encoder's movement.

use quadrature_decoder::Change;

use crate::mode::OperationMode;

use super::Movement;

/// The movement detected by a linear quadrature encoder.
#[repr(i8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum LinearMovement {
    /// Forward movement.
    Forward = 1,
    /// Backward movement.
    Backward = -1,
}

impl LinearMovement {
    /// Returns the direction of `self`, flipped.
    pub fn flipped(self) -> Self {
        match self {
            Self::Forward => Self::Backward,
            Self::Backward => Self::Forward,
        }
    }
}

impl From<Change> for LinearMovement {
    /// Interprets quadrature movement as a linear movement with the following mapping:
    ///
    /// - `Change::AB => LinearMovement::Forward`
    /// - `Change::BA => LinearMovement::Backward`
    fn from(change: Change) -> Self {
        match change {
            Change::Positive => Self::Forward,
            Change::Negative => Self::Backward,
        }
    }
}

impl Movement for LinearMovement {
    fn flipped(self) -> Self {
        match self {
            Self::Forward => Self::Backward,
            Self::Backward => Self::Forward,
        }
    }
}

/// The mode of a linear quadrature encoder.
pub struct Linear;

impl OperationMode for Linear {
    type Movement = LinearMovement;
}
