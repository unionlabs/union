use std::{fmt::Display, str::FromStr};

use unionlabs::{primitives::Bytes, tuple::AsTuple};
use voyager_primitives::Timestamp;

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
    pub timeout_height: MustBeZero,
    pub timeout_timestamp: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MustBeZero;

#[cfg(feature = "schemars")]
impl schemars::JsonSchema for MustBeZero {
    fn schema_name() -> String {
        todo!()
    }

    fn json_schema(_: &mut schemars::r#gen::SchemaGenerator) -> schemars::schema::Schema {
        use schemars::schema::{
            InstanceType, Metadata, NumberValidation, SchemaObject, SingleOrVec,
        };

        SchemaObject {
            metadata: Some(Box::new(Metadata {
                description: Some("a u64 that must be zero".to_owned()),
                ..Default::default()
            })),
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Number))),
            number: Some(Box::new(NumberValidation {
                minimum: Some(0.0),
                maximum: Some(0.0),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("value must be zero but found {0}")]
pub struct MustBeZeroFromStrError(String);

impl FromStr for MustBeZero {
    type Err = MustBeZeroFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s != "0" {
            Err(MustBeZeroFromStrError(s.into()))
        } else {
            Ok(MustBeZero)
        }
    }
}

impl Display for MustBeZero {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0")
    }
}

#[cfg(feature = "bincode")]
impl bincode::Encode for MustBeZero {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        bincode::Encode::encode(&0_u64, encoder)
    }
}

#[cfg(feature = "bincode")]
impl<Context> bincode::Decode<Context> for MustBeZero {
    fn decode<D: bincode::de::Decoder<Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let value = <u64 as bincode::Decode<Context>>::decode(decoder)?;
        if value != 0 {
            Err(bincode::error::DecodeError::Other("must be zero"))
        } else {
            Ok(Self)
        }
    }
}

#[cfg(feature = "bincode")]
bincode::impl_borrow_decode!(MustBeZero);

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for MustBeZero {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u64::deserialize(deserializer)?;
        if value != 0 {
            Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Unsigned(value),
                &"zero",
            ))
        } else {
            Ok(Self)
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for MustBeZero {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u64(0)
    }
}

impl Packet {
    /// Calculate the hash of this packet. This is the same as the commitment key for a single packet.
    #[cfg(feature = "ethabi")]
    pub fn hash(&self) -> unionlabs::primitives::H256 {
        use alloy_sol_types::SolValue;
        use unionlabs::ethereum::keccak256;

        keccak256(core::slice::from_ref(self).abi_encode())
    }
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use std::borrow::Cow;

    use alloy_sol_types::{
        SolStruct, SolType, SolValue,
        sol_data::{Bytes as SolBytes, Uint},
    };

    use super::*;

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
                timeout_height,
                _timeout_timestamp,
            ): &Self::Token<'_>,
        ) -> bool {
            (<Uint<32>>::valid_token(source_channel_id)
                && <Uint<32>>::detokenize(*source_channel_id) > 0)
                && (<Uint<32>>::valid_token(destination_channel_id)
                    && <Uint<32>>::detokenize(*destination_channel_id) > 0)
                && (<Uint<64>>::valid_token(timeout_height)
                    && <Uint<64>>::detokenize(*timeout_height) == 0)
        }

        fn detokenize(
            (
                source_channel_id,
                destination_channel_id,
                data,
                _timeout_height,
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
                timeout_height: MustBeZero,
                timeout_timestamp: Timestamp::from_nanos(<Uint<64>>::detokenize(timeout_timestamp)),
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
                <Uint<64> as SolType>::tokenize(&0),
                <Uint<64> as SolType>::tokenize(&self.timeout_timestamp.as_nanos()),
            )
        }

        fn stv_abi_encode_packed_to(&self, out: &mut Vec<u8>) {
            let (source_channel_id, destination_channel_id, data, MustBeZero, timeout_timestamp) =
                self.as_tuple();
            <SolTuple as SolType>::abi_encode_packed_to(
                &(
                    source_channel_id.raw(),
                    destination_channel_id.raw(),
                    data,
                    0,
                    timeout_timestamp.as_nanos(),
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
                <Uint<64> as SolType>::eip712_data_word(&0),
                <Uint<64> as SolType>::eip712_data_word(&self.timeout_timestamp.as_nanos()),
            ]
            .concat()
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy_sol_types::SolType;
    use unionlabs::primitives::H256;

    use super::*;

    #[test]
    fn packet_hash() {
        let packet = Packet {
            source_channel_id: ChannelId::new(1.try_into().unwrap()),
            destination_channel_id: ChannelId::new(9.try_into().unwrap()),
            data: "0xef2adec1b178443b9d06bb585939028d1d145463a272874f4188833bda0590bb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000078000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000003c000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001e00000000000000000000000000000000000000000000000014d1120d7b1600000000000000000000000000000000000000000000000000000000000000000022000000000000000000000000000000000000000000000000000000000000002600000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a00000000000000000000000000000000000000000000000014d1120d7b16000000000000000000000000000000000000000000000000000000000000000000014bd1b743615f903a630393f78234b4500fbe5691a000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a3078326336443733663430414365353132433433303430324463624463393438393431343330353034380000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000142be4bf88014a6574cb10df3b7826be8356aa24990000000000000000000000000000000000000000000000000000000000000000000000000000000000000007756e694254436400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000b756e694254432044656d6f000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003e62626e31683867717a763638713334723073386471343770756171726b6e74343835303533793330647235636a30667964766c307a7839716a7278617868000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000001f4000000000000000000000000000000000000000000000000000000000000022000000000000000000000000000000000000000000000000000000000000002600000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014bd1b743615f903a630393f78234b4500fbe5691a000000000000000000000000000000000000000000000000000000000000000000000000000000000000002a307832633644373366343041436535313243343330343032446362446339343839343134333035303438000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014420000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000000000000000000000000045745544800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000d5772617070656420457468657200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003e62626e31647530393867637661617234657573667a703035717070657433333879716339793637776338663574613472666d7978733734736a72683639730000".parse().unwrap(),
            timeout_height: MustBeZero,
            timeout_timestamp: Timestamp::from_nanos(9007199254740991),
        };

        assert_eq!(
            packet.hash(),
            "0xb204ec01ba72d32b884df0ecbfea727a15c82c1e0a08dff53fde4990d3ba589f"
                .parse::<H256>()
                .unwrap()
        );
    }

    #[test]
    fn smoke() {
        let packet = Packet {
            source_channel_id: ChannelId::new(3.try_into().unwrap()),
            destination_channel_id: ChannelId::new(4.try_into().unwrap()),
            data: "0xaf68ea97ce967fdc8ad331fd2481fbf7f5b3286bfab1c5e13a1b8151aa1fcd590000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000003200000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000014fb180000000000000000000000000000000000000000000000000000000000000022000000000000000000000000000000000000000000000000000000000014fb18000000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000260000000000000000000000000000000000000000000000000000000000000002b6f736d6f316c6e6b79746671636a776c6c77733633677666306d76397974303461737779346c7a6a6771640000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014c4c68628a4259eb4779b5330bc5f8762289fe37900000000000000000000000000000000000000000000000000000000000000000000000000000000000000446962632f42433236413741383035454344363832323731393437324243423738343241343845463039444632303631383246384632353942323539334542354432334642000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014a1a1d0b9182339e86e80db519218ea03ec09a1a100000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000014a1a1d0b9182339e86e80db519218ea03ec09a1a10000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".parse().unwrap(),
            timeout_height: MustBeZero,
            timeout_timestamp: Timestamp::from_nanos(1770987413320000000),
        };

        assert_eq!(
            packet.hash(),
            "0xb204ec01ba72d32b884df0ecbfea727a15c82c1e0a08dff53fde4990d3ba589f"
                .parse::<H256>()
                .unwrap()
        );
    }

    #[test]
    fn decode() {
        use alloy_sol_types::SolValue;

        let bz = hex_literal::hex!(
            "0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000018940c7ffa1c8300000000000000000000000000000000000000000000000000000000000000040084958e65227f5a4eb032dfdd430dfdeab6944a4bbe7fdd6d4088a39005d0416b0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000003200000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000260000000000000000000000000000000000000000000000000000000000000002b6f736d6f31396c6e7063733070767a39687463766d35386a6b7036616b35356d343978356e3935706a716300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000142c96e52fce14baa13868ca8182f8a7903e4e76e000000000000000000000000000000000000000000000000000000000000000000000000000000000000000446962632f44364530324335414538413337464332453341423146433841433136383837384144423837303534393338334446464541394644303230433233343532304137000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014222c042e17d94f4c83720583c75a37242921ba1c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000014222c042e17d94f4c83720583c75a37242921ba1c0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        );

        let packet = <Vec<Packet>>::abi_decode(&bz).unwrap();

        dbg!(packet);
    }
}
