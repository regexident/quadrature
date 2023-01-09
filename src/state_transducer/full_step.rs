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

    // columns: `B00`, `B01`, `B10`, `B11`
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
    use crate::state_transducer::{Input::*, Output::*, StateTransducer};

    use super::*;

    mod clean {
        use super::*;

        #[test]
        fn forwards() {
            let mut transducer = StateTransducer::new(&TRANSITIONS);

            // Full cycle without redundant inputs:
            assert_eq!(transducer.step(B01), N);
            assert_eq!(transducer.step(B00), N);
            assert_eq!(transducer.step(B10), N);
            assert_eq!(transducer.step(B11), F);
        }

        #[test]
        fn backwards() {
            let mut transducer = StateTransducer::new(&TRANSITIONS);

            // Full cycle without redundant inputs:
            assert_eq!(transducer.step(B10), N);
            assert_eq!(transducer.step(B00), N);
            assert_eq!(transducer.step(B01), N);
            assert_eq!(transducer.step(B11), R);
        }
    }

    mod stutter {
        use super::*;

        mod block {
            use super::*;

            #[test]
            fn forwards() {
                let mut transducer = StateTransducer::new(&TRANSITIONS);

                // Full cycle with block of redundant inputs:
                assert_eq!(transducer.step(B01), N);
                assert_eq!(transducer.step(B00), N); // Redundant input
                assert_eq!(transducer.step(B00), N); // Redundant input
                assert_eq!(transducer.step(B00), N); // Redundant input
                assert_eq!(transducer.step(B10), N);
                assert_eq!(transducer.step(B11), F);
            }

            #[test]
            fn backwards() {
                let mut transducer = StateTransducer::new(&TRANSITIONS);

                // Full cycle with block of redundant inputs:
                assert_eq!(transducer.step(B10), N);
                assert_eq!(transducer.step(B00), N); // Redundant input
                assert_eq!(transducer.step(B00), N); // Redundant input
                assert_eq!(transducer.step(B00), N); // Redundant input
                assert_eq!(transducer.step(B01), N);
                assert_eq!(transducer.step(B11), R);
            }
        }

        mod interleaved {
            use super::*;

            #[test]
            fn forwards() {
                let mut transducer = StateTransducer::new(&TRANSITIONS);

                // Full cycle with alternating redundant inputs:
                assert_eq!(transducer.step(B01), N);
                assert_eq!(transducer.step(B01), N); // Redundant input
                assert_eq!(transducer.step(B00), N);
                assert_eq!(transducer.step(B00), N); // Redundant input
                assert_eq!(transducer.step(B10), N);
                assert_eq!(transducer.step(B10), N); // Redundant input
                assert_eq!(transducer.step(B11), F);
            }

            #[test]
            fn backwards() {
                let mut transducer = StateTransducer::new(&TRANSITIONS);

                // Full cycle with alternating redundant inputs:
                assert_eq!(transducer.step(B10), N);
                assert_eq!(transducer.step(B10), N); // Redundant input
                assert_eq!(transducer.step(B00), N);
                assert_eq!(transducer.step(B00), N); // Redundant input
                assert_eq!(transducer.step(B01), N);
                assert_eq!(transducer.step(B01), N); // Redundant input
                assert_eq!(transducer.step(B11), R);
            }
        }
    }

    mod direction_change {
        use super::*;

        mod half {
            use super::*;

            #[test]
            fn forwards() {
                let mut transducer = StateTransducer::new(&TRANSITIONS);

                assert_eq!(transducer.step(B01), N);
                assert_eq!(transducer.step(B00), N);
                assert_eq!(transducer.step(B01), N);
                assert_eq!(transducer.step(B11), N);
            }

            #[test]
            fn backwards() {
                let mut transducer = StateTransducer::new(&TRANSITIONS);

                assert_eq!(transducer.step(B10), N);
                assert_eq!(transducer.step(B00), N);
                assert_eq!(transducer.step(B10), N);
                assert_eq!(transducer.step(B11), N);
            }
        }

        mod quarter {
            use super::*;

            #[test]
            fn forwards() {
                let mut transducer = StateTransducer::new(&TRANSITIONS);

                assert_eq!(transducer.step(B01), N);
                assert_eq!(transducer.step(B11), N);
            }

            #[test]
            fn backwards() {
                let mut transducer = StateTransducer::new(&TRANSITIONS);

                assert_eq!(transducer.step(B10), N);
                assert_eq!(transducer.step(B11), N);
            }
        }
    }

    mod noise {
        use super::*;

        mod single {
            use super::*;

            #[test]
            fn forwards() {
                let mut transducer = StateTransducer::new(&TRANSITIONS);

                assert_eq!(transducer.step(B01), N);
                assert_eq!(transducer.step(B00), N);
                assert_eq!(transducer.step(B10), N);
                assert_eq!(transducer.step(B01), E); // Noise input
                assert_eq!(transducer.step(B11), N);

                transducer.reset();

                assert_eq!(transducer.step(B01), N);
                assert_eq!(transducer.step(B10), E); // Noise input
                assert_eq!(transducer.step(B00), N);
                assert_eq!(transducer.step(B10), N);
                assert_eq!(transducer.step(B11), N);
            }

            #[test]
            fn backwards() {
                let mut transducer = StateTransducer::new(&TRANSITIONS);

                assert_eq!(transducer.step(B10), N);
                assert_eq!(transducer.step(B00), N);
                assert_eq!(transducer.step(B01), N);
                assert_eq!(transducer.step(B10), E); // Noise input
                assert_eq!(transducer.step(B11), N);

                transducer.reset();

                assert_eq!(transducer.step(B10), N);
                assert_eq!(transducer.step(B01), E); // Noise input
                assert_eq!(transducer.step(B00), N);
                assert_eq!(transducer.step(B01), N);
                assert_eq!(transducer.step(B11), N);
            }
        }

        mod double {
            use super::*;

            #[test]
            fn forwards() {
                let mut transducer = StateTransducer::new(&TRANSITIONS);

                assert_eq!(transducer.step(B01), N);
                assert_eq!(transducer.step(B10), E); // Noise input
                assert_eq!(transducer.step(B00), N);
                assert_eq!(transducer.step(B10), N);
                assert_eq!(transducer.step(B01), E); // Noise input
                assert_eq!(transducer.step(B11), N);
            }

            #[test]
            fn backwards() {
                let mut transducer = StateTransducer::new(&TRANSITIONS);

                assert_eq!(transducer.step(B10), N);
                assert_eq!(transducer.step(B01), E); // Noise input
                assert_eq!(transducer.step(B00), N);
                assert_eq!(transducer.step(B01), N);
                assert_eq!(transducer.step(B10), E); // Noise input
                assert_eq!(transducer.step(B11), N);
            }
        }

        mod quadruple {
            use super::*;

            #[test]
            fn forwards() {
                let mut transducer = StateTransducer::new(&TRANSITIONS);

                assert_eq!(transducer.step(B00), E); // Noise input
                assert_eq!(transducer.step(B01), N);
                assert_eq!(transducer.step(B10), E); // Noise input
                assert_eq!(transducer.step(B00), N);
                assert_eq!(transducer.step(B11), E); // Noise input
                assert_eq!(transducer.step(B10), N);
                assert_eq!(transducer.step(B01), E); // Noise input
                assert_eq!(transducer.step(B11), N);
            }

            #[test]
            fn backwards() {
                let mut transducer = StateTransducer::new(&TRANSITIONS);

                assert_eq!(transducer.step(B00), E); // Noise input
                assert_eq!(transducer.step(B10), N);
                assert_eq!(transducer.step(B01), E); // Noise input
                assert_eq!(transducer.step(B00), N);
                assert_eq!(transducer.step(B11), E); // Noise input
                assert_eq!(transducer.step(B01), N);
                assert_eq!(transducer.step(B10), E); // Noise input
                assert_eq!(transducer.step(B11), N);
            }
        }
    }
}
