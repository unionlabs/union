use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockIdFlag {
    Unknown = 0,
    Absent = 1,
    Commit = 2,
    Nil = 3,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::errors::UnknownEnumVariant;

    use crate::types::block_id_flag::BlockIdFlag;

    impl From<BlockIdFlag> for protos::tendermint::types::BlockIdFlag {
        fn from(value: BlockIdFlag) -> Self {
            match value {
                BlockIdFlag::Unknown => Self::Unknown,
                BlockIdFlag::Absent => Self::Absent,
                BlockIdFlag::Commit => Self::Commit,
                BlockIdFlag::Nil => Self::Nil,
            }
        }
    }

    impl From<BlockIdFlag> for i32 {
        fn from(value: BlockIdFlag) -> Self {
            protos::tendermint::types::BlockIdFlag::from(value).into()
        }
    }

    impl From<protos::tendermint::types::BlockIdFlag> for BlockIdFlag {
        fn from(value: protos::tendermint::types::BlockIdFlag) -> Self {
            match value {
                protos::tendermint::types::BlockIdFlag::Unknown => Self::Unknown,
                protos::tendermint::types::BlockIdFlag::Absent => Self::Absent,
                protos::tendermint::types::BlockIdFlag::Commit => Self::Commit,
                protos::tendermint::types::BlockIdFlag::Nil => Self::Nil,
            }
        }
    }

    impl TryFrom<i32> for BlockIdFlag {
        type Error = UnknownEnumVariant<i32>;

        fn try_from(value: i32) -> Result<Self, Self::Error> {
            Ok(
                match protos::tendermint::types::BlockIdFlag::try_from(value)
                    .map_err(|_| UnknownEnumVariant(value))?
                {
                    protos::tendermint::types::BlockIdFlag::Unknown => Self::Unknown,
                    protos::tendermint::types::BlockIdFlag::Absent => Self::Absent,
                    protos::tendermint::types::BlockIdFlag::Commit => Self::Commit,
                    protos::tendermint::types::BlockIdFlag::Nil => Self::Nil,
                },
            )
        }
    }
}
