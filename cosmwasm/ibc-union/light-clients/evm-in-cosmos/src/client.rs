use cometbls_light_client::client::CometblsLightClient;
use cosmwasm_std::Empty;
use ethereum_light_client_types::StorageProof;
use evm_in_cosmos_light_client_types::{
    client_state::ClientState, consensus_state::ConsensusState, header::Header,
};
use ibc_union_light_client::IbcClient;
use ibc_union_msg::lightclient::Status;
use ibc_union_spec::ConsensusStatePath;
use ics23::ibc_api::SDK_SPECS;
use unionlabs::encoding::{EncodeAs, EthAbi, Json};

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
            consensus_state.ibc_storage_root,
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
            consensus_state.ibc_storage_root,
            storage_proof,
        )
        .map_err(Error::EthereumLightClient)?;

        Ok(())
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> u64 {
        consensus_state.timestamp
    }

    fn get_latest_height(client_state: &Self::ClientState) -> u64 {
        client_state.latest_slot
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
        let client_state = ctx.read_self_client_state()?;
        let l1_consensus_state: cometbls_light_client_types::ConsensusState = ctx
            .read_consensus_state::<CometblsLightClient>(
                client_state.l1_client_id,
                header.l1_height.height(),
            )
            .map_err(Into::<Error>::into)?;
        let consensus_state_path = ConsensusStatePath {
            client_id: client_state.l2_client_id,
            height: header.l2_slot,
        }
        .key();

        let header_ = header.clone();
        // The ethereum consensus state is stored in proto-encoded wasm-wrapped form.
        // Verify inclusion of the ethereum consensus state against union.
        ics23::ibc_api::verify_membership(
            &header.l2_inclusion_proof,
            &SDK_SPECS,
            &l1_consensus_state.app_hash,
            &[
                b"wasm".to_vec(),
                3u8.to_le_bytes()
                    .into_iter()
                    .chain(client_state.l1_ibc_contract_address)
                    .chain(consensus_state_path)
                    .collect::<Vec<u8>>(),
            ],
            header.l2_consensus_state.encode_as::<EthAbi>(),
        )
        .map_err(Error::VerifyL2Membership)?;

        Ok(update_state(client_state, header_)?)
    }

    fn misbehaviour(
        _ctx: ibc_union_light_client::IbcClientCtx<Self>,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<Self::ClientState, ibc_union_light_client::IbcClientError<Self>> {
        unimplemented!()
    }
}

fn update_state(
    mut client_state: ClientState,
    header: Header,
) -> Result<(u64, ClientState, ConsensusState), Error> {
    if client_state.latest_slot < header.l1_height.height() {
        client_state.latest_slot = header.l1_height.height();
    }

    let consensus_state = ConsensusState {
        evm_state_root: header.l2_consensus_state.state_root,
        ibc_storage_root: header.l2_consensus_state.storage_root,
        timestamp: header.l2_consensus_state.timestamp,
    };

    Ok((header.l1_height.height(), client_state, consensus_state))
}
