// Polled pins must impliment ehal v1.0.0 InputPin trait,
// either directly or via embadded-hal-compat Forward-ing.
pub use eh1::digital::InputPin;
use embedded_hal_compat::eh1_0 as eh1;

// exported async traits
#[cfg(feature = "async")]
pub use embassy_futures::select::{select, Either};
#[cfg(feature = "async")]
pub use embassy_futures::select::{select3, Either3};
#[cfg(feature = "async")]
pub use embedded_hal_async::digital::Wait;
#[cfg(feature = "async")]
pub use futures::FutureExt;
