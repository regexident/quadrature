//! Quadrature-based decoder.

use core::marker::PhantomData;

use num_traits::{One, SaturatingAdd, Zero};

use crate::{
    state_transducer::{Input, Output},
    validator::InputValidator,
    Change, Error, FullStep, HalfStep, QuadStep, StateTransducer, StepMode,
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
    counter: T,
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
            counter: Zero::zero(),
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
    /// returning the direction if a change was detected, `None` if no change was detected,
    /// or `Err(_)` if an invalid input (i.e. a counteral "jump") was detected.
    ///
    /// Depending on whether it matters why the decoder did not detect a change
    /// (e.g. due to actual lack of change or an erroneous read)
    /// you would either call `decoder.update(a, b)` directly, or via `decoder.update(a, b).unwrap_or_default()`
    /// to fall back to `None` in case of `Err(_)`.
    pub fn update(&mut self, a: bool, b: bool) -> Result<Option<Change>, Error> {
        let input = Input::new(a, b);

        let validation_result = self.validator.validate(input);
        let transducer_output = self.transducer.step(input);

        match (validation_result, transducer_output) {
            (Err(error), output) => {
                debug_assert_eq!(output, Output::N, "Expected `None` output from transducer.");
                Err(error)
            }
            (Ok(_), Output::N) => Ok(None),
            (Ok(_), Output::AB) => {
                let change = Change::Positive;
                let delta: T = (change as i8).into();
                self.counter = self.counter.saturating_add(&delta);
                Ok(Some(change))
            }
            (Ok(_), Output::BA) => {
                let change = Change::Negative;
                let delta: T = (change as i8).into();
                self.counter = self.counter.saturating_add(&delta);
                Ok(Some(change))
            }
            (_, Output::E) => {
                // Transducers are expected to not return error outputs since their states tend to
                // be insufficient for reliable detection without false positives/negatives.
                panic!("Unexpected error output from transducer.")
            }
        }
    }

    /// Resets the decoder to its initial state and its counter counter back to `0`.
    pub fn reset(&mut self) {
        self.transducer.reset();
        self.validator.reset();
        self.counter = Zero::zero();
    }

    /// Returns the decoder's counter counter relative to its initial counter in number of cycles.
    ///
    /// A change of `Change::Positive` increments the counter counter,
    /// while a change of `Change::Negative` decrements it.
    pub fn counter(&self) -> T {
        self.counter
    }

    /// Sets the decoder's counter.
    pub fn set_counter(&mut self, counter: T) {
        self.counter = counter;
    }
}
