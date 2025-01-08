#[cfg(feature = "eh1")]
use embedded_hal_mock::eh1::digital::{Mock as PinMock, State as PinState, Transaction as PinTransaction};
#[cfg(feature = "eh0")]
use embedded_hal_mock::eh0::digital::{Mock as PinMock, State as PinState, Transaction as PinTransaction};
#[cfg(feature = "async")]
use embedded_hal_mock::eh1::digital::Edge;
#[cfg(feature = "async")]
use embassy_futures::block_on;

use quadrature_encoder::{LinearEncoder, LinearMovement};

fn main() {
    #[cfg(not(feature = "async"))]
    let pin_clk = PinMock::new(&[PinTransaction::get(PinState::High)]);
    #[cfg(not(feature = "async"))]
    let pin_dt = PinMock::new(&[PinTransaction::get(PinState::High)]);

    #[cfg(feature = "async")]
    let pin_clk = PinMock::new(&[PinTransaction::wait_for_edge(Edge::Any),PinTransaction::get(PinState::High)]);
    #[cfg(feature = "async")]
    let pin_dt = PinMock::new(&[PinTransaction::get(PinState::High)]);

    let mut encoder = LinearEncoder::<_, _>::new(pin_clk, pin_dt);

    #[cfg(not(feature = "async"))]
    match encoder.poll() {
        Ok(Some(movement)) => {
            let direction = match movement {
                LinearMovement::Forward => "forward",
                LinearMovement::Backward => "backward",
            };
            println!("Movement detected in {:?} direction.", direction)
        }
        Ok(_) => println!("No movement detected."),
        Err(error) => println!("Error detected: {:?}.", error),
    }
    #[cfg(feature = "async")]
    match block_on(encoder.poll_async()) {
        Ok(Some(movement)) => {
            let direction = match movement {
                LinearMovement::Forward => "forward",
                LinearMovement::Backward => "backward",
            };
            println!("Movement detected in {:?} direction.", direction)
        }
        Ok(_) => println!("No movement detected."),
        Err(error) => println!("Error detected: {:?}.", error),
    }
    println!("Encoder is at position: {:?}.", encoder.position());

    let (mut pin_clk, mut pin_dt) = encoder.release();
    pin_clk.done();
    pin_dt.done();
}
