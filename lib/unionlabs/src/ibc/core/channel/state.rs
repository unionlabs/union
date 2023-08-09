use serde::Serialize;

use crate::errors::UnknownEnumVariant;

#[derive(Debug, Clone, Serialize)]
pub enum State {
    UninitializedUnspecified,
    Init,
    Tryopen,
    Open,
    Closed,
}

impl TryFrom<i32> for State {
    type Error = UnknownEnumVariant<i32>;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(State::UninitializedUnspecified),
            1 => Ok(State::Init),
            2 => Ok(State::Tryopen),
            3 => Ok(State::Open),
            4 => Ok(State::Closed),
            state => Err(UnknownEnumVariant(state)),
        }
    }
}

impl TryFrom<u8> for State {
    type Error = UnknownEnumVariant<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(State::UninitializedUnspecified),
            1 => Ok(State::Init),
            2 => Ok(State::Tryopen),
            3 => Ok(State::Open),
            4 => Ok(State::Closed),
            state => Err(UnknownEnumVariant(state)),
        }
    }
}
