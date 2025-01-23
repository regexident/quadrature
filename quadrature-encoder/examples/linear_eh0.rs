use embedded_hal_compat::{markers::*, Forward, ForwardCompat};
use embedded_hal_mock::eh0::digital::{
    Mock as PinMock, State as PinState, Transaction as PinTransaction,
};
use quadrature_encoder::{LinearEncoder, LinearMovement};

fn main() {
    let pin_clk = PinMock::new(&[
        PinTransaction::get(PinState::High),
        PinTransaction::get(PinState::High),
    ]);
    let pin_dt = PinMock::new(&[
        PinTransaction::get(PinState::High),
        PinTransaction::get(PinState::High),
    ]);

    let pin_clk_eh1: Forward<embedded_hal_mock::common::Generic<PinTransaction>, ForwardInputPin> =
        pin_clk.forward();
    let pin_dt_eh1: Forward<embedded_hal_mock::common::Generic<PinTransaction>, ForwardInputPin> =
        pin_dt.forward();

    let mut encoder = LinearEncoder::<_, _>::new(pin_clk_eh1, pin_dt_eh1);

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

    println!("Encoder is at position: {:?}.", encoder.position());

    let (mut pin_clk, mut pin_dt) = encoder.release();
    pin_clk.inner_mut().done();
    pin_dt.inner_mut().done();
}
