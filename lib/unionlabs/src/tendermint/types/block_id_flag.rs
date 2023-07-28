use crate::errors::UnknownEnumVariant;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BlockIdFlag {
    Unknown = 0,
    Absent = 1,
    Commit = 2,
    Nil = 3,
}

impl TryFrom<u8> for BlockIdFlag {
    type Error = UnknownEnumVariant<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        i32::from(value)
            .try_into()
            .map_err(|_| UnknownEnumVariant(value))
    }
}

impl TryFrom<i32> for BlockIdFlag {
    type Error = UnknownEnumVariant<i32>;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        protos::tendermint::types::BlockIdFlag::from_i32(value)
            .ok_or(UnknownEnumVariant(value))
            .map(Into::into)
    }
}

impl From<protos::tendermint::types::BlockIdFlag> for BlockIdFlag {
    fn from(value: protos::tendermint::types::BlockIdFlag) -> Self {
        match value {
            protos::tendermint::types::BlockIdFlag::Unknown => BlockIdFlag::Unknown,
            protos::tendermint::types::BlockIdFlag::Absent => BlockIdFlag::Absent,
            protos::tendermint::types::BlockIdFlag::Commit => BlockIdFlag::Commit,
            protos::tendermint::types::BlockIdFlag::Nil => BlockIdFlag::Nil,
        }
    }
}

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

// impl TryFromProto for BlockIdFlag {
//     type Proto = protos::tendermint::types::BlockIdFlag;
// }

// impl IntoProto for BlockIdFlag {
//     type Proto = protos::tendermint::types::BlockIdFlag;
// }

// impl TypeUrl for protos::tendermint::types::BlockIdFlag {
//     const TYPE_URL: &'static str = "/tendermint.types.BlockIdFlag";
// }

// #[test]
// #[cfg(test)]
// fn proto_roundtrip() {
//     crate::assert_proto_roundtrip(&BlockIdFlag::Unknown);
//     crate::assert_proto_roundtrip(&BlockIdFlag::Absent);
//     crate::assert_proto_roundtrip(&BlockIdFlag::Commit);
//     crate::assert_proto_roundtrip(&BlockIdFlag::Nil);
// }
