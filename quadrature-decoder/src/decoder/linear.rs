//! A robust linear quadrature decoder with support for multiple step-modes.

use crate::{Error, FullStep, HalfStep, QuadStep, QuadratureDecoder, QuadratureMovement, StepMode};

/// The movement detected by a linear quadrature decoder.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum LinearMovement {
    /// Forward movement (i.e. channel A leads channel B).
    Forward = 0,
    /// Reverse movement (i.e. channel B trails channel A).
    Reverse = 1,
}

impl LinearMovement {
    /// Flips the direction of `self`.
    pub fn flip(&mut self) {
        *self = self.flipped()
    }

    /// Returns the direction of `self`, flipped.
    pub fn flipped(self) -> Self {
        match self {
            Self::Forward => Self::Reverse,
            Self::Reverse => Self::Forward,
        }
    }
}

impl From<QuadratureMovement> for LinearMovement {
    fn from(movement: QuadratureMovement) -> Self {
        match movement {
            QuadratureMovement::AB => Self::Forward,
            QuadratureMovement::BA => Self::Reverse,
        }
    }
}

/// A robust linear quadrature decoder with support for multiple step-modes.
#[derive(Debug)]
pub struct LinearDecoder<Mode> {
    decoder: QuadratureDecoder<Mode>,
}

impl Default for LinearDecoder<FullStep> {
    fn default() -> Self {
        Self::new(QuadratureDecoder::default())
    }
}

impl Default for LinearDecoder<HalfStep> {
    fn default() -> Self {
        Self::new(QuadratureDecoder::default())
    }
}

impl Default for LinearDecoder<QuadStep> {
    fn default() -> Self {
        Self::new(QuadratureDecoder::default())
    }
}

impl<Mode> LinearDecoder<Mode>
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
    /// use quadrature_decoder::{FullStep, LinearDecoder};
    ///
    /// let mut decoder = LinearDecoder::<FullStep>::default();
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
    /// use quadrature_decoder::{FullStep, LinearDecoder};
    ///
    /// let mut decoder = LinearDecoder::<FullStep>::default();
    /// match decoder.update(a, b).unwrap_or_default() {
    ///     Some(movement) => println!("PhaseShift detected: {:?}.", movement),
    ///     None => println!("No movement detected."),
    /// }
    /// ```
    pub fn update(&mut self, a: bool, b: bool) -> Result<Option<LinearMovement>, Error> {
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
    /// - A step mode with 1 pulse per cycle (e.g. `LinearDecoder<FullStep>`) results in effectively 100 pulses per revolution (100 PPR).
    /// - A step mode with 2 pulses per cycle (e.g. `LinearDecoder<HalfStep>`) results in effectively 200 pulses per revolution (200 PPR).
    /// - A step mode with 4 pulses per cycle (e.g. `LinearDecoder<QuadStep>`) results in effectively 400 pulses per revolution (400 PPR).
    pub fn pulses_per_cycle() -> usize {
        Mode::PULSES_PER_CYCLE
    }
}
