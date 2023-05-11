use alloc::format;
use alloc::string::String;
use core::fmt::Display;
use core::format_args;

use flex_error::{define_error, DisplayOnly};
use prost::{DecodeError, EncodeError};

use super::erased::TryFrom;

define_error! {
    Error {
        TryFromProtobuf
            { reason: String }
            | e | {
                format_args!("error converting message type into domain type: {}",
                    e.reason)
            },

        EncodeMessage
            [ DisplayOnly<EncodeError> ]
            | _ | { "error encoding message into buffer" },

        DecodeMessage
            [ DisplayOnly<DecodeError> ]
            | _ | { "error decoding buffer into message" },
    }
}

impl Error {
    pub fn try_from<Raw, T, E>(e: E) -> Error
    where
        E: Display,
        T: TryFrom<Raw, Error = E>,
    {
        Error::try_from_protobuf(format!("{e}"))
    }
}
