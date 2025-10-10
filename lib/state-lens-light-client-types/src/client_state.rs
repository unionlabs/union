use ibc_union_spec::ClientId;
use tuple_join::{Join, Joined};
use unionlabs::tuple::{AsTuple, Tuple, TupleAsRef};

/// Representation of the client state of a state lens client.
///
/// For a state lens client A->B->C, where the state lens is running on A and tracking C, the
/// terminology is as follows:
///
/// - B is L1
/// - C is L2
///
/// where C "settles" on B with the client `self.l2_client_id`, and B "settles" on A with
/// `self.l1_client_id`.
///
/// # Supported Encoding Formats
///
/// This struct implements bincode, json (serde) and ethabi serialization and deserialization by
/// flattening the generic param `Extra` into the top level structure.
///
/// The following struct will be used in all examples:
///
/// ```rust
/// struct Extra {
///     pub a: u64,
///     pub b: String,
/// }
/// ```
///
/// ## EthAbi
///
/// EthAbi encoding is supported for this structure through the [`DecodeExtra`][ethabi::DecodeExtra] and
/// [`ExtraToTokens`][ethabi::ExtraToTokens] traits.
///
/// The following is an example implementation of these traits for `Extra`:
///
/// ```rust,ignore
/// // decodes as (uint64,string)
/// impl DecodeExtra for Extra {
///     fn decode_extra(
///         decoder: &mut alloy::dyn_abi::Decoder,
///     ) -> Result<Self, alloy::dyn_abi::Error> {
///         Ok(Self {
///             a: u64::detokenize(decoder.take_word()?.into()),
///             b: String::detokenize(decoder.decode::<PackedSeqToken>()?),
///         })
///     }
/// }
///
///
/// // encodes as (uint64,string)
/// impl ExtraToTokens for Extra {
///     fn encode_extra_to_dyn_value(self) -> Vec<DynSolValue> {
///         vec![self.a.into(), self.b.into()]
///     }
/// }
/// ```
///
/// With these implementations, and the standalone client state tuple:
///
/// ```txt
/// (string,uint32,uint32,uint64)
/// ```
///
/// `ClientState<Extra>` is then encoded and decoded as if it were this tuple:
///
/// ```txt
/// (string,uint32,uint32,uint64,uint64,string)
/// ```
///
/// Which is equivalent to this solidity struct:
///
/// ```solidity
/// struct ClientState {
///     string l2_chain_id;
///     uint32 l1_client_id;
///     uint32 l2_client_id;
///     uint64 l2_latest_height;
///     uint64 a;
///     string b;
/// }
/// ```
///
/// The encoding of this tuple is ***unprefixed***. In solidity, `abi.encode(value)` ***MUST NOT***
/// be used, as this will wrap the entire structure in a single item tuple. Instead, use
/// `abi.encode(value.l2_chain_id,value.l1_client_id, value.l2_client_id, value.l2_latest_height,
/// value.a, value.b)`. Although this is more verbose, it results in a consistent and predictable
/// encoding and decoding. This also enables certain optimizations in solidity, such as directly
/// decoding the state from calldata:
///
/// ```solidity
/// ClientState calldata clientState;
/// assembly {
///     clientState := clientStateBytes.offset
/// }
/// ```
///
/// In rust, `Encode<EthAbi>` ***MUST*** be used (as it uses `abi_encode_params` internally). This
/// has the same effect as the per-field `abi.encode` in solidity.
///
/// ## JSON
///
/// JSON encoding is implemented via serde. `Extra` is `#[serde(flattened)]` into the top level
/// object.
///
/// NOTE: Due to limitations with serde, it is not possible to `#[serde(deny_unknown_fields)]` on
/// this struct with the flattened `Extra`. Any unknown fields will be silently dropped during
/// deserialization.
///
/// ## Bincode
///
/// Bincode encoding is implemented via [`bincode`]. Since bincode inlines nested objects, there is
/// no difference between `ClientState<Extra>` and the equivalent struct with the fields of `Extra`
/// inlined directly.
///
/// ## Bcs
///
/// Bcs encoding is implemented via serde and [`bcs`]. Since `bcs` leverages serde, it is
/// incompatible with the derived serde implementation due to `#[serde(flatten)]`. To work around
/// this, [`AsTuple`] is implemented for `ClientState`. To encode bcs, convert to tuple form with
/// [`AsTuple::as_tuple`] first and then encode that structure. To decode `bcs`, decode into
/// `<ClientState<Extra> as AsTuple>::Tuple` and then convert from that value with
/// [`AsTuple::from_tuple`].
///
/// [`bcs`]: (https://docs.rs/bcs/latest/bcs/)
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    // https://serde.rs/field-attrs.html#flatten
    // serde(deny_unknown_fields)
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ClientState<Extra> {
    /// L2 chain ID. This is the same as the ID of the chain being tracked by `self.l2_client_id`.
    ///
    /// ("C")
    pub l2_chain_id: String,

    /// L1 client ID. This is the ID of the L1 client running on A that is used to check the L2
    /// inclusion proof against.
    ///
    /// ("B" on "A")
    pub l1_client_id: ClientId,

    /// L2 client ID. This is the ID of the L2 client running on B (L1) tracking the C (L2).
    ///
    /// ("C" on "B")
    pub l2_client_id: ClientId,

    /// L2 latest height
    pub l2_latest_height: u64,

    #[cfg_attr(feature = "serde", serde(flatten))]
    pub extra: Extra,
}

type ClientStateFieldsTuple = (String, ClientId, ClientId, u64);

impl<Extra> AsTuple for ClientState<Extra>
where
    Extra: AsTuple,
    ClientStateFieldsTuple: Join<Extra::Tuple, Out: Tuple + TupleAsRef>,
    // can't use `<ClientStateFieldsTuple as Tuple>::Ref<'a>` here for some reason?
    for<'a> (&'a String, &'a ClientId, &'a ClientId, &'a u64): Join<
            <Extra::Tuple as Tuple>::Ref<'a>,
            Out = <<ClientStateFieldsTuple as Join<Extra::Tuple>>::Out as Tuple>::Ref<'a>,
        >,
{
    type Tuple = <ClientStateFieldsTuple as Join<Extra::Tuple>>::Out;

    fn as_tuple(&self) -> <Self::Tuple as Tuple>::Ref<'_> {
        (
            &self.l2_chain_id,
            &self.l1_client_id,
            &self.l2_client_id,
            &self.l2_latest_height,
        )
            .join(self.extra.as_tuple())
    }

    fn into_tuple(self) -> Self::Tuple {
        (
            self.l2_chain_id,
            self.l1_client_id,
            self.l2_client_id,
            self.l2_latest_height,
        )
            .join(self.extra.into_tuple())
    }

    fn from_tuple(tuple: Self::Tuple) -> Self {
        let ((l2_chain_id, l1_client_id, l2_client_id, l2_latest_height), extra_tuple): (
            ClientStateFieldsTuple,
            Extra::Tuple,
        ) = tuple.split();

        Self {
            l2_chain_id,
            l1_client_id,
            l2_client_id,
            l2_latest_height,
            extra: Extra::from_tuple(extra_tuple),
        }
    }
}

// avert your eyes, here be dragons
#[cfg(feature = "ethabi")]
pub mod ethabi {
    use alloy::{
        dyn_abi::{Decoder, DynSolValue, abi::token::PackedSeqToken},
        sol_types::SolValue,
    };
    use ibc_union_spec::ClientId;
    use unionlabs::encoding::{Decode, Encode, EthAbi};

    use crate::ClientState;

    /// Decode this type from the given decoder. This allows for dynamic decoding depending on the
    /// values decoded.
    pub trait DecodeExtra: Sized {
        fn decode_extra(decoder: &mut Decoder) -> Result<Self, alloy::dyn_abi::Error>;
    }

    /// Encode this value into a stream of tokens. This allows for dynamic encoding depending on the
    /// values of self.
    pub trait ExtraToTokens: Sized {
        fn encode_extra_to_dyn_value(self) -> Vec<DynSolValue>;
    }

    impl<Extra: ExtraToTokens> Encode<EthAbi> for ClientState<Extra> {
        fn encode(self) -> Vec<u8> {
            let mut dt = vec![
                DynSolValue::from(self.l2_chain_id),
                DynSolValue::from(self.l1_client_id.raw()),
                DynSolValue::from(self.l2_client_id.raw()),
                DynSolValue::from(self.l2_latest_height),
            ];

            dt.extend(self.extra.encode_extra_to_dyn_value());

            DynSolValue::Tuple(dt).abi_encode_params()
        }
    }

    impl<Extra: DecodeExtra> Decode<EthAbi> for ClientState<Extra> {
        type Error = alloy::dyn_abi::Error;

        fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
            let mut decoder = Decoder::new(bytes);

            let l2_chain_id =
                String::from_utf8(decoder.decode::<PackedSeqToken>()?.into_vec()).unwrap();
            let l1_client_id = decoder.take_word()?;
            let l2_client_id = decoder.take_word()?;
            let l2_latest_height = decoder.take_word()?;

            let extra = Extra::decode_extra(&mut decoder)?;

            Ok(Self {
                l2_chain_id,
                l1_client_id: ClientId::new(
                    u32::detokenize(l1_client_id.into()).try_into().unwrap(),
                ),
                l2_client_id: ClientId::new(
                    u32::detokenize(l2_client_id.into()).try_into().unwrap(),
                ),
                l2_latest_height: u64::detokenize(l2_latest_height.into()),
                extra,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy::{
        dyn_abi::{DynSolValue, abi::token::PackedSeqToken},
        sol_types::SolValue,
    };
    use unionlabs::{
        encoding::{Bcs, Bincode, DecodeAs, EncodeAs, EthAbi, Json},
        test_utils::assert_codec_iso,
        tuple::AsTuple,
    };

    use super::*;
    use crate::client_state::ethabi::{DecodeExtra, ExtraToTokens};

    #[derive(
        Debug,
        Clone,
        PartialEq,
        serde::Serialize,
        serde::Deserialize,
        bincode::Encode,
        bincode::Decode,
        AsTuple,
    )]
    struct Extra {
        pub a: u64,
        pub b: String,
    }

    impl DecodeExtra for Extra {
        fn decode_extra(
            decoder: &mut alloy::dyn_abi::Decoder,
        ) -> Result<Self, alloy::dyn_abi::Error> {
            Ok(Self {
                a: u64::detokenize(decoder.take_word()?.into()),
                b: String::detokenize(decoder.decode::<PackedSeqToken>()?),
            })
        }
    }

    impl ExtraToTokens for Extra {
        fn encode_extra_to_dyn_value(self) -> Vec<DynSolValue> {
            vec![self.a.into(), self.b.into()]
        }
    }

    #[derive(
        Debug,
        Clone,
        PartialEq,
        serde::Serialize,
        serde::Deserialize,
        bincode::Encode,
        bincode::Decode,
        AsTuple,
    )]
    pub struct ClientStateWithExtra {
        pub l2_chain_id: String,
        pub l1_client_id: ClientId,
        pub l2_client_id: ClientId,
        pub l2_latest_height: u64,
        pub a: u64,
        pub b: String,
    }

    alloy::sol! {
        #[derive(Debug, PartialEq)]
        struct SolClientStateWithExtra {
            string l2_chain_id;
            uint32 l1_client_id;
            uint32 l2_client_id;
            uint64 l2_latest_height;
            uint64 a;
            string b;
        }
    }

    #[test]
    fn test_bincode() {
        let cs = ClientState {
            l2_chain_id: "l2_chain_id".to_owned(),
            l1_client_id: ClientId!(1),
            l2_client_id: ClientId!(2),
            l2_latest_height: 100,
            extra: Extra {
                a: 5,
                b: "b".to_owned(),
            },
        };

        let cs_with_extra = ClientStateWithExtra {
            l2_chain_id: "l2_chain_id".to_owned(),
            l1_client_id: ClientId!(1),
            l2_client_id: ClientId!(2),
            l2_latest_height: 100,
            a: 5,
            b: "b".to_owned(),
        };

        let cs_bz = bincode::encode_to_vec(&cs, bincode::config::legacy()).unwrap();
        let cs_with_extra_bz =
            bincode::encode_to_vec(&cs_with_extra, bincode::config::legacy()).unwrap();

        assert_eq!(cs_bz, cs_with_extra_bz);

        let cs_with_extra_from_cs_bz: ClientStateWithExtra =
            bincode::decode_from_slice(&cs_bz, bincode::config::legacy())
                .unwrap()
                .0;

        let cs_from_cs_with_extra_bz: ClientState<Extra> =
            bincode::decode_from_slice(&cs_with_extra_bz, bincode::config::legacy())
                .unwrap()
                .0;

        assert_eq!(cs_from_cs_with_extra_bz, cs);
        assert_eq!(cs_with_extra_from_cs_bz, cs_with_extra);
    }

    #[test]
    fn test_serde() {
        let cs = ClientState {
            l2_chain_id: "l2_chain_id".to_owned(),
            l1_client_id: ClientId!(1),
            l2_client_id: ClientId!(2),
            l2_latest_height: 100,
            extra: Extra {
                a: 5,
                b: "b".to_owned(),
            },
        };

        let cs_with_extra = ClientStateWithExtra {
            l2_chain_id: "l2_chain_id".to_owned(),
            l1_client_id: ClientId!(1),
            l2_client_id: ClientId!(2),
            l2_latest_height: 100,
            a: 5,
            b: "b".to_owned(),
        };

        assert_eq!(
            cs_with_extra,
            serde_json::from_str::<ClientStateWithExtra>(&serde_json::to_string(&cs).unwrap())
                .unwrap()
        );

        assert_eq!(
            cs,
            serde_json::from_str::<ClientState<Extra>>(
                &serde_json::to_string(&cs_with_extra).unwrap()
            )
            .unwrap()
        );
    }

    #[test]
    fn test_bcs() {
        let cs = ClientState {
            l2_chain_id: "l2_chain_id".to_owned(),
            l1_client_id: ClientId!(1),
            l2_client_id: ClientId!(2),
            l2_latest_height: 100,
            extra: Extra {
                a: 5,
                b: "b".to_owned(),
            },
        };

        let cs_with_extra = ClientStateWithExtra {
            l2_chain_id: "l2_chain_id".to_owned(),
            l1_client_id: ClientId!(1),
            l2_client_id: ClientId!(2),
            l2_latest_height: 100,
            a: 5,
            b: "b".to_owned(),
        };

        assert_eq!(
            cs_with_extra,
            bcs::from_bytes::<ClientStateWithExtra>(&bcs::to_bytes(&cs.as_tuple()).unwrap())
                .unwrap()
        );

        assert_eq!(
            cs.into_tuple(),
            bcs::from_bytes::<<ClientState<Extra> as AsTuple>::Tuple>(
                &bcs::to_bytes(&cs_with_extra).unwrap()
            )
            .unwrap()
        );
    }

    #[test]
    fn test_ethabi() {
        let cs = ClientState {
            l2_chain_id: "l2_chain_id".to_owned(),
            l1_client_id: ClientId!(1),
            l2_client_id: ClientId!(2),
            l2_latest_height: 100,
            extra: Extra {
                a: 5,
                b: "b".to_owned(),
            },
        };

        let cs_with_extra = SolClientStateWithExtra {
            l2_chain_id: "l2_chain_id".to_owned(),
            l1_client_id: 1,
            l2_client_id: 2,
            l2_latest_height: 100,
            a: 5,
            b: "b".to_owned(),
        };

        assert_eq!(
            cs_with_extra,
            <SolClientStateWithExtra as SolValue>::abi_decode_params_validate(
                &cs.clone().encode_as::<EthAbi>(),
            )
            .unwrap()
        );

        assert_eq!(
            cs,
            <ClientState<Extra>>::decode_as::<EthAbi>(&cs_with_extra.abi_encode_params()).unwrap()
        );
    }

    #[test]
    fn test_bincode_unit() {
        let cs = ClientState {
            l2_chain_id: "l2_chain_id".to_owned(),
            l1_client_id: ClientId!(1),
            l2_client_id: ClientId!(2),
            l2_latest_height: 100,
            extra: (),
        };

        assert_codec_iso::<_, Bincode>(&cs);
    }

    #[test]
    fn test_serde_unit() {
        let cs = ClientState {
            l2_chain_id: "l2_chain_id".to_owned(),
            l1_client_id: ClientId!(1),
            l2_client_id: ClientId!(2),
            l2_latest_height: 100,
            extra: (),
        };

        assert_codec_iso::<_, Json>(&cs);
    }

    #[test]
    fn test_bcs_unit() {
        let cs = ClientState {
            l2_chain_id: "l2_chain_id".to_owned(),
            l1_client_id: ClientId!(1),
            l2_client_id: ClientId!(2),
            l2_latest_height: 100,
            extra: (),
        };

        assert_codec_iso::<_, Bcs>(&cs);
    }
}
