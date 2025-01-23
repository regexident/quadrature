mod linear;
mod rotary;

use core::marker::PhantomData;
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

/// A marker trait for initializing drivers in a specific mode.
/// Inspired by https://github.com/esp-rs/esp-hal
pub trait PollMode {}

/// Driver initialized in blocking mode.
#[derive(Debug)]
pub struct Blocking;

/// Driver initialized in async mode.
#[derive(Debug)]
pub struct Async(PhantomData<*const ()>);

impl crate::PollMode for Blocking {}
impl crate::PollMode for Async {}
