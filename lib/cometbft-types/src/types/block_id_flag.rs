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

    impl From<BlockIdFlag> for protos::cometbft::types::v1::BlockIdFlag {
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
            protos::cometbft::types::v1::BlockIdFlag::from(value).into()
        }
    }

    impl From<protos::cometbft::types::v1::BlockIdFlag> for BlockIdFlag {
        fn from(value: protos::cometbft::types::v1::BlockIdFlag) -> Self {
            match value {
                protos::cometbft::types::v1::BlockIdFlag::Unknown => Self::Unknown,
                protos::cometbft::types::v1::BlockIdFlag::Absent => Self::Absent,
                protos::cometbft::types::v1::BlockIdFlag::Commit => Self::Commit,
                protos::cometbft::types::v1::BlockIdFlag::Nil => Self::Nil,
            }
        }
    }

    impl TryFrom<i32> for BlockIdFlag {
        type Error = UnknownEnumVariant<i32>;

        fn try_from(value: i32) -> Result<Self, Self::Error> {
            Ok(
                match protos::cometbft::types::v1::BlockIdFlag::try_from(value)
                    .map_err(|_| UnknownEnumVariant(value))?
                {
                    protos::cometbft::types::v1::BlockIdFlag::Unknown => Self::Unknown,
                    protos::cometbft::types::v1::BlockIdFlag::Absent => Self::Absent,
                    protos::cometbft::types::v1::BlockIdFlag::Commit => Self::Commit,
                    protos::cometbft::types::v1::BlockIdFlag::Nil => Self::Nil,
                },
            )
        }
    }
}
