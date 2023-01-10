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

    // columns: `B00`, `B01`, `B10`, `B11`
    [
        [t!(N2, N), t!(F1, F), t!(R1, R), t!(N0, N)], // row: `N0`
        [t!(N2, F), t!(F1, N), t!(N0, N), t!(N0, R)], // row: `F1`
        [t!(N2, R), t!(N2, N), t!(F2, N), t!(N0, F)], // row: `F2`
        // This row is unused in half-step mode, but needs to be provided
        // as it expects a transition matrix of certain dimensions:
        [t!(N0, E), t!(N0, E), t!(N0, E), t!(N0, E)], // row: `F3`
        [t!(N2, R), t!(N0, N), t!(R1, N), t!(N0, F)], // row: `R1`
        [t!(N2, F), t!(R2, N), t!(N2, N), t!(N0, R)], // row: `R2`
        // This row is unused in half-step mode, but needs to be provided
        // as it expects a transition matrix of certain dimensions:
        [t!(N0, E), t!(N0, E), t!(N0, E), t!(N0, E)], // row: `R3`
        [t!(N2, N), t!(R2, R), t!(F2, F), t!(N0, N)], // row: `N2`
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
            assert_eq!(transducer.step(B01), F);
            assert_eq!(transducer.step(B00), F);
            assert_eq!(transducer.step(B10), F);
            assert_eq!(transducer.step(B11), F);
        }

        #[test]
        fn backwards() {
            let mut transducer = StateTransducer::new(&TRANSITIONS);

            // Full cycle without reduBdant inputs:
            assert_eq!(transducer.step(B10), R);
            assert_eq!(transducer.step(B00), R);
            assert_eq!(transducer.step(B01), R);
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
                assert_eq!(transducer.step(B01), F);
                assert_eq!(transducer.step(B00), F); // Redundant input
                assert_eq!(transducer.step(B00), N); // Redundant input
                assert_eq!(transducer.step(B00), N); // Redundant input
                assert_eq!(transducer.step(B10), F);
                assert_eq!(transducer.step(B11), F);
            }

            #[test]
            fn backwards() {
                let mut transducer = StateTransducer::new(&TRANSITIONS);

                // Full cycle with block of redundant inputs:
                assert_eq!(transducer.step(B10), R);
                assert_eq!(transducer.step(B00), R); // Redundant input
                assert_eq!(transducer.step(B00), N); // Redundant input
                assert_eq!(transducer.step(B00), N); // Redundant input
                assert_eq!(transducer.step(B01), R);
                assert_eq!(transducer.step(B11), R);
            }
        }

        mod interleaved {
            use super::*;

            #[test]
            fn forwards() {
                let mut transducer = StateTransducer::new(&TRANSITIONS);

                // Full cycle with alternating redundant inputs:
                assert_eq!(transducer.step(B01), F);
                assert_eq!(transducer.step(B01), N); // Redundant input
                assert_eq!(transducer.step(B00), F);
                assert_eq!(transducer.step(B00), N); // Redundant input
                assert_eq!(transducer.step(B10), F);
                assert_eq!(transducer.step(B10), N); // Redundant input
                assert_eq!(transducer.step(B11), F);
            }

            #[test]
            fn backwards() {
                let mut transducer = StateTransducer::new(&TRANSITIONS);

                // Full cycle with alternating redundant inputs:
                assert_eq!(transducer.step(B10), R);
                assert_eq!(transducer.step(B10), N); // Redundant input
                assert_eq!(transducer.step(B00), R);
                assert_eq!(transducer.step(B00), N); // Redundant input
                assert_eq!(transducer.step(B01), R);
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

                assert_eq!(transducer.step(B01), F);
                assert_eq!(transducer.step(B00), F);
                assert_eq!(transducer.step(B01), R);
                assert_eq!(transducer.step(B11), R);
            }

            #[test]
            fn backwards() {
                let mut transducer = StateTransducer::new(&TRANSITIONS);

                assert_eq!(transducer.step(B10), R);
                assert_eq!(transducer.step(B00), R);
                assert_eq!(transducer.step(B10), F);
                assert_eq!(transducer.step(B11), F);
            }
        }

        mod quarter {
            use super::*;

            #[test]
            fn forwards() {
                let mut transducer = StateTransducer::new(&TRANSITIONS);

                assert_eq!(transducer.step(B01), F);
                assert_eq!(transducer.step(B11), R);
            }

            #[test]
            fn backwards() {
                let mut transducer = StateTransducer::new(&TRANSITIONS);

                assert_eq!(transducer.step(B10), R);
                assert_eq!(transducer.step(B11), F);
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

                assert_eq!(transducer.step(B01), F);
                assert_eq!(transducer.step(B00), F);
                assert_eq!(transducer.step(B10), F);
                assert_eq!(transducer.step(B01), E); // Noise input
                assert_eq!(transducer.step(B11), N);

                transducer.reset();

                assert_eq!(transducer.step(B01), F);
                assert_eq!(transducer.step(B10), E); // Noise input
                assert_eq!(transducer.step(B00), N);
                assert_eq!(transducer.step(B10), F);
                assert_eq!(transducer.step(B11), F);
            }

            #[test]
            fn backwards() {
                let mut transducer = StateTransducer::new(&TRANSITIONS);

                assert_eq!(transducer.step(B10), R);
                assert_eq!(transducer.step(B00), R);
                assert_eq!(transducer.step(B01), R);
                assert_eq!(transducer.step(B10), E); // Noise input
                assert_eq!(transducer.step(B11), N);

                transducer.reset();

                assert_eq!(transducer.step(B10), R);
                assert_eq!(transducer.step(B01), E); // Noise input
                assert_eq!(transducer.step(B00), N);
                assert_eq!(transducer.step(B01), R);
                assert_eq!(transducer.step(B11), R);
            }
        }

        mod double {
            use super::*;

            #[test]
            fn forwards() {
                let mut transducer = StateTransducer::new(&TRANSITIONS);

                assert_eq!(transducer.step(B01), F);
                assert_eq!(transducer.step(B10), E); // Noise input
                assert_eq!(transducer.step(B00), N);
                assert_eq!(transducer.step(B10), F);
                assert_eq!(transducer.step(B01), E); // Noise input
                assert_eq!(transducer.step(B11), N);
            }

            #[test]
            fn backwards() {
                let mut transducer = StateTransducer::new(&TRANSITIONS);

                assert_eq!(transducer.step(B10), R);
                assert_eq!(transducer.step(B01), E); // Noise input
                assert_eq!(transducer.step(B00), N);
                assert_eq!(transducer.step(B01), R);
                assert_eq!(transducer.step(B10), E); // Noise input
                assert_eq!(transducer.step(B11), N);
            }
        }

        mod quadruple {
            use super::*;

            #[test]
            fn forwards() {
                let mut transducer = StateTransducer::new(&TRANSITIONS);

                assert_eq!(transducer.step(B01), F);
                assert_eq!(transducer.step(B10), E); // Noise input
                assert_eq!(transducer.step(B00), N);
                assert_eq!(transducer.step(B11), E); // Noise input
                assert_eq!(transducer.step(B10), R);
                assert_eq!(transducer.step(B01), E); // Noise input
                assert_eq!(transducer.step(B11), N);
            }

            #[test]
            fn backwards() {
                let mut transducer = StateTransducer::new(&TRANSITIONS);

                assert_eq!(transducer.step(B10), R);
                assert_eq!(transducer.step(B01), E); // Noise input
                assert_eq!(transducer.step(B00), N);
                assert_eq!(transducer.step(B11), E); // Noise input
                assert_eq!(transducer.step(B01), F);
                assert_eq!(transducer.step(B10), E); // Noise input
                assert_eq!(transducer.step(B11), N);
            }
        }
    }
}
