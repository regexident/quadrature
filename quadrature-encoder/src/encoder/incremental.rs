//! A robust incremental encoder driver with support for multiple step-modes.

use core::marker::PhantomData;

use num_traits::{One, SaturatingAdd, Zero};
use quadrature_decoder::{Change, FullStep, IncrementalDecoder, StepMode};

#[cfg(feature="async")]
use embassy_futures::select::{select,Either};

use crate::{
    traits::InputPin,
    mode::{Movement, OperationMode},
    Error, InputPinError, Linear, Rotary,
};

/// Rotary encoder.
pub type RotaryEncoder<Clk, Dt, Steps = FullStep, T = i32> =
    IncrementalEncoder<Rotary, Clk, Dt, Steps, T>;
/// Linear encoder.
pub type LinearEncoder<Clk, Dt, Steps = FullStep, T = i32> =
    IncrementalEncoder<Linear, Clk, Dt, Steps, T>;

/// A robust incremental encoder with support for multiple step-modes.
#[derive(Debug)]
pub struct IncrementalEncoder<Mode, Clk, Dt, Steps = FullStep, T = i32> {
    decoder: IncrementalDecoder<Steps, T>,
    pin_clk: Clk,
    pin_dt: Dt,
    is_reversed: bool,
    _mode: PhantomData<Mode>,
}

impl<Mode, Clk, Dt, Steps, T> IncrementalEncoder<Mode, Clk, Dt, Steps, T>
where
    Mode: OperationMode,
    Clk: InputPin,
    Dt: InputPin,
    Steps: StepMode,
    T: Zero,
{
    /// Creates an incremental encoder driver for the given pins.
    pub fn new(pin_clk: Clk, pin_dt: Dt) -> Self
    where
        IncrementalDecoder<Steps, T>: Default,
    {
        Self {
            decoder: Default::default(),
            pin_clk,
            pin_dt,
            is_reversed: false,
            _mode: PhantomData,
        }
    }
}

impl<Mode, Clk, Dt, Steps, T> IncrementalEncoder<Mode, Clk, Dt, Steps, T>
where
    Mode: OperationMode,
    Clk: InputPin,
    Dt: InputPin,
    Steps: StepMode,
    T: Copy + Zero + One + SaturatingAdd + From<i8>,
{
    /// Sets the encoder's reversed mode, making it report flipped movements and positions.
    pub fn reversed(mut self) -> Self {
        self.is_reversed = true;
        self
    }

    /// Returns `true` if the encoder is reversed, otherwise `false`.
    pub fn is_reversed(&self) -> bool {
        self.is_reversed
    }

    /// Returns mutable borrows for the signal channel pins.
    pub fn pins_mut(&mut self) -> (&mut Clk, &mut Dt) {
        (&mut self.pin_clk, &mut self.pin_dt)
    }

    /// Consumes self, returning the signal channel pins.
    pub fn release(self) -> (Clk, Dt) {
        (self.pin_clk, self.pin_dt)
    }

    /// Updates the encoder's state based on the given **clock** and **data** pins,
    /// returning the direction if a movement was detected, `None` if no movement was detected,
    /// or `Err(_)` if an invalid input (i.e. a positional "jump") was detected.
    ///
    /// Depending on whether it matters why the encoder did not detect a movement
    /// (e.g. due to actual lack of movement or an erroneous read)
    /// you would either call `encoder.poll()` directly, or via `encoder.poll().unwrap_or_default()`
    /// to fall back to `None` in case of `Err(_)`.
    pub fn poll(&mut self) -> Result<Option<Mode::Movement>, Error> {
        let a = self
            .pin_clk
            .is_high()
            .map_err(|_| Error::InputPin(InputPinError::PinClk))?;
        let b = self
            .pin_dt
            .is_high()
            .map_err(|_| Error::InputPin(InputPinError::PinDt))?;

        let change: Option<Change> = self.decoder.update(a, b).map_err(Error::Quadrature)?;
        let movement: Option<Mode::Movement> = change.map(From::from);

        Ok(movement.map(|movement| {
            if self.is_reversed() {
                movement.flipped()
            } else {
                movement
            }
        }))
    }

    /// Waits asyncronously for either two pins to change state, then runs poll()
    #[cfg(feature="async")]
    pub async fn poll_async(&mut self) -> Result<Option<Mode::Movement>, Error> {
        match select(self.pin_clk.wait_for_any_edge(),self.pin_dt.wait_for_any_edge()).await
        {
            Either::First(_) => {},
            Either::Second(_) => {},
        };
        self.poll()
    }

    /// Resets the encoder to its initial state.
    pub fn reset(&mut self) {
        self.decoder.reset();
    }

    /// Returns the encoder's position counter relative to its initial position in number of cycles.
    pub fn position(&self) -> T {
        self.decoder.counter()
    }

    /// Sets the encoder's position.
    pub fn set_position(&mut self, position: T) {
        self.decoder.set_counter(position);
    }
}
