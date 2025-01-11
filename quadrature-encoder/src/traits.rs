#[cfg(feature = "eh1")]
use eh1::digital::InputPin as EhalInputPin;

#[cfg(feature = "eh0")]
use eh0::digital::v2::InputPin as EhalInputPin;

#[cfg(not(feature = "async"))]
pub trait InputPin: EhalInputPin {}
#[cfg(not(feature = "async"))]
impl<T: EhalInputPin> InputPin for T {}

#[cfg(feature = "async")]
use embedded_hal_async::digital::Wait;
#[cfg(feature = "async")]
pub trait InputPin: EhalInputPin + Wait {}
#[cfg(feature = "async")]
impl<T: EhalInputPin + Wait> InputPin for T {}