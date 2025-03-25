use cosmwasm_std::{Addr, Empty};
use ethereum_light_client_types::StorageProof;
use ibc_union_light_client::{
    ClientCreationResult, IbcClient, IbcClientCtx, IbcClientError, StateUpdate,
};
use ibc_union_msg::lightclient::Status;
use mpt_trusted_light_client_types::{ClientState, ConsensusState, Header};
use unionlabs::encoding::Bincode;

use crate::errors::Error;

pub enum MptTrustedLightClient {}

impl IbcClient for MptTrustedLightClient {
    type Error = Error;

    type CustomQuery = Empty;

    type Header = Header;

    type Misbehaviour = Header;

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    type StorageProof = StorageProof;

    type Encoding = Bincode;

    fn verify_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
        value: Vec<u8>,
    ) -> Result<(), IbcClientError<Self>> {
        let consensus_state = ctx.read_self_consensus_state(height)?;
        Ok(ethereum_light_client::client::verify_membership(
            key,
            consensus_state.ibc_storage_root,
            storage_proof,
            value,
        )
        .map_err(Into::<Error>::into)?)
    }

    fn verify_non_membership(
        ctx: IbcClientCtx<Self>,
        height: u64,
        key: Vec<u8>,
        storage_proof: Self::StorageProof,
    ) -> Result<(), IbcClientError<Self>> {
        let consensus_state = ctx.read_self_consensus_state(height)?;
        Ok(ethereum_light_client::client::verify_non_membership(
            key,
            consensus_state.ibc_storage_root,
            storage_proof,
        )
        .map_err(Into::<Error>::into)?)
    }

    fn verify_header(
        ctx: IbcClientCtx<Self>,
        caller: Addr,
        header: Self::Header,
        _relayer: Addr,
    ) -> Result<StateUpdate<Self>, IbcClientError<Self>> {
        let mut client_state = ctx.read_self_client_state()?;
        if client_state
            .whitelisted_relayers
            .contains(&caller.to_string())
        {
            return Err(Error::Unauthorized(caller).into());
        }

        // We still verify the account storage root since we only trust `state_root`
        evm_storage_verifier::verify_account_storage_root(
            header.state_root,
            &client_state.ibc_contract_address,
            &header.ibc_account_proof.proof,
            &header.ibc_account_proof.storage_root,
        )
        .map_err(Error::InvalidContractAddressProof)?;

        let mut update = StateUpdate::new(
            header.height,
            ConsensusState {
                ibc_storage_root: header.ibc_account_proof.storage_root,
                timestamp: header.timestamp,
            },
        );

        if header.height > client_state.latest_height {
            client_state.latest_height = header.height;
            update = update.overwrite_client_state(client_state);
        }

        Ok(update)
    }

    fn misbehaviour(
        _ctx: IbcClientCtx<Self>,
        _caller: Addr,
        _misbehaviour: Self::Misbehaviour,
        _relayer: Addr,
    ) -> Result<Self::ClientState, IbcClientError<Self>> {
        Err(Error::Unimplemented.into())
    }

    fn status(ctx: IbcClientCtx<Self>, _client_state: &Self::ClientState) -> Status {
        let _ = ctx;

        Status::Active
    }

    fn verify_creation(
        _caller: Addr,
        _client_state: &Self::ClientState,
        _consensus_state: &Self::ConsensusState,
        _relayer: Addr,
    ) -> Result<ClientCreationResult<Self>, IbcClientError<MptTrustedLightClient>> {
        Ok(ClientCreationResult::new())
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> u64 {
        consensus_state.timestamp
    }

    fn get_latest_height(client_state: &Self::ClientState) -> u64 {
        client_state.latest_height
    }

    fn get_counterparty_chain_id(client_state: &Self::ClientState) -> String {
        client_state.chain_id.to_string()
    }
}

// #[cfg(test)]
// mod test {
//     use cosmwasm_std::{
//         testing::{MockApi, MockQuerier, MockStorage},
//         DepsMut, OwnedDeps,
//     };
//     use ics008_wasm_client::{
//         storage_utils::{
//             consensus_db_key, read_subject_client_state, HOST_CLIENT_STATE_KEY,
//             SUBJECT_CLIENT_STORE_PREFIX, SUBSTITUTE_CLIENT_STORE_PREFIX,
//         },
//         IbcClient,
//     };
//     use unionlabs::{
//         bounded::BoundedU32,
//         cosmwasm::wasm::union::custom_query::UnionCustomQuery,
//         encoding::{EncodeAs, Proto},
//         google::protobuf::any::Any,
//         primitives::{H160, H256},
//         ibc::core::client::height::Height,
//         id::ClientId,
//         primitives::U256,
//     };

//     use super::{
//         MptTrustedLightClient, ClientState, ConsensusState, WasmClientState, WasmConsensusState,
//     };
//     use crate::errors::Error;

//     const INITIAL_CONSENSUS_STATE_HEIGHT: Height = Height {
//         revision_number: 0,
//         revision_height: 950,
//     };

//     const INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT: Height = Height {
//         revision_number: 0,
//         revision_height: 970,
//     };

//     #[allow(clippy::type_complexity)]
//     fn mock_dependencies(
//     ) -> OwnedDeps<MockStorage, MockApi, MockQuerier<UnionCustomQuery>, UnionCustomQuery> {
//         OwnedDeps::<_, _, _, UnionCustomQuery> {
//             storage: MockStorage::default(),
//             api: MockApi::default(),
//             querier: MockQuerier::<UnionCustomQuery>::new(&[]),
//             custom_query_type: std::marker::PhantomData,
//         }
//     }

//     fn create_client_state(
//         l1_client_id: String,
//         chain_id: U256,
//         latest_slot: u64,
//         height: Height,
//         frozen_height: Height,
//     ) -> WasmClientState {
//         WasmClientState {
//             data: ClientState {
//                 l1_client_id: ClientId::new(l1_client_id.clone()).unwrap(),
//                 chain_id,
//                 l1_latest_slot: latest_slot,
//                 frozen_height,
//                 l1_contract_address: H160::default(),
//                 l1_next_node_num_slot: U256::from(10),
//                 l1_nodes_slot: U256::from(10),
//                 l1_next_node_num_slot_offset_bytes: BoundedU32::new(0).unwrap(),
//                 l1_nodes_confirm_data_offset: U256::from(10),
//                 l2_ibc_contract_address: H160::default(),
//             },
//             latest_height: height,
//             checksum: H256::default(),
//         }
//     }

//     fn save_states_to_migrate_store(
//         deps: DepsMut<UnionCustomQuery>,
//         subject_client_state: &WasmClientState,
//         substitute_client_state: &WasmClientState,
//         subject_consensus_state: &WasmConsensusState,
//         substitute_consensus_state: &WasmConsensusState,
//     ) {
//         deps.storage.set(
//             format!("{SUBJECT_CLIENT_STORE_PREFIX}{HOST_CLIENT_STATE_KEY}").as_bytes(),
//             &Any(subject_client_state.clone()).encode_as::<Proto>(),
//         );
//         deps.storage.set(
//             format!(
//                 "{SUBJECT_CLIENT_STORE_PREFIX}{}",
//                 consensus_db_key(&INITIAL_CONSENSUS_STATE_HEIGHT)
//             )
//             .as_bytes(),
//             &Any(subject_consensus_state.clone()).encode_as::<Proto>(),
//         );
//         deps.storage.set(
//             format!("{SUBSTITUTE_CLIENT_STORE_PREFIX}{HOST_CLIENT_STATE_KEY}").as_bytes(),
//             &Any(substitute_client_state.clone()).encode_as::<Proto>(),
//         );
//         deps.storage.set(
//             format!(
//                 "{SUBSTITUTE_CLIENT_STORE_PREFIX}{}",
//                 consensus_db_key(&INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT)
//             )
//             .as_bytes(),
//             &Any(substitute_consensus_state.clone()).encode_as::<Proto>(),
//         );
//     }

//     #[allow(clippy::type_complexity)]
//     fn prepare_migrate_tests() -> (
//         OwnedDeps<MockStorage, MockApi, MockQuerier<UnionCustomQuery>, UnionCustomQuery>,
//         WasmClientState,
//         WasmConsensusState,
//         WasmClientState,
//         WasmConsensusState,
//     ) {
//         let deps = mock_dependencies();

//         let subject_client_state = create_client_state(
//             "l1_client_1".to_string(),
//             U256::from(1),
//             INITIAL_CONSENSUS_STATE_HEIGHT.revision_height,
//             INITIAL_CONSENSUS_STATE_HEIGHT,
//             Height::default(),
//         );
//         let substitute_client_state = create_client_state(
//             "l1_client_1".to_string(),
//             U256::from(1),
//             INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT.revision_height,
//             INITIAL_SUBSTITUTE_CONSENSUS_STATE_HEIGHT,
//             Height::default(),
//         );

//         let subject_consensus_state = WasmConsensusState {
//             data: ConsensusState {
//                 ibc_storage_root: H256::default(),
//                 timestamp: 1000,
//             },
//         };
//         let substitute_consensus_state = WasmConsensusState {
//             data: ConsensusState {
//                 ibc_storage_root: H256::default(),
//                 timestamp: 2000,
//             },
//         };

//         (
//             deps,
//             subject_client_state,
//             subject_consensus_state,
//             substitute_client_state,
//             substitute_consensus_state,
//         )
//     }

//     #[test]
//     fn migrate_client_store_succeeds_with_valid_data() {
//         let (
//             mut deps,
//             mut subject_client_state,
//             subject_consensus_state,
//             mut substitute_client_state,
//             substitute_consensus_state,
//         ) = prepare_migrate_tests();

//         subject_client_state.data.frozen_height = Height {
//             revision_number: 0,
//             revision_height: 1000,
//         };

//         substitute_client_state.data.frozen_height = Height::default();

//         save_states_to_migrate_store(
//             deps.as_mut(),
//             &subject_client_state,
//             &substitute_client_state,
//             &subject_consensus_state,
//             &substitute_consensus_state,
//         );

//         let original_subject_client_state: WasmClientState =
//             read_subject_client_state::<MptTrustedLightClient>(deps.as_ref()).unwrap();

//         assert_eq!(
//             original_subject_client_state.data.frozen_height,
//             Height {
//                 revision_number: 0,
//                 revision_height: 1000,
//             }
//         );

//         // Perform migration
//         let result = MptTrustedLightClient::migrate_client_store(deps.as_mut());

//         // Assert success, print error if any
//         if let Err(ref e) = result {
//             println!("Migration failed with error: {:?}", e);
//         }
//         assert!(result.is_ok());

//         let updated_subject_client_state: WasmClientState =
//             read_subject_client_state::<MptTrustedLightClient>(deps.as_ref()).unwrap();
//         assert_eq!(
//             updated_subject_client_state.data.frozen_height,
//             Height::default()
//         );
//         assert_eq!(
//             updated_subject_client_state.latest_height,
//             substitute_client_state.latest_height
//         );
//     }

//     #[test]
//     fn migrate_client_store_fails_when_substitute_client_frozen() {
//         let (
//             mut deps,
//             subject_client_state,
//             subject_consensus_state,
//             mut substitute_client_state,
//             substitute_consensus_state,
//         ) = prepare_migrate_tests();

//         // Make the substitute client frozen
//         substitute_client_state.data.frozen_height = Height {
//             revision_number: 0,
//             revision_height: 100,
//         };

//         save_states_to_migrate_store(
//             deps.as_mut(),
//             &subject_client_state,
//             &substitute_client_state,
//             &subject_consensus_state,
//             &substitute_consensus_state,
//         );

//         // Perform migration
//         let result = MptTrustedLightClient::migrate_client_store(deps.as_mut());

//         // Assert failure
//         assert_eq!(result, Err(Error::SubstituteClientFrozen.into()));
//     }

//     #[test]
//     fn migrate_client_store_fails_when_fields_differ() {
//         let (
//             mut deps,
//             subject_client_state,
//             subject_consensus_state,
//             mut substitute_client_state,
//             substitute_consensus_state,
//         ) = prepare_migrate_tests();

//         // Alter the chain_id in the substitute client state
//         substitute_client_state.data.chain_id = U256::from(999);

//         save_states_to_migrate_store(
//             deps.as_mut(),
//             &subject_client_state,
//             &substitute_client_state,
//             &subject_consensus_state,
//             &substitute_consensus_state,
//         );

//         // Perform migration
//         let result = MptTrustedLightClient::migrate_client_store(deps.as_mut());

//         // Assert failure
//         assert_eq!(result, Err(Error::MigrateFieldsChanged.into()));
//     }

//     #[test]
//     fn migrate_client_store_fails_when_substitute_consensus_not_found() {
//         let (
//             mut deps,
//             subject_client_state,
//             subject_consensus_state,
//             mut substitute_client_state,
//             _substitute_consensus_state, // we won't save this to storage
//         ) = prepare_migrate_tests();

//         // Modify the latest height to a height where the consensus state is not found
//         substitute_client_state.latest_height = Height {
//             revision_number: 0,
//             revision_height: 15,
//         };

//         save_states_to_migrate_store(
//             deps.as_mut(),
//             &subject_client_state,
//             &substitute_client_state,
//             &subject_consensus_state,
//             &subject_consensus_state, // Reusing subject consensus intentionally
//         );

//         // Perform migration
//         let result = MptTrustedLightClient::migrate_client_store(deps.as_mut());

//         // Assert failure
//         assert_eq!(
//             result,
//             Err(Error::ConsensusStateNotFound(substitute_client_state.latest_height).into())
//         );
//     }
// }
