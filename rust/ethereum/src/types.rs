use crate::{
    commitment::decode_eip1184_rlp_proof,
    errors::Error,
    eth_types::{
        CapellaExecutionPayloadHeader, ConsensusUpdateInfo, LightClientHeader, LightClientUpdate,
        SYNC_COMMITTEE_SIZE,
    },
};
use ethereum_consensus::{
    beacon::BeaconBlockHeader,
    bls::{PublicKey, Signature},
    sync_protocol::{SyncAggregate, SyncCommittee},
    types::{Address, ByteList, ByteVector, H256, U256},
};
use ethereum_light_client_verifier::updates::{
    capella::ConsensusUpdateInfo as CapellaConsensusUpdateInfo, ConsensusUpdate,
};
use ibc::Height;
use ibc_proto::ibc::{
    core::client::v1::Height as ProtoHeight,
    lightclients::ethereum::v1::{
        AccountUpdate as ProtoAccountUpdate, BeaconBlockHeader as ProtoBeaconBlockHeader,
        ExecutionPayloadHeader as ProtoExecutionPayloadHeader,
        LightClientHeader as ProtoLightClientHeader, LightClientUpdate as ProtoLightClientUpdate,
        SyncAggregate as ProtoSyncAggregate, SyncCommittee as ProtoSyncCommittee,
        TrustedSyncCommittee as ProtoTrustedSyncCommittee,
    },
};
use ssz_rs::{Bitvector, Deserialize, Vector};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TrustedSyncCommittee {
    /// height(i.e. execution's block number) of consensus state to trusted sync committee stored at
    pub height: Height,
    /// trusted sync committee
    pub sync_committee: SyncCommittee<SYNC_COMMITTEE_SIZE>,
    /// since the consensus state contains a current and next sync committee, this flag determines which one to refer to
    pub is_next: bool,
}

impl TryFrom<ProtoTrustedSyncCommittee> for TrustedSyncCommittee {
    type Error = Error;

    fn try_from(value: ProtoTrustedSyncCommittee) -> Result<Self, Error> {
        Ok(TrustedSyncCommittee {
            height: Height::new(
                value
                    .trusted_height
                    .as_ref()
                    .ok_or(Error::decode(
                        "no `trusted_height` in `RawTrustedSyncCommittee`",
                    ))?
                    .revision_number,
                value
                    .trusted_height
                    .as_ref()
                    .ok_or(Error::decode(
                        "no `trusted_height` in `RawTrustedSyncCommittee`",
                    ))?
                    .revision_height,
            )
            .map_err(|_| Error::InvalidHeight)?,
            sync_committee: SyncCommittee {
                pubkeys: Vector::<PublicKey, SYNC_COMMITTEE_SIZE>::from_iter(
                    value
                        .sync_committee
                        .as_ref()
                        .ok_or(Error::decode(
                            "no `sync_committee` in `RawTrustedSyncCommittee`",
                        ))?
                        .pubkeys
                        .clone()
                        .into_iter()
                        .map(PublicKey::try_from)
                        .collect::<Result<Vec<PublicKey>, _>>()
                        .map_err(|_| Error::InvalidPublicKey)?,
                ),
                aggregate_pubkey: PublicKey::try_from(
                    value
                        .sync_committee
                        .ok_or(Error::decode(
                            "no `sync_committee` in `RawTrustedSyncCommittee`",
                        ))?
                        .aggregate_pubkey,
                )
                .map_err(|_| Error::InvalidPublicKey)?,
            },
            is_next: value.is_next,
        })
    }
}

impl From<TrustedSyncCommittee> for ProtoTrustedSyncCommittee {
    fn from(value: TrustedSyncCommittee) -> Self {
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

pub(crate) fn convert_proto_to_beacon_header(
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

pub(crate) fn convert_proto_to_header(
    header: &ProtoLightClientHeader,
) -> Result<LightClientHeader, Error> {
    let execution = header
        .execution
        .clone()
        .ok_or(Error::decode("no `execution` in `RawLightClientHeader`"))?;

    Ok(LightClientHeader {
        beacon: convert_proto_to_beacon_header(
            header
                .beacon
                .as_ref()
                .ok_or(Error::decode("no `beacon` in `RawLightClientHeader`"))?,
        )?,
        execution: CapellaExecutionPayloadHeader {
            parent_hash: H256::from_slice(&execution.parent_hash),
            fee_recipient: Address::try_from(execution.fee_recipient.as_slice()).map_err(|_| {
                Error::decode("cannot parse `fee_recipient` in `RawLightClientHeader`")
            })?,
            state_root: H256::from_slice(&execution.state_root),
            receipts_root: H256::from_slice(&execution.receipts_root),
            logs_bloom: ByteVector::try_from(execution.logs_bloom.as_slice()).map_err(|_| {
                Error::decode("cannot parse `logs_bloom` in `RawLightClientHeader`")
            })?,
            prev_randao: H256::from_slice(&execution.prev_randao),
            block_number: execution.block_number.into(),
            gas_limit: execution.gas_limit.into(),
            gas_used: execution.gas_used.into(),
            timestamp: execution.timestamp.into(),
            extra_data: ByteList::try_from(execution.extra_data.as_slice()).map_err(|_| {
                Error::decode("cannot parse `extra_data` in `RawLightClientHeader`")
            })?,
            base_fee_per_gas: U256::try_from_bytes_le(execution.base_fee_per_gas.as_slice())
                .map_err(|_| {
                    Error::decode("cannot parse `base_fee_per_gas` in `RawLightClientHeader`")
                })?,
            block_hash: H256::from_slice(&execution.block_hash),
            transactions_root: H256::from_slice(&execution.transactions_root),
            withdrawals_root: H256::from_slice(&execution.withdrawals_root),
        },
        execution_branch: header
            .execution_branch
            .iter()
            .map(|h| H256::from_slice(h))
            .collect::<Vec<H256>>()
            .try_into()
            .map_err(|_| {
                Error::decode("cannot parse `execution_branch` in `RawLightClientHeader`")
            })?,
    })
}

pub(crate) fn convert_beacon_header_to_proto(header: &BeaconBlockHeader) -> ProtoBeaconBlockHeader {
    ProtoBeaconBlockHeader {
        slot: header.slot.into(),
        proposer_index: header.proposer_index.into(),
        parent_root: header.parent_root.as_bytes().to_vec(),
        state_root: header.state_root.as_bytes().to_vec(),
        body_root: header.body_root.as_bytes().to_vec(),
    }
}

pub(crate) fn convert_header_to_proto(header: &LightClientHeader) -> ProtoLightClientHeader {
    ProtoLightClientHeader {
        beacon: Some(convert_beacon_header_to_proto(&header.beacon)),
        execution: Some(ProtoExecutionPayloadHeader {
            parent_hash: header.execution.parent_hash.as_bytes().into(),
            fee_recipient: header.execution.fee_recipient.0.into(),
            state_root: header.execution.state_root.as_bytes().into(),
            receipts_root: header.execution.receipts_root.as_bytes().into(),
            logs_bloom: header.execution.logs_bloom.as_ref().into(),
            prev_randao: header.execution.prev_randao.as_bytes().into(),
            block_number: header.execution.block_number.into(),
            gas_limit: header.execution.gas_limit.into(),
            gas_used: header.execution.gas_used.into(),
            timestamp: header.execution.timestamp.into(),
            extra_data: header.execution.extra_data.as_ref().into(),
            base_fee_per_gas: header.execution.base_fee_per_gas.to_bytes_le(),
            block_hash: header.execution.block_hash.as_bytes().into(),
            transactions_root: header.execution.transactions_root.as_bytes().into(),
            withdrawals_root: header.execution.transactions_root.as_bytes().into(),
        }),
        execution_branch: header
            .execution_branch
            .iter()
            .map(|h| h.as_bytes().into())
            .collect(),
    }
}

pub(crate) fn convert_sync_aggregate_to_proto(
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

pub(crate) fn convert_proto_sync_aggregate(
    sync_aggregate: ProtoSyncAggregate,
) -> Result<SyncAggregate<SYNC_COMMITTEE_SIZE>, Error> {
    Ok(SyncAggregate {
        sync_committee_bits: Bitvector::<SYNC_COMMITTEE_SIZE>::deserialize(
            sync_aggregate.sync_committee_bits.as_slice(),
        )
        .map_err(|_| Error::decode("cannot parse `sync_committee_bits` in `RawSyncAggregate`"))?,
        sync_committee_signature: Signature::try_from(sync_aggregate.sync_committee_signature)
            .map_err(|_| {
                Error::decode("cannot parse `sync_committee_signature` in `RawSyncAggregate`")
            })?,
    })
}

pub(crate) fn convert_consensus_update_to_proto(
    consensus_update: ConsensusUpdateInfo,
) -> ProtoLightClientUpdate {
    let sync_aggregate = consensus_update.0.sync_aggregate.clone();
    let signature_slot = consensus_update.signature_slot();
    let light_client_update = consensus_update.0;

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
            &light_client_update.finalized_header,
        )),
        finality_branch: light_client_update
            .finality_branch
            .iter()
            .map(|h| h.as_bytes().into())
            .collect(),
        sync_aggregate: Some(convert_sync_aggregate_to_proto(sync_aggregate)),
        signature_slot: signature_slot.into(),
    }
}

pub(crate) fn convert_proto_to_consensus_update(
    consensus_update: ProtoLightClientUpdate,
) -> Result<ConsensusUpdateInfo, Error> {
    let attested_header = convert_proto_to_header(
        consensus_update
            .attested_header
            .as_ref()
            .ok_or(Error::decode("no `attested_header` in consensus update"))?,
    )?;
    let finalized_header = convert_proto_to_header(
        consensus_update
            .finalized_header
            .as_ref()
            .ok_or(Error::decode("no `finalized_header` in consensus update"))?,
    )?;

    let light_client_update = LightClientUpdate {
        attested_header,
        next_sync_committee: if consensus_update.next_sync_committee.is_none()
            || consensus_update
                .next_sync_committee
                .as_ref()
                .ok_or(Error::decode(
                    "no `next_sync_committee` in consensus update",
                ))?
                .pubkeys
                .is_empty()
            || consensus_update.next_sync_committee_branch.is_empty()
        {
            None
        } else {
            Some((
                SyncCommittee {
                    pubkeys: Vector::<PublicKey, SYNC_COMMITTEE_SIZE>::from_iter(
                        consensus_update
                            .next_sync_committee
                            .clone()
                            .ok_or(Error::decode(
                                "no `next_sync_committee` in consensus update",
                            ))?
                            .pubkeys
                            .into_iter()
                            .map(PublicKey::try_from)
                            .collect::<Result<Vec<PublicKey>, _>>()
                            .map_err(|_| Error::InvalidPublicKey)?,
                    ),
                    aggregate_pubkey: PublicKey::try_from(
                        consensus_update
                            .next_sync_committee
                            .ok_or(Error::decode(
                                "no `next_sync_committee` in consensus update",
                            ))?
                            .aggregate_pubkey,
                    )
                    .map_err(|_| Error::InvalidPublicKey)?,
                },
                decode_branch(consensus_update.next_sync_committee_branch),
            ))
        },
        finalized_header,
        finality_branch: consensus_update
            .finality_branch
            .iter()
            .map(|h| H256::from_slice(h))
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| Error::decode("cannot parse `finality_branch` in consensus update"))?,
        sync_aggregate: convert_proto_sync_aggregate(
            consensus_update
                .sync_aggregate
                .ok_or(Error::decode("no `sync_aggregate` in consensus update"))?,
        )?,
        signature_slot: consensus_update.signature_slot.into(),
    };

    Ok(CapellaConsensusUpdateInfo(light_client_update))
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
