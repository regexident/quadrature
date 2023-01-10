//! Quadrature-based decoders.

mod linear;
mod quadrature;
mod rotary;

pub use self::{
    linear::{LinearDecoder, LinearMovement},
    quadrature::{QuadratureDecoder, QuadratureMovement},
    rotary::{RotaryDecoder, RotaryMovement},
};
