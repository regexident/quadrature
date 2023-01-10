//! A robust rotary quadrature encoder driver with support for multiple step-modes.

use embedded_hal::digital::v2::InputPin;
use quadrature_decoder::{Error, RotaryDecoder, RotaryMovement, StepMode};

/// A robust rotary quadrature encoder with support for multiple step-modes.
#[derive(Debug)]
pub struct RotaryEncoder<Mode, Clk, Dt>
where
    Mode: StepMode,
    Clk: InputPin,
    Dt: InputPin,
{
    decoder: RotaryDecoder<Mode>,
    pin_clk: Clk,
    pin_dt: Dt,
}

impl<Mode, Clk, Dt> RotaryEncoder<Mode, Clk, Dt>
where
    Mode: StepMode,
    Clk: InputPin,
    Dt: InputPin,
{
    /// Creates a linear encoder driver for the given pins for signal channels A and Dt.
    ///
    /// The
    ///
    /// |             | `KY-040` |       |
    /// |  ---------  |  ------  |  ---  |
    /// | Channel `A` | `Clk`    |       |
    /// | Channel `Dt` | `Dt`     |       |
    pub fn new(pin_clk: Clk, pin_dt: Dt) -> Self
    where
        RotaryDecoder<Mode>: Default,
    {
        Self {
            decoder: Default::default(),
            pin_clk,
            pin_dt,
        }
    }

    /// Returns mutable borrows for the signal channel pins.
    pub fn pins_mut(&mut self) -> (&mut Clk, &mut Dt) {
        (&mut self.pin_clk, &mut self.pin_dt)
    }

    /// Consumes self, returning the signal channel pins.
    pub fn release(self) -> (Clk, Dt) {
        (self.pin_clk, self.pin_dt)
    }

    /// Updates the encoder's state based on the given `a` and `b` pulse train readings,
    /// returning the direction if a movement was detected, `None` if no movement was detected,
    /// or `Err(_)` if an invalid input (i.e. a positional "jump") was detected.
    ///
    /// Depending on whether it matters why the encoder did not detect a movement
    /// (e.g. due to actual lack of movement or an erroneous read)
    /// you would either call `poll()` directly:
    ///
    /// ```rust
    /// use embedded_hal_mock::pin::{
    ///     Mock as PinMock,
    ///     Transaction as PinTransaction,
    ///     State as PinState
    /// };
    ///
    /// let pin_clk = PinMock::new(&[PinTransaction::get(PinState::High)]);
    /// let pin_dt = PinMock::new(&[PinTransaction::get(PinState::High)]);
    ///
    /// use quadrature_encoder::{FullStep, RotaryEncoder};
    ///
    /// let mut encoder = RotaryEncoder::<FullStep, _, _>::new(pin_clk, pin_dt);
    ///
    /// match encoder.poll() {
    ///     Ok(Some(movement)) => println!("Movement detected: {:?}.", movement),
    ///     Ok(None) => println!("No movement detected."),
    ///     Err(error) => println!("Error detected: {:?}.", error),
    /// }
    /// ```
    ///
    /// Or fall back to `None` in case of `Err(_)` by use of `.unwrap_or_default()`:
    ///
    /// ```rust
    /// use embedded_hal_mock::pin::{
    ///     Mock as PinMock,
    ///     Transaction as PinTransaction,
    ///     State as PinState
    /// };
    ///
    /// let pin_clk = PinMock::new(&[PinTransaction::get(PinState::High)]);
    /// let pin_dt = PinMock::new(&[PinTransaction::get(PinState::High)]);
    ///
    /// use quadrature_encoder::{FullStep, RotaryEncoder};
    ///
    /// let mut encoder = RotaryEncoder::<FullStep, _, _>::new(pin_clk, pin_dt);
    ///
    /// match encoder.poll().unwrap_or_default() {
    ///     Some(movement) => println!("Movement detected: {:?}.", movement),
    ///     None => println!("No movement detected."),
    /// }
    /// ```
    pub fn poll(&mut self) -> Result<Option<RotaryMovement>, Error> {
        let a = self.pin_clk.is_high().unwrap_or_default();
        let b = self.pin_dt.is_high().unwrap_or_default();

        self.decoder.update(a, b)
    }

    /// Resets the encoder to its initial state.
    pub fn reset(&mut self) {
        self.decoder.reset();
    }

    /// The encoder's number of pulses per (quadrature) cycle (PPC).
    ///
    /// As an example, consider the effectively pulses per revolution (PPR)
    /// of a rotary encoder with 100 cycles per revolution (CPR):
    ///
    /// - A step mode with 1 pulse per cycle (e.g. `RotaryEncoder<FullStep>`) results in effectively 100 pulses per revolution (100 PPR).
    /// - A step mode with 2 pulses per cycle (e.g. `RotaryEncoder<HalfStep>`) results in effectively 200 pulses per revolution (200 PPR).
    /// - A step mode with 4 pulses per cycle (e.g. `RotaryEncoder<QuadStep>`) results in effectively 400 pulses per revolution (400 PPR).
    pub fn pulses_per_cycle() -> usize {
        Mode::PULSES_PER_CYCLE
    }
}
