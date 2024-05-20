//! Quadrature-based decoder.

use core::marker::PhantomData;

use num_traits::{One, SaturatingAdd, Zero};

use crate::{
    state_transducer::{Input, Output},
    validator::InputValidator,
    Error, FullStep, HalfStep, QuadStep, QuadratureMovement, StateTransducer, StepMode,
};

/// A robust quadrature decoder with support for multiple step-modes,
/// based on which channel (A vs. B) is leading the other.
///
/// ```plain
///                ┌ ─ ┐   ┌───┐   ┌───┐   ┌───┐   ┌ ─ ─ high
///            A           │   │   │   │   │                  
///              ─ ┘   └───┘   └───┘   └───┘   └ ─ ┘     low  
/// AB:                                                  
///                  ┌ ─ ┐   ┌───┐   ┌───┐   ┌───┐   ┌ ─ high
///            B             │   │   │   │   │                
///              ─ ─ ┘   └───┘   └───┘   └───┘   └ ─ ┘   low  
/// Time: ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶
///                  ┌ ─ ┐   ┌───┐   ┌───┐   ┌───┐   ┌ ─ high
///            A             │   │   │   │   │                
///              ─ ─ ┘   └───┘   └───┘   └───┘   └ ─ ┘   low  
/// BA:                                                  
///                ┌ ─ ┐   ┌───┐   ┌───┐   ┌───┐   ┌ ─ ─ high
///            B           │   │   │   │   │                  
///              ─ ┘   └───┘   └───┘   └───┘   └ ─ ┘     low  
/// ```
#[derive(Debug)]
pub struct IncrementalDecoder<Mode, T = i32> {
    transducer: StateTransducer<'static, 8, 4>,
    validator: InputValidator,
    position: T,
    _phantom: PhantomData<Mode>,
}

impl<T> Default for IncrementalDecoder<FullStep, T>
where
    T: Zero,
{
    fn default() -> Self {
        Self::new(StateTransducer::new(
            &crate::state_transducer::full_step::TRANSITIONS,
        ))
    }
}

impl<T> Default for IncrementalDecoder<HalfStep, T>
where
    T: Zero,
{
    fn default() -> Self {
        Self::new(StateTransducer::new(
            &crate::state_transducer::half_step::TRANSITIONS,
        ))
    }
}

impl<T> Default for IncrementalDecoder<QuadStep, T>
where
    T: Zero,
{
    fn default() -> Self {
        Self::new(StateTransducer::new(
            &crate::state_transducer::quad_step::TRANSITIONS,
        ))
    }
}

impl<Mode, T> IncrementalDecoder<Mode, T>
where
    Mode: StepMode,
    T: Zero,
{
    pub(crate) fn new(transducer: StateTransducer<'static, 8, 4>) -> Self {
        Self {
            transducer,
            validator: Default::default(),
            position: Zero::zero(),
            _phantom: PhantomData,
        }
    }
}

impl<Mode, T> IncrementalDecoder<Mode, T>
where
    Mode: StepMode,
    T: Copy + Zero + One + SaturatingAdd + From<i8>,
{
    /// Updates the decoder's state based on the given `a` and `b` pulse train (aka channel) readings,
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
    /// use quadrature_decoder::{FullStep, IncrementalDecoder};
    ///
    /// let mut decoder = IncrementalDecoder::<FullStep>::default();
    /// match decoder.update(a, b) {
    ///     Ok(Some(movement)) => println!("Movement detected: {:?}.", movement),
    ///     Ok(None) => println!("No movement detected."),
    ///     Err(error) => println!("Error detected: {:?}.", error),
    /// }
    /// println!("position: {:?}", decoder.position());
    /// ```
    ///
    /// Or fall back to `None` in case of `Err(_)` by use of `.unwrap_or_default()`:
    ///
    /// ```rust
    /// # let a: bool = true;
    /// # let b: bool = true;
    /// use quadrature_decoder::{FullStep, IncrementalDecoder};
    ///
    /// let mut decoder = IncrementalDecoder::<FullStep>::default();
    /// match decoder.update(a, b).unwrap_or_default() {
    ///     Some(movement) => println!("Movement detected: {:?}.", movement),
    ///     None => println!("No movement detected."),
    /// }
    /// println!("position: {:?}", decoder.position());
    /// ```
    pub fn update(&mut self, a: bool, b: bool) -> Result<Option<QuadratureMovement>, Error> {
        let input = Input::new(a, b);

        let validation_result = self.validator.validate(input);
        let transducer_output = self.transducer.step(input);

        match (validation_result, transducer_output) {
            (Err(error), output) => {
                debug_assert_eq!(output, Output::N, "Expected `None` output from transducer.");
                Err(error)
            }
            (Ok(_), Output::N) => Ok(None),
            (Ok(_), Output::F) => {
                let movement = QuadratureMovement::AB;
                let delta: T = (movement as i8).into();
                self.position = self.position.saturating_add(&delta);
                Ok(Some(movement))
            }
            (Ok(_), Output::R) => {
                let movement = QuadratureMovement::BA;
                let delta: T = (movement as i8).into();
                self.position = self.position.saturating_add(&delta);
                Ok(Some(movement))
            }
            (_, Output::E) => {
                // Transducers are expected to not return error outputs since their states tend to
                // be insufficient for reliable detection without false positives/negatives.
                panic!("Unexpected error output from transducer.")
            }
        }
    }

    /// Resets the decoder to its initial state and its position counter back to `0`.
    pub fn reset(&mut self) {
        self.transducer.reset();
        self.validator.reset();
        self.position = Zero::zero();
    }

    /// Returns the decoder's position counter relative to its initial position in number of cycles.
    ///
    /// A movement of direction `QuadratureMovement::AB` increments the position counter,
    /// while a movement of direction `QuadratureMovement::BA` decrements it.
    pub fn position(&self) -> T {
        self.position
    }

    /// Sets the decoder's position.
    pub fn set_position(&mut self, position: T) {
        self.position = position;
    }
}
