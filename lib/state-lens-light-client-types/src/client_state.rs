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
/// EthAbi encoding is supported for this structure. This is achieved by flattening the tuple of
/// `Extra` into the tuple of `ClientState`.
///
/// The standalone client state tuple:
///
/// ```txt
/// (string,uint32,uint32,uint64)
/// ```
///
/// And the tuple of `Extra`:
///
/// ```txt
/// (uint64,string)
/// ```
///
/// This then becomes:
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
/// The expected encoding of this tuple is ***unprefixed***. In solidity, `abi.encode(value)`
/// ***MUST NOT*** be used, as this will wrap the entire structure in a single item tuple. Instead,
/// use `abi.encode(value.l2_chain_id,value.l1_client_id, value.l2_client_id,
/// value.l2_latest_height, value.a, value.b)`. Although this is more verbose, it results in a
/// consistent and predictable encoding and decoding. This also enables certain optimizations in
/// solidity, such as directly decoding the state from calldata:
///
/// ```solidity
/// ClientState calldata clientState;
/// assembly {
///     clientState := clientStateBytes.offset
/// }
/// ```
///
/// In rust, `abi_encode_params` ***MUST*** be used. This has the same effect as the per-field
/// `abi.encode` in solidity.
///
/// NOTE: For ethabi encoding, `Extra` must implement [`SolType`][alloy::sol_types::SolType],
/// [`SolValue`][alloy::sol_types::SolValue], and
/// [`SolTypeValue`][alloy::sol_types::private::SolTypeValue].
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
        dyn_abi::abi::{
            token::{PackedSeqToken, WordToken},
            Token, TokenSeq,
        },
        sol_types::{
            private::SolTypeValue,
            sol_data::{String as SolString, Uint},
            SolType, SolValue,
        },
    };
    use ibc_union_spec::ClientId;
    use tuple_join::{Join, Joined};
    use unionlabs::encoding::{Decode, Encode, EthAbi};

    use crate::ClientState;

    impl<Extra> Encode<EthAbi> for ClientState<Extra>
    where
        for<'a> Extra:
            SolValue<SolType: SolType<RustType = Extra, Token<'a>: TokenSeq<'a>>> + 'static,
        for<'a> ClientStateTokenTuple<'a>: Join<TokenOfSolValue<'a, Extra>, Out: Clone + Token<'a>>,
        for<'a> <(PackedSeqToken<'a>, WordToken, WordToken, WordToken) as Join<
            TokenOfSolValue<'a, Extra>,
        >>::Out: Clone + TokenSeq<'a>,
        for<'a> (PackedSeqToken<'a>, WordToken, WordToken, WordToken):
            Join<<<Extra as SolValue>::SolType as SolType>::Token<'a>>,
    {
        fn encode(self) -> Vec<u8> {
            self.abi_encode_params()
        }
    }

    impl<Extra> Decode<EthAbi> for ClientState<Extra>
    where
        for<'a> Extra:
            SolValue<SolType: SolType<RustType = Extra, Token<'a>: TokenSeq<'a>>> + 'static,
        for<'a> ClientStateTokenTuple<'a>: Join<TokenOfSolValue<'a, Extra>, Out: Clone + Token<'a>>,
        for<'a> <(PackedSeqToken<'a>, WordToken, WordToken, WordToken) as Join<
            TokenOfSolValue<'a, Extra>,
        >>::Out: Clone + TokenSeq<'a>,
        for<'a> (PackedSeqToken<'a>, WordToken, WordToken, WordToken):
            Join<<<Extra as SolValue>::SolType as SolType>::Token<'a>>,
    {
        type Error = alloy::sol_types::Error;

        fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
            <ClientState<Extra> as SolValue>::abi_decode_params(bytes, true)
        }
    }

    type ClientStateTokenTuple<'a> = (PackedSeqToken<'a>, WordToken, WordToken, WordToken);
    type TokenOfSolValue<'a, T> = <<T as SolValue>::SolType as SolType>::Token<'a>;

    impl<Extra> SolType for ClientState<Extra>
    where
        for<'a> Extra:
            SolValue<SolType: SolType<RustType = Extra, Token<'a>: TokenSeq<'a>>> + 'static,
        for<'a> ClientStateTokenTuple<'a>: Join<TokenOfSolValue<'a, Extra>, Out: Clone + Token<'a>>,
    {
        type RustType = Self;

        type Token<'a> = <ClientStateTokenTuple<'a> as Join<TokenOfSolValue<'a, Extra>>>::Out;

        const SOL_NAME: &'static str = "ClientState";

        // dynamic due to containing string
        const ENCODED_SIZE: Option<usize> = None;

        // dynamic due to containing string
        const PACKED_ENCODED_SIZE: Option<usize> = None;

        fn valid_token(token: &Self::Token<'_>) -> bool {
            // incredibly poor api, we must first validate the input in this method and then actually decode it in Self::detokenize. as such we must clone here, since we cannot .split() on a ref tuple
            let token: Self::Token<'_> = token.clone();

            let ((l2_chain_id, l1_client_id, l2_client_id, l2_latest_height), extra_tokens): (
                ClientStateTokenTuple,
                TokenOfSolValue<Extra>,
            ) = token.split();

            SolString::valid_token(&l2_chain_id)
                && <Uint<32>>::valid_token(&l1_client_id)
                && <Uint<32>>::valid_token(&l2_client_id)
                && <Uint<64>>::valid_token(&l2_latest_height)
                && Extra::SolType::valid_token(&extra_tokens)
        }

        fn detokenize(token: Self::Token<'_>) -> Self::RustType {
            let ((l2_chain_id, l1_client_id, l2_client_id, l2_latest_height), extra_tokens): (
                ClientStateTokenTuple,
                TokenOfSolValue<Extra>,
            ) = token.split();

            Self {
                l2_chain_id: <SolString as SolType>::detokenize(l2_chain_id),
                l1_client_id: ClientId::from_raw(<Uint<32> as SolType>::detokenize(l1_client_id))
                    .expect("???"),
                l2_client_id: ClientId::from_raw(<Uint<32> as SolType>::detokenize(l2_client_id))
                    .expect("???"),
                l2_latest_height: <Uint<64> as SolType>::detokenize(l2_latest_height),
                extra: <Extra::SolType as SolType>::detokenize(extra_tokens),
            }
        }
    }

    impl<Extra> SolTypeValue<Self> for ClientState<Extra>
    where
        for<'a> Extra:
            SolValue<SolType: SolType<RustType = Extra, Token<'a>: TokenSeq<'a>>> + 'static,
        for<'a> ClientStateTokenTuple<'a>: Join<TokenOfSolValue<'a, Extra>, Out: Clone + Token<'a>>,
    {
        fn stv_to_tokens(&self) -> <Self as SolType>::Token<'_> {
            let cs_tuple: ClientStateTokenTuple = (
                SolString::tokenize(&self.l2_chain_id),
                <Uint<32>>::tokenize(&self.l1_client_id.raw()),
                <Uint<32>>::tokenize(&self.l2_client_id.raw()),
                <Uint<64>>::tokenize(&self.l2_latest_height),
            );

            let extra_tuple = Extra::stv_to_tokens(&self.extra);

            cs_tuple.join(extra_tuple)
        }

        fn stv_abi_encode_packed_to(&self, _out: &mut Vec<u8>) {
            todo!()
        }

        fn stv_eip712_data_word(&self) -> alloy::sol_types::Word {
            todo!()
        }
    }

    impl<Extra> SolValue for ClientState<Extra>
    where
        for<'a> Extra:
            SolValue<SolType: SolType<RustType = Extra, Token<'a>: TokenSeq<'a>>> + 'static,
        for<'a> ClientStateTokenTuple<'a>: Join<TokenOfSolValue<'a, Extra>, Out: Clone + Token<'a>>,
    {
        type SolType = Self;
    }
}

#[cfg(test)]
mod tests {
    use alloy::sol_types::{
        private::SolTypeValue,
        sol_data::{String as SolString, Uint},
        SolType, SolValue,
    };
    use unionlabs::{
        encoding::{Bcs, Bincode, EthAbi, Json},
        test_utils::assert_codec_iso,
        tuple::AsTuple,
    };

    use super::*;

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

    impl SolType for Extra {
        type RustType = Self;

        type Token<'a> = <(Uint<64>, SolString) as SolType>::Token<'a>;

        const SOL_NAME: &'static str = "Extra";

        const ENCODED_SIZE: Option<usize> = None;

        const PACKED_ENCODED_SIZE: Option<usize> = None;

        fn valid_token(_token: &Self::Token<'_>) -> bool {
            true
        }

        fn detokenize((a, b): Self::Token<'_>) -> Self::RustType {
            Self {
                a: <Uint<64>>::detokenize(a),
                b: SolString::detokenize(b),
            }
        }
    }

    impl SolValue for Extra {
        type SolType = Self;
    }

    impl SolTypeValue<Self> for Extra {
        fn stv_to_tokens(&self) -> <Self as SolType>::Token<'_> {
            (
                <Uint<64> as SolType>::tokenize(&self.a),
                <SolString as SolType>::tokenize(&self.b),
            )
        }

        fn stv_abi_encode_packed_to(&self, _out: &mut Vec<u8>) {
            todo!()
        }

        fn stv_eip712_data_word(&self) -> alloy::sol_types::Word {
            todo!()
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
            <SolClientStateWithExtra as SolValue>::abi_decode_params(
                &cs.clone().abi_encode_params(),
                true
            )
            .unwrap()
        );

        assert_eq!(
            cs,
            <ClientState<Extra> as SolValue>::abi_decode_params(
                &cs_with_extra.abi_encode_params(),
                true
            )
            .unwrap()
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

    #[test]
    fn test_ethabi_unit() {
        let cs = ClientState {
            l2_chain_id: "l2_chain_id".to_owned(),
            l1_client_id: ClientId!(1),
            l2_client_id: ClientId!(2),
            l2_latest_height: 100,
            extra: (),
        };

        assert_codec_iso::<_, EthAbi>(&cs);
    }
}
