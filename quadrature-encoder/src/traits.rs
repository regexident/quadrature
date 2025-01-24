// Polled pins must implement the `InputPin` trait from embedded-hal v1.0.0,
// either directly or via `embedded-hal-compat` forward-ing.
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
