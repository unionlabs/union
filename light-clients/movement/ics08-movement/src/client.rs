use cosmwasm_std::Deps;
use ics008_wasm_client::{
    storage_utils::{
        read_client_state, read_consensus_state, save_client_state, save_consensus_state,
    },
    IbcClient, IbcClientError, StorageState,
};
use unionlabs::{
    aptos::{
        account::AccountAddress, storage_proof::StorageProof, transaction_info::TransactionInfo,
    },
    cosmwasm::wasm::union::custom_query::UnionCustomQuery,
    encoding::{DecodeAs, Proto},
    hash::H256,
    ibc::{
        core::{client::height::Height, commitment::merkle_path::MerklePath},
        lightclients::{
            movement::{
                client_state::ClientState, consensus_state::ConsensusState, header::Header,
            },
            wasm,
        },
    },
};

use crate::errors::Error;

type WasmClientState = wasm::client_state::ClientState<ClientState>;
type WasmConsensusState = wasm::consensus_state::ConsensusState<ConsensusState>;
// type WasmL1ConsensusState =
//     wasm::consensus_state::ConsensusState<ethereum::consensus_state::ConsensusState>;

// #[derive(rlp::RlpEncodable)]
// struct BlockCommitment {
//     pub height: U256,
//     pub commitment: U256,
//     pub block_id: U256,
// }

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
        deps: Deps<Self::CustomQuery>,
        height: Height,
        _delay_time_period: u64,
        _delay_block_period: u64,
        proof: Vec<u8>,
        mut path: MerklePath,
        value: StorageState,
    ) -> Result<(), IbcClientError<Self>> {
        #[cfg(feature = "union-movement")]
        {
            let consensus_state: WasmConsensusState = read_consensus_state(deps, &height)?
                .ok_or(Error::ConsensusStateNotFound(height))?;
            let client_state: WasmClientState = read_client_state(deps)?;

            let path = path.key_path.pop().ok_or(Error::EmptyIbcPath)?;

            match value {
                StorageState::Occupied(value) => do_verify_membership(
                    path,
                    consensus_state.data.state_root,
                    client_state.data.table_handle,
                    proof,
                    value,
                )?,
                StorageState::Empty => unimplemented!(),
            }
        }
        Ok(())
    }

    fn verify_header(
        deps: Deps<Self::CustomQuery>,
        env: cosmwasm_std::Env,
        header: Header,
    ) -> Result<(), IbcClientError<Self>> {
        // NOTE(aeryz): FOR AUDITORS and NERDS:
        // Movement's current REST API's don't provide state and transaction proofs. We added those to our custom
        // Movement node which we also work on getting them to be upstreamed. Hence, we use the following feature-flag with
        // a custom setup.
        // Also see the related PR: https://github.com/movementlabsxyz/movement/pull/645

        #[cfg(feature = "union-movement")]
        {
            aptos_verifier::verify_tx_state(
                &header.tx_proof,
                *header
                    .state_proof
                    .latest_ledger_info()
                    .commit_info
                    .executed_state_id
                    .get(),
                header.tx_index,
            )
            .map_err(Into::<Error>::into)?;

            // TODO(aeryz): make sure the given state_proof_hash_proof.key matches the correct slot

            let client_state: WasmClientState = read_client_state(deps)?;

            let l1_consensus_state = query_consensus_state::<WasmL1ConsensusState>(
                deps,
                &env,
                client_state.data.l1_client_id.to_string(),
                header.l1_height,
            )
            .map_err(Error::CustomQuery)?;

            let expected_commitment = BlockCommitment {
                height: header.new_height.into(),
                commitment: U256::from_be_bytes(header.state_proof.hash()),
                block_id: U256::from_be_bytes(
                    header.state_proof.latest_ledger_info().commit_info.id.0,
                ),
            };

            ethereum_verifier::verify::verify_account_storage_root(
                l1_consensus_state.data.state_root,
                &client_state.data.l1_contract_address,
                &header.settlement_contract_proof.proof,
                &header.settlement_contract_proof.storage_root,
            )
            .unwrap();

            ethereum_verifier::verify::verify_storage_proof(
                header.settlement_contract_proof.storage_root,
                header.state_proof_hash_proof.key,
                &rlp::encode(&expected_commitment),
                header.state_proof_hash_proof.proof,
            )
            .unwrap();
        }
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
            state_root: H256::new(*tx_info.state_checkpoint_hash.unwrap().get()), // TODO(aeryz): we always need this, no need to make this not an option
            timestamp: header
                .state_proof
                .latest_ledger_info()
                .commit_info
                .timestamp_usecs,
            state_proof_hash: H256::default(), // TODO(aeryz): im not sure if we need this
        };

        if header.new_height > client_state.data.latest_block_num {
            client_state.data.latest_block_num = header.new_height;
            *client_state.latest_height.height_mut() = header.new_height;
            save_client_state::<MovementLightClient>(deps.branch(), client_state);
        }

        let update_height = Height::new(header.new_height);

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
        deps: Deps<Self::CustomQuery>,
        height: Height,
    ) -> Result<u64, IbcClientError<Self>> {
        Ok(read_consensus_state::<Self>(deps, &height)?
            .ok_or(Error::ConsensusStateNotFound(height))?
            .data
            .timestamp)
    }
}

fn do_verify_membership(
    path: String,
    state_root: H256,
    table_handle: AccountAddress,
    proof: Vec<u8>,
    value: Vec<u8>,
) -> Result<(), IbcClientError<MovementLightClient>> {
    let proof = StorageProof::decode_as::<Proto>(&proof).map_err(Error::StorageProofDecode)?;

    let Some(proof_value) = &proof.state_value else {
        return Err(Error::MembershipProofWithoutValue.into());
    };

    // `aptos_std::table` stores the value as bcs encoded
    let given_value = bcs::to_bytes(&value).expect("cannot fail");
    if proof_value.data() != given_value {
        return Err(Error::ProofValueMismatch(proof_value.data().to_vec(), given_value).into());
    }

    let Some(proof_leaf) = proof.proof.leaf.as_ref() else {
        return Err(Error::MembershipProofWithoutValue.into());
    };

    if aptos_verifier::hash_state_value(proof_value) != *proof_leaf.value_hash.get() {
        return Err(Error::ProofValueHashMismatch.into());
    }

    let key = aptos_verifier::hash_table_key(
        &bcs::to_bytes(path.as_bytes()).expect("cannot fail"),
        &table_handle,
    );

    if key != *proof_leaf.key.get() {
        return Err(Error::ProofKeyMismatch.into());
    }

    Ok(
        aptos_verifier::verify_membership(proof.proof, state_root.into())
            .map_err(Into::<Error>::into)?,
    )
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename = "StateValue")]
enum PersistedStateValue {
    V0(Vec<u8>),
    WithMetadata {
        data: Vec<u8>,
        metadata: PersistedStateValueMetadata,
    },
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename = "StateValueMetadata")]
pub enum PersistedStateValueMetadata {
    V0 {
        deposit: u64,
        creation_time_usecs: u64,
    },
    V1 {
        slot_deposit: u64,
        bytes_deposit: u64,
        creation_time_usecs: u64,
    },
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use unionlabs::{
        encoding::{DecodeAs, Proto},
        ibc::core::channel::channel::Channel,
    };

    #[test]
    fn test_proto() {
        let channel_end = hex!("6d080110011a470a457761736d2e756e696f6e3134686a32746176713866706573647778786375343472747933686839307668756a7276636d73746c347a723374786d6676773973336539666532220c636f6e6e656374696f6e2d302a1075637330302d70696e67706f6e672d31");
        println!(
            "end 1: {:?}",
            Channel::decode_as::<Proto>(&channel_end).unwrap()
        );
    }
}
