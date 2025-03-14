use unionlabs::{primitives::Bytes, tuple::AsTuple};

use crate::types::ChannelId;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, AsTuple)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Packet {
    pub source_channel_id: ChannelId,
    pub destination_channel_id: ChannelId,
    pub data: Bytes,
    // TODO: Group these into a single PacketTimeout type (one of these fields must be set, but both *can*)
    pub timeout_height: u64,
    pub timeout_timestamp: u64,
}

impl Packet {
    /// Calculate the hash of this packet. This is the same as the commitment key for a single packet.
    #[cfg(feature = "ethabi")]
    pub fn hash(&self) -> unionlabs::primitives::H256 {
        use alloy_sol_types::SolValue;
        use unionlabs::ethereum::keccak256;

        keccak256(self.abi_encode())
    }
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use std::borrow::Cow;

    use alloy_sol_types::{
        sol_data::{Bytes as SolBytes, Uint},
        SolStruct, SolType, SolValue,
    };

    use super::*;

    #[cfg(feature = "ibc-solidity-compat")]
    impl From<Packet> for ibc_solidity::Packet {
        fn from(value: Packet) -> Self {
            Self {
                source_channel_id: value.source_channel_id.raw(),
                destination_channel_id: value.destination_channel_id.raw(),
                data: value.data.into(),
                timeout_height: value.timeout_height,
                timeout_timestamp: value.timeout_timestamp,
            }
        }
    }

    #[cfg(feature = "ibc-solidity-compat")]
    impl TryFrom<ibc_solidity::Packet> for Packet {
        type Error = Error;

        fn try_from(value: ibc_solidity::Packet) -> Result<Self, Self::Error> {
            Ok(Self {
                source_channel_id: ChannelId::from_raw(value.source_channel_id)
                    .ok_or(Error::InvalidSourceChannelId)?,
                destination_channel_id: ChannelId::from_raw(value.destination_channel_id)
                    .ok_or(Error::InvalidDestinationChannelId)?,
                data: value.data.into(),
                timeout_height: value.timeout_height,
                timeout_timestamp: value.timeout_timestamp,
            })
        }
    }

    #[cfg(feature = "ibc-solidity-compat")]
    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("invalid source channel id")]
        InvalidSourceChannelId,
        #[error("invalid destination channel id")]
        InvalidDestinationChannelId,
    }

    type SolTuple = (Uint<32>, Uint<32>, SolBytes, Uint<64>, Uint<64>);

    impl SolValue for Packet {
        type SolType = Self;
    }

    impl SolType for Packet {
        type RustType = Self;

        type Token<'a> = <SolTuple as SolType>::Token<'a>;

        const SOL_NAME: &'static str = <SolTuple as SolType>::SOL_NAME;

        const ENCODED_SIZE: Option<usize> = <SolTuple as SolType>::ENCODED_SIZE;

        const PACKED_ENCODED_SIZE: Option<usize> = <SolTuple as SolType>::PACKED_ENCODED_SIZE;

        fn valid_token(
            (
                source_channel_id,
                destination_channel_id,
                _data,
                _timeout_height,
                _timeout_timestamp,
            ): &Self::Token<'_>,
        ) -> bool {
            (<Uint<32>>::valid_token(source_channel_id)
                && <Uint<32>>::detokenize(*source_channel_id) > 0)
                && (<Uint<32>>::valid_token(destination_channel_id)
                    && <Uint<32>>::detokenize(*destination_channel_id) > 0)
        }

        fn detokenize(
            (
                source_channel_id,
                destination_channel_id,
                data,
                timeout_height,
                timeout_timestamp,
            ): Self::Token<'_>,
        ) -> Self::RustType {
            Self {
                source_channel_id: ChannelId::from_raw(<Uint<32>>::detokenize(source_channel_id))
                    .expect("???"),
                destination_channel_id: ChannelId::from_raw(<Uint<32>>::detokenize(
                    destination_channel_id,
                ))
                .expect("???"),
                data: SolBytes::detokenize(data).into(),
                timeout_height: <Uint<64>>::detokenize(timeout_height),
                timeout_timestamp: <Uint<64>>::detokenize(timeout_timestamp),
            }
        }
    }

    impl SolType for &Packet {
        type RustType = Packet;

        type Token<'a> = <Packet as SolType>::Token<'a>;

        const SOL_NAME: &'static str = <Packet as SolType>::SOL_NAME;

        const ENCODED_SIZE: Option<usize> = <Packet as SolType>::ENCODED_SIZE;

        const PACKED_ENCODED_SIZE: Option<usize> = <Packet as SolType>::PACKED_ENCODED_SIZE;

        fn valid_token(token: &Self::Token<'_>) -> bool {
            <Packet as SolType>::valid_token(token)
        }

        fn detokenize(token: Self::Token<'_>) -> Self::RustType {
            <Packet as SolType>::detokenize(token)
        }
    }

    impl alloy_sol_types::private::SolTypeValue<Self> for Packet {
        fn stv_to_tokens(&self) -> <Self as SolType>::Token<'_> {
            (
                <Uint<32> as SolType>::tokenize(&self.source_channel_id.raw()),
                <Uint<32> as SolType>::tokenize(&self.destination_channel_id.raw()),
                <SolBytes as SolType>::tokenize(&self.data),
                <Uint<64> as SolType>::tokenize(&self.timeout_height),
                <Uint<64> as SolType>::tokenize(&self.timeout_timestamp),
            )
        }

        fn stv_abi_encode_packed_to(&self, out: &mut Vec<u8>) {
            let (
                source_channel_id,
                destination_channel_id,
                data,
                timeout_height,
                timeout_timestamp,
            ) = self.as_tuple();
            <SolTuple as SolType>::abi_encode_packed_to(
                &(
                    source_channel_id.raw(),
                    destination_channel_id.raw(),
                    data,
                    timeout_height,
                    timeout_timestamp,
                ),
                out,
            )
        }

        fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
            <Self as SolStruct>::eip712_hash_struct(self)
        }
    }

    impl alloy_sol_types::private::SolTypeValue<Self> for &Packet {
        fn stv_to_tokens(&self) -> <Self as SolType>::Token<'_> {
            <Packet as alloy_sol_types::private::SolTypeValue<Packet>>::stv_to_tokens(self)
        }

        fn stv_abi_encode_packed_to(&self, out: &mut Vec<u8>) {
            <Packet as alloy_sol_types::private::SolTypeValue<Packet>>::stv_abi_encode_packed_to(
                self, out,
            )
        }

        fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
            <Packet as alloy_sol_types::private::SolTypeValue<Packet>>::stv_eip712_data_word(self)
        }
    }

    impl alloy_sol_types::private::SolTypeValue<Packet> for &Packet {
        fn stv_to_tokens(&self) -> <Self as SolType>::Token<'_> {
            <Packet as alloy_sol_types::private::SolTypeValue<Packet>>::stv_to_tokens(self)
        }

        fn stv_abi_encode_packed_to(&self, out: &mut Vec<u8>) {
            <Packet as alloy_sol_types::private::SolTypeValue<Packet>>::stv_abi_encode_packed_to(
                self, out,
            )
        }

        fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
            <Packet as alloy_sol_types::private::SolTypeValue<Packet>>::stv_eip712_data_word(self)
        }
    }

    impl alloy_sol_types::private::SolTypeValue<&Packet> for Packet {
        fn stv_to_tokens(&self) -> <Self as SolType>::Token<'_> {
            <Packet as alloy_sol_types::private::SolTypeValue<Packet>>::stv_to_tokens(self)
        }

        fn stv_abi_encode_packed_to(&self, out: &mut Vec<u8>) {
            <Packet as alloy_sol_types::private::SolTypeValue<Packet>>::stv_abi_encode_packed_to(
                self, out,
            )
        }

        fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
            <Packet as alloy_sol_types::private::SolTypeValue<Packet>>::stv_eip712_data_word(self)
        }
    }

    impl SolStruct for Packet {
        const NAME: &'static str = "Packet";
        #[inline]
        fn eip712_root_type() -> Cow<'static, str> {
            "Packet(uint8 state,uint32 connection_id,uint32 counterparty_channel_id,bytes counterparty_port_id,string version)".into()
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
                <Uint<32> as SolType>::eip712_data_word(&self.source_channel_id.raw()),
                <Uint<32> as SolType>::eip712_data_word(&self.destination_channel_id.raw()),
                <SolBytes as SolType>::eip712_data_word(&self.data),
                <Uint<64> as SolType>::eip712_data_word(&self.timeout_height),
                <Uint<64> as SolType>::eip712_data_word(&self.timeout_timestamp),
            ]
            .concat()
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy_sol_types::SolValue;

    use super::*;

    #[test]
    fn abi_encode() {
        let ibc_solidity_packet = ibc_solidity::Packet {
            source_channel_id: 1,
            destination_channel_id: 1,
            data: b"data".into(),
            timeout_height: 1,
            timeout_timestamp: 0,
        };

        let packet = Packet {
            source_channel_id: ChannelId::from_raw(1).unwrap(),
            destination_channel_id: ChannelId::from_raw(1).unwrap(),
            data: b"data".into(),
            timeout_height: 1,
            timeout_timestamp: 0,
        };

        let ibc_solidity_bz = ibc_solidity_packet.abi_encode_params();
        let bz = packet.abi_encode_params();
        assert_eq!(ibc_solidity_bz, bz);

        let ibc_solidity_bz = ibc_solidity_packet.abi_encode();
        let bz = packet.abi_encode();
        assert_eq!(ibc_solidity_bz, bz);
    }

    #[test]
    fn abi_decode() {
        let ibc_solidity_packet = ibc_solidity::Packet {
            source_channel_id: 1,
            destination_channel_id: 1,
            data: b"data".into(),
            timeout_height: 1,
            timeout_timestamp: 0,
        };

        let packet = Packet {
            source_channel_id: ChannelId::from_raw(1).unwrap(),
            destination_channel_id: ChannelId::from_raw(1).unwrap(),
            data: b"data".into(),
            timeout_height: 1,
            timeout_timestamp: 0,
        };

        let ibc_solidity_bz = ibc_solidity_packet.abi_encode();
        let decoded_packet = Packet::abi_decode(&ibc_solidity_bz, true).unwrap();
        assert_eq!(packet, decoded_packet);

        let ibc_solidity_bz = ibc_solidity_packet.abi_encode_params();
        let decoded_packet = Packet::abi_decode_params(&ibc_solidity_bz, true).unwrap();
        assert_eq!(packet, decoded_packet);
    }

    // NOTE: No validation is currently required for the current packet struct definition
    // #[test]
    // fn abi_decode_invalid() {
    //     let ibc_solidity_connection = ibc_solidity::Packet {
    //         source_channel_id: ChannelId::from_raw(1).unwrap(),
    //         destination_channel_id: ChannelId::from_raw(1).unwrap(),
    //         data: b"data".into(),
    //         timeout_height: 1,
    //         timeout_timestamp: 0,
    //     };

    //     let expected_err = alloy_sol_types::Error::type_check_fail_token::<Packet>(&(
    //         U256::from(0_u32).into(),
    //         U256::from(1_u32).into(),
    //         U256::from(1_u32).into(),
    //         b"port".as_slice().into(),
    //         b"version".as_slice().into(),
    //     ));

    //     let ibc_solidity_bz = ibc_solidity_connection.abi_encode_params();
    //     let err = Packet::abi_decode_params(&ibc_solidity_bz, true).unwrap_err();
    //     assert_eq!(expected_err, err);

    //     let ibc_solidity_bz = ibc_solidity_connection.abi_encode();
    //     let err = Packet::abi_decode(&ibc_solidity_bz, true).unwrap_err();
    //     assert_eq!(expected_err, err);
    // }
}
