use sui_light_client_types::{Digest, ObjectID};
use unionlabs_primitives::Bytes;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Client(Box<dyn core::error::Error>),

    #[error(
        "proven object ({proven}) does not match the given ({given}) ({commitments_object}, {key})"
    )]
    ObjectMismatch {
        given: ObjectID,
        proven: ObjectID,
        commitments_object: Bytes,
        key: Bytes,
    },

    #[error("proven key ({proven}) does not match the given ({given})")]
    KeyMismatch { given: Bytes, proven: Bytes },

    #[error("proven value ({proven}) does not match the given ({given})")]
    ValueMismatch { given: Bytes, proven: Bytes },

    #[error("an effect to object ({0}) not found in the given effects")]
    EffectNotFound(ObjectID),

    #[error(
        "proven object's digest ({proven}) does not match the given object's digest ({given})"
    )]
    ObjectDigestMismatch { given: Digest, proven: Digest },

    #[error(
        "proven contents digest ({proven}) does not match the given contents digest ({given})"
    )]
    ContentsDigestMismatch { given: Digest, proven: Digest },
}
