//! A robust quadrature decoder with support for multiple step-modes

use core::marker::PhantomData;

use crate::{
    state_transducer::{Input, Output},
    validator::InputValidator,
    Error, FullStep, HalfStep, Movement, QuadStep, StateTransducer, StepMode,
};

/// A robust quadrature decoder with support for multiple step-modes,
/// based on which channel (A vs. B) is leading the other.
///
/// ```plain
///                ┌ ─ ┐   ┌───┐   ┌───┐   ┌───┐   ┌ ─ ─ high
///            A           │   │   │   │   │                  
///              ─ ┘   └───┘   └───┘   └───┘   └ ─ ┘     low  
/// Forward:                                                  
///                  ┌ ─ ┐   ┌───┐   ┌───┐   ┌───┐   ┌ ─ high
///            B             │   │   │   │   │                
///              ─ ─ ┘   └───┘   └───┘   └───┘   └ ─ ┘   low  
/// Time: ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶
///                  ┌ ─ ┐   ┌───┐   ┌───┐   ┌───┐   ┌ ─ high
///            A             │   │   │   │   │                
///              ─ ─ ┘   └───┘   └───┘   └───┘   └ ─ ┘   low  
/// Reverse:                                                  
///                ┌ ─ ┐   ┌───┐   ┌───┐   ┌───┐   ┌ ─ ─ high
///            B           │   │   │   │   │                  
///              ─ ┘   └───┘   └───┘   └───┘   └ ─ ┘     low  
/// ```
#[derive(Debug)]
pub struct QuadratureDecoder<Mode> {
    transducer: StateTransducer<'static, 8, 4>,
    validator: InputValidator,
    _phantom: PhantomData<Mode>,
}

impl Default for QuadratureDecoder<FullStep> {
    fn default() -> Self {
        Self::new(StateTransducer::new(
            &crate::state_transducer::full_step::TRANSITIONS,
        ))
    }
}

impl Default for QuadratureDecoder<HalfStep> {
    fn default() -> Self {
        Self::new(StateTransducer::new(
            &crate::state_transducer::half_step::TRANSITIONS,
        ))
    }
}

impl Default for QuadratureDecoder<QuadStep> {
    fn default() -> Self {
        Self::new(StateTransducer::new(
            &crate::state_transducer::quad_step::TRANSITIONS,
        ))
    }
}

impl<Mode> QuadratureDecoder<Mode>
where
    Mode: StepMode,
{
    pub(crate) fn new(transducer: StateTransducer<'static, 8, 4>) -> Self {
        Self {
            transducer,
            validator: Default::default(),
            _phantom: PhantomData,
        }
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
    /// use quadrature_decoder::{FullStep, QuadratureDecoder};
    ///
    /// let mut decoder = QuadratureDecoder::<FullStep>::default();
    /// match decoder.update(a, b) {
    ///     Ok(Some(movement)) => println!("Movement detected: {:?}.", movement),
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
    /// use quadrature_decoder::{FullStep, QuadratureDecoder};
    ///
    /// let mut decoder = QuadratureDecoder::<FullStep>::default();
    /// match decoder.update(a, b).unwrap_or_default() {
    ///     Some(movement) => println!("Movement detected: {:?}.", movement),
    ///     None => println!("No movement detected."),
    /// }
    /// ```
    pub fn update(&mut self, a: bool, b: bool) -> Result<Option<Movement>, Error> {
        let input = Input::new(a, b);

        let validation_result = self.validator.validate(input);
        let transducer_output = self.transducer.step(input);

        match (validation_result, transducer_output) {
            (Err(error), output) => {
                debug_assert_eq!(output, Output::N, "Expected `None` output from transducer.");
                Err(error)
            }
            (Ok(_), Output::N) => Ok(None),
            (Ok(_), Output::F) => Ok(Some(Movement::Forward)),
            (Ok(_), Output::R) => Ok(Some(Movement::Reverse)),
            (_, Output::E) => {
                // Transducers are expected to not return error outputs since their states tend to
                // be insufficient for reliable detection without false positives/negatives.
                panic!("Unexpected error output from transducer.")
            }
        }
    }

    /// Resets the decoder to its initial state.
    pub fn reset(&mut self) {
        self.transducer.reset();
        self.validator.reset();
    }

    /// The decoder's number of pulses per (quadrature) cycle (PPC).
    ///
    /// As an example, consider the effectively pulses per revolution (PPR)
    /// of a rotary encoder with 100 cycles per revolution (CPR):
    ///
    /// - A step mode with 1 pulse per cycle (e.g. `QuadratureDecoder<FullStep>`) results in effectively 100 pulses per revolution (100 PPR).
    /// - A step mode with 2 pulses per cycle (e.g. `QuadratureDecoder<HalfStep>`) results in effectively 200 pulses per revolution (200 PPR).
    /// - A step mode with 4 pulses per cycle (e.g. `QuadratureDecoder<QuadStep>`) results in effectively 400 pulses per revolution (400 PPR).
    pub fn pulses_per_cycle() -> usize {
        Mode::PULSES_PER_CYCLE
    }
}
