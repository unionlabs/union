use crate::{
    commitment::decode_eip1184_rlp_proof,
    errors::Error,
    update::{new_consensus_update, ConsensusUpdateInfo, ExecutionUpdateInfo, LightClientUpdate},
};
use ethereum_consensus::{
    beacon::BeaconBlockHeader,
    bls::{PublicKey, Signature},
    sync_protocol::{SyncAggregate, SyncCommittee},
    types::H256,
};

use ethereum_light_client_verifier::updates::ConsensusUpdate;
use ibc::Height;
use ibc_proto::ibc::{
    core::client::v1::Height as ProtoHeight,
    lightclients::ethereum::v1::{
        AccountUpdate as ProtoAccountUpdate, BeaconBlockHeader as ProtoBeaconBlockHeader,
        ExecutionUpdate as ProtoExecutionUpdate, LightClientUpdate as ProtoLightClientUpdate,
        SyncAggregate as ProtoSyncAggregate, SyncCommittee as ProtoSyncCommittee,
        TrustedSyncCommittee as ProtoTrustedSyncCommittee,
    },
};
use ssz_rs::{Bitvector, Deserialize, Vector};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TrustedSyncCommittee<const SYNC_COMMITTEE_SIZE: usize> {
    /// height(i.e. execution's block number) of consensus state to trusted sync committee stored at
    pub height: Height,
    /// trusted sync committee
    pub sync_committee: SyncCommittee<SYNC_COMMITTEE_SIZE>,
    /// since the consensus state contains a current and next sync committee, this flag determines which one to refer to
    pub is_next: bool,
}

impl<const SYNC_COMMITTEE_SIZE: usize> TryFrom<ProtoTrustedSyncCommittee>
    for TrustedSyncCommittee<SYNC_COMMITTEE_SIZE>
{
    type Error = Error;

    fn try_from(value: ProtoTrustedSyncCommittee) -> Result<Self, Error> {
        Ok(TrustedSyncCommittee {
            height: Height::new(
                value
                    .trusted_height
                    .as_ref()
                    .ok_or(Error::DecodeError)?
                    .revision_number,
                value
                    .trusted_height
                    .as_ref()
                    .ok_or(Error::DecodeError)?
                    .revision_height,
            )
            .map_err(|_| Error::InvalidHeight)?,
            sync_committee: SyncCommittee {
                pubkeys: Vector::<PublicKey, SYNC_COMMITTEE_SIZE>::from_iter(
                    value
                        .sync_committee
                        .as_ref()
                        .ok_or(Error::DecodeError)?
                        .pubkeys
                        .clone()
                        .into_iter()
                        .map(|pk| PublicKey::try_from(pk))
                        .collect::<Result<Vec<PublicKey>, _>>()
                        .map_err(|_| Error::InvalidPublicKey)?,
                ),
                aggregate_pubkey: PublicKey::try_from(
                    value
                        .sync_committee
                        .ok_or(Error::DecodeError)?
                        .aggregate_pubkey,
                )
                .map_err(|_| Error::InvalidPublicKey)?,
            },
            is_next: value.is_next,
        })
    }
}

impl<const SYNC_COMMITTEE_SIZE: usize> From<TrustedSyncCommittee<SYNC_COMMITTEE_SIZE>>
    for ProtoTrustedSyncCommittee
{
    fn from(value: TrustedSyncCommittee<SYNC_COMMITTEE_SIZE>) -> Self {
        Self {
            trusted_height: Some(ProtoHeight {
                revision_number: value.height.revision_number(),
                revision_height: value.height.revision_height(),
            }),
            sync_committee: Some(ProtoSyncCommittee {
                pubkeys: value
                    .sync_committee
                    .pubkeys
                    .iter()
                    .map(|pk| pk.to_vec())
                    .collect(),
                aggregate_pubkey: value.sync_committee.aggregate_pubkey.to_vec(),
            }),
            is_next: value.is_next,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AccountUpdateInfo {
    pub account_proof: Vec<Vec<u8>>,
    pub account_storage_root: H256,
}

impl From<AccountUpdateInfo> for ProtoAccountUpdate {
    fn from(value: AccountUpdateInfo) -> Self {
        Self {
            account_proof: encode_account_proof(value.account_proof),
            account_storage_root: value.account_storage_root.as_bytes().to_vec(),
        }
    }
}

impl TryFrom<ProtoAccountUpdate> for AccountUpdateInfo {
    type Error = Error;
    fn try_from(value: ProtoAccountUpdate) -> Result<Self, Self::Error> {
        Ok(Self {
            account_proof: decode_eip1184_rlp_proof(value.account_proof)?,
            account_storage_root: H256::from_slice(&value.account_storage_root),
        })
    }
}

fn encode_account_proof(bz: Vec<Vec<u8>>) -> Vec<u8> {
    let proof: Vec<Vec<u8>> = bz.into_iter().map(|b| b.to_vec()).collect();
    let mut stream = rlp::RlpStream::new();
    stream.begin_list(proof.len());
    for p in proof.iter() {
        stream.append_raw(p, 1);
    }
    stream.out().freeze().into()
}

pub(crate) fn convert_proto_to_header(
    header: &ProtoBeaconBlockHeader,
) -> Result<BeaconBlockHeader, Error> {
    Ok(BeaconBlockHeader {
        slot: header.slot.into(),
        proposer_index: header.proposer_index.into(),
        parent_root: H256::from_slice(&header.parent_root),
        state_root: H256::from_slice(&header.state_root),
        body_root: H256::from_slice(&header.body_root),
    })
}

pub(crate) fn convert_header_to_proto(header: &BeaconBlockHeader) -> ProtoBeaconBlockHeader {
    ProtoBeaconBlockHeader {
        slot: header.slot.into(),
        proposer_index: header.proposer_index.into(),
        parent_root: header.parent_root.as_bytes().to_vec(),
        state_root: header.state_root.as_bytes().to_vec(),
        body_root: header.body_root.as_bytes().to_vec(),
    }
}

pub(crate) fn convert_proto_to_execution_update(
    execution_update: ProtoExecutionUpdate,
) -> ExecutionUpdateInfo {
    ExecutionUpdateInfo {
        state_root: H256::from_slice(&execution_update.state_root),
        state_root_branch: execution_update
            .state_root_branch
            .into_iter()
            .map(|n| H256::from_slice(&n))
            .collect(),
        block_number: execution_update.block_number.into(),
        block_number_branch: execution_update
            .block_number_branch
            .into_iter()
            .map(|n| H256::from_slice(&n))
            .collect(),
    }
}

pub(crate) fn convert_execution_update_to_proto(
    execution_update: ExecutionUpdateInfo,
) -> ProtoExecutionUpdate {
    ProtoExecutionUpdate {
        state_root: execution_update.state_root.as_bytes().into(),
        state_root_branch: execution_update
            .state_root_branch
            .into_iter()
            .map(|n| n.as_bytes().to_vec())
            .collect(),
        block_number: execution_update.block_number.into(),
        block_number_branch: execution_update
            .block_number_branch
            .into_iter()
            .map(|n| n.as_bytes().to_vec())
            .collect(),
    }
}

pub(crate) fn convert_sync_aggregate_to_proto<const SYNC_COMMITTEE_SIZE: usize>(
    sync_aggregate: SyncAggregate<SYNC_COMMITTEE_SIZE>,
) -> ProtoSyncAggregate {
    ProtoSyncAggregate {
        sync_committee_bits: sync_aggregate
            .sync_committee_bits
            .iter()
            .map(|b| if b == true { 1 } else { 0 })
            .collect(),
        sync_committee_signature: sync_aggregate.sync_committee_signature.0.to_vec(),
    }
}

pub(crate) fn convert_proto_sync_aggregate<const SYNC_COMMITTEE_SIZE: usize>(
    sync_aggregate: ProtoSyncAggregate,
) -> Result<SyncAggregate<SYNC_COMMITTEE_SIZE>, Error> {
    Ok(SyncAggregate {
        sync_committee_bits: Bitvector::<SYNC_COMMITTEE_SIZE>::deserialize(
            sync_aggregate.sync_committee_bits.as_slice(),
        )
        .map_err(|_| Error::DecodeError)?,
        sync_committee_signature: Signature::try_from(sync_aggregate.sync_committee_signature)
            .map_err(|_| Error::DecodeError)?,
    })
}

pub(crate) fn convert_consensus_update_to_proto<const SYNC_COMMITTEE_SIZE: usize>(
    consensus_update: ConsensusUpdateInfo<SYNC_COMMITTEE_SIZE>,
) -> ProtoLightClientUpdate {
    let finalized_beacon_header_branch = consensus_update.finalized_beacon_header_branch().clone();
    let sync_aggregate = consensus_update.light_client_update.sync_aggregate.clone();
    let signature_slot = consensus_update.signature_slot();
    let light_client_update = consensus_update.light_client_update;

    ProtoLightClientUpdate {
        attested_header: Some(convert_header_to_proto(
            &light_client_update.attested_header,
        )),
        next_sync_committee: light_client_update.next_sync_committee.clone().map(|c| {
            ProtoSyncCommittee {
                pubkeys: c.0.pubkeys.iter().map(|pk| pk.to_vec()).collect(),
                aggregate_pubkey: c.0.aggregate_pubkey.to_vec(),
            }
        }),
        next_sync_committee_branch: light_client_update
            .next_sync_committee
            .map_or(Vec::new(), |(_, branch)| {
                branch.into_iter().map(|n| n.as_bytes().to_vec()).collect()
            }),
        finalized_header: Some(convert_header_to_proto(
            &light_client_update.finalized_header.0,
        )),
        finalized_header_branch: finalized_beacon_header_branch
            .into_iter()
            .map(|n| n.as_bytes().to_vec())
            .collect(),
        finalized_execution_root: consensus_update.finalized_execution_root.as_bytes().into(),
        finalized_execution_branch: consensus_update
            .finalized_execution_branch
            .into_iter()
            .map(|n| n.as_bytes().to_vec())
            .collect(),
        sync_aggregate: Some(convert_sync_aggregate_to_proto(sync_aggregate)),
        signature_slot: signature_slot.into(),
    }
}

pub(crate) fn convert_proto_to_consensus_update<const SYNC_COMMITTEE_SIZE: usize>(
    consensus_update: ProtoLightClientUpdate,
) -> Result<ConsensusUpdateInfo<SYNC_COMMITTEE_SIZE>, Error> {
    let attested_header = convert_proto_to_header(
        consensus_update
            .attested_header
            .as_ref()
            .ok_or(Error::DecodeError)?,
    )?;
    let finalized_header = convert_proto_to_header(
        consensus_update
            .finalized_header
            .as_ref()
            .ok_or(Error::DecodeError)?,
    )?;

    let light_client_update = LightClientUpdate {
        attested_header,
        next_sync_committee: if consensus_update.next_sync_committee.is_none()
            || consensus_update
                .next_sync_committee
                .as_ref()
                .ok_or(Error::DecodeError)?
                .pubkeys
                .len()
                == 0
            || consensus_update.next_sync_committee_branch.len() == 0
        {
            None
        } else {
            Some((
                SyncCommittee {
                    pubkeys: Vector::<PublicKey, SYNC_COMMITTEE_SIZE>::from_iter(
                        consensus_update
                            .next_sync_committee
                            .clone()
                            .ok_or(Error::DecodeError)?
                            .pubkeys
                            .into_iter()
                            .map(|pk| PublicKey::try_from(pk))
                            .collect::<Result<Vec<PublicKey>, _>>()
                            .map_err(|_| Error::InvalidPublicKey)?,
                    ),
                    aggregate_pubkey: PublicKey::try_from(
                        consensus_update
                            .next_sync_committee
                            .ok_or(Error::DecodeError)?
                            .aggregate_pubkey,
                    )
                    .map_err(|_| Error::InvalidPublicKey)?,
                },
                decode_branch(consensus_update.next_sync_committee_branch),
            ))
        },
        finalized_header: (
            finalized_header,
            decode_branch(consensus_update.finalized_header_branch),
        ),
        sync_aggregate: convert_proto_sync_aggregate(
            consensus_update.sync_aggregate.ok_or(Error::DecodeError)?,
        )?,
        signature_slot: consensus_update.signature_slot.into(),
    };

    Ok(new_consensus_update(
        light_client_update,
        H256::from_slice(&consensus_update.finalized_execution_root),
        consensus_update
            .finalized_execution_branch
            .into_iter()
            .map(|n| H256::from_slice(&n))
            .collect(),
    ))
}

pub(crate) fn decode_branch<const N: usize>(bz: Vec<Vec<u8>>) -> [H256; N]
where
    [H256; N]: Default,
{
    let mut array: [H256; N] = Default::default();
    let v: Vec<H256> = bz.into_iter().map(|b| H256::from_slice(&b)).collect();
    array.clone_from_slice(v.as_slice());
    array
}
