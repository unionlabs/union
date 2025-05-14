use sui_light_client_types::{
    checkpoint_summary::CheckpointSummary,
    committee::Committee,
    crypto::{
        AggregateAuthoritySignature, AuthorityPublicKeyBytes, AuthorityStrongQuorumSignInfo,
        BLS_DST,
    },
    object::ObjectInner,
    transaction_effects::TransactionEffects,
    AppId, Intent, IntentMessage, IntentScope, IntentVersion,
};

mod error;

pub use error::*;
use unionlabs_primitives::Bytes;

pub trait SignatureVerification {
    type Error: Into<error::Error>;

    const BLS_DST: &[u8] = BLS_DST;

    const BLS_GENERATOR: [u8; 96] = [
        147, 224, 43, 96, 82, 113, 159, 96, 125, 172, 211, 160, 136, 39, 79, 101, 89, 107, 208,
        208, 153, 32, 182, 26, 181, 218, 97, 187, 220, 127, 80, 73, 51, 76, 241, 18, 19, 148, 93,
        87, 229, 172, 125, 5, 93, 4, 43, 126, 2, 74, 162, 178, 240, 143, 10, 145, 38, 8, 5, 39, 45,
        197, 16, 81, 198, 228, 122, 212, 250, 64, 59, 2, 180, 81, 11, 100, 122, 227, 209, 119, 11,
        172, 3, 38, 168, 5, 187, 239, 212, 128, 86, 200, 193, 33, 189, 184,
    ];

    /// verify bls12_381 signature
    ///
    /// - `msg` must be hashed to g1 using Sha256 and Self::BLS_DST
    /// - `signature` must be checked with `BLS_GENERATOR`
    fn verify_signature(
        &self,
        public_keys: &[AuthorityPublicKeyBytes],
        msg: &[u8],
        signature: &AggregateAuthoritySignature,
    ) -> Result<(), Self::Error>;
}

pub fn verify_checkpoint<V: SignatureVerification>(
    committee: &Committee,
    checkpoint: &CheckpointSummary,
    sign_info: &AuthorityStrongQuorumSignInfo,
    signature_verifier: &V,
) -> Result<(), Error> {
    let mut selected_public_keys = vec![];

    let mut seen = std::collections::BTreeSet::new();

    for authority_index in sign_info.signers_map.0.iter() {
        if !seen.insert(authority_index) {
            continue;
        }

        selected_public_keys.push(committee.voting_rights[authority_index as usize].0.clone());
    }

    let intent_msg = IntentMessage {
        intent: Intent {
            scope: IntentScope::CheckpointSummary,
            version: IntentVersion::V0,
            app_id: AppId::Sui,
        },
        value: checkpoint.clone(),
    };
    let mut intent_msg_bytes =
        bcs::to_bytes(&intent_msg).expect("Message serialization should not fail");
    bcs::serialize_into(&mut intent_msg_bytes, &checkpoint.epoch)
        .expect("Message serialization should not fail");

    signature_verifier
        .verify_signature(
            &selected_public_keys,
            &intent_msg_bytes,
            &sign_info.signature,
        )
        .map_err(Into::into)
}

pub fn verify_membership(
    key: Bytes,
    value: Bytes,
    object: ObjectInner,
    effects: TransactionEffects,
) -> Result<(), Error> {
    Ok(())
}
