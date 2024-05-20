//! Quadrature-based decoder.

use num_traits::{One, SaturatingAdd, Zero};

use crate::{index_decoder::IndexDecoder, Change, Error, IncrementalDecoder, StepMode};

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
    /// returning the direction if a change was detected, `None` if no change was detected,
    /// or `Err(_)` if an invalid input (i.e. a counteral "jump") was detected.
    ///
    /// Upon detection of a raising edge on the `z` pulse train the counter gets reset back to `0`.
    ///
    /// Depending on whether it matters why the decoder did not detect a change
    /// (e.g. due to actual lack of change or an erroneous read)
    /// you would either call `decoder.update(a, b)` directly, or via `decoder.update(a, b).unwrap_or_default()`
    /// to fall back to `None` in case of `Err(_)`.
    pub fn update(&mut self, a: bool, b: bool, z: bool) -> Result<Option<Change>, Error> {
        let result = self.decoder.update(a, b);

        if self.indexer.update(z) {
            self.decoder.set_counter(Zero::zero());
        }

        result
    }

    /// Resets the decoder to its initial state and its counter counter back to `0`.
    pub fn reset(&mut self) {
        self.decoder.reset();
        self.indexer.reset();
    }

    /// Returns the decoder's counter counter relative to its initial counter in number of cycles.
    ///
    /// A change of `Change::Positive` increments the counter counter,
    /// while a change of `Change::Negative` decrements it.
    pub fn counter(&self) -> T {
        self.decoder.counter()
    }

    /// Sets the decoder's counter.
    pub fn set_counter(&mut self, counter: T) {
        self.decoder.set_counter(counter);
    }
}

#[cfg(test)]
mod tests {
    use crate::HalfStep;

    use super::*;

    #[test]
    fn index() {
        let a: Vec<bool> = vec![false, false, true, true, false, false, true, true];
        let b: Vec<bool> = vec![true, false, false, true, true, false, false, true];
        let z: Vec<bool> = vec![false, false, false, false, true, false, false, false];

        let pulse_trains = a.into_iter().zip(b.into_iter()).zip(z.into_iter());

        let changes: Vec<Option<Change>> = vec![
            None,
            Some(Change::Positive),
            None,
            Some(Change::Positive),
            None,
            Some(Change::Positive),
            None,
            Some(Change::Positive),
        ];
        let counters: Vec<i32> = vec![0, 1, 1, 2, 0, 1, 1, 2];

        let expected = changes.into_iter().zip(counters.into_iter());

        let mut decoder: IndexedIncrementalDecoder<HalfStep> = Default::default();

        for (input, expected) in pulse_trains.zip(expected) {
            let ((a, b), z) = input;
            let (expected_change, expected_counter) = expected;

            let change = decoder.update(a, b, z).unwrap();

            assert_eq!(change, expected_change);
            assert_eq!(decoder.counter(), expected_counter);
        }
    }
}
