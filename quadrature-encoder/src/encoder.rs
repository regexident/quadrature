//! Quadrature-based encoder drivers.

mod linear;
mod rotary;

pub use self::{linear::LinearEncoder, rotary::RotaryEncoder};
