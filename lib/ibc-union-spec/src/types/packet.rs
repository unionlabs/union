use unionlabs::{primitives::Bytes, tuple::AsTuple};

use crate::types::ChannelId;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, AsTuple)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct Packet {
    pub source_channel_id: ChannelId,
    pub destination_channel_id: ChannelId,
    pub data: Bytes,
    // TODO: Group these into a single PacketTimeout type (one of these fields must be set, but both *can*)
    pub timeout_height: u64,
    pub timeout_timestamp: u64,
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
                source_channel_id: value.source_channel_id,
                destination_channel_id: value.destination_channel_id,
                data: value.data.into(),
                timeout_height: value.timeout_height,
                timeout_timestamp: value.timeout_timestamp,
            }
        }
    }

    #[cfg(feature = "ibc-solidity-compat")]
    impl From<ibc_solidity::Packet> for Packet {
        fn from(value: ibc_solidity::Packet) -> Self {
            Self {
                source_channel_id: value.source_channel_id,
                destination_channel_id: value.destination_channel_id,
                data: value.data.into(),
                timeout_height: value.timeout_height,
                timeout_timestamp: value.timeout_timestamp,
            }
        }
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
                _source_channel_id,
                _destination_channel_id,
                _data,
                _timeout_height,
                _timeout_timestamp,
            ): &Self::Token<'_>,
        ) -> bool {
            true
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
                source_channel_id: <Uint<32>>::detokenize(source_channel_id),
                destination_channel_id: <Uint<32>>::detokenize(destination_channel_id),
                data: SolBytes::detokenize(data).into(),
                timeout_height: <Uint<64>>::detokenize(timeout_height),
                timeout_timestamp: <Uint<64>>::detokenize(timeout_timestamp),
            }
        }
    }

    impl alloy_sol_types::private::SolTypeValue<Self> for Packet {
        fn stv_to_tokens(&self) -> <Self as SolType>::Token<'_> {
            (
                <Uint<32> as SolType>::tokenize(&self.source_channel_id),
                <Uint<32> as SolType>::tokenize(&self.destination_channel_id),
                <SolBytes as SolType>::tokenize(&self.data),
                <Uint<64> as SolType>::tokenize(&self.timeout_height),
                <Uint<64> as SolType>::tokenize(&self.timeout_timestamp),
            )
        }

        fn stv_abi_encode_packed_to(&self, out: &mut Vec<u8>) {
            let tuple = self.as_tuple();
            <SolTuple as SolType>::abi_encode_packed_to(&tuple, out)
        }

        fn stv_eip712_data_word(&self) -> alloy_sol_types::Word {
            <Self as SolStruct>::eip712_hash_struct(self)
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
                <Uint<32> as SolType>::eip712_data_word(&self.source_channel_id),
                <Uint<32> as SolType>::eip712_data_word(&self.destination_channel_id),
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
        let ibc_solidity_connection = ibc_solidity::Packet {
            source_channel_id: 1,
            destination_channel_id: 1,
            data: b"data".into(),
            timeout_height: 1,
            timeout_timestamp: 0,
        };

        let connection = Packet {
            source_channel_id: 1,
            destination_channel_id: 1,
            data: b"data".into(),
            timeout_height: 1,
            timeout_timestamp: 0,
        };

        let ibc_solidity_bz = ibc_solidity_connection.abi_encode_params();
        let bz = connection.abi_encode_params();
        assert_eq!(ibc_solidity_bz, bz);

        let ibc_solidity_bz = ibc_solidity_connection.abi_encode();
        let bz = connection.abi_encode();
        assert_eq!(ibc_solidity_bz, bz);
    }

    #[test]
    fn abi_decode() {
        let ibc_solidity_connection = ibc_solidity::Packet {
            source_channel_id: 1,
            destination_channel_id: 1,
            data: b"data".into(),
            timeout_height: 1,
            timeout_timestamp: 0,
        };

        let connection = Packet {
            source_channel_id: 1,
            destination_channel_id: 1,
            data: b"data".into(),
            timeout_height: 1,
            timeout_timestamp: 0,
        };

        let ibc_solidity_bz = ibc_solidity_connection.abi_encode();
        let decoded_connection = Packet::abi_decode(&ibc_solidity_bz, true).unwrap();
        assert_eq!(connection, decoded_connection);

        let ibc_solidity_bz = ibc_solidity_connection.abi_encode_params();
        let decoded_connection = Packet::abi_decode_params(&ibc_solidity_bz, true).unwrap();
        assert_eq!(connection, decoded_connection);
    }

    // NOTE: No validation is currently required for the current packet struct definition
    // #[test]
    // fn abi_decode_invalid() {
    //     let ibc_solidity_connection = ibc_solidity::Packet {
    //         source_channel_id: 1,
    //         destination_channel_id: 1,
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
