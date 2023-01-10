//! An efficient `no_std`-compatible implementation of a quadrature encoder driver,
//! based on a finite-state-transducer with support for different step-modes.
#![warn(missing_docs)]
#![cfg_attr(not(test), no_std)]

mod encoder;

pub use quadrature_decoder::{Error, FullStep, HalfStep, LinearMovement, QuadStep, RotaryMovement};

pub use self::encoder::{LinearEncoder, RotaryEncoder};
