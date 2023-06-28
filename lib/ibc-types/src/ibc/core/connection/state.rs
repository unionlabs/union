use crate::errors::UnknownEnumVariant;

#[derive(Debug, Clone)]
pub enum State {
    /// Default State
    UninitializedUnspecified = 0,
    /// A connection end has just started the opening handshake.
    Init = 1,
    /// A connection end has acknowledged the handshake step on the counterparty
    /// chain.
    Tryopen = 2,
    /// A connection end has completed the handshake.
    Open = 3,
}

impl TryFrom<i32> for State {
    type Error = UnknownEnumVariant<i32>;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(State::UninitializedUnspecified),
            1 => Ok(State::Init),
            2 => Ok(State::Tryopen),
            3 => Ok(State::Open),
            _ => Err(UnknownEnumVariant(value)),
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
            _ => Err(UnknownEnumVariant(value)),
        }
    }
}

impl From<State> for protos::ibc::core::connection::v1::State {
    fn from(value: State) -> Self {
        match value {
            State::UninitializedUnspecified => {
                protos::ibc::core::connection::v1::State::UninitializedUnspecified
            }
            State::Init => protos::ibc::core::connection::v1::State::Init,
            State::Tryopen => protos::ibc::core::connection::v1::State::Tryopen,
            State::Open => protos::ibc::core::connection::v1::State::Open,
        }
    }
}

impl From<protos::ibc::core::connection::v1::State> for State {
    fn from(value: protos::ibc::core::connection::v1::State) -> Self {
        match value {
            protos::ibc::core::connection::v1::State::UninitializedUnspecified => {
                State::UninitializedUnspecified
            }
            protos::ibc::core::connection::v1::State::Init => State::Init,
            protos::ibc::core::connection::v1::State::Tryopen => State::Tryopen,
            protos::ibc::core::connection::v1::State::Open => State::Open,
        }
    }
}
