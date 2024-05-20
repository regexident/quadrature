//! An efficient `no_std`-compatible implementation of a quadrature decoder,
//! based on a finite-state-transducer with support for different step-modes.
#![warn(missing_docs)]
#![cfg_attr(not(test), no_std)]

mod decoder;
mod index_decoder;
mod state_transducer;
mod validator;

pub use self::{
    decoder::{IncrementalDecoder, IndexedIncrementalDecoder},
    index_decoder::IndexDecoder,
};

use self::state_transducer::StateTransducer;

mod sealed {
    pub trait Sealed {}
}

/// An error indicating an invalid quadrature signal sequence.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(non_camel_case_types)]
pub enum Error {
    /// Invalid gray-code sequence [00, 11].
    E00_11 = 0b_00_11,
    /// Invalid gray-code sequence [11, 00].
    E11_00 = 0b_11_00,
    /// Invalid gray-code sequence [01, 10].
    E01_10 = 0b_01_10,
    /// Invalid gray-code sequence [10, 01].
    E10_01 = 0b_10_01,
}

/// The movement detected by a quadrature decoder.
#[repr(i8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum QuadratureMovement {
    /// Channel A leads channel B, commonly describing a forwards movement.
    AB = 1,
    /// Channel B leads channel A, commonly describing a backwards movement.
    BA = -1,
}

/// A quadrature-based decoder's step mode.
pub trait StepMode: sealed::Sealed {
    /// The step-mode's number of pulses per (quadrature) cycle (PPC).
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
    /// The number of pulses per (quadrature) cycle (PPC).
    ///
    /// As an example, consider the effective pulses per revolution (PPR)
    /// of a rotary encoder with 100 cycles per revolution (CPR): 100 PPR.
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
    /// The number of pulses per (quadrature) cycle (PPC).
    ///
    /// As an example, consider the effective pulses per revolution (PPR)
    /// of a rotary encoder with 100 cycles per revolution (CPR): 200 PPR.
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
    /// The number of pulses per (quadrature) cycle (PPC).
    ///
    /// As an example, consider the effective pulses per revolution (PPR)
    /// of a rotary encoder with 100 cycles per revolution (CPR): 400 PPR.
    const PULSES_PER_CYCLE: usize = 4;
}
