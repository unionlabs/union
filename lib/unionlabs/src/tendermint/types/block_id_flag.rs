use crate::macros::wrapper_enum;

wrapper_enum! {
    #[proto(protos::tendermint::types::BlockIdFlag)]
    pub enum BlockIdFlag {
        Unknown = 0,
        Absent = 1,
        Commit = 2,
        Nil = 3,
    }
}

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
