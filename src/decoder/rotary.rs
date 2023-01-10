//! A robust rotary quadrature decoder with support for multiple step-modes.

use crate::{Error, FullStep, HalfStep, QuadStep, QuadratureDecoder, QuadratureMovement, StepMode};

/// The movement detected by a rotary quadrature decoder.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum RotaryMovement {
    /// Clockwise movement (i.e. channel A leads channel B).
    Clockwise = 0,
    /// Counter-clockwise movement (i.e. channel B trails channel A).
    CounterClockwise = 1,
}

impl RotaryMovement {
    /// Flips the direction of `self`.
    pub fn flip(&mut self) {
        *self = self.flipped()
    }

    /// Returns the direction of `self`, flipped.
    pub fn flipped(self) -> Self {
        match self {
            Self::Clockwise => Self::CounterClockwise,
            Self::CounterClockwise => Self::Clockwise,
        }
    }
}

impl From<QuadratureMovement> for RotaryMovement {
    fn from(movement: QuadratureMovement) -> Self {
        match movement {
            QuadratureMovement::AB => Self::Clockwise,
            QuadratureMovement::BA => Self::CounterClockwise,
        }
    }
}

/// A robust rotary quadrature decoder with support for multiple step-modes.
#[derive(Debug)]
pub struct RotaryDecoder<Mode> {
    decoder: QuadratureDecoder<Mode>,
}

impl Default for RotaryDecoder<FullStep> {
    fn default() -> Self {
        Self::new(QuadratureDecoder::default())
    }
}

impl Default for RotaryDecoder<HalfStep> {
    fn default() -> Self {
        Self::new(QuadratureDecoder::default())
    }
}

impl Default for RotaryDecoder<QuadStep> {
    fn default() -> Self {
        Self::new(QuadratureDecoder::default())
    }
}

impl<Mode> RotaryDecoder<Mode>
where
    Mode: StepMode,
{
    pub(crate) fn new(decoder: QuadratureDecoder<Mode>) -> Self {
        Self { decoder }
    }

    /// Updates the decoder's state based on the given `a` and `b` pulse train readings,
    /// returning the direction if a movement was detected, `None` if no movement was detected,
    /// or `Err(_)` if an invalid input (i.e. a positional "jump") was detected.
    ///
    /// Depending on whether it matters why the decoder did not detect a movement
    /// (e.g. due to actual lack of movement or an erroneous read)
    /// you would either call `update()` directly:
    ///
    /// ```rust
    /// # let a: bool = true;
    /// # let b: bool = true;
    /// use quadrature_decoder::{FullStep, RotaryDecoder};
    ///
    /// let mut decoder = RotaryDecoder::<FullStep>::default();
    /// match decoder.update(a, b) {
    ///     Ok(Some(movement)) => println!("PhaseShift detected: {:?}.", movement),
    ///     Ok(None) => println!("No movement detected."),
    ///     Err(error) => println!("Error detected: {:?}.", error),
    /// }
    /// ```
    ///
    /// Or fall back to `None` in case of `Err(_)` by use of `.unwrap_or_default()`:
    ///
    /// ```rust
    /// # let a: bool = true;
    /// # let b: bool = true;
    /// use quadrature_decoder::{FullStep, RotaryDecoder};
    ///
    /// let mut decoder = RotaryDecoder::<FullStep>::default();
    /// match decoder.update(a, b).unwrap_or_default() {
    ///     Some(movement) => println!("PhaseShift detected: {:?}.", movement),
    ///     None => println!("No movement detected."),
    /// }
    /// ```
    pub fn update(&mut self, a: bool, b: bool) -> Result<Option<RotaryMovement>, Error> {
        self.decoder
            .update(a, b)
            .map(|option| option.map(From::from))
    }

    /// Resets the decoder to its initial state.
    pub fn reset(&mut self) {
        self.decoder.reset();
    }

    /// The decoder's number of pulses per (quadrature) cycle (PPC).
    ///
    /// As an example, consider the effectively pulses per revolution (PPR)
    /// of a rotary encoder with 100 cycles per revolution (CPR):
    ///
    /// - A step mode with 1 pulse per cycle (e.g. `RotaryDecoder<FullStep>`) results in effectively 100 pulses per revolution (100 PPR).
    /// - A step mode with 2 pulses per cycle (e.g. `RotaryDecoder<HalfStep>`) results in effectively 200 pulses per revolution (200 PPR).
    /// - A step mode with 4 pulses per cycle (e.g. `RotaryDecoder<QuadStep>`) results in effectively 400 pulses per revolution (400 PPR).
    pub fn pulses_per_cycle() -> usize {
        Mode::PULSES_PER_CYCLE
    }
}
