use crate::{
    errors::Error,
    eth_types::{
        ExecutionPayloadHeader, LightClientHeader, LightClientUpdate, SyncAggregate, SyncCommittee,
        SYNC_COMMITTEE_SIZE,
    },
};
use ethereum_verifier::{
    capella::{BeaconBlockHeader, NEXT_SYNC_COMMITTEE_INDEX_FLOOR_LOG_2},
    crypto::{BlsPublicKey, BlsSignature},
    primitives::{Bytes32, ExecutionAddress, Hash32, Root},
    ByteList, ByteVector,
};
use ibc::Height;
use protos::ibc::core::client::v1::Height as ProtoHeight;
use protos::union::ibc::lightclients::ethereum::v1::{
    AccountUpdate as ProtoAccountUpdate, BeaconBlockHeader as ProtoBeaconBlockHeader,
    ExecutionPayloadHeader as ProtoExecutionPayloadHeader,
    LightClientHeader as ProtoLightClientHeader, LightClientUpdate as ProtoLightClientUpdate,
    Proof as ProtoProof, SyncAggregate as ProtoSyncAggregate, SyncCommittee as ProtoSyncCommittee,
    TrustedSyncCommittee as ProtoTrustedSyncCommittee,
};
use ssz_rs::{Bitvector, Deserialize, U256};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TrustedSyncCommittee {
    /// height(i.e. execution's block number) of consensus state to trusted sync committee stored at
    pub height: Height,
    /// trusted sync committee
    pub sync_committee: SyncCommittee,
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
                public_keys: value
                    .sync_committee
                    .as_ref()
                    .ok_or(Error::decode(
                        "no `sync_committee` in `RawTrustedSyncCommittee`",
                    ))?
                    .pubkeys
                    .clone()
                    .into_iter()
                    .map(|v| BlsPublicKey::try_from(v.as_slice()))
                    .collect::<Result<Vec<BlsPublicKey>, _>>()
                    .map_err(|_| Error::InvalidPublicKey)?
                    .try_into()
                    .map_err(|_| Error::InvalidPublicKey)?,
                aggregate_public_key: BlsPublicKey::try_from(
                    value
                        .sync_committee
                        .ok_or(Error::decode(
                            "no `sync_committee` in `RawTrustedSyncCommittee`",
                        ))?
                        .aggregate_pubkey
                        .as_slice(),
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
                    .public_keys
                    .iter()
                    .map(|pk| pk.to_vec())
                    .collect(),
                aggregate_pubkey: value.sync_committee.aggregate_public_key.to_vec(),
            }),
            is_next: value.is_next,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Fraction {
    pub numerator: u64,
    pub denominator: u64,
}

impl Fraction {
    pub fn new(numerator: u64, denominator: u64) -> Self {
        Self {
            numerator,
            denominator,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AccountProof {
    pub address: Vec<u8>,
    pub storage_hash: Vec<u8>,
    pub proof: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AccountUpdateInfo {
    pub proofs: Vec<AccountProof>,
}

impl From<AccountUpdateInfo> for ProtoAccountUpdate {
    fn from(value: AccountUpdateInfo) -> Self {
        Self {
            proof: value.proofs.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<AccountProof> for ProtoProof {
    fn from(value: AccountProof) -> Self {
        Self {
            key: value.address,
            value: value.storage_hash,
            proof: value.proof,
        }
    }
}

impl TryFrom<ProtoProof> for AccountProof {
    type Error = Error;

    fn try_from(value: ProtoProof) -> Result<Self, Self::Error> {
        Ok(Self {
            address: value.key,
            storage_hash: value.value,
            proof: value.proof,
        })
    }
}

impl TryFrom<ProtoAccountUpdate> for AccountUpdateInfo {
    type Error = Error;
    fn try_from(value: ProtoAccountUpdate) -> Result<Self, Self::Error> {
        Ok(Self {
            proofs: value
                .proof
                .into_iter()
                .map(TryInto::<AccountProof>::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

pub(crate) fn convert_proto_to_beacon_header(
    header: &ProtoBeaconBlockHeader,
) -> Result<BeaconBlockHeader, Error> {
    Ok(BeaconBlockHeader {
        slot: header.slot,
        proposer_index: header
            .proposer_index
            .try_into()
            .map_err(|_| Error::decode("`proposer_index` overflow"))?,
        parent_root: Root::try_from(header.parent_root.as_slice())
            .map_err(|_| Error::decode("`parent_root` must be 32 bytes long"))?,
        state_root: Root::try_from(header.state_root.as_slice())
            .map_err(|_| Error::decode("`state_root` must be 32 bytes long"))?,
        body_root: Root::try_from(header.body_root.as_slice())
            .map_err(|_| Error::decode("`body_root` must be 32 bytes long"))?,
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

        execution: ExecutionPayloadHeader {
            parent_hash: Hash32::try_from(execution.parent_hash.as_slice())
                .map_err(|_| Error::decode("`parent_hash` must be 32 bytes long"))?,
            fee_recipient: ExecutionAddress::try_from(execution.fee_recipient.as_slice()).map_err(
                |_| Error::decode("cannot parse `fee_recipient` in `RawLightClientHeader`"),
            )?,
            state_root: Bytes32::try_from(execution.state_root.as_slice()).map_err(|_| {
                Error::decode("cannot parse `state_root` in `RawLightClientHeader`")
            })?,
            receipts_root: Bytes32::try_from(execution.receipts_root.as_slice()).map_err(|_| {
                Error::decode("cannot parse `receipts_root` in `RawLightClientHeader`")
            })?,
            logs_bloom: ByteVector::try_from(execution.logs_bloom.as_slice()).map_err(|_| {
                Error::decode("cannot parse `logs_bloom` in `RawLightClientHeader`")
            })?,
            prev_randao: Hash32::try_from(execution.prev_randao.as_slice())
                .map_err(|_| Error::decode("`prev_randao` must be 32 bytes long"))?,
            block_number: execution.block_number,
            gas_limit: execution.gas_limit,
            gas_used: execution.gas_used,
            timestamp: execution.timestamp,
            extra_data: ByteList::try_from(execution.extra_data.as_slice()).map_err(|_| {
                Error::decode("cannot parse `extra_data` in `RawLightClientHeader`")
            })?,
            base_fee_per_gas: U256::try_from_bytes_le(execution.base_fee_per_gas.as_slice())
                .map_err(|_| {
                    Error::decode("cannot parse `base_fee_per_gas` in `RawLightClientHeader`")
                })?,
            block_hash: Hash32::try_from(execution.block_hash.as_slice())
                .map_err(|_| Error::decode("`block_hash` must be 32 bytes long"))?,
            transactions_root: Root::try_from(execution.transactions_root.as_slice())
                .map_err(|_| Error::decode("`transactions_root` must be 32 bytes long"))?,
            withdrawals_root: Root::try_from(execution.withdrawals_root.as_slice())
                .map_err(|_| Error::decode("`withdrawals_root` must be 32 bytes long"))?,
        },
        execution_branch: header
            .execution_branch
            .iter()
            .map(|h| Hash32::try_from(h.as_slice()))
            .collect::<Result<Vec<Hash32>, _>>()
            .map_err(|_| {
                Error::decode("cannot parse `execution_branch` in `RawLightClientHeader`")
            })?
            .try_into()
            .map_err(|_| {
                Error::decode("cannot parse `execution_branch` in `RawLightClientHeader`")
            })?,
    })
}

pub(crate) fn convert_beacon_header_to_proto(header: &BeaconBlockHeader) -> ProtoBeaconBlockHeader {
    ProtoBeaconBlockHeader {
        slot: header.slot,
        proposer_index: header
            .proposer_index
            .try_into()
            .expect("`proposer_index` cannot be larger than u64"),
        parent_root: header.parent_root.as_ref().into(),
        state_root: header.state_root.as_ref().into(),
        body_root: header.body_root.as_ref().into(),
    }
}

pub(crate) fn convert_header_to_proto(header: &LightClientHeader) -> ProtoLightClientHeader {
    ProtoLightClientHeader {
        beacon: Some(convert_beacon_header_to_proto(&header.beacon)),
        execution: Some(ProtoExecutionPayloadHeader {
            parent_hash: header.execution.parent_hash.as_ref().into(),
            fee_recipient: header.execution.fee_recipient.as_ref().into(),
            state_root: header.execution.state_root.as_ref().into(),
            receipts_root: header.execution.receipts_root.as_ref().into(),
            logs_bloom: header.execution.logs_bloom.as_ref().into(),
            prev_randao: header.execution.prev_randao.as_ref().into(),
            block_number: header.execution.block_number,
            gas_limit: header.execution.gas_limit,
            gas_used: header.execution.gas_used,
            timestamp: header.execution.timestamp,
            extra_data: header.execution.extra_data.as_ref().into(),
            base_fee_per_gas: header.execution.base_fee_per_gas.to_bytes_le(),
            block_hash: header.execution.block_hash.as_ref().into(),
            transactions_root: header.execution.transactions_root.as_ref().into(),
            withdrawals_root: header.execution.transactions_root.as_ref().into(),
        }),
        execution_branch: header
            .execution_branch
            .iter()
            .map(|h| h.as_ref().into())
            .collect(),
    }
}

pub(crate) fn convert_sync_aggregate_to_proto(sync_aggregate: SyncAggregate) -> ProtoSyncAggregate {
    ProtoSyncAggregate {
        sync_committee_bits: sync_aggregate
            .sync_committee_bits
            .iter()
            .map(|b| if b == true { 1 } else { 0 })
            .collect(),
        sync_committee_signature: sync_aggregate.sync_committee_signature.to_vec(),
    }
}

pub(crate) fn convert_proto_sync_aggregate(
    sync_aggregate: ProtoSyncAggregate,
) -> Result<SyncAggregate, Error> {
    Ok(SyncAggregate {
        sync_committee_bits: Bitvector::<SYNC_COMMITTEE_SIZE>::deserialize(
            sync_aggregate.sync_committee_bits.as_slice(),
        )
        .map_err(|_| Error::decode("cannot parse `sync_committee_bits` in `RawSyncAggregate`"))?,
        sync_committee_signature: BlsSignature::try_from(
            sync_aggregate.sync_committee_signature.as_slice(),
        )
        .map_err(|_| {
            Error::decode("cannot parse `sync_committee_signature` in `RawSyncAggregate`")
        })?,
    })
}

pub(crate) fn convert_consensus_update_to_proto(
    consensus_update: LightClientUpdate,
) -> ProtoLightClientUpdate {
    let sync_aggregate = consensus_update.sync_aggregate.clone();
    let signature_slot = consensus_update.signature_slot;
    let light_client_update = consensus_update;

    ProtoLightClientUpdate {
        attested_header: Some(convert_header_to_proto(
            &light_client_update.attested_header,
        )),
        next_sync_committee: light_client_update.next_sync_committee.clone().map(|c| {
            ProtoSyncCommittee {
                pubkeys: c.public_keys.iter().map(|pk| pk.to_vec()).collect(),
                aggregate_pubkey: c.aggregate_public_key.to_vec(),
            }
        }),
        next_sync_committee_branch: light_client_update
            .next_sync_committee_branch
            .iter()
            .map(|n| n.as_ref().into())
            .collect(),
        finalized_header: Some(convert_header_to_proto(
            &light_client_update.finalized_header,
        )),
        finality_branch: light_client_update
            .finality_branch
            .iter()
            .map(|h| h.as_ref().into())
            .collect(),
        sync_aggregate: Some(convert_sync_aggregate_to_proto(sync_aggregate)),
        signature_slot,
    }
}

pub(crate) fn convert_proto_to_consensus_update(
    consensus_update: ProtoLightClientUpdate,
) -> Result<LightClientUpdate, Error> {
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
            Some(SyncCommittee {
                public_keys: consensus_update
                    .next_sync_committee
                    .clone()
                    .ok_or(Error::decode(
                        "no `next_sync_committee` in consensus update",
                    ))?
                    .pubkeys
                    .into_iter()
                    .map(|v| BlsPublicKey::try_from(v.as_slice()))
                    .collect::<Result<Vec<BlsPublicKey>, _>>()
                    .map_err(|_| Error::InvalidPublicKey)?
                    .try_into()
                    .map_err(|_| Error::InvalidPublicKey)?,
                aggregate_public_key: BlsPublicKey::try_from(
                    consensus_update
                        .next_sync_committee
                        .ok_or(Error::decode(
                            "no `next_sync_committee` in consensus update",
                        ))?
                        .aggregate_pubkey
                        .as_slice(),
                )
                .map_err(|_| Error::InvalidPublicKey)?,
            })
        },
        next_sync_committee_branch: if consensus_update.next_sync_committee_branch.is_empty() {
            Default::default()
        } else {
            decode_branch::<_, NEXT_SYNC_COMMITTEE_INDEX_FLOOR_LOG_2>(
                consensus_update.next_sync_committee_branch.into_iter(),
            )?
            .to_vec()
            .try_into()
            .map_err(|_| Error::decode("branch items must be 32 bytes"))?
        },
        finalized_header,
        finality_branch: consensus_update
            .finality_branch
            .iter()
            .map(|h| Hash32::try_from(h.as_slice()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| Error::decode("branch items must be 32 bytes"))?
            .try_into()
            .map_err(|_| Error::decode("cannot parse `finality_branch` in consensus update"))?,
        sync_aggregate: convert_proto_sync_aggregate(
            consensus_update
                .sync_aggregate
                .ok_or(Error::decode("no `sync_aggregate` in consensus update"))?,
        )?,
        signature_slot: consensus_update.signature_slot,
    };

    Ok(light_client_update)
}

pub(crate) fn decode_branch<B: Iterator<Item = Vec<u8>>, const N: usize>(
    bz: B,
) -> Result<[Hash32; N], Error>
where
    [Hash32; N]: Default,
{
    let mut array: [Hash32; N] = Default::default();
    let v: Vec<Hash32> = bz
        .into_iter()
        .map(|b| Hash32::try_from(b.as_slice()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| Error::decode("branch items must be 32 bytes"))?;
    array.clone_from_slice(v.as_slice());
    Ok(array)
}
