use serde::{Deserialize, Serialize};

use crate::header::Header;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Misbehaviour {
    pub header_a: Header,
    pub header_b: Header,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{errors::MissingField, impl_proto_via_try_from_into, required};

    use crate::{header, misbehaviour::Misbehaviour};

    impl_proto_via_try_from_into!(Misbehaviour => protos::union::ibc::lightclients::cometbls::v1::Misbehaviour);

    impl From<Misbehaviour> for protos::union::ibc::lightclients::cometbls::v1::Misbehaviour {
        fn from(value: Misbehaviour) -> Self {
            Self {
                header_a: Some(value.header_a.into()),
                header_b: Some(value.header_b.into()),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum TryFromMisbehaviourError {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid signed header")]
        Header(#[from] header::proto::Error),
    }

    impl TryFrom<protos::union::ibc::lightclients::cometbls::v1::Misbehaviour> for Misbehaviour {
        type Error = TryFromMisbehaviourError;

        fn try_from(
            value: protos::union::ibc::lightclients::cometbls::v1::Misbehaviour,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                header_a: required!(value.header_a)?.try_into()?,
                header_b: required!(value.header_b)?.try_into()?,
            })
        }
    }
}
