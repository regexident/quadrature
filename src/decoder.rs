//! Quadrature-based decoders.

mod linear;
mod quadrature;

pub use self::{
    linear::{LinearDecoder, LinearMovement},
    quadrature::{QuadratureDecoder, QuadratureMovement},
};
