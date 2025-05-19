use blake2::{Blake2b, Digest as _};
use sui_light_client_types::{
    checkpoint_summary::{CheckpointContents, CheckpointSummary, ExecutionDigests},
    committee::Committee,
    crypto::{
        AggregateAuthoritySignature, AuthorityPublicKeyBytes, AuthorityStrongQuorumSignInfo,
        BLS_DST,
    },
    digest::Digest,
    object::{Data, MoveObject, ObjectInner, TypeTag},
    transaction_effects::{EffectsObjectChange, ObjectOut, TransactionEffects},
    AppId, Intent, IntentMessage, IntentScope, IntentVersion, ObjectID,
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
    commitments_object: ObjectID,
    key: Bytes,
    value: Bytes,
    object: ObjectInner,
    effects: TransactionEffects,
    checkpoint_contents: CheckpointContents,
    contents_digest: Digest,
) -> Result<(), Error> {
    // STEP 1: check if the given `object` has the correct object address
    let commitment_object = calculate_dynamic_field_object_id(*commitments_object.get(), &key);

    let Data::Move(ref object_data) = object.data;
    let (proven_object, proven_key, proven_value): (ObjectID, Bytes, Bytes) =
        bcs::from_bytes(&object_data.contents).unwrap();

    if commitment_object != proven_object {
        return Err(Error::ObjectMismatch {
            given: commitment_object,
            proven: proven_object,
        });
    }

    // STEP 2: check if the given `key` and the `value` belongs to the `object`
    if key != proven_key {
        return Err(Error::KeyMismatch {
            given: key,
            proven: proven_value,
        });
    }

    if value != proven_value {
        return Err(Error::ValueMismatch {
            given: value,
            proven: proven_value,
        });
    }

    // STEP 3: find the effect in the `effects` and compare the digest with the given `object`s digest
    let digest = find_write_effect(&effects, commitment_object)
        .ok_or(Error::EffectNotFound(commitment_object))?;

    {
        let object_digest = object.digest();
        if digest != object_digest {
            return Err(Error::ObjectDigestMismatch {
                given: object_digest,
                proven: digest,
            });
        }
    }

    // STEP 4: find the effect digest in `checkpoint_contents.transactions` to verify it exists
    let CheckpointContents::V1(checkpoint_contents) = checkpoint_contents;
    let effects_digest = effects.digest();
    let _ = checkpoint_contents
        .transactions
        .iter()
        .find(|e| e.effects == effects_digest)
        .expect("execution digests do not contain the given effect");

    // STEP 5: compare the digest of `checkpoint_contents` with the `contents_digest` which is verified previously by the client
    if contents_digest != CheckpointContents::V1(checkpoint_contents.clone()).digest() {
        panic!(
            "digest mismatch: {:?} {:?}",
            contents_digest,
            CheckpointContents::V1(checkpoint_contents).digest()
        );
    }

    Ok(())
}

/// find a write effect in `effects` that effects the `object` and return the object's digest
fn find_write_effect(effects: &TransactionEffects, object: ObjectID) -> Option<Digest> {
    match effects {
        TransactionEffects::V1(_) => None,
        TransactionEffects::V2(effects) => {
            let effect = effects.changed_objects.iter().find(|eff| {
                eff.0 == object && matches!(eff.1.output_state, ObjectOut::ObjectWrite(..))
            })?;

            let ObjectOut::ObjectWrite(write) = &effect.1.output_state else {
                panic!("wut?");
            };

            Some(write.0)
        }
    }
}

/// Calculate the object_id of the dynamic field within the commitments mapping
pub fn calculate_dynamic_field_object_id(parent: [u8; 32], key_bytes: &[u8]) -> ObjectID {
    #[repr(u8)]
    enum HashingIntentScope {
        ChildObjectId = 0xf0,
        RegularObjectId = 0xf1,
    }

    // hash(parent || len(key) || key || key_type_tag)
    let mut hasher = Blake2b::<typenum::U32>::default();
    hasher.update([HashingIntentScope::ChildObjectId as u8]);
    hasher.update(parent);
    // +1 since `key_bytes` should be prefixed with its length (bcs encoding)
    hasher.update((key_bytes.len() + 1).to_le_bytes());
    // instead of calling bcs::serialize, we just prefix the bytes with the its length
    // since the table we are verifying uses `vector<u8>` keys
    hasher.update([key_bytes.len() as u8]);
    hasher.update(key_bytes);
    hasher.update(
        bcs::to_bytes(&TypeTag::Vector(Box::new(TypeTag::U8))).expect("bcs serialization works"),
    );
    let hash = hasher.finalize();

    ObjectID::new(hash.into())
}
