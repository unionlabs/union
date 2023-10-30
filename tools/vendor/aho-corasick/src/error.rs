use std::error;
use std::fmt;
use std::result;

pub type Result<T> = result::Result<T, Error>;

/// An error that occurred during the construction of an Aho-Corasick
/// automaton.
#[derive(Clone, Debug)]
pub struct Error {
    kind: ErrorKind,
}

/// The kind of error that occurred.
#[derive(Clone, Debug)]
pub enum ErrorKind {
    /// An error that occurs when constructing an automaton would require the
    /// use of a state ID that overflows the chosen state ID representation.
    /// For example, if one is using `u8` for state IDs and builds a DFA with
    /// 257 states, then the last state's ID will be `256` which cannot be
    /// represented with `u8`.
    StateIDOverflow {
        /// The maximum possible state ID.
        max: usize,
    },
    /// An error that occurs when premultiplication of state IDs is requested
    /// when constructing an Aho-Corasick DFA, but doing so would overflow the
    /// chosen state ID representation.
    ///
    /// When `max == requested_max`, then the state ID would overflow `usize`.
    PremultiplyOverflow {
        /// The maximum possible state id.
        max: usize,
        /// The maximum ID required by premultiplication.
        requested_max: usize,
    },
}

impl Error {
    /// Return the kind of this error.
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub(crate) fn state_id_overflow(max: usize) -> Error {
        Error { kind: ErrorKind::StateIDOverflow { max } }
    }

    pub(crate) fn premultiply_overflow(
        max: usize,
        requested_max: usize,
    ) -> Error {
        Error { kind: ErrorKind::PremultiplyOverflow { max, requested_max } }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self.kind {
            ErrorKind::StateIDOverflow { .. } => {
                "state id representation too small"
            }
            ErrorKind::PremultiplyOverflow { .. } => {
                "state id representation too small for premultiplication"
            }
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ErrorKind::StateIDOverflow { max } => write!(
                f,
                "building the automaton failed because it required \
                 building more states that can be identified, where the \
                 maximum ID for the chosen representation is {}",
                max,
            ),
            ErrorKind::PremultiplyOverflow { max, requested_max } => {
                if max == requested_max {
                    write!(
                        f,
                        "premultiplication of states requires the ability to \
                         represent a state ID greater than what can fit on \
                         this platform's usize, which is {}",
                        ::std::usize::MAX,
                    )
                } else {
                    write!(
                        f,
                        "premultiplication of states requires the ability to \
                         represent at least a state ID of {}, but the chosen \
                         representation only permits a maximum state ID of {}",
                        requested_max, max,
                    )
                }
            }
        }
    }
}
