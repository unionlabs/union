use crate::macros::wrapper_enum;

wrapper_enum! {
    #[model(proto(protos::tendermint::types::BlockIdFlag))]
    pub enum BlockIdFlag {
        Unknown = 0,
        Absent = 1,
        Commit = 2,
        Nil = 3,
    }
}
