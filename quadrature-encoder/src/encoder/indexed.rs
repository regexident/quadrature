//! A robust incremental encoder driver with support for multiple step-modes.

use core::marker::PhantomData;

use num_traits::{One, SaturatingAdd, Zero};
use quadrature_decoder::{Change, FullStep, IndexedIncrementalDecoder, StepMode};

#[allow(unused_imports)]
use crate::{
    mode::{Async, Blocking, Movement, OperationMode, PollMode},
    traits::*,
    Error, InputPinError, Linear, Rotary,
};

/// Rotary encoder.
pub type IndexedRotaryEncoder<Clk, Dt, Steps = FullStep, T = i32, PM = Blocking> =
    IndexedIncrementalEncoder<Rotary, Clk, Dt, Steps, T, PM>;
/// Linear encoder.
pub type IndexedLinearEncoder<Clk, Dt, Steps = FullStep, T = i32, PM = Blocking> =
    IndexedIncrementalEncoder<Linear, Clk, Dt, Steps, T, PM>;

/// A robust incremental encoder with support for multiple step-modes.
#[derive(Debug)]
pub struct IndexedIncrementalEncoder<Mode, Clk, Dt, Idx, Steps = FullStep, T = i32, PM = Blocking> {
    decoder: IndexedIncrementalDecoder<Steps, T>,
    pin_clk: Clk,
    pin_dt: Dt,
    pin_idx: Idx,
    pin_clk_state: bool,
    pin_dt_state: bool,
    pin_idx_state: bool,
    is_reversed: bool,
    _mode: PhantomData<Mode>,
    _poll_mode: PhantomData<PM>,
}

impl<Mode, Clk, Dt, Idx, Steps, T, PM> IndexedIncrementalEncoder<Mode, Clk, Dt, Idx, Steps, T, PM>
where
    Mode: OperationMode,
    Clk: InputPin,
    Dt: InputPin,
    Idx: InputPin,
    Steps: StepMode,
    T: Zero,
    PM: PollMode,
{
    /// Creates an indexed incremental encoder driver for the given pins.
    pub fn new(mut pin_clk: Clk, mut pin_dt: Dt, mut pin_idx: Idx) -> Self
    where
        IndexedIncrementalDecoder<Steps, T>: Default,
    {
        let pin_clk_state = pin_clk.is_high().unwrap_or(false);
        let pin_dt_state = pin_dt.is_high().unwrap_or(false);
        let pin_idx_state = pin_idx.is_high().unwrap_or(false);

        Self {
            decoder: Default::default(),
            pin_clk,
            pin_dt,
            pin_idx,
            pin_clk_state,
            pin_dt_state,
            pin_idx_state,
            is_reversed: false,
            _mode: PhantomData,
            _poll_mode: PhantomData,
        }
    }
}

impl<Mode, Clk, Dt, Idx, Steps, T, PM> IndexedIncrementalEncoder<Mode, Clk, Dt, Idx, Steps, T, PM>
where
    Mode: OperationMode,
    Clk: InputPin,
    Dt: InputPin,
    Idx: InputPin,
    Steps: StepMode,
    T: Copy + Zero + One + SaturatingAdd + From<i8>,
    PM: PollMode,
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

    /// Updates the internal decoder state, from the latest IO readings.
    /// This is called within poll() / poll_async()
    fn update(&mut self) -> Result<Option<Mode::Movement>, Error> {
        let change: Option<Change> = self
            .decoder
            .update(self.pin_clk_state, self.pin_dt_state, self.pin_idx_state)
            .map_err(Error::Quadrature)?;
        let movement: Option<Mode::Movement> = change.map(From::from);

        Ok(movement.map(|movement| {
            if self.is_reversed() {
                movement.flipped()
            } else {
                movement
            }
        }))
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

impl<Mode, Clk, Dt, Idx, Steps, T> IndexedIncrementalEncoder<Mode, Clk, Dt, Idx, Steps, T, Blocking>
where
    Mode: OperationMode,
    Clk: InputPin,
    Dt: InputPin,
    Idx: InputPin,
    Steps: StepMode,
    T: Copy + Zero + One + SaturatingAdd + From<i8>,
{
    /// Updates the encoder's state based on the given **clock**, **data**, and **index** pins,
    /// returning the direction if a movement was detected, `None` if no movement was detected,
    /// or `Err(_)` if an invalid input (i.e. a positional "jump") was detected.
    ///
    /// Depending on whether it matters why the encoder did not detect a movement
    /// (e.g. due to actual lack of movement or an erroneous read)
    /// you would either call `encoder.poll()` directly, or via `encoder.poll().unwrap_or_default()`
    /// to fall back to `None` in case of `Err(_)`.
    pub fn poll(&mut self) -> Result<Option<Mode::Movement>, Error> {
        self.pin_clk_state = self
            .pin_clk
            .is_high()
            .map_err(|_| Error::InputPin(InputPinError::PinClk))?;
        self.pin_dt_state = self
            .pin_dt
            .is_high()
            .map_err(|_| Error::InputPin(InputPinError::PinDt))?;
        self.pin_idx_state = self
            .pin_idx
            .is_high()
            .map_err(|_| Error::InputPin(InputPinError::PinIdx))?;
        self.update()
    }
}

/// If async is enabled, and the pins provided satisfy the AsyncInputPin trait, the into_async() method is exposed.
#[cfg(feature = "async")]
impl<Mode, Clk, Dt, Idx, Steps, T> IndexedIncrementalEncoder<Mode, Clk, Dt, Idx, Steps, T, Blocking>
where
    Mode: OperationMode,
    Clk: InputPin + Wait,
    Dt: InputPin + Wait,
    Idx: InputPin + Wait,
    Steps: StepMode,
    T: Copy + Zero + One + SaturatingAdd + From<i8>,
{
    /// Reconfigure the driver so that poll() is an async fn
    pub fn into_async(self) -> IndexedIncrementalEncoder<Mode, Clk, Dt, Idx, Steps, T, Async>
    where
        IndexedIncrementalDecoder<Steps, T>: Default,
    {
        IndexedIncrementalEncoder::<Mode, Clk, Dt, Idx, Steps, T, Async> {
            decoder: self.decoder,
            pin_clk: self.pin_clk,
            pin_dt: self.pin_dt,
            pin_idx: self.pin_idx,
            pin_clk_state: self.pin_clk_state,
            pin_dt_state: self.pin_dt_state,
            pin_idx_state: self.pin_idx_state,
            is_reversed: self.is_reversed,
            _mode: PhantomData,
            _poll_mode: PhantomData,
        }
    }
}

#[cfg(feature = "async")]
impl<Mode, Clk, Dt, Idx, Steps, T> IndexedIncrementalEncoder<Mode, Clk, Dt, Idx, Steps, T, Async>
where
    Mode: OperationMode,
    Clk: InputPin + Wait,
    Dt: InputPin + Wait,
    Idx: InputPin + Wait,
    Steps: StepMode,
    T: Copy + Zero + One + SaturatingAdd + From<i8>,
{
    /// Updates the encoder's state based on the given **clock**, **data**, and **index** pins,
    /// returning the direction if a movement was detected, `None` if no movement was detected,
    /// or `Err(_)` if an invalid input (i.e. a positional "jump") was detected.
    ///
    /// Depending on whether it matters why the encoder did not detect a movement
    /// (e.g. due to actual lack of movement or an erroneous read)
    /// you would either call `encoder.poll()` directly, or via `encoder.poll().unwrap_or_default()`
    /// to fall back to `None` in case of `Err(_)`.
    ///
    /// Waits asyncronously for any of the pins to change state, before returning.
    pub async fn poll(&mut self) -> Result<Option<Mode::Movement>, Error> {
        let clk_fut = match self.pin_clk_state {
            true => self.pin_clk.wait_for_falling_edge().left_future(),
            false => self.pin_clk.wait_for_rising_edge().right_future(),
        };

        let dt_fut = match self.pin_dt_state {
            true => self.pin_dt.wait_for_falling_edge().left_future(),
            false => self.pin_dt.wait_for_rising_edge().right_future(),
        };

        let idx_fut = match self.pin_idx_state {
            true => self.pin_idx.wait_for_falling_edge().left_future(),
            false => self.pin_idx.wait_for_rising_edge().right_future(),
        };

        match select3(clk_fut, dt_fut, idx_fut).await {
            Either3::First(_) => {
                self.pin_clk_state = !self.pin_clk_state;
            }
            Either3::Second(_) => {
                self.pin_dt_state = !self.pin_dt_state;
            }
            Either3::Third(_) => {
                self.pin_idx_state = !self.pin_idx_state;
            }
        };

        self.update()
    }

    /// Reconfigure the driver so that poll() is a blocking function
    pub fn into_blocking(self) -> IndexedIncrementalEncoder<Mode, Clk, Dt, Idx, Steps, T, Blocking>
    where
        IndexedIncrementalDecoder<Steps, T>: Default,
    {
        IndexedIncrementalEncoder::<Mode, Clk, Dt, Idx, Steps, T, Blocking> {
            decoder: self.decoder,
            pin_clk: self.pin_clk,
            pin_dt: self.pin_dt,
            pin_idx: self.pin_idx,
            pin_clk_state: self.pin_clk_state,
            pin_dt_state: self.pin_dt_state,
            pin_idx_state: self.pin_idx_state,
            is_reversed: self.is_reversed,
            _mode: PhantomData,
            _poll_mode: PhantomData,
        }
    }
}
