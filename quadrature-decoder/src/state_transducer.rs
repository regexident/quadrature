//! A finite-state transducer (FST), i.e. a type of finite-state machine (FSM)
//! that maps between two sets of symbols: inputs and outputs.

pub(crate) mod full_step;
pub(crate) mod half_step;
pub(crate) mod quad_step;

/// A type defining the FST's inputs.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Input {
    A0B0,
    A0B1,
    A1B0,
    A1B1,
}

impl Input {
    pub(crate) const fn new(a: bool, b: bool) -> Self {
        match (a, b) {
            (false, false) => Self::A0B0,
            (false, true) => Self::A0B1,
            (true, false) => Self::A1B0,
            (true, true) => Self::A1B1,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) const fn a(&self) -> bool {
        match self {
            Input::A0B0 | Input::A0B1 => false,
            Input::A1B0 | Input::A1B1 => true,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) const fn b(&self) -> bool {
        match self {
            Input::A0B0 | Input::A1B0 => false,
            Input::A0B1 | Input::A1B1 => true,
        }
    }

    pub(crate) const fn bits(&self) -> u8 {
        *self as u8
    }
}

/// A type defining the FST's outputs.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Output {
    /// Neutral
    N = 0b_00,
    /// AB
    AB = 0b_01,
    /// BA
    BA = 0b_10,
    /// Error
    E = 0b_11,
}

impl Output {
    const BITS: usize = 2;
    const MASK: u8 = (1 << Self::BITS) - 1;

    pub(crate) const fn from_bits(bits: u8) -> Option<Self> {
        match bits {
            x if x == (Output::N as u8) => Some(Output::N),
            x if x == (Output::AB as u8) => Some(Output::AB),
            x if x == (Output::BA as u8) => Some(Output::BA),
            x if x == (Output::E as u8) => Some(Output::E),
            _ => None,
        }
    }

    pub(crate) unsafe fn from_bits_unchecked(bits: u8) -> Self {
        Self::from_bits(bits).unwrap()
    }

    pub(crate) const fn bits(&self) -> u8 {
        *self as u8
    }
}

/// A type defining the FST's states.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) enum State {
    N0, // Neutral: 0/4 cycle
    F1, // A --> B: 1/4 cycle
    F2, // A --> B: 2/4 cycle
    F3, // A --> B: 3/4 cycle
    R1, // B --> A: 1/4 cycle
    R2, // B --> A: 2/4 cycle
    R3, // B --> A: 3/4 cycle
    N2, // Neutral: 2/4 cycle
}

impl State {
    const BITS: usize = 3;
    const MASK: u8 = (1 << Self::BITS) - 1;

    pub(crate) const fn from_bits(bits: u8) -> Option<Self> {
        match bits {
            x if x == (State::N0 as u8) => Some(State::N0),
            x if x == (State::F1 as u8) => Some(State::F1),
            x if x == (State::F2 as u8) => Some(State::F2),
            x if x == (State::F3 as u8) => Some(State::F3),
            x if x == (State::R1 as u8) => Some(State::R1),
            x if x == (State::R2 as u8) => Some(State::R2),
            x if x == (State::R3 as u8) => Some(State::R3),
            x if x == (State::N2 as u8) => Some(State::N2),
            _ => None,
        }
    }

    pub(crate) unsafe fn from_bits_unchecked(bits: u8) -> Self {
        Self::from_bits(bits).unwrap()
    }

    pub(crate) const fn bits(&self) -> u8 {
        *self as u8
    }
}

/// A type defining the FST's transitions.
///
/// ```plain
///       ┌───────────┬───────┬───────────┐
/// Bits: │ 0   1   2 │ 3   4 │ 5   6   7 │
///       └───────────┴───────┴───────────┘
///        ╰┬────────╯╰┬─────╯ ╰┬────────╯
///         │          │        └── State bits
///         │          └── Output bits
///         └── Unused bits
/// ```
#[repr(Rust, packed)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Transition {
    bits: u8,
}

impl Transition {
    const OUTPUT_OFFSET: usize = State::BITS;

    pub(crate) const fn new(state: State, output: Output) -> Self {
        let state_bits = state.bits() & State::MASK;
        let output_bits = (output.bits() & Output::MASK) << Self::OUTPUT_OFFSET;
        Transition {
            bits: output_bits | state_bits,
        }
    }

    pub(crate) fn state(&self) -> State {
        let bits = self.bits & State::MASK;
        unsafe { State::from_bits_unchecked(bits) }
    }

    pub(crate) fn output(&self) -> Output {
        let bits = (self.bits >> Self::OUTPUT_OFFSET) & Output::MASK;
        unsafe { Output::from_bits_unchecked(bits) }
    }
}

pub(crate) type Transitions<const STATES: usize, const INPUTS: usize> =
    [[Transition; INPUTS]; STATES];

/// A finite-state transducer (FST), i.e. a type of finite-state machine (FSM)
/// that maps between two sets of symbols: inputs and outputs.
///
/// The inputs in this particular use-case are the concatenated 2-bit binary states
/// corresponding to the readings from the A and B pulse trains (aka channels) of a quadrature encoder.
#[derive(Debug)]
pub(crate) struct StateTransducer<'a, const STATES: usize, const INPUTS: usize> {
    state: State,
    transitions: &'a Transitions<STATES, INPUTS>,
}

impl<'a, const STATES: usize, const INPUTS: usize> StateTransducer<'a, STATES, INPUTS> {
    const INITIAL_STATE: State = State::N0;

    pub(crate) const fn new(transitions: &'a Transitions<STATES, INPUTS>) -> Self {
        Self {
            transitions,
            state: Self::INITIAL_STATE,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn state(&self) -> State {
        self.state
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn set_state(&mut self, state: State) {
        self.state = state;
    }

    pub(crate) fn reset(&mut self) {
        self.state = State::N0;
    }

    pub(crate) fn step(&mut self, input: Input) -> Output {
        let state_index = self.state.bits() as usize;
        let input_index = input.bits() as usize;
        let transition = self.transitions[state_index][input_index];

        self.state = transition.state();
        let output = transition.output();

        debug_assert_ne!(
            output,
            Output::E,
            "Transitions should not produce error outputs."
        );

        output
    }
}
