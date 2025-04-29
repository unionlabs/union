use std::borrow::Borrow;

use unionlabs::{errors::UnknownEnumVariant, primitives::Bytes, tuple::AsTuple};

use crate::types::{ChannelId, ConnectionId};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, AsTuple)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct Channel {
    pub state: ChannelState,
    pub connection_id: ConnectionId,
    // can be None when the channel is in the init state
    pub counterparty_channel_id: Option<ChannelId>,
    pub counterparty_port_id: Bytes,
    pub version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
#[repr(u8)]
pub enum ChannelState {
    Init = 1,
    TryOpen = 2,
    Open = 3,
    Closed = 4,
}

impl TryFrom<u8> for ChannelState {
    type Error = UnknownEnumVariant<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Init),
            2 => Ok(Self::TryOpen),
            3 => Ok(Self::Open),
            4 => Ok(Self::Closed),
            _ => Err(UnknownEnumVariant(value)),
        }
    }
}

impl Borrow<u8> for ChannelState {
    fn borrow(&self) -> &u8 {
        // SAFETY: Self is repr(u8)
        unsafe { core::mem::transmute_copy::<&Self, &u8>(&self) }
    }
}

impl Borrow<u8> for &ChannelState {
    fn borrow(&self) -> &u8 {
        <ChannelState as Borrow<u8>>::borrow(self)
    }
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use std::borrow::Cow;

    use alloy_sol_types::{
        sol_data::{Bytes as SolBytes, String as SolString, Uint},
        SolStruct, SolType, SolValue,
    };

    use super::*;

    type SolTuple = (Uint<8>, Uint<32>, Uint<32>, SolBytes, SolString);

    impl SolValue for Channel {
        type SolType = Self;
    }

    impl SolType for Channel {
        type RustType = Self;

        type Token<'a> = <SolTuple as SolType>::Token<'a>;

        const SOL_NAME: &'static str = <SolTuple as SolType>::SOL_NAME;

        const ENCODED_SIZE: Option<usize> = <SolTuple as SolType>::ENCODED_SIZE;

        const PACKED_ENCODED_SIZE: Option<usize> = <SolTuple as SolType>::PACKED_ENCODED_SIZE;

        fn valid_token(
            (state,
            connection_id,
            _counterparty_channel_id,
            _counterparty_port_id,
            _version,
            ): &Self::Token<
                '_,
            >,
        ) -> bool {
            (<Uint<8>>::valid_token(state)
                && ChannelState::try_from(<Uint<8>>::detokenize(*state)).is_ok())
                && (<Uint<32>>::valid_token(connection_id)
                    && <Uint<32>>::detokenize(*connection_id) > 0)
        }

        fn detokenize(
            (state,
            connection_id,
            counterparty_channel_id,
            counterparty_port_id,
            version,
            ): Self::Token<'_>,
        ) -> Self::RustType {
            Self {
                state: ChannelState::try_from(<Uint<8>>::detokenize(state)).expect("???"),
                connection_id: ConnectionId::from_raw(<Uint<32>>::detokenize(connection_id))
                    .expect("???"),
                counterparty_channel_id: ChannelId::from_raw(<Uint<32>>::detokenize(
                    counterparty_channel_id,
                )),
                counterparty_port_id: <SolBytes>::detokenize(counterparty_port_id).into(),
                version: SolString::detokenize(version),
            }
        }
    }

    impl alloy_sol_types::private::SolTypeValue<Self> for Channel {
        fn stv_to_tokens(&self) -> <Self as SolType>::Token<'_> {
            (
                <Uint<8> as SolType>::tokenize(&(self.state as u8)),
                <Uint<32> as SolType>::tokenize(&self.connection_id.raw()),
                <Uint<32> as SolType>::tokenize(
                    &self
                        .counterparty_channel_id
                        .map(|counterparty_channel_id| counterparty_channel_id.raw())
                        .unwrap_or_default(),
                ),
                <SolBytes as SolType>::tokenize(&self.counterparty_port_id),
                <SolString as SolType>::tokenize(&self.version),
            )
        }

        fn stv_abi_encode_packed_to(&self, out: &mut Vec<u8>) {
            let (state, connection_id, counterpartry_channel_id, counterparty_port_id, version) =
                self.as_tuple();
            <SolTuple as SolType>::abi_encode_packed_to(
                &(
                    state,
                    connection_id.raw(),
                    counterpartry_channel_id
                        .map(|counterpartry_channel_id| counterpartry_channel_id.raw())
                        .unwrap_or_default(),
                    counterparty_port_id,
                    version,
                ),
                out,
            )
        }

        fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
            <Self as SolStruct>::eip712_hash_struct(self)
        }
    }

    impl SolStruct for Channel {
        const NAME: &'static str = "Channel";
        #[inline]
        fn eip712_root_type() -> Cow<'static, str> {
            "Channel(uint8 state,uint32 connection_id,uint32 counterparty_channel_id,bytes counterparty_port_id,string version)".into()
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
                <Uint<32> as SolType>::eip712_data_word(&self.connection_id.raw()).0,
                <Uint<32> as SolType>::eip712_data_word(
                    &self
                        .counterparty_channel_id
                        .map(|counterparty_channel_id| counterparty_channel_id.raw())
                        .unwrap_or_default(),
                )
                .0,
                <SolBytes as SolType>::eip712_data_word(&self.counterparty_port_id).0,
                <SolString as SolType>::eip712_data_word(&self.version).0,
            ]
            .concat()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: Explicit type annotations are intentional, to ensure the intended impls are called
    #[test]
    fn channel_state_borrow_u8() {
        let connection_state: ChannelState = ChannelState::Open;
        let borrowed_u8: &u8 = <ChannelState as Borrow<u8>>::borrow(&connection_state);
        assert_eq!(&(connection_state as u8), borrowed_u8);

        let connection_state: &ChannelState = &ChannelState::TryOpen;
        let borrowed_u8: &u8 = <&ChannelState as Borrow<u8>>::borrow(&connection_state);
        assert_eq!(&(*connection_state as u8), borrowed_u8);

        let connection_state: &&ChannelState = &&ChannelState::Init;
        // ayutoref coercion
        let borrowed_u8: &u8 = connection_state.borrow();
        assert_eq!(&(**connection_state as u8), borrowed_u8);
    }
}
