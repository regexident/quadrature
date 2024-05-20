//! Quadrature-based index decoder.

/// A decoder for detecting the rising edge of the index signal (z channel).
///
/// ```plain
///               ┌─┐                         high
/// Z             │ │                              
///   ─ ─ ─ ──────┘ └────────────────── ─ ─ ─ low  
/// ```
#[derive(Default, Debug)]
pub struct IndexDecoder {
    z: bool,
}

impl IndexDecoder {
    /// Creates a decoder with the given initial channel state.
    pub fn new(z: bool) -> Self {
        Self { z }
    }

    /// Resets the decoder to the default state
    pub fn reset(&mut self) {
        self.z = false;
    }

    /// Updates the internal state and returns `true` iff it
    /// detects a raising edge on the z channel, otherwise `false`.
    pub fn update(&mut self, z: bool) -> bool {
        let is_at_edge = self.z != z;

        self.z = z;

        z && is_at_edge
    }
}
