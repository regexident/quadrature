use embedded_hal_mock::eh1::digital::{
    Mock as PinMock, State as PinState, Transaction as PinTransaction,
};

use quadrature_encoder::{RotaryEncoder, RotaryMovement};

fn main() {
    let pin_clk = PinMock::new(&[PinTransaction::get(PinState::High)]);
    let pin_dt = PinMock::new(&[PinTransaction::get(PinState::High)]);

    let mut encoder = RotaryEncoder::<_, _>::new(pin_clk, pin_dt);

    match encoder.poll() {
        Ok(Some(movement)) => {
            let direction = match movement {
                RotaryMovement::Clockwise => "clockwise",
                RotaryMovement::CounterClockwise => "counter-clockwise",
            };
            println!("Movement detected in {:?} direction.", direction)
        }
        Ok(None) => println!("No movement detected."),
        Err(error) => println!("Error detected: {:?}.", error),
    }
    println!("Encoder is at position: {:?}.", encoder.position());

    let (mut pin_clk, mut pin_dt) = encoder.release();
    pin_clk.done();
    pin_dt.done();
}
