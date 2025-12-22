use cosmwasm_std::{Addr, Empty, ensure};
use ethereum_light_client::client::EthereumLightClient;
use ibc_union_light_client::{
    ClientCreationResult, IbcClient, IbcClientCtx, IbcClientError, StateUpdate,
    spec::{Status, Timestamp},
};
use starknet_light_client_types::{ClientState, ConsensusState, Header, StorageProof};
use starknet_storage_verifier::{Membership, PoseidonHash};
use starknet_types::{Felt, commitment_key};
use unionlabs::encoding::Bincode;

use crate::errors::Error;

pub struct StarknetLightClient;

impl IbcClient for StarknetLightClient {
    type Error = Error;

    type Header = Header;

    type Misbehaviour = ();

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    type Encoding = Bincode;

    type CustomQuery = Empty;

    type StorageProof = StorageProof;

    fn verify_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
        value: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        let consensus_state = ctx.read_self_consensus_state(height)?;

        let res = starknet_storage_verifier::verify_proof::<PoseidonHash>(
            consensus_state.ibc_storage_root,
            commitment_key(
                key.try_into()
                    .map_err(|bz: Vec<u8>| Error::InvalidProofKey(bz.into()))?,
            ),
            Felt::from_be_bytes(
                value
                    .try_into()
                    .map_err(|bz: Vec<u8>| Error::InvalidProofValue(bz.into()))?,
            ),
            &storage_proof.nodes,
        )
        .map_err(Into::<Error>::into)?;

        ensure!(matches!(res, Membership::Membership), Error::InvalidProof);

        Ok(())
    }

    fn verify_non_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
    ) -> Result<(), IbcClientError<Self>> {
        let consensus_state = ctx.read_self_consensus_state(height)?;

        let res = starknet_storage_verifier::verify_proof::<PoseidonHash>(
            consensus_state.ibc_storage_root,
            commitment_key(
                key.try_into()
                    .map_err(|bz: Vec<u8>| Error::InvalidProofKey(bz.into()))?,
            ),
            Felt::ZERO,
            &storage_proof.nodes,
        )
        .map_err(Into::<Error>::into)?;

        ensure!(
            matches!(res, Membership::NonMembership),
            Error::InvalidProof
        );

        Ok(())
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> Timestamp {
        consensus_state.timestamp
    }

    fn get_latest_height(ClientState::V1(client_state): &Self::ClientState) -> u64 {
        client_state.latest_height
    }

    fn get_counterparty_chain_id(ClientState::V1(client_state): &Self::ClientState) -> String {
        client_state.chain_id.to_string()
    }

    fn status(ctx: IbcClientCtx<Self>, client_state: &Self::ClientState) -> Status {
        let _ = client_state;
        let _ = ctx;
        // FIXME: expose the ctx to this call to allow threading this call to L1
        // client. generally, we want to thread if a client is an L2 so always
        // provide the ctx?
        Status::Active
    }

    fn verify_creation(
        _caller: Addr,
        _client_state: &Self::ClientState,
        _consensus_state: &Self::ConsensusState,
        _relayer: Addr,
    ) -> Result<ClientCreationResult<Self>, IbcClientError<Self>> {
        Ok(ClientCreationResult::new())
    }

    fn verify_header(
        ctx: IbcClientCtx<Self>,
        _caller: Addr,
        header: Self::Header,
        _relayer: Addr,
    ) -> Result<StateUpdate<Self>, IbcClientError<Self>> {
        let ClientState::V1(mut client_state) = ctx.read_self_client_state()?;

        let l1_consensus_state = ctx
            .read_consensus_state::<EthereumLightClient>(
                client_state.l1_client_id,
                header.l1_height,
            )
            .map_err(Into::<Error>::into)?;

        starknet_verifier::verify_header(&client_state, &header, l1_consensus_state.state_root)
            .map_err(Into::<Error>::into)?;

        let update_height = header.l2_block.block_number;

        let consensus_state = ConsensusState {
            timestamp: Timestamp::from_secs(header.l2_block.block_timestamp),
            contracts_trie_root: header.l2_block.contracts_trie_root,
            classes_trie_root: header.l2_block.classes_trie_root,
            ibc_storage_root: header.l2_ibc_contract_proof.contract_leaf_data.storage_root,
        };

        let state_update = StateUpdate::new(update_height, consensus_state);

        if client_state.latest_height < update_height {
            client_state.latest_height = update_height;
            Ok(state_update.overwrite_client_state(ClientState::V1(client_state)))
        } else {
            Ok(state_update)
        }
    }

    fn misbehaviour(
        _ctx: IbcClientCtx<Self>,
        _caller: Addr,
        _misbehaviour: Self::Misbehaviour,
        _relayer: Addr,
    ) -> Result<Self::ClientState, IbcClientError<Self>> {
        Err(Error::Unimplemented.into())
    }
}
