//! A finite-state-transducer implementing quad-step decoding.
//!
//! ```plain
//!       ┌────────────────────────────ε──────────────────────────┐
//!       │       ┌──────┐     ╔══════╗     ┌──────┐     ╔══════╗ │
//!       │ ┌─────│  F1  │─00─▶║  F!  ║┌────│  F2  │─11─▶║  F!  ║─┘
//!       │ │  ┌─▶└──────┘     ╚══════╝│ ┌─▶└──────┘     ╚══════╝  
//!       │ │  └01─┘ ▲  └──11──┐   │   │ └10─┘ ▲  └───00───┐                  
//!       │ 10       └──ε─┐    ▼   └─┐ │       └───ε──┐    ▼        
//!       │ │    ╔══════╗ │ ╔══════╗ │ 01    ╔══════╗ │  ╔══════╗           
//!       │ │ ┌─▶║  F!  ║─┘ ║  R!  ║ ε │ ┌──▶║  F!  ║─┘  ║  R!  ║           
//!       │ │ 01 ╚══════╝   ╚══════╝ │ │ 10  ╚══════╝    ╚══════╝           
//!       │ │ │               │      │ │ │                  │
//! ┌11─┐ ▼ ▼ │               │ ┌00─┐▼ ▼ │                  │        
//! └─▶┌──────┐◀───────ε──────┘ └─▶┌──────┐◀────────ε───────┘                             
//! ●─▶│  N0  │────────00─────────▶│  N2  │                             
//!    └──────┘◀───────11──────────└──────┘◀────────ε───────┐                             
//!     ▲ ▲ │ ▲                      ▲ ▲ │                  │                   
//!     │ │ │ └────────ε───────┐     │ │ │                  │  
//!     │ │ 10   ╔══════╗   ╔══════╗ │ │ 01  ╔══════╗     ╔══════╗               
//!     │ │ └───▶║  R!  ║─┐ ║  F!  ║ ε │ └──▶║  R!  ║──┐  ║  F!  ║              
//!     │ │      ╚══════╝ │ ╚══════╝ │ │     ╚══════╝  │  ╚══════╝               
//!     │ 01         ┌──ε─┘   ▲    ┌─┘ 10       ┌───ε──┘    ▲         
//!     │ │    ┌10─┐ ▼  ┌──11─┘    │   └┐ ┌01─┐ ▼  ┌───00───┘                  
//!     │ │    └─▶┌──────┐     ╔══════╗ │ └─▶┌──────┐     ╔══════╗  
//!     │ └───────│  R1  │─00─▶║  R!  ║ └────│  R2  │─11─▶║  R!  ║─┐
//!     │         └──────┘     ╚══════╝      └──────┘     ╚══════╝ │
//!     └─────────────────────────────ε────────────────────────────┘
//! ```
//!
//! Double-bordered states are accepting (and also transitive) states that emit an output.

use crate::state_transducer::{Output, State, Transition, Transitions};

/// The transition table that defines the quad-step finite-state-transducer.
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
        [t!(N2, N), t!(F1, AB), t!(R1, BA), t!(N0, N)], // row: `N0`
        [t!(N2, AB), t!(F1, N), t!(N0, N), t!(N0, BA)], // row: `F1`
        [t!(N2, BA), t!(N2, N), t!(F2, N), t!(N0, AB)], // row: `F2`
        // This row is unused in half-step mode, but needs to be provided
        // as it expects a transition matrix of certain dimensions:
        [t!(N0, E), t!(N0, E), t!(N0, E), t!(N0, E)], // row: `F3`
        [t!(N2, BA), t!(N0, N), t!(R1, N), t!(N0, AB)], // row: `R1`
        [t!(N2, AB), t!(R2, N), t!(N2, N), t!(N0, BA)], // row: `R2`
        // This row is unused in half-step mode, but needs to be provided
        // as it expects a transition matrix of certain dimensions:
        [t!(N0, E), t!(N0, E), t!(N0, E), t!(N0, E)], // row: `R3`
        [t!(N2, N), t!(R2, BA), t!(F2, AB), t!(N0, N)], // row: `N2`
    ]
};

#[cfg(test)]
mod tests {
    use crate::{
        state_transducer::{
            quad_step::TRANSITIONS,
            Input::{self, *},
            Output, State, StateTransducer,
        },
        Change::{self, *},
        Error, IncrementalDecoder, QuadStep,
    };

    type Decoder = IncrementalDecoder<QuadStep>;

    fn update(decoder: &mut Decoder, input: Input) -> Result<Option<Change>, Error> {
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
            (State::F2, Input::A1B0),
            // State::F3 is not used by the quad-step transducer.
            (State::R1, Input::A1B0),
            (State::R2, Input::A0B1),
            // State::R3 is not used by the quad-step transducer.
            (State::N2, Input::A0B0),
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
            assert_eq!(update(&mut decoder, A0B1), Ok(Some(Positive)));
            assert_eq!(update(&mut decoder, A0B0), Ok(Some(Positive)));
            assert_eq!(update(&mut decoder, A1B0), Ok(Some(Positive)));
            assert_eq!(update(&mut decoder, A1B1), Ok(Some(Positive)));
        }

        #[test]
        fn backwards() {
            let mut decoder = Decoder::default();

            // Full cycle without reduBdant inputs:
            assert_eq!(update(&mut decoder, A1B0), Ok(Some(Negative)));
            assert_eq!(update(&mut decoder, A0B0), Ok(Some(Negative)));
            assert_eq!(update(&mut decoder, A0B1), Ok(Some(Negative)));
            assert_eq!(update(&mut decoder, A1B1), Ok(Some(Negative)));
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
                assert_eq!(update(&mut decoder, A0B1), Ok(Some(Positive)));
                assert_eq!(update(&mut decoder, A0B0), Ok(Some(Positive))); // Redundant input
                assert_eq!(update(&mut decoder, A0B0), Ok(None)); // Redundant input
                assert_eq!(update(&mut decoder, A0B0), Ok(None)); // Redundant input
                assert_eq!(update(&mut decoder, A1B0), Ok(Some(Positive)));
                assert_eq!(update(&mut decoder, A1B1), Ok(Some(Positive)));
            }

            #[test]
            fn backwards() {
                let mut decoder = Decoder::default();

                // Full cycle with block of redundant inputs:
                assert_eq!(update(&mut decoder, A1B0), Ok(Some(Negative)));
                assert_eq!(update(&mut decoder, A0B0), Ok(Some(Negative))); // Redundant input
                assert_eq!(update(&mut decoder, A0B0), Ok(None)); // Redundant input
                assert_eq!(update(&mut decoder, A0B0), Ok(None)); // Redundant input
                assert_eq!(update(&mut decoder, A0B1), Ok(Some(Negative)));
                assert_eq!(update(&mut decoder, A1B1), Ok(Some(Negative)));
            }
        }

        mod interleaved {
            use super::*;

            #[test]
            fn forwards() {
                let mut decoder = Decoder::default();

                // Full cycle with alternating redundant inputs:
                assert_eq!(update(&mut decoder, A0B1), Ok(Some(Positive)));
                assert_eq!(update(&mut decoder, A0B1), Ok(None)); // Redundant input
                assert_eq!(update(&mut decoder, A0B0), Ok(Some(Positive)));
                assert_eq!(update(&mut decoder, A0B0), Ok(None)); // Redundant input
                assert_eq!(update(&mut decoder, A1B0), Ok(Some(Positive)));
                assert_eq!(update(&mut decoder, A1B0), Ok(None)); // Redundant input
                assert_eq!(update(&mut decoder, A1B1), Ok(Some(Positive)));
            }

            #[test]
            fn backwards() {
                let mut decoder = Decoder::default();

                // Full cycle with alternating redundant inputs:
                assert_eq!(update(&mut decoder, A1B0), Ok(Some(Negative)));
                assert_eq!(update(&mut decoder, A1B0), Ok(None)); // Redundant input
                assert_eq!(update(&mut decoder, A0B0), Ok(Some(Negative)));
                assert_eq!(update(&mut decoder, A0B0), Ok(None)); // Redundant input
                assert_eq!(update(&mut decoder, A0B1), Ok(Some(Negative)));
                assert_eq!(update(&mut decoder, A0B1), Ok(None)); // Redundant input
                assert_eq!(update(&mut decoder, A1B1), Ok(Some(Negative)));
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

                assert_eq!(update(&mut decoder, A0B1), Ok(Some(Positive)));
                assert_eq!(update(&mut decoder, A0B0), Ok(Some(Positive)));
                assert_eq!(update(&mut decoder, A0B1), Ok(Some(Negative)));
                assert_eq!(update(&mut decoder, A1B1), Ok(Some(Negative)));
            }

            #[test]
            fn backwards() {
                let mut decoder = Decoder::default();

                assert_eq!(update(&mut decoder, A1B0), Ok(Some(Negative)));
                assert_eq!(update(&mut decoder, A0B0), Ok(Some(Negative)));
                assert_eq!(update(&mut decoder, A1B0), Ok(Some(Positive)));
                assert_eq!(update(&mut decoder, A1B1), Ok(Some(Positive)));
            }
        }

        mod quarter {
            use super::*;

            #[test]
            fn forwards() {
                let mut decoder = Decoder::default();

                assert_eq!(update(&mut decoder, A0B1), Ok(Some(Positive)));
                assert_eq!(update(&mut decoder, A1B1), Ok(Some(Negative)));
            }

            #[test]
            fn backwards() {
                let mut decoder = Decoder::default();

                assert_eq!(update(&mut decoder, A1B0), Ok(Some(Negative)));
                assert_eq!(update(&mut decoder, A1B1), Ok(Some(Positive)));
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

                assert_eq!(update(&mut decoder, A0B1), Ok(Some(Positive)));
                assert_eq!(update(&mut decoder, A0B0), Ok(Some(Positive)));
                assert_eq!(update(&mut decoder, A1B0), Ok(Some(Positive)));
                assert_eq!(update(&mut decoder, A0B1), Err(Error::E10_01)); // Noise input
                assert_eq!(update(&mut decoder, A1B1), Ok(None));

                decoder.reset();

                assert_eq!(update(&mut decoder, A0B1), Ok(Some(Positive)));
                assert_eq!(update(&mut decoder, A1B0), Err(Error::E01_10)); // Noise input
                assert_eq!(update(&mut decoder, A0B0), Ok(None));
                assert_eq!(update(&mut decoder, A1B0), Ok(Some(Positive)));
                assert_eq!(update(&mut decoder, A1B1), Ok(Some(Positive)));
            }

            #[test]
            fn backwards() {
                let mut decoder = Decoder::default();

                assert_eq!(update(&mut decoder, A1B0), Ok(Some(Negative)));
                assert_eq!(update(&mut decoder, A0B0), Ok(Some(Negative)));
                assert_eq!(update(&mut decoder, A0B1), Ok(Some(Negative)));
                assert_eq!(update(&mut decoder, A1B0), Err(Error::E01_10)); // Noise input
                assert_eq!(update(&mut decoder, A1B1), Ok(None));

                decoder.reset();

                assert_eq!(update(&mut decoder, A1B0), Ok(Some(Negative)));
                assert_eq!(update(&mut decoder, A0B1), Err(Error::E10_01)); // Noise input
                assert_eq!(update(&mut decoder, A0B0), Ok(None));
                assert_eq!(update(&mut decoder, A0B1), Ok(Some(Negative)));
                assert_eq!(update(&mut decoder, A1B1), Ok(Some(Negative)));
            }
        }

        mod double {
            use super::*;

            #[test]
            fn forwards() {
                let mut decoder = Decoder::default();

                assert_eq!(update(&mut decoder, A0B1), Ok(Some(Positive)));
                assert_eq!(update(&mut decoder, A1B0), Err(Error::E01_10)); // Noise input
                assert_eq!(update(&mut decoder, A0B0), Ok(None));
                assert_eq!(update(&mut decoder, A1B0), Ok(Some(Positive)));
                assert_eq!(update(&mut decoder, A0B1), Err(Error::E10_01)); // Noise input
                assert_eq!(update(&mut decoder, A1B1), Ok(None));
            }

            #[test]
            fn backwards() {
                let mut decoder = Decoder::default();

                assert_eq!(update(&mut decoder, A1B0), Ok(Some(Negative)));
                assert_eq!(update(&mut decoder, A0B1), Err(Error::E10_01)); // Noise input
                assert_eq!(update(&mut decoder, A0B0), Ok(None));
                assert_eq!(update(&mut decoder, A0B1), Ok(Some(Negative)));
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
                assert_eq!(update(&mut decoder, A0B1), Ok(Some(Negative)));
                assert_eq!(update(&mut decoder, A1B0), Err(Error::E01_10)); // Noise input
                assert_eq!(update(&mut decoder, A0B0), Ok(None));
                assert_eq!(update(&mut decoder, A1B1), Err(Error::E00_11)); // Noise input
                assert_eq!(update(&mut decoder, A1B0), Ok(Some(Negative)));
                assert_eq!(update(&mut decoder, A0B1), Err(Error::E10_01)); // Noise input
                assert_eq!(update(&mut decoder, A1B1), Ok(None));
            }

            #[test]
            fn backwards() {
                let mut decoder = Decoder::default();

                assert_eq!(update(&mut decoder, A0B0), Err(Error::E11_00)); // Noise input
                assert_eq!(update(&mut decoder, A1B0), Ok(Some(Positive)));
                assert_eq!(update(&mut decoder, A0B1), Err(Error::E10_01)); // Noise input
                assert_eq!(update(&mut decoder, A0B0), Ok(None));
                assert_eq!(update(&mut decoder, A1B1), Err(Error::E00_11)); // Noise input
                assert_eq!(update(&mut decoder, A0B1), Ok(Some(Positive)));
                assert_eq!(update(&mut decoder, A1B0), Err(Error::E01_10)); // Noise input
                assert_eq!(update(&mut decoder, A1B1), Ok(None));
            }
        }
    }
}
