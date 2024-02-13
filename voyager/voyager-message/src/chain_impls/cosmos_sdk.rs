use std::{fmt::Debug, marker::PhantomData};

use chain_utils::cosmos_sdk::CosmosSdkChain;
use prost::Message;
use queue_msg::data;
use unionlabs::{
    encoding::Decode,
    ibc::core::{client::height::IsHeight, commitment::merkle_proof::MerkleProof},
    proof::{ClientStatePath, Path},
    traits::HeightOf,
    TryFromProto,
};

use crate::{
    chain_impls::cosmos_sdk::fetch::AbciQueryType,
    data::{AnyData, Data, IbcProof, IbcState},
    identified,
    use_aggregate::IsAggregateData,
    AnyLightClientIdentified, ChainExt, Identified, RelayerMsg,
};

pub async fn fetch_abci_query<Hc, Tr>(
    c: &Hc,
    path: Path<Hc::ClientId, Tr::Height>,
    height: HeightOf<Hc>,
    ty: AbciQueryType,
) -> RelayerMsg
where
    Hc: CosmosSdkChain + ChainExt<StateProof = MerkleProof>,
    Tr: ChainExt,
    AnyLightClientIdentified<AnyData>: From<identified!(Data<Hc, Tr>)>,
    Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>: IsAggregateData,

    Hc::StoredClientState<Tr>: TryFromProto,
    Hc::StoredConsensusState<Tr>: TryFromProto,
    <Hc::StoredClientState<Tr> as TryFrom<
        <Hc::StoredClientState<Tr> as unionlabs::Proto>::Proto,
    >>::Error: Debug,
    <Hc::StoredConsensusState<Tr> as TryFrom<
        <Hc::StoredConsensusState<Tr> as unionlabs::Proto>::Proto,
    >>::Error: Debug,
{
    let mut client =
        protos::cosmos::base::tendermint::v1beta1::service_client::ServiceClient::connect(
            c.grpc_url().clone(),
        )
        .await
        .unwrap();

    let query_result = client
        .abci_query(
            protos::cosmos::base::tendermint::v1beta1::AbciQueryRequest {
                data: path.to_string().into_bytes(),
                path: "store/ibc/key".to_string(),
                height: i64::try_from(height.revision_height()).unwrap() - 1_i64,
                prove: matches!(ty, AbciQueryType::Proof),
            },
        )
        .await
        .unwrap()
        .into_inner();

    match ty {
        AbciQueryType::State => match path {
            Path::ClientStatePath(path) => data(Identified::<Hc, Tr, _>::new(
                c.chain_id(),
                IbcState::<Hc, Tr, ClientStatePath<Hc::ClientId>> {
                    height,
                    state: Hc::StoredClientState::<Tr>::try_from_proto_bytes(&query_result.value)
                        .unwrap(),
                    path,
                },
            )),
            Path::ClientConsensusStatePath(path) => data(Identified::<Hc, Tr, _>::new(
                c.chain_id(),
                IbcState {
                    height,
                    state: Hc::StoredConsensusState::<Tr>::try_from_proto_bytes(
                        &query_result.value,
                    )
                    .unwrap(),
                    path,
                },
            )),
            Path::ConnectionPath(path) => data(Identified::<Hc, Tr, _>::new(
                c.chain_id(),
                IbcState {
                    height,
                    state: Decode::<unionlabs::encoding::Proto>::decode(&query_result.value)
                        .unwrap(),
                    path,
                },
            )),
            Path::ChannelEndPath(path) => data(Identified::<Hc, Tr, _>::new(
                c.chain_id(),
                IbcState {
                    height,
                    state: Decode::<unionlabs::encoding::Proto>::decode(&query_result.value)
                        .unwrap(),
                    path,
                },
            )),
            Path::CommitmentPath(path) => data(Identified::<Hc, Tr, _>::new(
                c.chain_id(),
                IbcState {
                    height,
                    state: query_result.value.try_into().unwrap(),
                    path,
                },
            )),
            Path::AcknowledgementPath(path) => data(Identified::<Hc, Tr, _>::new(
                c.chain_id(),
                IbcState {
                    height,
                    state: query_result.value.try_into().unwrap(),
                    path,
                },
            )),
        },
        AbciQueryType::Proof => {
            let proof = MerkleProof::try_from(protos::ibc::core::commitment::v1::MerkleProof {
                proofs: query_result
                    .proof_ops
                    .unwrap()
                    .ops
                    .into_iter()
                    .map(|op| {
                        protos::cosmos::ics23::v1::CommitmentProof::decode(op.data.as_slice())
                            .unwrap()
                    })
                    .collect::<Vec<_>>(),
            })
            .unwrap();

            match path {
                Path::ClientStatePath(path) => data(Identified::<Hc, Tr, _>::new(
                    c.chain_id(),
                    IbcProof::<Hc, Tr, _> {
                        proof,
                        height,
                        path,
                        __marker: PhantomData,
                    },
                )),
                Path::ClientConsensusStatePath(path) => data(Identified::<Hc, Tr, _>::new(
                    c.chain_id(),
                    IbcProof::<Hc, Tr, _> {
                        proof,
                        height,
                        path,
                        __marker: PhantomData,
                    },
                )),
                Path::ConnectionPath(path) => data(Identified::<Hc, Tr, _>::new(
                    c.chain_id(),
                    IbcProof::<Hc, Tr, _> {
                        proof,
                        height,
                        path,
                        __marker: PhantomData,
                    },
                )),
                Path::ChannelEndPath(path) => data(Identified::<Hc, Tr, _>::new(
                    c.chain_id(),
                    IbcProof::<Hc, Tr, _> {
                        proof,
                        height,
                        path,
                        __marker: PhantomData,
                    },
                )),
                Path::CommitmentPath(path) => data(Identified::<Hc, Tr, _>::new(
                    c.chain_id(),
                    IbcProof::<Hc, Tr, _> {
                        proof,
                        height,
                        path,
                        __marker: PhantomData,
                    },
                )),
                Path::AcknowledgementPath(path) => data(Identified::<Hc, Tr, _>::new(
                    c.chain_id(),
                    IbcProof::<Hc, Tr, _> {
                        proof,
                        height,
                        path,
                        __marker: PhantomData,
                    },
                )),
            }
        }
    }
}
pub mod fetch {
    use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
    use serde::{Deserialize, Serialize};
    use unionlabs::traits::HeightOf;

    use crate::{ChainExt, PathOf};

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
    #[cfg_attr(
        feature = "arbitrary",
        derive(arbitrary::Arbitrary),
        arbitrary(bound = "Hc: ChainExt, Tr: ChainExt")
    )]
    pub struct FetchAbciQuery<Hc: ChainExt, Tr: ChainExt> {
        pub path: PathOf<Hc, Tr>,
        pub height: HeightOf<Hc>,
        pub ty: AbciQueryType,
    }

    #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
    #[serde(
        bound(serialize = "", deserialize = ""),
        deny_unknown_fields,
        rename_all = "snake_case"
    )]
    #[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
    pub enum AbciQueryType {
        State,
        Proof,
    }
}
