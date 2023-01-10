//! An efficient `no_std`-compatible implementation of a quadrature decoder,
//! based on a finite-state-transducer with support for different step-modes.
#![warn(missing_docs)]
#![cfg_attr(not(test), no_std)]

mod decoder;
mod state_transducer;

pub use self::decoder::QuadratureDecoder;

use self::state_transducer::{Output, StateTransducer};

/// An error indicating an inconsistency between two channel-value reads.
pub struct Error;

/// The movement detected by a quadrature decoder.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Movement {
    /// Forward movement (i.e. channel A leads channel B).
    Forward = 0,
    /// Forward movement (i.e. channel A trails channel B).
    Reverse = 1,
}

impl Movement {
    /// Flips the direction of `self`.
    pub fn flip(&mut self) {
        *self = self.flipped()
    }

    /// Returns the direction of `self`, flipped.
    pub fn flipped(self) -> Self {
        match self {
            Self::Forward => Self::Reverse,
            Self::Reverse => Self::Forward,
        }
    }
}

impl From<Movement> for Output {
    fn from(movement: Movement) -> Self {
        match movement {
            Movement::Forward => Self::F,
            Movement::Reverse => Self::R,
        }
    }
}

impl From<Option<Movement>> for Output {
    fn from(movement: Option<Movement>) -> Self {
        match movement {
            Some(movement) => movement.into(),
            None => Self::N,
        }
    }
}

impl From<Output> for Result<Option<Movement>, Error> {
    fn from(output: Output) -> Self {
        match output {
            Output::N => Ok(None),
            Output::F => Ok(Some(Movement::Forward)),
            Output::R => Ok(Some(Movement::Reverse)),
            Output::E => Err(Error),
        }
    }
}

mod sealed {
    pub trait Sealed {}
}

/// A quadrature-decoder's step mode.
pub trait StepMode: sealed::Sealed {
    /// The step-mode's number of pulses per cycle.
    const PULSES_PER_CYCLE: usize;
}

/// A step mode producing movement for every stable full cycle
/// (i.e. 1 movement per quadrature cycle).
///
/// Full-step mode provides:
/// - high noise-resistance (factor 4× relative to naïve decoding)
/// - low resolution (factor 1× relative to native resolution)
pub struct FullStep;

impl sealed::Sealed for FullStep {}
impl StepMode for FullStep {
    const PULSES_PER_CYCLE: usize = 1;
}

/// A step mode producing movement for every stable half cycle
/// (i.e. 2 movements per quadrature cycle),
/// resulting in an effective 2× resolution multiplication.
///
/// Half-step mode effectively doubles the resolution of the decoder.
///
/// Half-step mode provides:
/// - medium noise-resistance (factor 2× relative to naïve decoding)
/// - medium resolution (factor 1× relative to native resolution)
pub struct HalfStep;

impl sealed::Sealed for HalfStep {}
impl StepMode for HalfStep {
    const PULSES_PER_CYCLE: usize = 2;
}

/// A step mode producing movement for every stable quarter cycle
/// (i.e. 4 movement per quadrature cycle),
/// resulting in an effective 4× resolution multiplication.
///
/// Quad-step mode effectively quadruples the resolution of the decoder.
///
/// Quad-step mode provides:
/// - low noise-resistance (factor 1× relative to naïve decoding)
/// - high resolution (factor 1× relative to native resolution)
pub struct QuadStep;

impl sealed::Sealed for QuadStep {}
impl StepMode for QuadStep {
    const PULSES_PER_CYCLE: usize = 4;
}
