use std::{fmt::Debug, marker::PhantomData};

use chain_utils::cosmos_sdk::CosmosSdkChain;
use prost::Message;
use queue_msg::{data, QueueMsg};
use unionlabs::{
    encoding::{Decode, Proto},
    ibc::core::client::height::IsHeight,
    proof::{ClientStatePath, Path},
    traits::{Chain, HeightOf},
};

use crate::{
    chain_impls::cosmos_sdk::fetch::AbciQueryType,
    data::{AnyData, Data, IbcProof, IbcState},
    id, identified,
    use_aggregate::IsAggregateData,
    AnyLightClientIdentified, ChainExt, Identified, RelayMessageTypes,
};

pub async fn fetch_abci_query<Hc, Tr>(
    c: &Hc,
    path: Path<Hc::ClientId, Tr::Height>,
    height: HeightOf<Hc>,
    ty: AbciQueryType,
) -> QueueMsg<RelayMessageTypes>
where
    Hc: CosmosSdkChain + ChainExt,
    <Hc as Chain>::StateProof: TryFrom<protos::ibc::core::commitment::v1::MerkleProof>,
    <<Hc as Chain>::StateProof as TryFrom<protos::ibc::core::commitment::v1::MerkleProof>>::Error:
        Debug,
    Tr: ChainExt,
    AnyLightClientIdentified<AnyData>: From<identified!(Data<Hc, Tr>)>,
    Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>: IsAggregateData,

    Hc::StoredClientState<Tr>: Decode<Proto>,
    Hc::StoredConsensusState<Tr>: Decode<Proto>,
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
            Path::ClientStatePath(path) => data(id::<Hc, Tr, _>(
                c.chain_id(),
                IbcState::<ClientStatePath<Hc::ClientId>, Hc, Tr> {
                    height,
                    state: Hc::StoredClientState::<Tr>::decode(&query_result.value).unwrap(),
                    path,
                },
            )),
            Path::ClientConsensusStatePath(path) => data(id::<Hc, Tr, _>(
                c.chain_id(),
                IbcState {
                    height,
                    state: Hc::StoredConsensusState::<Tr>::decode(&query_result.value).unwrap(),
                    path,
                },
            )),
            Path::ConnectionPath(path) => data(id::<Hc, Tr, _>(
                c.chain_id(),
                IbcState {
                    height,
                    state: Decode::<unionlabs::encoding::Proto>::decode(&query_result.value)
                        .unwrap(),
                    path,
                },
            )),
            Path::ChannelEndPath(path) => data(id::<Hc, Tr, _>(
                c.chain_id(),
                IbcState {
                    height,
                    state: Decode::<unionlabs::encoding::Proto>::decode(&query_result.value)
                        .unwrap(),
                    path,
                },
            )),
            Path::CommitmentPath(path) => data(id::<Hc, Tr, _>(
                c.chain_id(),
                IbcState {
                    height,
                    state: query_result.value.try_into().unwrap(),
                    path,
                },
            )),
            Path::AcknowledgementPath(path) => data(id::<Hc, Tr, _>(
                c.chain_id(),
                IbcState {
                    height,
                    state: query_result.value.try_into().unwrap(),
                    path,
                },
            )),
        },
        AbciQueryType::Proof => {
            let proof = Hc::StateProof::try_from(protos::ibc::core::commitment::v1::MerkleProof {
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
                Path::ClientStatePath(path) => data(id::<Hc, Tr, _>(
                    c.chain_id(),
                    IbcProof::<_, Hc, Tr> {
                        proof,
                        height,
                        path,
                        __marker: PhantomData,
                    },
                )),
                Path::ClientConsensusStatePath(path) => data(id::<Hc, Tr, _>(
                    c.chain_id(),
                    IbcProof::<_, Hc, Tr> {
                        proof,
                        height,
                        path,
                        __marker: PhantomData,
                    },
                )),
                Path::ConnectionPath(path) => data(id::<Hc, Tr, _>(
                    c.chain_id(),
                    IbcProof::<_, Hc, Tr> {
                        proof,
                        height,
                        path,
                        __marker: PhantomData,
                    },
                )),
                Path::ChannelEndPath(path) => data(id::<Hc, Tr, _>(
                    c.chain_id(),
                    IbcProof::<_, Hc, Tr> {
                        proof,
                        height,
                        path,
                        __marker: PhantomData,
                    },
                )),
                Path::CommitmentPath(path) => data(id::<Hc, Tr, _>(
                    c.chain_id(),
                    IbcProof::<_, Hc, Tr> {
                        proof,
                        height,
                        path,
                        __marker: PhantomData,
                    },
                )),
                Path::AcknowledgementPath(path) => data(id::<Hc, Tr, _>(
                    c.chain_id(),
                    IbcProof::<_, Hc, Tr> {
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
    use std::marker::PhantomData;

    use chain_utils::cosmos_sdk::CosmosSdkChain;
    use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
    use queue_msg::{data, msg_struct, QueueMsg};
    use serde::{Deserialize, Serialize};
    use tendermint_rpc::Client;
    use unionlabs::{ibc::core::client::height::IsHeight, traits::HeightOf};

    use crate::{
        chain_impls::cosmos_sdk::{
            data::{TrustedCommit, TrustedValidators, UntrustedCommit, UntrustedValidators},
            tendermint_helpers::{
                tendermint_commit_to_signed_header, tendermint_validator_info_to_validator,
            },
        },
        data::{AnyData, Data},
        id, identified, AnyLightClientIdentified, ChainExt, PathOf, RelayMessageTypes,
    };

    #[msg_struct]
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

    #[msg_struct]
    pub struct FetchTrustedCommit<Hc: ChainExt, #[cover] Tr: ChainExt> {
        pub height: Hc::Height,
    }

    #[msg_struct]
    pub struct FetchUntrustedCommit<Hc: ChainExt, #[cover] Tr: ChainExt> {
        pub height: Hc::Height,
    }

    #[msg_struct]
    pub struct FetchTrustedValidators<Hc: ChainExt, #[cover] Tr: ChainExt> {
        pub height: Hc::Height,
    }

    #[msg_struct]
    pub struct FetchUntrustedValidators<Hc: ChainExt, #[cover] Tr: ChainExt> {
        pub height: Hc::Height,
    }

    pub async fn fetch_trusted_commit<Hc, Tr>(
        hc: &Hc,
        height: Hc::Height,
    ) -> QueueMsg<RelayMessageTypes>
    where
        Hc: CosmosSdkChain + ChainExt,
        <Hc as ChainExt>::Data<Tr>: From<TrustedCommit<Hc, Tr>>,
        Tr: ChainExt,
        AnyLightClientIdentified<AnyData>: From<identified!(Data<Hc, Tr>)>,
    {
        let commit = hc
            .tm_client()
            .commit(
                TryInto::<::tendermint::block::Height>::try_into(height.revision_height()).unwrap(),
            )
            .await
            .unwrap();

        let signed_header = tendermint_commit_to_signed_header(commit);

        data(id::<Hc, Tr, _>(
            hc.chain_id(),
            Data::specific(TrustedCommit {
                height,
                // REVIEW: Ensure `commit.canonical`?
                signed_header,
                __marker: PhantomData,
            }),
        ))
    }

    pub async fn fetch_untrusted_commit<Hc, Tr>(
        hc: &Hc,
        height: Hc::Height,
    ) -> QueueMsg<RelayMessageTypes>
    where
        Hc: CosmosSdkChain + ChainExt,
        <Hc as ChainExt>::Data<Tr>: From<UntrustedCommit<Hc, Tr>>,
        Tr: ChainExt,
        AnyLightClientIdentified<AnyData>: From<identified!(Data<Hc, Tr>)>,
    {
        let commit = hc
            .tm_client()
            .commit(
                TryInto::<::tendermint::block::Height>::try_into(height.revision_height()).unwrap(),
            )
            .await
            .unwrap();

        let signed_header = tendermint_commit_to_signed_header(commit);

        data(id::<Hc, Tr, _>(
            hc.chain_id(),
            Data::specific(UntrustedCommit {
                height,
                // REVIEW: Ensure `commit.canonical`?
                signed_header,
                __marker: PhantomData,
            }),
        ))
    }

    pub async fn fetch_trusted_validators<Hc, Tr>(
        hc: &Hc,
        height: Hc::Height,
    ) -> QueueMsg<RelayMessageTypes>
    where
        Hc: CosmosSdkChain + ChainExt,
        <Hc as ChainExt>::Data<Tr>: From<TrustedValidators<Hc, Tr>>,
        Tr: ChainExt,
        AnyLightClientIdentified<AnyData>: From<identified!(Data<Hc, Tr>)>,
    {
        let validators = hc
            .tm_client()
            .validators(
                TryInto::<::tendermint::block::Height>::try_into(height.revision_height()).unwrap(),
                tendermint_rpc::Paging::All,
            )
            .await
            .unwrap()
            .validators
            .into_iter()
            .map(tendermint_validator_info_to_validator)
            .collect();

        data(id::<Hc, Tr, _>(
            hc.chain_id(),
            Data::specific(TrustedValidators {
                height,
                validators,
                __marker: PhantomData,
            }),
        ))
    }

    pub async fn fetch_untrusted_validators<Hc, Tr>(
        hc: &Hc,
        height: Hc::Height,
    ) -> QueueMsg<RelayMessageTypes>
    where
        Hc: CosmosSdkChain + ChainExt,
        <Hc as ChainExt>::Data<Tr>: From<UntrustedValidators<Hc, Tr>>,
        Tr: ChainExt,
        AnyLightClientIdentified<AnyData>: From<identified!(Data<Hc, Tr>)>,
    {
        let validators = hc
            .tm_client()
            .validators(
                TryInto::<::tendermint::block::Height>::try_into(height.revision_height()).unwrap(),
                tendermint_rpc::Paging::All,
            )
            .await
            .unwrap()
            .validators
            .into_iter()
            .map(tendermint_validator_info_to_validator)
            .collect();

        data(id::<Hc, Tr, _>(
            hc.chain_id(),
            Data::specific(UntrustedValidators {
                height,
                validators,
                __marker: PhantomData,
            }),
        ))
    }
}

pub mod data {
    use queue_msg::msg_struct;
    use unionlabs::tendermint::types::{signed_header::SignedHeader, validator::Validator};

    use crate::ChainExt;

    #[msg_struct]
    pub struct UntrustedCommit<Hc: ChainExt, #[cover] Tr: ChainExt> {
        pub height: Hc::Height,
        pub signed_header: SignedHeader,
    }

    #[msg_struct]
    pub struct TrustedCommit<Hc: ChainExt, #[cover] Tr: ChainExt> {
        pub height: Hc::Height,
        pub signed_header: SignedHeader,
    }

    #[msg_struct]
    pub struct TrustedValidators<Hc: ChainExt, #[cover] Tr: ChainExt> {
        pub height: Hc::Height,
        pub validators: Vec<Validator>,
    }

    #[msg_struct]
    pub struct UntrustedValidators<Hc: ChainExt, #[cover] Tr: ChainExt> {
        pub height: Hc::Height,
        pub validators: Vec<Validator>,
    }
}

pub mod tendermint_helpers {
    use unionlabs::{
        bounded::BoundedI64,
        google::protobuf::timestamp::Timestamp,
        hash::H256,
        tendermint::{
            crypto::public_key::PublicKey,
            types::{
                block_id::BlockId, commit::Commit, commit_sig::CommitSig,
                part_set_header::PartSetHeader, signed_header::SignedHeader, validator::Validator,
            },
        },
    };

    pub fn tendermint_commit_to_signed_header(
        commit: tendermint_rpc::endpoint::commit::Response,
    ) -> SignedHeader {
        let header_timestamp =
            tendermint_proto::google::protobuf::Timestamp::from(commit.signed_header.header.time);

        SignedHeader {
            header: unionlabs::tendermint::types::header::Header {
                version: unionlabs::tendermint::version::consensus::Consensus {
                    block: commit.signed_header.header.version.block,
                    app: commit.signed_header.header.version.app,
                },
                chain_id: commit.signed_header.header.chain_id.into(),
                height: tendermint_height_to_bounded_i64(commit.signed_header.header.height),
                time: Timestamp {
                    seconds: header_timestamp.seconds.try_into().unwrap(),
                    nanos: header_timestamp.nanos.try_into().unwrap(),
                },
                last_block_id: BlockId {
                    hash: tendermint_hash_to_h256(
                        commit.signed_header.header.last_block_id.unwrap().hash,
                    ),
                    part_set_header: PartSetHeader {
                        total: commit
                            .signed_header
                            .header
                            .last_block_id
                            .unwrap()
                            .part_set_header
                            .total,
                        hash: tendermint_hash_to_h256(
                            commit
                                .signed_header
                                .header
                                .last_block_id
                                .unwrap()
                                .part_set_header
                                .hash,
                        ),
                    },
                },
                last_commit_hash: tendermint_hash_to_h256(
                    commit.signed_header.header.last_commit_hash.unwrap(),
                ),
                data_hash: tendermint_hash_to_h256(commit.signed_header.header.data_hash.unwrap()),
                validators_hash: tendermint_hash_to_h256(
                    commit.signed_header.header.validators_hash,
                ),
                next_validators_hash: tendermint_hash_to_h256(
                    commit.signed_header.header.next_validators_hash,
                ),
                consensus_hash: tendermint_hash_to_h256(commit.signed_header.header.consensus_hash),
                app_hash: commit
                    .signed_header
                    .header
                    .app_hash
                    .as_bytes()
                    .try_into()
                    .unwrap(),
                last_results_hash: tendermint_hash_to_h256(
                    commit.signed_header.header.last_results_hash.unwrap(),
                ),
                evidence_hash: tendermint_hash_to_h256(
                    commit.signed_header.header.evidence_hash.unwrap(),
                ),
                proposer_address: commit
                    .signed_header
                    .header
                    .proposer_address
                    .as_bytes()
                    .try_into()
                    .expect("value is a [u8; 20] internally, this should not fail; qed;"),
            },
            commit: Commit {
                height: tendermint_height_to_bounded_i64(commit.signed_header.commit.height),
                round: i32::from(commit.signed_header.commit.round)
                    .try_into()
                    .unwrap(),
                block_id: BlockId {
                    hash: tendermint_hash_to_h256(commit.signed_header.commit.block_id.hash),
                    part_set_header: PartSetHeader {
                        total: commit.signed_header.commit.block_id.part_set_header.total,
                        hash: tendermint_hash_to_h256(
                            commit.signed_header.commit.block_id.part_set_header.hash,
                        ),
                    },
                },
                signatures: commit
                    .signed_header
                    .commit
                    .signatures
                    .into_iter()
                    .map(tendermint_commit_sig_to_commit_sig)
                    .collect(),
            },
        }
    }

    fn tendermint_commit_sig_to_commit_sig(sig: tendermint::block::CommitSig) -> CommitSig {
        match sig {
            ::tendermint::block::CommitSig::BlockIdFlagAbsent => CommitSig::Absent,
            ::tendermint::block::CommitSig::BlockIdFlagCommit {
                validator_address,
                timestamp,
                signature,
            } => CommitSig::Commit {
                validator_address: Vec::from(validator_address).try_into().unwrap(),
                timestamp: {
                    let ts = tendermint_proto::google::protobuf::Timestamp::from(timestamp);

                    Timestamp {
                        seconds: ts.seconds.try_into().unwrap(),
                        nanos: ts.nanos.try_into().unwrap(),
                    }
                },
                signature: signature.unwrap().into_bytes().try_into().unwrap(),
            },
            ::tendermint::block::CommitSig::BlockIdFlagNil {
                validator_address,
                timestamp,
                signature,
            } => CommitSig::Nil {
                validator_address: Vec::from(validator_address).try_into().unwrap(),
                timestamp: {
                    let ts = tendermint_proto::google::protobuf::Timestamp::from(timestamp);

                    Timestamp {
                        seconds: ts.seconds.try_into().unwrap(),
                        nanos: ts.nanos.try_into().unwrap(),
                    }
                },
                signature: signature.unwrap().into_bytes().try_into().unwrap(),
            },
        }
    }

    pub fn tendermint_validator_info_to_validator(val: ::tendermint::validator::Info) -> Validator {
        Validator {
            address: val
                .address
                .as_bytes()
                .try_into()
                .expect("value is 20 bytes internally; should not fail; qed"),
            pub_key: match val.pub_key {
                ::tendermint::PublicKey::Ed25519(key) => PublicKey::Ed25519(key.as_bytes().into()),
                ::tendermint::PublicKey::Bn254(key) => PublicKey::Bn254(key.to_vec()),
                _ => todo!(),
            },
            voting_power: BoundedI64::new(val.power.value().try_into().unwrap()).unwrap(),
            proposer_priority: val.proposer_priority.value(),
        }
    }

    fn tendermint_hash_to_h256(hash: tendermint::Hash) -> H256 {
        match hash {
            tendermint::Hash::Sha256(hash) => hash.into(),
            tendermint::Hash::None => panic!("empty hash???"),
        }
    }

    pub fn tendermint_height_to_bounded_i64(
        height: ::tendermint::block::Height,
    ) -> BoundedI64<0, { i64::MAX }> {
        i64::from(height).try_into().unwrap()
    }
}
