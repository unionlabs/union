use cosmwasm_std::Empty;
use ibc_union_light_client::IbcClientError;
use ibc_union_msg::lightclient::{Status, VerifyCreationResponseEvent};
use movement_light_client_types::{
    client_state::ClientState, consensus_state::ConsensusState, header::Header,
};
use unionlabs::{
    aptos::{
        account::AccountAddress, storage_proof::StorageProof, transaction_info::TransactionInfo,
    },
    encoding::Bincode,
    primitives::{H256, U256},
};

use crate::error::Error;

pub enum MovementLightClient {}

#[derive(rlp::RlpEncodable)]
pub struct BlockCommitment {
    pub height: U256,
    pub commitment: U256,
    pub block_id: U256,
}

impl ibc_union_light_client::IbcClient for MovementLightClient {
    type Error = Error;

    type CustomQuery = Empty;

    type Header = Header;

    type Misbehaviour = Header;

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    type StorageProof = StorageProof;

    type Encoding = Bincode;

    fn verify_membership(
        _ctx: ibc_union_light_client::IbcClientCtx<Self>,
        _height: u64,
        _key: Vec<u8>,
        _storage_proof: Self::StorageProof,
        _value: Vec<u8>,
    ) -> Result<(), ibc_union_light_client::IbcClientError<Self>> {
        // let client_state = ctx.read_self_client_state()?;
        // let consensus_state = ctx.read_self_consensus_state(height)?;
        // verify_membership(
        //     &key,
        //     consensus_state.state_root,
        //     client_state.table_handle,
        //     storage_proof,
        //     &value,
        // )
        // .map_err(Into::into)
        Ok(())
    }

    fn verify_non_membership(
        _ctx: ibc_union_light_client::IbcClientCtx<Self>,
        _height: u64,
        _key: Vec<u8>,
        _storage_proof: Self::StorageProof,
    ) -> Result<(), ibc_union_light_client::IbcClientError<Self>> {
        unimplemented!()
    }

    fn get_timestamp(consensus_state: &Self::ConsensusState) -> u64 {
        consensus_state.timestamp
    }

    fn get_latest_height(client_state: &Self::ClientState) -> u64 {
        client_state.latest_block_num
    }

    fn get_counterparty_chain_id(client_state: &Self::ClientState) -> String {
        client_state.chain_id.clone()
    }

    fn status(client_state: &Self::ClientState) -> Status {
        if client_state.frozen_height.height() != 0 {
            Status::Frozen
        } else {
            Status::Active
        }
    }

    fn verify_creation(
        _client_state: &Self::ClientState,
        _consensus_state: &Self::ConsensusState,
    ) -> Result<Option<Vec<VerifyCreationResponseEvent>>, IbcClientError<MovementLightClient>> {
        Ok(None)
    }

    fn verify_header(
        ctx: ibc_union_light_client::IbcClientCtx<Self>,
        header: Self::Header,
        caller: cosmwasm_std::Addr,
    ) -> Result<
        (u64, Self::ClientState, Self::ConsensusState),
        ibc_union_light_client::IbcClientError<Self>,
    > {
        let client_state = ctx.read_self_client_state()?;
        // Check if caller is whitelisted
        if !client_state
            .whitelisted_relayers
            .contains(&caller.to_string())
        {
            return Err(ibc_union_light_client::IbcClientError::UnauthorizedCaller(
                caller.to_string(),
            ));
        }

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

            let l1_consensus_state =
                ctx.read_consensus_state(client_state.l1_client_id, header.l1_height)?;

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
        }
        update_state(client_state, header).map_err(Into::into)
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
    if proof_value.data() != given_value {
        return Err(Error::ProofValueMismatch(
            proof_value.data().to_vec(),
            given_value,
        ));
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

    Ok(aptos_verifier::verify_membership(
        proof.proof,
        state_root.into(),
    )?)
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
        let channel_end = hex!(
            "6d080110011a470a457761736d2e756e696f6e3134686a32746176713866706573647778786375343472747933686839307668756a7276636d73746c347a723374786d6676773973336539666532220c636f6e6e656374696f6e2d302a1075637330302d70696e67706f6e672d31"
        );
        println!(
            "end 1: {:?}",
            Channel::decode_as::<Proto>(&channel_end).unwrap()
        );
    }
}
