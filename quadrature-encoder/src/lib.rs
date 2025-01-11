//! An efficient `no_std`-compatible implementation of a quadrature encoder driver,
//! based on a finite-state-transducer with support for different step-modes.
#![warn(missing_docs)]
#![cfg_attr(not(test), no_std)]

mod traits;
mod encoder;
mod mode;
pub use quadrature_decoder::{Error as QuadratureError, FullStep, HalfStep, QuadStep};

pub use self::{
    encoder::{
        IncrementalEncoder, IndexedIncrementalEncoder, IndexedLinearEncoder, IndexedRotaryEncoder,
        LinearEncoder, RotaryEncoder,
    },
    mode::{Linear, LinearMovement, OperationMode, Rotary, RotaryMovement},
};

/// An error indicating an input pin issue.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputPinError {
    /// Failed reading clock pin.
    PinClk,
    /// Failed reading data pin.
    PinDt,
    /// Failed reading index pin.
    PinIdx,
}

/// An error indicating quadrature or input pin issues.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Error {
    /// Quadrature error.
    Quadrature(QuadratureError),
    /// Input pin error.
    InputPin(InputPinError),
}
