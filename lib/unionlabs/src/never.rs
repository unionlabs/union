use core::fmt::Display;

use serde::{Deserialize, Serialize};

/// The empty/ "bottom" type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Never {}

impl Display for Never {
    fn fmt(&self, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {}
    }
}

impl std::error::Error for Never {}
