//! A robust indexed incremental encoder driver with support for multiple step-modes.

use embedded_hal::digital::InputPin;

use num_traits::{One, SaturatingAdd, Zero};
use quadrature_decoder::{IncrementalDecoder, IndexDecoder, StepMode};

use crate::{mode::OperationMode, Error, InputPinError, Linear, Rotary};

use super::IncrementalEncoder;

/// Indexed rotary encoder.
pub type IndexedRotaryEncoder<Steps, Clk, Dt, Idx, T = i32> =
    IndexedIncrementalEncoder<Rotary, Steps, Clk, Dt, Idx, T>;

/// Indexed linear encoder.
pub type IndexedLinearEncoder<Steps, Clk, Dt, Idx, T = i32> =
    IndexedIncrementalEncoder<Linear, Steps, Clk, Dt, Idx, T>;

/// A robust indexed incremental encoder with support for multiple step-modes.
#[derive(Debug)]
pub struct IndexedIncrementalEncoder<Mode, Steps, Clk, Dt, Idx, T = i32> {
    encoder: IncrementalEncoder<Mode, Steps, Clk, Dt, T>,
    indexer: IndexDecoder,
    pin_idx: Idx,
}

impl<Mode, Steps, Clk, Dt, Idx, T> IndexedIncrementalEncoder<Mode, Steps, Clk, Dt, Idx, T>
where
    Mode: OperationMode,
    Steps: StepMode,
    Clk: InputPin,
    Dt: InputPin,
    Idx: InputPin,
    T: Zero,
{
    /// Creates an indexed incremental encoder driver for the given pins.
    pub fn new(pin_clk: Clk, pin_dt: Dt, pin_idx: Idx) -> Self
    where
        IncrementalDecoder<Steps, T>: Default,
    {
        Self {
            encoder: IncrementalEncoder::new(pin_clk, pin_dt),
            indexer: Default::default(),
            pin_idx,
        }
    }
}

impl<Mode, Steps, Clk, Dt, Idx, T> IndexedIncrementalEncoder<Mode, Steps, Clk, Dt, Idx, T>
where
    Mode: OperationMode,
    Steps: StepMode,
    Clk: InputPin,
    Dt: InputPin,
    Idx: InputPin,
    T: Copy + Zero + One + SaturatingAdd + From<i8>,
{
    /// Sets the encoder's reversed mode, making it report flipped movements and positions.
    pub fn reversed(mut self) -> Self {
        self.encoder = self.encoder.reversed();
        self
    }

    /// Returns `true` if the encoder is reversed, otherwise `false`.
    pub fn is_reversed(&self) -> bool {
        self.encoder.is_reversed()
    }

    /// Returns mutable borrows for the signal channel pins.
    pub fn pins_mut(&mut self) -> (&mut Clk, &mut Dt, &mut Idx) {
        let (pin_clk, pin_dt) = self.encoder.pins_mut();
        (pin_clk, pin_dt, &mut self.pin_idx)
    }

    /// Consumes self, returning the signal channel pins.
    pub fn release(self) -> (Clk, Dt, Idx) {
        let (pin_clk, pin_dt) = self.encoder.release();
        (pin_clk, pin_dt, self.pin_idx)
    }

    /// Updates the encoder's state based on the given **clock**, **data** and **index** pins,
    /// returning the direction if a movement was detected, `None` if no movement was detected,
    /// or `Err(_)` if an invalid input (i.e. a positional "jump") was detected.
    ///
    /// Upon detection of a raising edge on the `index` pin the position gets reset back to `0`.
    ///
    /// Depending on whether it matters why the encoder did not detect a movement
    /// (e.g. due to actual lack of movement or an erroneous read)
    /// you would either call `poll()` directly:
    ///
    /// ```rust
    /// # use embedded_hal_mock::eh1::pin::{
    /// #     Mock as PinMock,
    /// #     Transaction as PinTransaction,
    /// #     State as PinState
    /// # };
    /// #
    /// # let pin_clk = PinMock::new(&[PinTransaction::get(PinState::High)]);
    /// # let pin_dt = PinMock::new(&[PinTransaction::get(PinState::High)]);
    /// # let pin_idx = PinMock::new(&[PinTransaction::get(PinState::Low)]);
    ///
    /// use quadrature_encoder::{FullStep, IndexedIncrementalEncoder, Rotary};
    ///
    /// let mut encoder = IndexedIncrementalEncoder::<Rotary, FullStep, _, _, _>::new(pin_clk, pin_dt, pin_idx);
    ///
    /// match encoder.poll() {
    ///     Ok(Some(movement)) => println!("Movement detected: {:?}.", movement),
    ///     Ok(None) => println!("No movement detected."),
    ///     Err(error) => println!("Error detected: {:?}.", error),
    /// }
    /// println!("position: {:?}", encoder.position());
    ///
    /// # let (mut pin_clk, mut pin_dt, mut pin_idx) = encoder.release();
    /// # pin_clk.done();
    /// # pin_dt.done();
    /// # pin_idx.done();
    /// ```
    ///
    /// Or fall back to `None` in case of `Err(_)` by use of `.unwrap_or_default()`:
    ///
    /// ```rust
    /// # use embedded_hal_mock::eh1::pin::{
    /// #     Mock as PinMock,
    /// #     Transaction as PinTransaction,
    /// #     State as PinState
    /// # };
    /// #
    /// # let pin_clk = PinMock::new(&[PinTransaction::get(PinState::High)]);
    /// # let pin_dt = PinMock::new(&[PinTransaction::get(PinState::High)]);
    /// # let pin_idx = PinMock::new(&[PinTransaction::get(PinState::Low)]);
    ///
    /// use quadrature_encoder::{FullStep, IndexedIncrementalEncoder, Rotary};
    ///
    /// let mut encoder = IndexedIncrementalEncoder::<Rotary, FullStep, _, _, _>::new(pin_clk, pin_dt, pin_idx);
    ///
    /// match encoder.poll().unwrap_or_default() {
    ///     Some(movement) => println!("Movement detected: {:?}.", movement),
    ///     None => println!("No movement detected."),
    /// }
    /// println!("position: {:?}", encoder.position());
    ///
    /// # let (mut pin_clk, mut pin_dt, mut pin_idx) = encoder.release();
    /// # pin_clk.done();
    /// # pin_dt.done();
    /// # pin_idx.done();
    /// ```
    pub fn poll(&mut self) -> Result<Option<Mode::Movement>, Error> {
        let z = self
            .pin_idx
            .is_high()
            .map_err(|_| Error::InputPin(InputPinError::PinIdx))?;

        let result = self.encoder.poll();

        if self.indexer.update(z) {
            self.encoder.set_position(Zero::zero());
        }

        result
    }

    /// Resets the encoder to its initial state.
    pub fn reset(&mut self) {
        self.encoder.reset();
        self.indexer.reset();
    }

    /// Returns the encoder's position counter relative to its initial position in number of cycles.
    pub fn position(&self) -> T {
        self.encoder.position()
    }

    /// Sets the encoder's position.
    pub fn set_position(&mut self, position: T) {
        self.encoder.set_position(position);
    }
}
