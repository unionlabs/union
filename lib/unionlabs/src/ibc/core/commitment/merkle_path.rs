use macros::model;

#[model(proto(raw(protos::ibc::core::commitment::v1::MerklePath), into, from))]
pub struct MerklePath {
    pub key_path: Vec<String>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::ibc::core::commitment::merkle_path::MerklePath;

    impl From<MerklePath> for protos::ibc::core::commitment::v1::MerklePath {
        fn from(value: MerklePath) -> Self {
            Self {
                key_path: value.key_path,
            }
        }
    }

    impl From<protos::ibc::core::commitment::v1::MerklePath> for MerklePath {
        fn from(value: protos::ibc::core::commitment::v1::MerklePath) -> Self {
            Self {
                key_path: value.key_path,
            }
        }
    }
}
