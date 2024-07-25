use ibc_vm_rs::{IbcQuery, IbcResponse, Status};
use ics23::ibc_api::SDK_SPECS;
use near_sdk::{
    borsh::{BorshDeserialize, BorshSerialize},
    env, near_bindgen,
    store::LookupMap,
    PanicOnDefault,
};
use unionlabs::{
    encoding::{DecodeAs, EncodeAs as _, Proto},
    ibc::{
        core::{
            client::height::Height,
            commitment::{
                merkle_path::MerklePath, merkle_proof::MerkleProof, merkle_root::MerkleRoot,
            },
        },
        lightclients::cometbls::{
            client_state::ClientState, consensus_state::ConsensusState, header::Header,
        },
    },
    id::ClientId,
};

use crate::{error::Error, verifier::verify_zkp};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    consensus_states: LookupMap<String, LookupMap<Height, ConsensusState>>,
    client_states: LookupMap<String, ClientState>,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            consensus_states: LookupMap::new(0),
            client_states: LookupMap::new(1),
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn new(
        &mut self,
        #[allow(unused)] client_id: ClientId,
        client_state: Vec<u8>,
        consensus_state: Vec<u8>,
    ) {
        let client_state = ClientState::decode_as::<Proto>(&client_state).unwrap();
        let consensus_state = ConsensusState::decode_as::<Proto>(&consensus_state).unwrap();

        if self.client_states.contains_key(&client_id.to_string()) {
            env::panic_str("already have the client");
        }

        // TODO(aeryz): we might wanna store a cursor for this to reduce storage
        let mut inner_consensus_states: LookupMap<Height, ConsensusState> =
            LookupMap::new(client_id.to_string().as_bytes());
        inner_consensus_states.insert(client_state.latest_height, consensus_state);

        self.consensus_states
            .insert(client_id.to_string(), inner_consensus_states);

        self.client_states
            .insert(client_id.to_string(), client_state);
    }

    pub fn query(&self, client_id: ClientId, query: Vec<IbcQuery>) -> Vec<IbcResponse> {
        query
            .into_iter()
            .map(|q| match q {
                IbcQuery::Status => IbcResponse::Status {
                    status: self.status(),
                },
                IbcQuery::LatestHeight => IbcResponse::LatestHeight {
                    height: self.latest_height(&client_id),
                },
                IbcQuery::VerifyMembership {
                    height,
                    delay_time_period,
                    delay_block_period,
                    proof,
                    path,
                    value,
                } => IbcResponse::VerifyMembership {
                    valid: self.verify_membership(
                        &client_id,
                        height,
                        delay_time_period,
                        delay_block_period,
                        proof,
                        path,
                        value,
                    ),
                },
                IbcQuery::VerifyClientMessage(msg) => IbcResponse::VerifyClientMessage {
                    valid: self.verify_client_message(&client_id, msg),
                },
                IbcQuery::CheckForMisbehaviour(msg) => IbcResponse::CheckForMisbehaviour {
                    misbehaviour_found: self.check_for_misbehaviour(msg),
                },
                IbcQuery::TimestampAtHeight(_) => IbcResponse::TimestampAtHeight { timestamp: 100 },
            })
            .collect()
    }

    pub fn status(&self) -> Status {
        Status::Active
    }

    fn latest_height(&self, client_id: &str) -> Height {
        self.client_states.get(client_id).unwrap().latest_height
    }

    #[allow(unused)]
    fn verify_membership(
        &self,
        client_id: &str,
        height: Height,
        // TODO(aeryz): delay times might not be relevant for other chains we could make it optional
        delay_time_period: u64,
        delay_block_period: u64,
        proof: Vec<u8>,
        path: MerklePath,
        value: Vec<u8>,
        // TODO(aeryz): make this a proper error type this is stupid
    ) -> bool {
        let Ok(consensus_state) = self
            .consensus_states
            .get(client_id)
            .unwrap()
            .get(&height)
            .ok_or(Error::ConsensusStateNotFound(height))
        else {
            return false;
        };

        let Ok(merkle_proof) =
            MerkleProof::decode_as::<Proto>(proof.as_ref()).map_err(Error::MerkleProofDecode)
        else {
            return false;
        };

        ics23::ibc_api::verify_membership(
            &merkle_proof,
            &SDK_SPECS,
            &consensus_state.app_hash,
            &path
                .key_path
                .into_iter()
                .map(|x| x.into_bytes())
                .collect::<Vec<_>>(),
            value,
        )
        .is_ok()
    }

    // TODO(aeryz): client_msg can be Misbehaviour or Header
    fn verify_client_message(&self, client_id: &str, client_msg: Vec<u8>) -> bool {
        let header = Header::decode_as::<Proto>(&client_msg).unwrap();
        let client_state = self.client_states.get(client_id).unwrap();
        let consensus_states = self.consensus_states.get(client_id).unwrap();
        let consensus_state = consensus_states.get(&header.trusted_height).unwrap();

        // SAFETY: height is bound to be 0..i64::MAX which makes it within the bounds of u64
        let untrusted_height_number = header.signed_header.height.inner() as u64;
        let trusted_height_number = header.trusted_height.revision_height;

        if untrusted_height_number <= trusted_height_number {
            // return Err(InvalidHeaderError::SignedHeaderHeightMustBeMoreRecent {
            //     signed_height: untrusted_height_number,
            //     trusted_height: trusted_height_number,
            // }
            // .into());
            return false;
        }

        let trusted_timestamp = consensus_state.timestamp;
        // Normalized to nanoseconds to follow tendermint convention
        let untrusted_timestamp = header.signed_header.time.as_unix_nanos();

        if untrusted_timestamp <= trusted_timestamp {
            // return Err(InvalidHeaderError::SignedHeaderTimestampMustBeMoreRecent {
            //     signed_timestamp: untrusted_timestamp,
            //     trusted_timestamp,
            // }
            // .into());
            return false;
        }

        if is_client_expired(
            untrusted_timestamp,
            client_state.trusting_period,
            env::block_timestamp(),
        ) {
            // return Err(InvalidHeaderError::HeaderExpired(consensus_state.data.timestamp).into());
            return false;
        }

        let max_clock_drift = env::block_timestamp()
            .checked_add(client_state.max_clock_drift)
            .unwrap();
        // .ok_or(Error::MathOverflow)?;

        if untrusted_timestamp >= max_clock_drift {
            // return Err(InvalidHeaderError::SignedHeaderCannotExceedMaxClockDrift {
            //     signed_timestamp: untrusted_timestamp,
            //     max_clock_drift,
            // }
            // .into());
            return false;
        }

        let trusted_validators_hash = consensus_state.next_validators_hash;

        if untrusted_height_number == trusted_height_number + 1
            && header.signed_header.validators_hash != trusted_validators_hash
        {
            // return Err(InvalidHeaderError::InvalidValidatorsHash {
            //     expected: trusted_validators_hash,
            //     actual: header.signed_header.validators_hash,
            // }
            // .into());
            return false;
        }

        verify_zkp(
            &client_state.chain_id,
            trusted_validators_hash,
            &header.signed_header,
            header.zero_knowledge_proof,
        )
        .unwrap();

        true
    }

    #[allow(unused)]
    pub fn check_for_misbehaviour(&self, client_msg: Vec<u8>) -> bool {
        false
    }

    pub fn test_circuit(
        &self,
        chain_id: String,
        trusted_validators_hash: unionlabs::hash::H256,
        header: unionlabs::ibc::lightclients::cometbls::light_header::LightHeader,
        zkp: Vec<u8>,
    ) {
        verify_zkp(&chain_id, trusted_validators_hash, &header, zkp).unwrap()
    }

    pub fn update_client(
        &mut self,
        client_id: String,
        client_msg: Vec<u8>,
    ) -> (Vec<u8>, Vec<(Height, Vec<u8>)>) {
        let header = Header::decode_as::<Proto>(&client_msg).unwrap();

        let untrusted_height = Height {
            revision_number: header.trusted_height.revision_number,
            revision_height: header.signed_header.height.inner() as u64,
        };

        let client_state = self.client_states.get_mut(&client_id).unwrap();

        if untrusted_height > client_state.latest_height {
            client_state.latest_height = untrusted_height;
        }

        let consensus_state = ConsensusState {
            timestamp: header.signed_header.time.as_unix_nanos(),
            app_hash: MerkleRoot {
                hash: header.signed_header.app_hash,
            },
            next_validators_hash: header.signed_header.next_validators_hash,
        };

        // TODO(aeryz): handle metadata
        // save_consensus_state_metadata(
        //     deps.branch(),
        //     consensus_state.data.timestamp,
        //     untrusted_height,
        // );
        self.consensus_states
            .get_mut(&client_id)
            .unwrap()
            .insert(untrusted_height, consensus_state.clone());

        (
            client_state.clone().encode_as::<Proto>(),
            vec![(untrusted_height, consensus_state.encode_as::<Proto>())],
        )
    }

    #[allow(unused)]
    pub fn update_client_on_misbehaviour(&mut self, client_msg: Vec<u8>) {}
}

fn is_client_expired(
    consensus_state_timestamp: u64,
    trusting_period: u64,
    current_block_time: u64,
) -> bool {
    if let Some(sum) = consensus_state_timestamp.checked_add(trusting_period) {
        sum < current_block_time
    } else {
        true
    }
}
