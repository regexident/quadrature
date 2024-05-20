//! A finite-state-transducer implementing full-step decoding.
//!
//! ```plain
//!          ┌───────────────────────────────ε────────────────────────┐    
//!          │             ┌─────01─────┐ ┌─────00─────┐              │    
//!          │             ▼            │ ▼            │              │    
//!          │        ┌──────┐       ┌──────┐       ┌──────┐       ╔══════╗
//!          │  ┌────▶│  F1  │──00──▶│  F2  │──10──▶│  F3  │──11──▶║  F!  ║
//!          │  01 ┌─▶└──────┘    ┌─▶└──────┘    ┌─▶└──────┘       ╚══════╝
//!          │  │  └01─┘ │        └00─┘ │        └10─┘ │                    
//! ┌00,11─┐ ▼  │        10,11          11             01                   
//! └────▶┌──────┐◀──────┘              ▼              ▼                    
//!    ●─▶│  N0  │◀───────────────────◀─┼────────────◀─┤                    
//!       └──────┘◀──────┐              ▲              ▲                    
//!          ▲  │        01,11          11             10                   
//!          │  │  ┌10─┐ │        ┌00─┐ │        ┌01─┐ │                    
//!          │  10 └─▶┌──────┐    └─▶┌──────┐    └─▶┌──────┐       ╔══════╗
//!          │  └────▶│  R1  │──00──▶│  R2  │──01──▶│  R3  │──11──▶║  F!  ║
//!          │        └──────┘       └──────┘       └──────┘       ╚══════╝
//!          │             ▲            │   ▲             │           │    
//!          │             └─────10─────┘   └─────00──────┘           │    
//!          └───────────────────────────────ε────────────────────────┘    
//! ```
//!
//! Double-bordered states are accepting (and also transitive) states that emit an output.

use crate::state_transducer::{Output, State, Transition, Transitions};

/// The transition table that defines the full-step finite-state-transducer.
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

    // columns: `A0B0`, `A0B1`, `A1B0`, `A1B1`
    [
        [t!(N0, N), t!(F1, N), t!(R1, N), t!(N0, N)], // row: `N0`
        [t!(F2, N), t!(F1, N), t!(N0, N), t!(N0, N)], // row: `F1`
        [t!(F2, N), t!(F1, N), t!(F3, N), t!(N0, N)], // row: `F2`
        [t!(F2, N), t!(N0, N), t!(F3, N), t!(N0, F)], // row: `F3`
        [t!(R2, N), t!(N0, N), t!(R1, N), t!(N0, N)], // row: `R1`
        [t!(R2, N), t!(R3, N), t!(R1, N), t!(N0, N)], // row: `R2`
        [t!(R2, N), t!(R3, N), t!(N0, N), t!(N0, R)], // row: `R3`
        // This row is unused in full-step mode, but needs to be provided
        // as it expects a transition matrix of certain dimensions:
        [t!(N0, E), t!(N0, E), t!(N0, E), t!(N0, E)], // row: `N2`
    ]
};

#[cfg(test)]
mod tests {
    use crate::{
        state_transducer::{
            full_step::TRANSITIONS,
            Input::{self, *},
            Output, State, StateTransducer,
        },
        Error, FullStep, IncrementalDecoder,
        QuadratureMovement::{self, *},
    };

    type Decoder = IncrementalDecoder<FullStep>;

    fn update(decoder: &mut Decoder, input: Input) -> Result<Option<QuadratureMovement>, Error> {
        decoder.update(input.a(), input.b())
    }

    #[test]
    fn initial_state() {
        let transducer = StateTransducer::new(&TRANSITIONS);

        assert_eq!(transducer.state(), State::N0);
    }

    #[test]
    fn identity() {
        let mut transducer = StateTransducer::new(&TRANSITIONS);

        let scenarios = [
            (State::N0, Input::A1B1),
            (State::F1, Input::A0B1),
            (State::F2, Input::A0B0),
            (State::F3, Input::A1B0),
            (State::R1, Input::A1B0),
            (State::R2, Input::A0B0),
            (State::R3, Input::A0B1),
            // State::N2 is not used by the full-step transducer.
        ];

        for (state, input) in scenarios {
            transducer.set_state(state);
            let output = transducer.step(input);
            assert_eq!(output, Output::N);
            assert_eq!(transducer.state(), state);
        }
    }

    mod clean {

        use super::*;

        #[test]
        fn forwards() {
            let mut decoder = Decoder::default();

            // Full cycle without redundant inputs:
            assert_eq!(update(&mut decoder, A0B1), Ok(None));
            assert_eq!(update(&mut decoder, A0B0), Ok(None));
            assert_eq!(update(&mut decoder, A1B0), Ok(None));
            assert_eq!(update(&mut decoder, A1B1), Ok(Some(AB)));
        }

        #[test]
        fn backwards() {
            let mut decoder = Decoder::default();

            // Full cycle without redundant inputs:
            assert_eq!(update(&mut decoder, A1B0), Ok(None));
            assert_eq!(update(&mut decoder, A0B0), Ok(None));
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
                assert_eq!(update(&mut decoder, A0B0), Ok(None)); // Redundant input
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
                assert_eq!(update(&mut decoder, A0B0), Ok(None)); // Redundant input
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
                assert_eq!(update(&mut decoder, A0B0), Ok(None));
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
                assert_eq!(update(&mut decoder, A0B0), Ok(None));
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
                assert_eq!(update(&mut decoder, A0B0), Ok(None));
                assert_eq!(update(&mut decoder, A0B1), Ok(None));
                assert_eq!(update(&mut decoder, A1B1), Ok(None));
            }

            #[test]
            fn backwards() {
                let mut decoder = Decoder::default();

                assert_eq!(update(&mut decoder, A1B0), Ok(None));
                assert_eq!(update(&mut decoder, A0B0), Ok(None));
                assert_eq!(update(&mut decoder, A1B0), Ok(None));
                assert_eq!(update(&mut decoder, A1B1), Ok(None));
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
                assert_eq!(update(&mut decoder, A0B0), Ok(None));
                assert_eq!(update(&mut decoder, A1B0), Ok(None));
                assert_eq!(update(&mut decoder, A0B1), Err(Error::E10_01)); // Noise input
                assert_eq!(update(&mut decoder, A1B1), Ok(None));

                decoder.reset();

                assert_eq!(update(&mut decoder, A0B1), Ok(None));
                assert_eq!(update(&mut decoder, A1B0), Err(Error::E01_10)); // Noise input
                assert_eq!(update(&mut decoder, A0B0), Ok(None));
                assert_eq!(update(&mut decoder, A1B0), Ok(None));
                assert_eq!(update(&mut decoder, A1B1), Ok(None));
            }

            #[test]
            fn backwards() {
                let mut decoder = Decoder::default();

                assert_eq!(update(&mut decoder, A1B0), Ok(None));
                assert_eq!(update(&mut decoder, A0B0), Ok(None));
                assert_eq!(update(&mut decoder, A0B1), Ok(None));
                assert_eq!(update(&mut decoder, A1B0), Err(Error::E01_10)); // Noise input
                assert_eq!(update(&mut decoder, A1B1), Ok(None));

                decoder.reset();

                assert_eq!(update(&mut decoder, A1B0), Ok(None));
                assert_eq!(update(&mut decoder, A0B1), Err(Error::E10_01)); // Noise input
                assert_eq!(update(&mut decoder, A0B0), Ok(None));
                assert_eq!(update(&mut decoder, A0B1), Ok(None));
                assert_eq!(update(&mut decoder, A1B1), Ok(None));
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
