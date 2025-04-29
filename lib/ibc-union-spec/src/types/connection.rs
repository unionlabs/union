use core::borrow::Borrow;

use unionlabs::{errors::UnknownEnumVariant, tuple::AsTuple};

use crate::types::{ClientId, ConnectionId};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, AsTuple)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct Connection {
    pub state: ConnectionState,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    // can be None if the connection is in the init state
    pub counterparty_connection_id: Option<ConnectionId>,
}

// pub enum Connection {
//     Init {
//         client_id: ClientId,
//         counterparty_client_id: ClientId,
//     },
//     TryOpen {
//         client_id: ClientId,
//         counterparty_client_id: ClientId,
//         counterparty_connection_id: ConnectionId,
//     },
//     Open {
//         client_id: ClientId,
//         counterparty_client_id: ClientId,
//         counterparty_connection_id: ConnectionId,
//     },
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
#[repr(u8)]
pub enum ConnectionState {
    Init = 1,
    TryOpen = 2,
    Open = 3,
}

impl TryFrom<u8> for ConnectionState {
    type Error = UnknownEnumVariant<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Init),
            2 => Ok(Self::TryOpen),
            3 => Ok(Self::Open),
            _ => Err(UnknownEnumVariant(value)),
        }
    }
}

impl Borrow<u8> for ConnectionState {
    fn borrow(&self) -> &u8 {
        // SAFETY: Self is repr(u8)
        unsafe { core::mem::transmute_copy::<&Self, &u8>(&self) }
    }
}

impl Borrow<u8> for &ConnectionState {
    fn borrow(&self) -> &u8 {
        <ConnectionState as Borrow<u8>>::borrow(self)
    }
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use std::borrow::Cow;

    use alloy_sol_types::{sol_data::Uint, SolStruct, SolType, SolValue};

    use super::*;

    type SolTuple = (Uint<8>, Uint<32>, Uint<32>, Uint<32>);

    impl SolValue for Connection {
        type SolType = Self;
    }

    impl SolType for Connection {
        type RustType = Self;

        type Token<'a> = <SolTuple as SolType>::Token<'a>;

        const SOL_NAME: &'static str = <SolTuple as SolType>::SOL_NAME;

        const ENCODED_SIZE: Option<usize> = <SolTuple as SolType>::ENCODED_SIZE;

        const PACKED_ENCODED_SIZE: Option<usize> = <SolTuple as SolType>::PACKED_ENCODED_SIZE;

        fn valid_token(
            (state, client_id, counterparty_client_id, _counterparty_connection_id): &Self::Token<
                '_,
            >,
        ) -> bool {
            (<Uint<8>>::valid_token(state)
                && ConnectionState::try_from(<Uint<8>>::detokenize(*state)).is_ok())
                && (<Uint<32>>::valid_token(client_id) && <Uint<32>>::detokenize(*client_id) > 0)
                && (<Uint<32>>::valid_token(counterparty_client_id)
                    && <Uint<32>>::detokenize(*counterparty_client_id) > 0)
        }

        fn detokenize(
            (state, client_id, counterparty_client_id, counterparty_connection_id): Self::Token<'_>,
        ) -> Self::RustType {
            Self {
                state: ConnectionState::try_from(<Uint<8>>::detokenize(state)).expect("???"),
                client_id: ClientId::from_raw(<Uint<32>>::detokenize(client_id)).expect("???"),
                counterparty_client_id: ClientId::from_raw(<Uint<32>>::detokenize(
                    counterparty_client_id,
                ))
                .expect("???"),
                counterparty_connection_id: ConnectionId::from_raw(<Uint<32>>::detokenize(
                    counterparty_connection_id,
                )),
            }
        }
    }

    impl alloy_sol_types::private::SolTypeValue<Self> for Connection {
        fn stv_to_tokens(&self) -> <Self as SolType>::Token<'_> {
            (
                <Uint<8> as SolType>::tokenize(&(self.state as u8)),
                <Uint<32> as SolType>::tokenize(&self.client_id.raw()),
                <Uint<32> as SolType>::tokenize(&self.counterparty_client_id.raw()),
                <Uint<32> as SolType>::tokenize(
                    &self
                        .counterparty_connection_id
                        .map(|counterparty_connection_id| counterparty_connection_id.raw())
                        .unwrap_or_default(),
                ),
            )
        }

        fn stv_abi_encode_packed_to(&self, out: &mut Vec<u8>) {
            let (state, client_id, counterparty_client_id, counterparty_connection_id) =
                self.as_tuple();
            <SolTuple as SolType>::abi_encode_packed_to(
                &(
                    state,
                    client_id.raw(),
                    counterparty_client_id.raw(),
                    counterparty_connection_id
                        .map(|counterparty_connection_id| counterparty_connection_id.raw())
                        .unwrap_or_default(),
                ),
                out,
            )
        }

        fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
            <Self as SolStruct>::eip712_hash_struct(self)
        }
    }

    impl SolStruct for Connection {
        const NAME: &'static str = "Connection";
        #[inline]
        fn eip712_root_type() -> Cow<'static, str> {
            "Connection(uint8 state,uint32 client_id,uint32 counterparty_client_id,uint32 counterparty_connection_id)".into()
        }
        #[inline]
        fn eip712_components() -> Vec<Cow<'static, str>> {
            Vec::new()
        }
        #[inline]
        fn eip712_encode_type() -> Cow<'static, str> {
            <Self as SolStruct>::eip712_root_type()
        }
        #[inline]
        fn eip712_encode_data(&self) -> Vec<u8> {
            [
                <Uint<8> as SolType>::eip712_data_word(&self.state).0,
                <Uint<32> as SolType>::eip712_data_word(&self.client_id.raw()).0,
                <Uint<32> as SolType>::eip712_data_word(&self.counterparty_client_id.raw()).0,
                <Uint<32> as SolType>::eip712_data_word(
                    &self
                        .counterparty_connection_id
                        .map(|counterparty_connection_id| counterparty_connection_id.raw())
                        .unwrap_or_default(),
                )
                .0,
            ]
            .concat()
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy_sol_types::SolValue;

    use super::*;

    // NOTE: Explicit type annotations are intentional, to ensure the intended impls are called
    #[test]
    fn connection_state_borrow_u8() {
        let connection_state: ConnectionState = ConnectionState::Open;
        let borrowed_u8: &u8 = <ConnectionState as Borrow<u8>>::borrow(&connection_state);
        assert_eq!(&(connection_state as u8), borrowed_u8);

        let connection_state: &ConnectionState = &ConnectionState::TryOpen;
        let borrowed_u8: &u8 = <&ConnectionState as Borrow<u8>>::borrow(&connection_state);
        assert_eq!(&(*connection_state as u8), borrowed_u8);

        let connection_state: &&ConnectionState = &&ConnectionState::Init;
        // ayutoref coercion
        let borrowed_u8: &u8 = connection_state.borrow();
        assert_eq!(&(**connection_state as u8), borrowed_u8);
    }
}
