use std::fmt;

use crate::error::{Error, ErrorKind};
use crate::result::ExecutionFailure;

impl From<ExecutionFailure> for Error {
    fn from(error: ExecutionFailure) -> Self {
        ErrorKind::Execution.detailed(error)
    }
}

impl std::error::Error for ExecutionFailure {}

impl fmt::Display for ExecutionFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
