use cosmwasm_std::{Addr, Deps};
use ethereum_light_client_types::consensus_state::ConsensusState as EthereumConsensusState;
use movement_light_client_types::{
    client_state::ClientState, consensus_state::ConsensusState, header::Header,
};
use union_ibc::state::CLIENT_CONSENSUS_STATES;
use unionlabs::{
    aptos::{
        account::AccountAddress, storage_proof::StorageProof, transaction_info::TransactionInfo,
    },
    cosmwasm::wasm::union::custom_query::UnionCustomQuery,
    encoding::{DecodeAs, Proto},
    hash::H256,
    uint::U256,
};

use crate::{error::Error, state::IBC_HOST};

#[derive(rlp::RlpEncodable)]
struct BlockCommitment {
    pub height: U256,
    pub commitment: U256,
    pub block_id: U256,
}

pub fn verify_header(
    client_state: ClientState,
    // TODO(aeryz): why are we not using this?
    _consensus_state: ConsensusState,
    deps: Deps<UnionCustomQuery>,
    _env: cosmwasm_std::Env,
    header: Header,
) -> Result<(u64, ClientState, ConsensusState), Error> {
    // NOTE(aeryz): FOR AUDITORS and NERDS:
    // Movement's current REST API's don't provide state and transaction proofs. We added those to our custom
    // Movement node which we also work on getting them to be upstreamed. Hence, we use the following feature-flag with
    // a custom setup.
    // Also see the related PR: https://github.com/movementlabsxyz/movement/pull/645

    // #[cfg(feature = "union-movement")]
    // {
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

    let l1_consensus_state = read_l1_consensus_state(
        deps,
        &IBC_HOST.load(deps.storage)?,
        client_state.l1_client_id,
        header.l1_height,
    )?;

    let expected_commitment = BlockCommitment {
        height: header.new_height.into(),
        commitment: U256::from_be_bytes(header.state_proof.hash()),
        // TODO(aeryz): check if hash here is big endian
        block_id: U256::from_be_bytes(
            header
                .state_proof
                .latest_ledger_info()
                .commit_info
                .id
                .into(),
        ),
    };

    evm_storage_verifier::verify_account_storage_root(
        l1_consensus_state.state_root,
        &client_state.l1_contract_address,
        &header.settlement_contract_proof.proof,
        &header.settlement_contract_proof.storage_root,
    )
    .unwrap();

    evm_storage_verifier::verify_storage_proof(
        header.settlement_contract_proof.storage_root,
        header.state_proof_hash_proof.key,
        &rlp::encode(&expected_commitment),
        &header.state_proof_hash_proof.proof,
    )
    .unwrap();
    // }
    // #[cfg(not(feature = "union-movement"))]
    // {
    //     let _ = (deps, env, header);
    // }
    update_state(client_state, header)
}

fn update_state(
    mut client_state: ClientState,
    header: Header,
) -> Result<(u64, ClientState, ConsensusState), Error> {
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

    if header.new_height > client_state.latest_block_num {
        client_state.latest_block_num = header.new_height;
    }

    Ok((header.new_height, client_state, consensus_state))
}

// #[cfg(feature = "union-movement")]
pub fn verify_membership(
    path: &[u8],
    state_root: H256,
    table_handle: AccountAddress,
    proof: StorageProof,
    value: &[u8],
) -> Result<(), Error> {
    let Some(proof_value) = &proof.state_value else {
        return Err(Error::MembershipProofWithoutValue);
    };

    // `aptos_std::table` stores the value as bcs encoded
    let given_value = bcs::to_bytes(&value).expect("cannot fail");
    if proof_value.data() != &given_value {
        return Err(Error::ProofValueMismatch(proof_value.data().to_vec(), given_value).into());
    }

    let Some(proof_leaf) = proof.proof.leaf.as_ref() else {
        return Err(Error::MembershipProofWithoutValue);
    };

    if aptos_verifier::hash_state_value(proof_value) != *proof_leaf.value_hash.get() {
        return Err(Error::ProofValueHashMismatch);
    }

    let key =
        aptos_verifier::hash_table_key(&bcs::to_bytes(path).expect("cannot fail"), &table_handle);

    if key != *proof_leaf.key.get() {
        return Err(Error::ProofKeyMismatch);
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

fn read_l1_consensus_state(
    deps: Deps<UnionCustomQuery>,
    ibc_host: &Addr,
    client_id: u32,
    height: u64,
) -> Result<EthereumConsensusState, Error> {
    let consensus_state = deps
        .querier
        .query_wasm_raw(
            ibc_host.to_string(),
            CLIENT_CONSENSUS_STATES.key((client_id, height)).to_vec(),
        )?
        .unwrap();
    Ok(EthereumConsensusState::decode_as::<Proto>(&consensus_state).unwrap())
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
