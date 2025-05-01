use cosmwasm_std::{testing::mock_dependencies, Api, HashFunction, BLS12_381_G2_GENERATOR};
use serde::{Deserialize, Serialize};

use crate::{
    checkpoint_summary::CheckpointSummary,
    crypto::{AuthorityPublicKeyBytes, AuthorityStrongQuorumSignInfo, BLS_DST},
    Intent, IntentMessage, U64,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Committee {
    pub epoch: U64,
    pub voting_rights: Vec<(AuthorityPublicKeyBytes, U64)>,
}

impl Committee {
    pub fn verify_signature(
        &self,
        checkpoint: CheckpointSummary,
        sign_info: AuthorityStrongQuorumSignInfo,
    ) {
        let mut selected_public_keys = vec![];

        let mut seen = std::collections::BTreeSet::new();

        for authority_index in &sign_info.signers_map {
            if !seen.insert(authority_index) {
                continue;
            }

            selected_public_keys.push(self.voting_rights[authority_index as usize].0.clone());
        }

        let intent_msg = IntentMessage {
            intent: Intent {
                scope: crate::IntentScope::CheckpointSummary,
                version: crate::IntentVersion::V0,
                app_id: crate::AppId::Sui,
            },
            value: checkpoint.clone(),
        };
        let mut intent_msg_bytes =
            bcs::to_bytes(&intent_msg).expect("Message serialization should not fail");
        bcs::serialize_into(&mut intent_msg_bytes, &checkpoint.epoch)
            .expect("Message serialization should not fail");

        let pubkeys = selected_public_keys
            .into_iter()
            .flat_map(|x| x.0)
            .collect::<Vec<u8>>();

        let deps = mock_dependencies();
        let aggregate_pubkey = deps.api.bls12_381_aggregate_g2(&pubkeys).unwrap();

        let hashed_msg = deps
            .api
            .bls12_381_hash_to_g1(HashFunction::Sha256, &intent_msg_bytes, BLS_DST)
            .unwrap();

        let valid = deps
            .api
            .bls12_381_pairing_equality(
                sign_info.signature.0.as_ref(),
                &BLS12_381_G2_GENERATOR,
                &hashed_msg,
                &aggregate_pubkey,
            )
            .unwrap();

        panic!("valid? {}", valid);
    }
}
