use cosmwasm_std::Deps;
use ics008_wasm_client::{
    storage_utils::{
        read_client_state, read_consensus_state, save_client_state, save_consensus_state,
    },
    IbcClient, IbcClientError, StorageState,
};
use unionlabs::{
    aptos::{account::AccountAddress, transaction_info::TransactionInfo},
    cosmwasm::wasm::union::custom_query::{query_consensus_state, UnionCustomQuery},
    encoding::Proto,
    hash::H256,
    ibc::{
        core::{client::height::Height, commitment::merkle_path::MerklePath},
        lightclients::{
            ethereum,
            movement::{
                client_state::ClientState, consensus_state::ConsensusState, header::Header,
            },
            wasm,
        },
    },
    uint::U256,
};

use crate::errors::Error;

type WasmClientState = wasm::client_state::ClientState<ClientState>;
type WasmConsensusState = wasm::consensus_state::ConsensusState<ConsensusState>;
type WasmL1ConsensusState =
    wasm::consensus_state::ConsensusState<ethereum::consensus_state::ConsensusState>;

#[derive(rlp::RlpEncodable)]
struct BlockCommitment {
    pub height: U256,
    pub commitment: U256,
    pub block_id: U256,
}

pub struct MovementLightClient;

impl IbcClient for MovementLightClient {
    type Error = Error;

    type CustomQuery = UnionCustomQuery;

    type Header = Header;

    type Misbehaviour = Header;

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    type Encoding = Proto;

    fn verify_membership(
        _deps: Deps<Self::CustomQuery>,
        _height: Height,
        _delay_time_period: u64,
        _delay_block_period: u64,
        _proof: Vec<u8>,
        mut _path: MerklePath,
        _value: StorageState,
    ) -> Result<(), IbcClientError<Self>> {
        // let consensus_state: WasmConsensusState =
        //     read_consensus_state(deps, &height)?.ok_or(Error::ConsensusStateNotFound(height))?;
        // let client_state: WasmClientState = read_client_state(deps)?;

        // let path = path.key_path.pop().ok_or(Error::EmptyIbcPath)?;

        // match value {
        //     StorageState::Occupied(value) => do_verify_membership(
        //         path,
        //         consensus_state.data.state_root,
        //         client_state.data.table_handle,
        //         proof,
        //         value,
        //     ),
        //     StorageState::Empty => unimplemented!(),
        // }
        Ok(())
    }

    fn verify_header(
        _deps: Deps<Self::CustomQuery>,
        _env: cosmwasm_std::Env,
        _header: Header,
    ) -> Result<(), IbcClientError<Self>> {
        // let client_state: WasmClientState = read_client_state(deps)?;

        // let l1_consensus_state = query_consensus_state::<WasmL1ConsensusState>(
        //     deps,
        //     &env,
        //     client_state.data.l1_client_id.to_string(),
        //     header.l1_height,
        // )
        // .map_err(Error::CustomQuery)?;

        // aptos_verifier::verify_tx_state(
        //     &header.tx_proof,
        //     header
        //         .state_proof
        //         .latest_ledger_info()
        //         .commit_info
        //         .executed_state_id,
        //     header.state_proof.latest_ledger_info().commit_info.version,
        // )
        // .map_err(Into::<Error>::into)?;

        // let expected_commitment = BlockCommitment {
        //     height: header.new_height.into(),
        //     commitment: U256::from_be_bytes(header.state_proof.hash()),
        //     block_id: U256::from_be_bytes(header.state_proof.latest_ledger_info().commit_info.id.0),
        // };

        // // TODO(aeryz): make sure the given state_proof_hash_proof.key matches the correct slot

        // ethereum_verifier::verify::verify_account_storage_root(
        //     l1_consensus_state.data.state_root,
        //     &client_state.data.l1_contract_address,
        //     &header.settlement_contract_proof.proof,
        //     &header.settlement_contract_proof.storage_root,
        // )
        // .unwrap();

        // ethereum_verifier::verify::verify_storage_proof(
        //     header.settlement_contract_proof.storage_root,
        //     header.state_proof_hash_proof.key,
        //     &rlp::encode(&expected_commitment),
        //     header.state_proof_hash_proof.proof,
        // )
        // .unwrap();

        Ok(())
    }

    fn verify_misbehaviour(
        _deps: Deps<Self::CustomQuery>,
        _env: cosmwasm_std::Env,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<(), IbcClientError<Self>> {
        todo!()
    }

    fn update_state(
        mut deps: cosmwasm_std::DepsMut<Self::CustomQuery>,
        _env: cosmwasm_std::Env,
        header: Self::Header,
    ) -> Result<Vec<Height>, IbcClientError<Self>> {
        let mut client_state: WasmClientState = read_client_state(deps.as_ref())?;

        let TransactionInfo::V0(tx_info) = header.tx_proof.transaction_info;

        let consensus_state = ConsensusState {
            state_root: tx_info.state_checkpoint_hash.unwrap(), // TODO(aeryz): we always need this, no need to make this an option
            timestamp: header
                .state_proof
                .latest_ledger_info()
                .commit_info
                .timestamp_usecs,
            state_proof_hash: H256::default(), // TODO(aeryz): im not sure if we need this
        };

        if header.new_height > client_state.data.latest_block_num {
            client_state.data.latest_block_num = header.new_height;
            client_state.latest_height.revision_height = header.new_height;
            save_client_state::<MovementLightClient>(deps.branch(), client_state);
        }

        let update_height = Height {
            revision_number: 0,
            revision_height: header.new_height,
        };

        save_consensus_state::<MovementLightClient>(
            deps,
            WasmConsensusState {
                data: consensus_state,
            },
            &update_height,
        );

        Ok(vec![update_height])
    }

    fn update_state_on_misbehaviour(
        _deps: cosmwasm_std::DepsMut<Self::CustomQuery>,
        _env: cosmwasm_std::Env,
        _client_message: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        todo!()
    }

    fn check_for_misbehaviour_on_header(
        _deps: Deps<Self::CustomQuery>,
        _header: Self::Header,
    ) -> Result<bool, IbcClientError<Self>> {
        todo!()
    }

    fn check_for_misbehaviour_on_misbehaviour(
        _deps: Deps<Self::CustomQuery>,
        _misbehaviour: Self::Misbehaviour,
    ) -> Result<bool, IbcClientError<Self>> {
        todo!()
    }

    fn verify_upgrade_and_update_state(
        _deps: cosmwasm_std::DepsMut<Self::CustomQuery>,
        _upgrade_client_state: Self::ClientState,
        _upgrade_consensus_state: Self::ConsensusState,
        _proof_upgrade_client: Vec<u8>,
        _proof_upgrade_consensus_state: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        todo!()
    }

    fn migrate_client_store(
        _deps: cosmwasm_std::DepsMut<Self::CustomQuery>,
    ) -> Result<(), IbcClientError<Self>> {
        todo!()
    }

    fn status(
        _deps: Deps<Self::CustomQuery>,
        _env: &cosmwasm_std::Env,
    ) -> Result<ics008_wasm_client::Status, IbcClientError<Self>> {
        Ok(ics008_wasm_client::Status::Active)
    }

    fn export_metadata(
        _deps: Deps<Self::CustomQuery>,
        _env: &cosmwasm_std::Env,
    ) -> Result<
        Vec<unionlabs::ibc::core::client::genesis_metadata::GenesisMetadata>,
        IbcClientError<Self>,
    > {
        todo!()
    }

    fn timestamp_at_height(
        _deps: Deps<Self::CustomQuery>,
        _height: Height,
    ) -> Result<u64, IbcClientError<Self>> {
        todo!()
    }
}

fn do_verify_membership(
    _path: String,
    state_root: H256,
    _table_handle: AccountAddress,
    proof: Vec<u8>,
    value: Vec<u8>,
) -> Result<(), IbcClientError<MovementLightClient>> {
    aptos_verifier::verify_existence_proof(
        &proof,
        state_root,
        H256::default(),
        value.try_into().unwrap(),
    )
    .unwrap();

    Ok(())
}
