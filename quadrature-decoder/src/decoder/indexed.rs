//! Quadrature-based decoder.

use num_traits::{One, SaturatingAdd, Zero};

use crate::{Error, IncrementalDecoder, IndexDecoder, QuadratureMovement, StepMode};

/// A robust indexed quadrature decoder with support for multiple step-modes,
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
///
///                          ┌─┐                         high
///            Z             │ │                              
///              ─ ─ ─ ──────┘ └────────────────── ─ ─ ─ low  
/// Time: ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶
///                  ┌ ─ ┐   ┌───┐   ┌───┐   ┌───┐   ┌ ─ high
///            A             │   │   │   │   │                
///              ─ ─ ┘   └───┘   └───┘   └───┘   └ ─ ┘   low  
/// BA:                                                  
///                ┌ ─ ┐   ┌───┐   ┌───┐   ┌───┐   ┌ ─ ─ high
///            B           │   │   │   │   │                  
///              ─ ┘   └───┘   └───┘   └───┘   └ ─ ┘     low  
///
///                          ┌─┐                         high
///            Z             │ │                              
///              ─ ─ ─ ──────┘ └────────────────── ─ ─ ─ low  
/// ```
#[derive(Debug)]
pub struct IndexedIncrementalDecoder<Mode, T = i32> {
    decoder: IncrementalDecoder<Mode, T>,
    indexer: IndexDecoder,
}

impl<Mode, T> Default for IndexedIncrementalDecoder<Mode, T>
where
    Mode: StepMode,
    IncrementalDecoder<Mode, T>: Default,
{
    fn default() -> Self {
        Self::new(IncrementalDecoder::default())
    }
}

impl<Mode, T> IndexedIncrementalDecoder<Mode, T>
where
    Mode: StepMode,
{
    pub(crate) fn new(decoder: IncrementalDecoder<Mode, T>) -> Self {
        Self {
            decoder,
            indexer: Default::default(),
        }
    }
}

impl<Mode, T> IndexedIncrementalDecoder<Mode, T>
where
    Mode: StepMode,
    T: Copy + Zero + One + SaturatingAdd + From<i8>,
{
    /// Updates the decoder's state based on the given `a` and `b` pulse train (aka channel) readings,
    /// returning the direction if a movement was detected, `None` if no movement was detected,
    /// or `Err(_)` if an invalid input (i.e. a positional "jump") was detected.
    ///
    /// Upon detection of a raising edge on the `z` pulse train the position gets reset back to `0`.
    ///
    /// Depending on whether it matters why the decoder did not detect a movement
    /// (e.g. due to actual lack of movement or an erroneous read)
    /// you would either call `update()` directly:
    ///
    /// ```rust
    /// # let a: bool = true;
    /// # let b: bool = true;
    /// # let z: bool = false;
    ///
    /// use quadrature_decoder::{FullStep, IndexedIncrementalDecoder};
    ///
    /// let mut decoder = IndexedIncrementalDecoder::<FullStep>::default();
    /// match decoder.update(a, b, z) {
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
    /// # let z: bool = false;
    ///
    /// use quadrature_decoder::{FullStep, IndexedIncrementalDecoder};
    ///
    /// let mut decoder = IndexedIncrementalDecoder::<FullStep>::default();
    /// match decoder.update(a, b, z).unwrap_or_default() {
    ///     Some(movement) => println!("Movement detected: {:?}.", movement),
    ///     None => println!("No movement detected."),
    /// }
    /// println!("position: {:?}", decoder.position());
    /// ```
    pub fn update(
        &mut self,
        a: bool,
        b: bool,
        z: bool,
    ) -> Result<Option<QuadratureMovement>, Error> {
        let result = self.decoder.update(a, b);

        if self.indexer.update(z) {
            self.decoder.set_position(Zero::zero());
        }

        result
    }

    /// Resets the decoder to its initial state and its position counter back to `0`.
    pub fn reset(&mut self) {
        self.decoder.reset();
        self.indexer.reset();
    }

    /// Returns the decoder's position counter relative to its initial position in number of cycles.
    ///
    /// A movement of direction `QuadratureMovement::AB` increments the position counter,
    /// while a movement of direction `QuadratureMovement::BA` decrements it.
    pub fn position(&self) -> T {
        self.decoder.position()
    }

    /// Sets the decoder's position.
    pub fn set_position(&mut self, position: T) {
        self.decoder.set_position(position);
    }
}
