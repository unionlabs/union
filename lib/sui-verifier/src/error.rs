use sui_light_client_types::{digest::Digest, ObjectID};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Client(Box<dyn core::error::Error>),

    #[error("proven object ({proven}) does not match the given ({given})")]
    ObjectMismatch {
        pub given: ObjectID,
        pub proven: ObjectID,
    },

    #[error("proven key ({proven}) does not match the given ({given})")]
    KeyMismatch { pub given: Bytes, pub proven: Bytes },

    #[error("proven value ({proven}) does not match the given ({given})")]
    ValueMismatch { pub given: Bytes, pub proven: Bytes },

    #[error("an effect to object ({object}) not found in the given effects")]
    EffectNotFound(ObjectID),

    #[error(
        "proven object's digest ({proven}) does not match the given object's digest ({given})"
    )]
    ObjectDigestMismatch {
        pub given: Digest,
        pub proven: Digest,
    },
}
