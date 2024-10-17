use crate::macros::wrapper_enum;

wrapper_enum! {
    #[model(proto(protos::cometbft::types::v1::BlockIdFlag))]
    pub enum BlockIdFlag {
        Unknown = 0,
        Absent = 1,
        Commit = 2,
        Nil = 3,
    }
}
