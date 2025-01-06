use cometbls_light_client::client::CometblsLightClient;
use cosmwasm_std::Empty;
use ethereum_light_client_types::StorageProof;
use evm_state_lens_light_client_types::{ClientState, ConsensusState, Header};
use ibc_union_light_client::IbcClient;
use ibc_union_msg::lightclient::Status;
use ibc_union_spec::ConsensusStatePath;
use unionlabs::{encoding::Json, hash::H256};

use crate::errors::Error;

pub struct EvmInCosmosLightClient;

impl IbcClient for EvmInCosmosLightClient {
    type Error = Error;

    type CustomQuery = Empty;

    type Header = Header;

    type Misbehaviour = Header;

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    type StorageProof = StorageProof;

    type Encoding = Json;

    fn verify_membership(
        ctx: ibc_union_light_client::IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
        value: Vec<u8>,
    ) -> Result<(), ibc_union_light_client::IbcClientError<Self>> {
        let consensus_state = ctx.read_self_consensus_state(height)?;

        ethereum_light_client::client::verify_membership(
            key,
            consensus_state.storage_root,
            storage_proof,
            value,
        )
        .map_err(Error::EthereumLightClient)?;

        Ok(())
    }

    fn verify_non_membership(
        ctx: ibc_union_light_client::IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
    ) -> Result<(), ibc_union_light_client::IbcClientError<Self>> {
        let consensus_state = ctx.read_self_consensus_state(height)?;

        ethereum_light_client::client::verify_non_membership(
            key,
            consensus_state.storage_root,
            storage_proof,
        )
        .map_err(Error::EthereumLightClient)?;

        Ok(())
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> u64 {
        consensus_state.timestamp
    }

    fn get_latest_height(client_state: &Self::ClientState) -> u64 {
        client_state.l2_latest_height
    }

    fn status(_client_state: &Self::ClientState) -> Status {
        // FIXME: expose the ctx to this call to allow threading this call to L1
        // client. generally, we want to thread if a client is an L2 so always
        // provide the ctx?
        // let client_state: WasmClientState = read_client_state(deps)?;
        // let l1_client_state = query_client_state::<WasmL1ClientState>(
        //     deps,
        //     env,
        //     client_state.data.l1_client_id.clone(),
        // )
        // .map_err(Error::CustomQuery)?;

        // if l1_client_state.data.frozen_height != Height::default() {
        //     return Ok(Status::Frozen);
        // }

        // let Some(_) = read_consensus_state::<Self>(deps, &client_state.latest_height)? else {
        //     return Ok(Status::Expired);
        // };

        // Ok(Status::Active)
        Status::Active
    }

    fn verify_creation(
        _client_state: &Self::ClientState,
        _consensus_state: &Self::ConsensusState,
    ) -> Result<(), ibc_union_light_client::IbcClientError<Self>> {
        Ok(())
    }

    fn verify_header(
        ctx: ibc_union_light_client::IbcClientCtx<Self>,
        header: Self::Header,
    ) -> Result<
        (u64, Self::ClientState, Self::ConsensusState),
        ibc_union_light_client::IbcClientError<Self>,
    > {
        let mut client_state = ctx.read_self_client_state()?;

        ctx.verify_membership::<CometblsLightClient>(
            client_state.l1_client_id,
            header.l1_height.height(),
            ConsensusStatePath {
                client_id: client_state.l2_client_id,
                height: header.l2_height.height(),
            }
            .key()
            .into_bytes(),
            header.l2_consensus_state_proof.clone(),
            header.l2_consensus_state.clone(),
        )
        .map_err(Error::L1Error)?;

        let l2_timestamp = extract_uint64(
            &header.l2_consensus_state,
            client_state.timestamp_offset as usize,
        );

        let l2_state_root = extract_bytes32(
            &header.l2_consensus_state,
            client_state.state_root_offset as usize,
        );

        let l2_storage_root = extract_bytes32(
            &header.l2_consensus_state,
            client_state.storage_root_offset as usize,
        );

        if client_state.l2_latest_height < header.l2_height.height() {
            client_state.l2_latest_height = header.l2_height.height();
        }

        let consensus_state = ConsensusState {
            timestamp: l2_timestamp,
            state_root: l2_state_root,
            storage_root: l2_storage_root,
        };

        Ok((header.l2_height.height(), client_state, consensus_state))
    }

    fn misbehaviour(
        _ctx: ibc_union_light_client::IbcClientCtx<Self>,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<Self::ClientState, ibc_union_light_client::IbcClientError<Self>> {
        unimplemented!()
    }
}

fn extract_uint64(data: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(
        data[offset..offset + 8]
            .try_into()
            .expect("impossible; qed"),
    )
}

fn extract_bytes32(data: &[u8], offset: usize) -> H256 {
    H256::new(
        data[offset..offset + 32]
            .try_into()
            .expect("impossible; qed"),
    )
}
