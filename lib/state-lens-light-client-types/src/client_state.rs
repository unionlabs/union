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
/// This struct implements bincode, json (serde) and ethabi serialization and deserialization by
/// flattening the generic param `Extra` into the top level structure.
///
/// NOTE: For ethabi encoding, `Extra` must implement [`AsTuple`][unionlabs::tuple::AsTuple].
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
    pub l1_client_id: u32,

    /// L2 client ID. This is the ID of the L2 client running on B (L1) tracking the C (L2).
    ///
    /// ("C" on "B")
    pub l2_client_id: u32,

    /// L2 latest height
    pub l2_latest_height: u64,

    #[cfg_attr(feature = "serde", serde(flatten))]
    pub extra: Extra,
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use alloy::{
        dyn_abi::abi::TokenSeq,
        sol_types::{SolType, SolValue},
    };
    use tuple_join::{Join, Joined};
    use unionlabs::{
        encoding::{Decode, Encode, EthAbi},
        tuple::{AsTuple, Tuple},
    };

    use crate::ClientState;

    type ClientStateFieldsTuple = (String, u32, u32, u64);

    impl<Extra: AsTuple> Encode<EthAbi> for ClientState<Extra>
    where
        for<'a, 'b> (&'a String, &'a u32, &'a u32, &'a u64): Join<
            <Extra::Tuple as Tuple>::Ref<'a>,
            Out: SolValue<SolType: SolType<Token<'b>: TokenSeq<'b>>>,
        >,
    {
        fn encode(self) -> Vec<u8> {
            let cs_tuple: (&String, &u32, &u32, &u64) = (
                &self.l2_chain_id,
                &self.l1_client_id,
                &self.l2_client_id,
                &self.l2_latest_height,
            );

            let extra_tuple = self.extra.as_tuple();

            cs_tuple.join(extra_tuple).abi_encode_params()
        }
    }

    impl<Extra: AsTuple> Decode<EthAbi> for ClientState<Extra>
    where
        ClientStateFieldsTuple: Join<
            Extra::Tuple,
            Out: From<
                    <<<ClientStateFieldsTuple as Join<Extra::Tuple>>::Out as SolValue>::SolType as SolType>::RustType
                >
                + SolValue<SolType: for<'a> SolType<Token<'a>: TokenSeq<'a>>>
        >,
    {
        type Error = alloy::sol_types::Error;

        fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
            let raw = <<ClientStateFieldsTuple as Join<Extra::Tuple>>::Out as SolValue>::abi_decode_params(
                bytes,
                true,
            )?;

            let ((
                    l2_chain_id,
                    l1_client_id,
                    l2_client_id,
                    l2_latest_height
                ),
                extra): (ClientStateFieldsTuple, Extra::Tuple) = raw.split();

            Ok(Self {
                l2_chain_id,
                l1_client_id,
                l2_client_id,
                l2_latest_height,
                extra: Extra::from_tuple(extra)
             })
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy::sol_types::SolValue;
    use unionlabs::{
        encoding::{Bincode, DecodeAs, EncodeAs, EthAbi, Json},
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

    #[derive(
        Debug,
        Clone,
        PartialEq,
        serde::Serialize,
        serde::Deserialize,
        bincode::Encode,
        bincode::Decode,
    )]
    pub struct ClientStateWithExtra {
        pub l2_chain_id: String,
        pub l1_client_id: u32,
        pub l2_client_id: u32,
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
            l1_client_id: 1,
            l2_client_id: 2,
            l2_latest_height: 100,
            extra: Extra {
                a: 5,
                b: "b".to_owned(),
            },
        };

        let cs_with_extra = ClientStateWithExtra {
            l2_chain_id: "l2_chain_id".to_owned(),
            l1_client_id: 1,
            l2_client_id: 2,
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
            l1_client_id: 1,
            l2_client_id: 2,
            l2_latest_height: 100,
            extra: Extra {
                a: 5,
                b: "b".to_owned(),
            },
        };

        let cs_with_extra = ClientStateWithExtra {
            l2_chain_id: "l2_chain_id".to_owned(),
            l1_client_id: 1,
            l2_client_id: 2,
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
    fn test_ethabi() {
        let cs = ClientState {
            l2_chain_id: "l2_chain_id".to_owned(),
            l1_client_id: 1,
            l2_client_id: 2,
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
            SolClientStateWithExtra::abi_decode_params(&cs.clone().encode_as::<EthAbi>(), true)
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
            l1_client_id: 1,
            l2_client_id: 2,
            l2_latest_height: 100,
            extra: (),
        };

        assert_codec_iso::<_, Bincode>(&cs);
    }

    #[test]
    fn test_serde_unit() {
        let cs = ClientState {
            l2_chain_id: "l2_chain_id".to_owned(),
            l1_client_id: 1,
            l2_client_id: 2,
            l2_latest_height: 100,
            extra: (),
        };

        assert_codec_iso::<_, Json>(&cs);
    }

    #[test]
    fn test_ethabi_unit() {
        let cs = ClientState {
            l2_chain_id: "l2_chain_id".to_owned(),
            l1_client_id: 1,
            l2_client_id: 2,
            l2_latest_height: 100,
            extra: (),
        };

        assert_codec_iso::<_, EthAbi>(&cs);
    }
}
