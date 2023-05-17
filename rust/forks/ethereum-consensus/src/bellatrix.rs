use crate::{
    beacon::{
        Attestation, AttesterSlashing, BeaconBlockHeader, Deposit, Eth1Data, ProposerSlashing,
        Root, SignedVoluntaryExit, Slot, ValidatorIndex,
    },
    bls::Signature,
    compute::hash_tree_root,
    errors::Error,
    execution::BlockNumber,
    internal_prelude::*,
    sync_protocol::{
        SyncAggregate, SyncCommittee, CURRENT_SYNC_COMMITTEE_DEPTH, FINALIZED_ROOT_DEPTH,
        NEXT_SYNC_COMMITTEE_DEPTH,
    },
    types::{Address, ByteList, ByteVector, Bytes32, H256, U256, U64},
};
use ssz_rs::{Deserialize, List, Merkleized, Sized};
use ssz_rs_derive::SimpleSerialize;

/// Beacon Block
/// https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#beaconblock
#[derive(
    Clone, Debug, PartialEq, Eq, Default, SimpleSerialize, serde::Serialize, serde::Deserialize,
)]
pub struct BeaconBlock<
    const MAX_PROPOSER_SLASHINGS: usize,
    const MAX_VALIDATORS_PER_COMMITTEE: usize,
    const MAX_ATTESTER_SLASHINGS: usize,
    const MAX_ATTESTATIONS: usize,
    const DEPOSIT_CONTRACT_TREE_DEPTH: usize,
    const MAX_DEPOSITS: usize,
    const MAX_VOLUNTARY_EXITS: usize,
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
    const MAX_BYTES_PER_TRANSACTION: usize,
    const MAX_TRANSACTIONS_PER_PAYLOAD: usize,
    const SYNC_COMMITTEE_SIZE: usize,
> {
    pub slot: Slot,
    pub proposer_index: ValidatorIndex,
    pub parent_root: Root,
    pub state_root: Root,
    pub body: BeaconBlockBody<
        MAX_PROPOSER_SLASHINGS,
        MAX_VALIDATORS_PER_COMMITTEE,
        MAX_ATTESTER_SLASHINGS,
        MAX_ATTESTATIONS,
        DEPOSIT_CONTRACT_TREE_DEPTH,
        MAX_DEPOSITS,
        MAX_VOLUNTARY_EXITS,
        BYTES_PER_LOGS_BLOOM,
        MAX_EXTRA_DATA_BYTES,
        MAX_BYTES_PER_TRANSACTION,
        MAX_TRANSACTIONS_PER_PAYLOAD,
        SYNC_COMMITTEE_SIZE,
    >,
}

impl<
        const MAX_PROPOSER_SLASHINGS: usize,
        const MAX_VALIDATORS_PER_COMMITTEE: usize,
        const MAX_ATTESTER_SLASHINGS: usize,
        const MAX_ATTESTATIONS: usize,
        const DEPOSIT_CONTRACT_TREE_DEPTH: usize,
        const MAX_DEPOSITS: usize,
        const MAX_VOLUNTARY_EXITS: usize,
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
        const MAX_BYTES_PER_TRANSACTION: usize,
        const MAX_TRANSACTIONS_PER_PAYLOAD: usize,
        const SYNC_COMMITTEE_SIZE: usize,
    >
    BeaconBlock<
        MAX_PROPOSER_SLASHINGS,
        MAX_VALIDATORS_PER_COMMITTEE,
        MAX_ATTESTER_SLASHINGS,
        MAX_ATTESTATIONS,
        DEPOSIT_CONTRACT_TREE_DEPTH,
        MAX_DEPOSITS,
        MAX_VOLUNTARY_EXITS,
        BYTES_PER_LOGS_BLOOM,
        MAX_EXTRA_DATA_BYTES,
        MAX_BYTES_PER_TRANSACTION,
        MAX_TRANSACTIONS_PER_PAYLOAD,
        SYNC_COMMITTEE_SIZE,
    >
{
    pub fn to_header(self) -> Result<BeaconBlockHeader, Error> {
        Ok(BeaconBlockHeader {
            slot: self.slot,
            proposer_index: self.proposer_index,
            parent_root: self.parent_root,
            state_root: self.state_root,
            body_root: hash_tree_root(self.body)?,
        })
    }
}

/// Beacon Block Body
/// https://github.com/ethereum/consensus-specs/blob/dev/specs/bellatrix/beacon-chain.md#beaconblockbody
#[derive(
    Clone, Debug, PartialEq, Eq, Default, SimpleSerialize, serde::Serialize, serde::Deserialize,
)]
pub struct BeaconBlockBody<
    const MAX_PROPOSER_SLASHINGS: usize,
    const MAX_VALIDATORS_PER_COMMITTEE: usize,
    const MAX_ATTESTER_SLASHINGS: usize,
    const MAX_ATTESTATIONS: usize,
    const DEPOSIT_CONTRACT_TREE_DEPTH: usize,
    const MAX_DEPOSITS: usize,
    const MAX_VOLUNTARY_EXITS: usize,
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
    const MAX_BYTES_PER_TRANSACTION: usize,
    const MAX_TRANSACTIONS_PER_PAYLOAD: usize,
    const SYNC_COMMITTEE_SIZE: usize,
> {
    pub randao_reveal: Signature,
    pub eth1_data: Eth1Data,
    pub graffiti: Bytes32,
    pub proposer_slashings: List<ProposerSlashing, MAX_PROPOSER_SLASHINGS>,
    pub attester_slashings:
        List<AttesterSlashing<MAX_VALIDATORS_PER_COMMITTEE>, MAX_ATTESTER_SLASHINGS>,
    pub attestations: List<Attestation<MAX_VALIDATORS_PER_COMMITTEE>, MAX_ATTESTATIONS>,
    pub deposits: List<Deposit<DEPOSIT_CONTRACT_TREE_DEPTH>, MAX_DEPOSITS>,
    pub voluntary_exits: List<SignedVoluntaryExit, MAX_VOLUNTARY_EXITS>,
    pub sync_aggregate: SyncAggregate<SYNC_COMMITTEE_SIZE>,
    pub execution_payload: ExecutionPayload<
        BYTES_PER_LOGS_BLOOM,
        MAX_EXTRA_DATA_BYTES,
        MAX_BYTES_PER_TRANSACTION,
        MAX_TRANSACTIONS_PER_PAYLOAD,
    >,
}

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/bellatrix/beacon-chain.md#executionpayload
#[derive(
    Clone, Debug, PartialEq, Eq, Default, SimpleSerialize, serde::Serialize, serde::Deserialize,
)]
pub struct ExecutionPayload<
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
    const MAX_BYTES_PER_TRANSACTION: usize,
    const MAX_TRANSACTIONS_PER_PAYLOAD: usize,
> {
    /// Execution block header fields
    pub parent_hash: H256,
    pub fee_recipient: Address,
    pub state_root: H256,
    pub receipts_root: H256,
    pub logs_bloom: ByteVector<BYTES_PER_LOGS_BLOOM>,
    /// 'difficulty' in the yellow paper
    pub prev_randao: H256,
    /// 'number' in the yellow paper
    pub block_number: BlockNumber,
    pub gas_limit: U64,
    pub gas_used: U64,
    pub timestamp: U64,
    pub extra_data: ByteList<MAX_EXTRA_DATA_BYTES>,
    pub base_fee_per_gas: U256,
    /// Extra payload fields
    /// Hash of execution block
    pub block_hash: H256,
    pub transactions: List<ByteList<MAX_BYTES_PER_TRANSACTION>, MAX_TRANSACTIONS_PER_PAYLOAD>,
}

impl<
        const BYTES_PER_LOGS_BLOOM: usize,
        const MAX_EXTRA_DATA_BYTES: usize,
        const MAX_BYTES_PER_TRANSACTION: usize,
        const MAX_TRANSACTIONS_PER_PAYLOAD: usize,
    >
    ExecutionPayload<
        BYTES_PER_LOGS_BLOOM,
        MAX_EXTRA_DATA_BYTES,
        MAX_BYTES_PER_TRANSACTION,
        MAX_TRANSACTIONS_PER_PAYLOAD,
    >
{
    pub fn to_header(
        mut self,
    ) -> Result<ExecutionPayloadHeader<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>, Error> {
        Ok(ExecutionPayloadHeader {
            parent_hash: self.parent_hash,
            fee_recipient: self.fee_recipient,
            state_root: self.state_root,
            receipts_root: self.receipts_root,
            logs_bloom: self.logs_bloom,
            prev_randao: self.prev_randao,
            block_number: self.block_number,
            gas_limit: self.gas_limit,
            gas_used: self.gas_used,
            timestamp: self.timestamp,
            extra_data: self.extra_data,
            base_fee_per_gas: self.base_fee_per_gas,
            block_hash: self.block_hash,
            transactions_root: Root::from_slice(self.transactions.hash_tree_root()?.as_bytes()),
        })
    }
}

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/bellatrix/beacon-chain.md#executionpayloadheader
#[derive(
    Clone, Debug, PartialEq, Eq, Default, SimpleSerialize, serde::Serialize, serde::Deserialize,
)]
pub struct ExecutionPayloadHeader<
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
> {
    /// Execution block header fields
    pub parent_hash: H256,
    pub fee_recipient: Address,
    pub state_root: H256,
    pub receipts_root: H256,
    pub logs_bloom: ByteVector<BYTES_PER_LOGS_BLOOM>,
    /// 'difficulty' in the yellow paper
    pub prev_randao: H256,
    /// 'number' in the yellow paper
    pub block_number: U64,
    pub gas_limit: U64,
    pub gas_used: U64,
    pub timestamp: U64,
    pub extra_data: ByteList<MAX_EXTRA_DATA_BYTES>,
    pub base_fee_per_gas: U256,
    /// Extra payload fields
    /// Hash of execution block
    pub block_hash: H256,
    pub transactions_root: Root,
}

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#lightclientbootstrap
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LightClientBootstrap<const SYNC_COMMITTEE_SIZE: usize> {
    pub beacon_header: BeaconBlockHeader,
    /// Current sync committee corresponding to `beacon_header.state_root`
    pub current_sync_committee: SyncCommittee<SYNC_COMMITTEE_SIZE>,
    pub current_sync_committee_branch: [H256; CURRENT_SYNC_COMMITTEE_DEPTH],
}

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#lightclientupdate
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LightClientUpdate<const SYNC_COMMITTEE_SIZE: usize> {
    /// Header attested to by the sync committee
    pub attested_header: BeaconBlockHeader,
    /// Next sync committee corresponding to `attested_header.state_root`
    pub next_sync_committee: Option<(
        SyncCommittee<SYNC_COMMITTEE_SIZE>,
        [H256; NEXT_SYNC_COMMITTEE_DEPTH],
    )>,
    /// Finalized header corresponding to `attested_header.state_root`
    pub finalized_header: (BeaconBlockHeader, [H256; FINALIZED_ROOT_DEPTH]),
    /// Sync committee aggregate signature
    pub sync_aggregate: SyncAggregate<SYNC_COMMITTEE_SIZE>,
    /// Slot at which the aggregate signature was created (untrusted)
    pub signature_slot: Slot,
}

// TODO each fork's prover implementation is redundant

pub fn gen_execution_payload_proof<
    const MAX_PROPOSER_SLASHINGS: usize,
    const MAX_VALIDATORS_PER_COMMITTEE: usize,
    const MAX_ATTESTER_SLASHINGS: usize,
    const MAX_ATTESTATIONS: usize,
    const DEPOSIT_CONTRACT_TREE_DEPTH: usize,
    const MAX_DEPOSITS: usize,
    const MAX_VOLUNTARY_EXITS: usize,
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
    const MAX_BYTES_PER_TRANSACTION: usize,
    const MAX_TRANSACTIONS_PER_PAYLOAD: usize,
    const SYNC_COMMITTEE_SIZE: usize,
>(
    body: &BeaconBlockBody<
        MAX_PROPOSER_SLASHINGS,
        MAX_VALIDATORS_PER_COMMITTEE,
        MAX_ATTESTER_SLASHINGS,
        MAX_ATTESTATIONS,
        DEPOSIT_CONTRACT_TREE_DEPTH,
        MAX_DEPOSITS,
        MAX_VOLUNTARY_EXITS,
        BYTES_PER_LOGS_BLOOM,
        MAX_EXTRA_DATA_BYTES,
        MAX_BYTES_PER_TRANSACTION,
        MAX_TRANSACTIONS_PER_PAYLOAD,
        SYNC_COMMITTEE_SIZE,
    >,
) -> Result<(Root, Vec<H256>), Error> {
    let tree = rs_merkle::MerkleTree::<rs_merkle::algorithms::Sha256>::from_leaves(&[
        hash_tree_root(body.randao_reveal.clone())?.0,
        hash_tree_root(body.eth1_data.clone())?.0,
        body.graffiti.0,
        hash_tree_root(body.proposer_slashings.clone())?.0,
        hash_tree_root(body.attester_slashings.clone())?.0,
        hash_tree_root(body.attestations.clone())?.0,
        hash_tree_root(body.deposits.clone())?.0,
        hash_tree_root(body.voluntary_exits.clone())?.0,
        hash_tree_root(body.sync_aggregate.clone())?.0,
        hash_tree_root(body.execution_payload.clone())?.0,
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
    ]);
    Ok((
        H256(
            tree.root()
                .expect("tree is not empty, root shouldn't be empty"),
        ),
        tree.proof(&[9])
            .proof_hashes()
            .iter()
            .map(|h| H256::from_slice(h))
            .collect(),
    ))
}

pub fn gen_execution_payload_fields_proof<
    const BYTES_PER_LOGS_BLOOM: usize,
    const MAX_EXTRA_DATA_BYTES: usize,
>(
    payload: &ExecutionPayloadHeader<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>,
    leaf_indices: &[usize],
) -> Result<(Root, Vec<H256>), Error> {
    let tree = rs_merkle::MerkleTree::<rs_merkle::algorithms::Sha256>::from_leaves(&[
        payload.parent_hash.0,
        hash_tree_root(payload.fee_recipient.clone())?.0,
        payload.state_root.0,
        payload.receipts_root.0,
        hash_tree_root(payload.logs_bloom.clone())?.0,
        payload.prev_randao.0,
        hash_tree_root(payload.block_number)?.0,
        hash_tree_root(payload.gas_limit)?.0,
        hash_tree_root(payload.gas_used)?.0,
        hash_tree_root(payload.timestamp)?.0,
        hash_tree_root(payload.extra_data.clone())?.0,
        hash_tree_root(payload.base_fee_per_gas.clone())?.0,
        payload.block_hash.0,
        payload.transactions_root.0,
        Default::default(),
        Default::default(),
    ]);
    Ok((
        H256(
            tree.root()
                .expect("tree is not empty, root shouldn't be empty"),
        ),
        tree.proof(leaf_indices)
            .proof_hashes()
            .iter()
            .map(|h| H256::from_slice(h))
            .collect(),
    ))
}

#[cfg(test)]
mod test {
    use super::{
        gen_execution_payload_fields_proof, gen_execution_payload_proof, BeaconBlockHeader,
    };
    use crate::bellatrix::LightClientUpdate;
    use crate::errors::Error;
    use crate::merkle::is_valid_merkle_branch;
    use crate::sync_protocol::SyncCommittee;
    use crate::{beacon::Root, compute::hash_tree_root, types::H256};
    use crate::{
        beacon::DOMAIN_SYNC_COMMITTEE,
        bls::fast_aggregate_verify,
        compute::{
            compute_domain, compute_epoch_at_slot, compute_fork_version, compute_signing_root,
        },
        config,
        context::DefaultChainContext,
        preset,
    };
    pub use milagro_bls::PublicKey as BLSPublicKey;
    use rs_merkle::algorithms::Sha256;
    use rs_merkle::MerkleProof;
    use ssz_rs::Merkleized;
    use std::fs;

    #[test]
    fn beacon_block_serialization() {
        use crate::execution::{
            EXECUTION_PAYLOAD_BLOCK_NUMBER_INDEX, EXECUTION_PAYLOAD_STATE_ROOT_INDEX,
        };
        let mut header: BeaconBlockHeader = serde_json::from_str(
            &fs::read_to_string("./data/goerli_bellatrix_header_4825088.json").unwrap(),
        )
        .unwrap();

        let mut block: crate::preset::mainnet::BellatrixBeaconBlock = serde_json::from_str(
            &fs::read_to_string("./data/goerli_bellatrix_block_4825088.json").unwrap(),
        )
        .unwrap();

        assert_eq!(header, block.clone().to_header().unwrap());
        assert_eq!(
            header.hash_tree_root().unwrap(),
            block.hash_tree_root().unwrap()
        );

        let (block_root, payload_proof) = gen_execution_payload_proof(&block.body).unwrap();
        assert_eq!(
            block_root.as_bytes(),
            block.body.hash_tree_root().unwrap().as_bytes()
        );

        let payload_root = block.body.execution_payload.hash_tree_root().unwrap();
        let payload_header = block.body.execution_payload.clone().to_header().unwrap();

        assert!(is_valid_merkle_branch(
            H256::from_slice(payload_root.as_bytes()),
            &payload_proof,
            9,
            block_root
        )
        .is_ok());

        let (root, proof) = gen_execution_payload_fields_proof(
            &payload_header,
            &[
                EXECUTION_PAYLOAD_STATE_ROOT_INDEX,
                EXECUTION_PAYLOAD_BLOCK_NUMBER_INDEX,
            ],
        )
        .unwrap();
        assert_eq!(root.as_bytes(), payload_root.as_bytes());

        assert!(is_valid_multiproofs_branch(
            root,
            &proof,
            &[
                EXECUTION_PAYLOAD_STATE_ROOT_INDEX,
                EXECUTION_PAYLOAD_BLOCK_NUMBER_INDEX
            ],
            &[
                hash_tree_root(payload_header.state_root).unwrap().0.into(),
                hash_tree_root(payload_header.block_number)
                    .unwrap()
                    .0
                    .into()
            ]
        )
        .unwrap());
    }

    fn is_valid_multiproofs_branch(
        root: Root,
        proof: &[H256],
        leaf_indices: &[usize],
        leaf_hashes: &[H256],
    ) -> Result<bool, Error> {
        let proof: Vec<[u8; 32]> = proof.iter().map(|h| h.0).collect();
        let proof = MerkleProof::<Sha256>::new(proof);
        let leaf_hashes: Vec<[u8; 32]> = leaf_hashes.iter().map(|h| h.0).collect();
        // TODO execution payload specific
        Ok(proof.verify(root.0, leaf_indices, &leaf_hashes, 16))
    }

    #[derive(Clone, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
    struct NetworkContext {
        pub genesis_validators_root: H256,
    }

    #[test]
    fn test_light_client_update_verification() {
        let sync_committee: SyncCommittee<{ preset::mainnet::PRESET.SYNC_COMMITTEE_SIZE }> =
            serde_json::from_str(
                &fs::read_to_string("./data/mainnet_sync_committee_period_713.json").unwrap(),
            )
            .unwrap();
        assert!(sync_committee.validate().is_ok());

        let update: LightClientUpdate<{ preset::mainnet::PRESET.SYNC_COMMITTEE_SIZE }> =
            serde_json::from_str(
                &fs::read_to_string("./data/mainnet_light_client_update_slot_5841038.json")
                    .unwrap(),
            )
            .unwrap();

        let network: NetworkContext =
            serde_json::from_str(&fs::read_to_string("./data/mainnet_context.json").unwrap())
                .unwrap();

        // ensure that signing_root calculation is correct

        let ctx = DefaultChainContext::new_with_config(0.into(), config::mainnet::CONFIG);
        let fork_version =
            compute_fork_version(&ctx, compute_epoch_at_slot(&ctx, update.signature_slot));
        let domain = compute_domain(
            &ctx,
            DOMAIN_SYNC_COMMITTEE,
            Some(fork_version),
            Some(network.genesis_validators_root),
        )
        .unwrap();
        let signing_root = compute_signing_root(update.attested_header, domain).unwrap();
        let expected_signing_root: H256 = serde_json::from_str(
            &fs::read_to_string("./data/mainnet_signing_root_slot_5841037.json").unwrap(),
        )
        .unwrap();
        assert_eq!(expected_signing_root, signing_root);

        // ensure that bls verification is correct

        let participant_pubkeys: Vec<BLSPublicKey> = update
            .sync_aggregate
            .sync_committee_bits
            .iter()
            .zip(sync_committee.pubkeys.iter())
            .filter(|it| it.0 == true)
            .map(|t| t.1.clone().try_into().unwrap())
            .collect();

        let res = fast_aggregate_verify(
            participant_pubkeys,
            signing_root,
            update
                .sync_aggregate
                .sync_committee_signature
                .try_into()
                .unwrap(),
        );
        assert!(res.is_ok());
        assert!(res.unwrap());
    }
}
