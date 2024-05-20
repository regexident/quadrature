use crate::{state_transducer::Input, Error};

/// A validator for checking conformance of inputs against quadrature protocol.
#[derive(Debug)]
pub(crate) struct InputValidator {
    input: Input,
}

impl InputValidator {
    const INITIAL_INPUT: Input = Input::A1B1;

    pub(crate) fn validate(&mut self, input: Input) -> Result<(), Error> {
        let last_input = core::mem::replace(&mut self.input, input);
        match (last_input, input) {
            (Input::A0B0, Input::A1B1) => Err(Error::E00_11),
            (Input::A0B1, Input::A1B0) => Err(Error::E01_10),
            (Input::A1B0, Input::A0B1) => Err(Error::E10_01),
            (Input::A1B1, Input::A0B0) => Err(Error::E11_00),
            _ => Ok(()),
        }
    }

    /// Resets the validator to its initial state.
    pub(crate) fn reset(&mut self) {
        self.input = Self::INITIAL_INPUT;
    }
}

impl Default for InputValidator {
    fn default() -> Self {
        // We expect transducers to have `State::N0` as initial state,
        // and the identity input of `State::N0` (i.e. an input that
        // does not cause a state change) is `Input::A1B1`:
        Self {
            input: Self::INITIAL_INPUT,
        }
    }
}
