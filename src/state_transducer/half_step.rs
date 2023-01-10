//! A finite-state-transducer implementing half-step decoding.
//!
//! ```plain
//!       ┌───────────────────────────────ε─────────────────────────┐  
//!       │         ┌──────┐       ╔══════╗       ┌──────┐       ╔══════╗
//!       │ ┌──────▶│  F1  │──00──▶║  F!  ║ ┌────▶│  F3  │──11──▶║  F!  ║
//!       │ 01   ┌─▶└──────┘       ╚══════╝ │  ┌─▶└──────┘       ╚══════╝
//!       │ │    └01─┘ │              │     10 └10─┘  │                 
//!       │ │ ┌──10,11─┘              ε  ┌──┘         │                 
//! ┌11─┐ ▼ │ ▼                 ┌00─┐ ▼  │            │                 
//! └─▶┌──────┐                 └─▶┌──────┐◀───01,00──┘               
//! ●─▶│  N0  │────────00─────────▶│  N2  │                            
//!    └──────┘◀───────11──────────└──────┘◀───10,00──┐               
//!       ▲ │ ▲                       ▲  │            │                 
//!       │ │ └──01,11─┐              ε  └──┐         │                 
//!       │ │    ┌10─┐ │              │     01 ┌01─┐  │                 
//!       │ 10   └─▶┌──────┐       ╔══════╗ │  └─▶┌──────┐       ╔══════╗
//!       │ └──────▶│  R1  │──00──▶║  R!  ║ └────▶│  R3  │──11──▶║  R!  ║
//!       │         └──────┘       ╚══════╝       └──────┘       ╚══════╝
//!       └───────────────────────────────ε─────────────────────────┘    
//! ```
//!
//! Double-bordered states are accepting (and also transitive) states that emit an output.

use crate::state_transducer::{Output, State, Transition, Transitions};

/// The transition table that defines the half-step finite-state-transducer.
///
/// Rows correspond to a set of transitions per state,
/// with the integer value of the state indicating the row index.
/// Columns correspond to individual transitions per state,
/// with the integer value of the input indicating the column index.
pub(crate) static TRANSITIONS: Transitions<8, 4> = {
    use self::{Output::*, State::*};

    macro_rules! t {
        ($s:expr, $o:expr) => {
            Transition::new($s, $o)
        };
    }

    // Columns: `A0B0`, `A0B1`, `A1B0`, `A1B1`
    [
        [t!(N2, N), t!(F1, N), t!(R1, N), t!(N0, N)], // row: `N0`
        [t!(N2, F), t!(F1, N), t!(N0, N), t!(N0, N)], // row: `F1`
        // This row is unused in half-step mode, but needs to be provided
        // as it expects a transition matrix of certain dimensions:
        [t!(N0, E), t!(N0, E), t!(N0, E), t!(N0, E)], // row: `F2`
        [t!(N2, N), t!(N2, N), t!(F3, N), t!(N0, F)], // row: `F3`
        [t!(N2, R), t!(N0, N), t!(R1, N), t!(N0, N)], // row: `R1`
        // This row is unused in half-step mode, but needs to be provided
        // as it expects a transition matrix of certain dimensions:
        [t!(N0, E), t!(N0, E), t!(N0, E), t!(N0, E)], // row: `R2`
        [t!(N2, N), t!(R3, N), t!(N2, N), t!(N0, R)], // row: `R3`
        [t!(N2, N), t!(R3, N), t!(F3, N), t!(N0, N)], // row: `N2`
    ]
};

#[cfg(test)]
mod tests {
    use crate::{
        state_transducer::Input::{self, *},
        Error, HalfStep, QuadratureDecoder,
        QuadratureMovement::{self, *},
    };

    type Decoder = QuadratureDecoder<HalfStep>;

    fn update(decoder: &mut Decoder, input: Input) -> Result<Option<QuadratureMovement>, Error> {
        decoder.update(input.a(), input.b())
    }

    mod clean {
        use super::*;

        #[test]
        fn forwards() {
            let mut decoder = Decoder::default();

            // Full cycle without redundant inputs:
            assert_eq!(update(&mut decoder, A0B1), Ok(None));
            assert_eq!(update(&mut decoder, A0B0), Ok(Some(AB)));
            assert_eq!(update(&mut decoder, A1B0), Ok(None));
            assert_eq!(update(&mut decoder, A1B1), Ok(Some(AB)));
        }

        #[test]
        fn backwards() {
            let mut decoder = Decoder::default();

            // Full cycle without redundant inputs:
            assert_eq!(update(&mut decoder, A1B0), Ok(None));
            assert_eq!(update(&mut decoder, A0B0), Ok(Some(BA)));
            assert_eq!(update(&mut decoder, A0B1), Ok(None));
            assert_eq!(update(&mut decoder, A1B1), Ok(Some(BA)));
        }
    }

    mod stutter {
        use super::*;

        mod block {
            use super::*;

            #[test]
            fn forwards() {
                let mut decoder = Decoder::default();

                // Full cycle with block of redundant inputs:
                assert_eq!(update(&mut decoder, A0B1), Ok(None));
                assert_eq!(update(&mut decoder, A0B0), Ok(Some(AB))); // Redundant input
                assert_eq!(update(&mut decoder, A0B0), Ok(None)); // Redundant input
                assert_eq!(update(&mut decoder, A0B0), Ok(None)); // Redundant input
                assert_eq!(update(&mut decoder, A1B0), Ok(None));
                assert_eq!(update(&mut decoder, A1B1), Ok(Some(AB)));
            }

            #[test]
            fn backwards() {
                let mut decoder = Decoder::default();

                // Full cycle with block of redundant inputs:
                assert_eq!(update(&mut decoder, A1B0), Ok(None));
                assert_eq!(update(&mut decoder, A0B0), Ok(Some(BA))); // Redundant input
                assert_eq!(update(&mut decoder, A0B0), Ok(None)); // Redundant input
                assert_eq!(update(&mut decoder, A0B0), Ok(None)); // Redundant input
                assert_eq!(update(&mut decoder, A0B1), Ok(None));
                assert_eq!(update(&mut decoder, A1B1), Ok(Some(BA)));
            }
        }

        mod interleaved {
            use super::*;

            #[test]
            fn forwards() {
                let mut decoder = Decoder::default();

                // Full cycle with alternating redundant inputs:
                assert_eq!(update(&mut decoder, A0B1), Ok(None));
                assert_eq!(update(&mut decoder, A0B1), Ok(None)); // Redundant input
                assert_eq!(update(&mut decoder, A0B0), Ok(Some(AB)));
                assert_eq!(update(&mut decoder, A0B0), Ok(None)); // Redundant input
                assert_eq!(update(&mut decoder, A1B0), Ok(None));
                assert_eq!(update(&mut decoder, A1B0), Ok(None)); // Redundant input
                assert_eq!(update(&mut decoder, A1B1), Ok(Some(AB)));
            }

            #[test]
            fn backwards() {
                let mut decoder = Decoder::default();

                // Full cycle with alternating redundant inputs:
                assert_eq!(update(&mut decoder, A1B0), Ok(None));
                assert_eq!(update(&mut decoder, A1B0), Ok(None)); // Redundant input
                assert_eq!(update(&mut decoder, A0B0), Ok(Some(BA)));
                assert_eq!(update(&mut decoder, A0B0), Ok(None)); // Redundant input
                assert_eq!(update(&mut decoder, A0B1), Ok(None));
                assert_eq!(update(&mut decoder, A0B1), Ok(None)); // Redundant input
                assert_eq!(update(&mut decoder, A1B1), Ok(Some(BA)));
            }
        }
    }

    mod direction_change {
        use super::*;

        mod half {
            use super::*;

            #[test]
            fn forwards() {
                let mut decoder = Decoder::default();

                assert_eq!(update(&mut decoder, A0B1), Ok(None));
                assert_eq!(update(&mut decoder, A0B0), Ok(Some(AB)));
                assert_eq!(update(&mut decoder, A0B1), Ok(None));
                assert_eq!(update(&mut decoder, A1B1), Ok(Some(BA)));
            }

            #[test]
            fn backwards() {
                let mut decoder = Decoder::default();

                assert_eq!(update(&mut decoder, A1B0), Ok(None));
                assert_eq!(update(&mut decoder, A0B0), Ok(Some(BA)));
                assert_eq!(update(&mut decoder, A1B0), Ok(None));
                assert_eq!(update(&mut decoder, A1B1), Ok(Some(AB)));
            }
        }

        mod quarter {
            use super::*;

            #[test]
            fn forwards() {
                let mut decoder = Decoder::default();

                assert_eq!(update(&mut decoder, A0B1), Ok(None));
                assert_eq!(update(&mut decoder, A1B1), Ok(None));
            }

            #[test]
            fn backwards() {
                let mut decoder = Decoder::default();

                assert_eq!(update(&mut decoder, A1B0), Ok(None));
                assert_eq!(update(&mut decoder, A1B1), Ok(None));
            }
        }
    }

    mod noise {
        use super::*;

        mod single {
            use super::*;

            #[test]
            fn forwards() {
                let mut decoder = Decoder::default();

                assert_eq!(update(&mut decoder, A0B1), Ok(None));
                assert_eq!(update(&mut decoder, A0B0), Ok(Some(AB)));
                assert_eq!(update(&mut decoder, A1B0), Ok(None));
                assert_eq!(update(&mut decoder, A0B1), Err(Error::E10_01)); // Noise input
                assert_eq!(update(&mut decoder, A1B1), Ok(None));

                decoder.reset();

                assert_eq!(update(&mut decoder, A0B1), Ok(None));
                assert_eq!(update(&mut decoder, A1B0), Err(Error::E01_10)); // Noise input
                assert_eq!(update(&mut decoder, A0B0), Ok(None));
                assert_eq!(update(&mut decoder, A1B0), Ok(None));
                assert_eq!(update(&mut decoder, A1B1), Ok(Some(AB)));
            }

            #[test]
            fn backwards() {
                let mut decoder = Decoder::default();

                assert_eq!(update(&mut decoder, A1B0), Ok(None));
                assert_eq!(update(&mut decoder, A0B0), Ok(Some(BA)));
                assert_eq!(update(&mut decoder, A0B1), Ok(None));
                assert_eq!(update(&mut decoder, A1B0), Err(Error::E01_10)); // Noise input
                assert_eq!(update(&mut decoder, A1B1), Ok(None));

                decoder.reset();

                assert_eq!(update(&mut decoder, A1B0), Ok(None));
                assert_eq!(update(&mut decoder, A0B1), Err(Error::E10_01)); // Noise input
                assert_eq!(update(&mut decoder, A0B0), Ok(None));
                assert_eq!(update(&mut decoder, A0B1), Ok(None));
                assert_eq!(update(&mut decoder, A1B1), Ok(Some(BA)));
            }
        }

        mod double {
            use super::*;

            #[test]
            fn forwards() {
                let mut decoder = Decoder::default();

                assert_eq!(update(&mut decoder, A0B1), Ok(None));
                assert_eq!(update(&mut decoder, A1B0), Err(Error::E01_10)); // Noise input
                assert_eq!(update(&mut decoder, A0B0), Ok(None));
                assert_eq!(update(&mut decoder, A1B0), Ok(None));
                assert_eq!(update(&mut decoder, A0B1), Err(Error::E10_01)); // Noise input
                assert_eq!(update(&mut decoder, A1B1), Ok(None));
            }

            #[test]
            fn backwards() {
                let mut decoder = Decoder::default();

                assert_eq!(update(&mut decoder, A1B0), Ok(None));
                assert_eq!(update(&mut decoder, A0B1), Err(Error::E10_01)); // Noise input
                assert_eq!(update(&mut decoder, A0B0), Ok(None));
                assert_eq!(update(&mut decoder, A0B1), Ok(None));
                assert_eq!(update(&mut decoder, A1B0), Err(Error::E01_10)); // Noise input
                assert_eq!(update(&mut decoder, A1B1), Ok(None));
            }
        }

        mod quadruple {
            use super::*;

            #[test]
            fn forwards() {
                let mut decoder = Decoder::default();

                assert_eq!(update(&mut decoder, A0B0), Err(Error::E11_00)); // Noise input
                assert_eq!(update(&mut decoder, A0B1), Ok(None));
                assert_eq!(update(&mut decoder, A1B0), Err(Error::E01_10)); // Noise input
                assert_eq!(update(&mut decoder, A0B0), Ok(None));
                assert_eq!(update(&mut decoder, A1B1), Err(Error::E00_11)); // Noise input
                assert_eq!(update(&mut decoder, A1B0), Ok(None));
                assert_eq!(update(&mut decoder, A0B1), Err(Error::E10_01)); // Noise input
                assert_eq!(update(&mut decoder, A1B1), Ok(None));
            }

            #[test]
            fn backwards() {
                let mut decoder = Decoder::default();

                assert_eq!(update(&mut decoder, A0B0), Err(Error::E11_00)); // Noise input
                assert_eq!(update(&mut decoder, A1B0), Ok(None));
                assert_eq!(update(&mut decoder, A0B1), Err(Error::E10_01)); // Noise input
                assert_eq!(update(&mut decoder, A0B0), Ok(None));
                assert_eq!(update(&mut decoder, A1B1), Err(Error::E00_11)); // Noise input
                assert_eq!(update(&mut decoder, A0B1), Ok(None));
                assert_eq!(update(&mut decoder, A1B0), Err(Error::E01_10)); // Noise input
                assert_eq!(update(&mut decoder, A1B1), Ok(None));
            }
        }
    }
}
