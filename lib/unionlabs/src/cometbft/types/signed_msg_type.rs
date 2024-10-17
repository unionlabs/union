use crate::macros::wrapper_enum;

wrapper_enum! {
    #[model(proto(protos::cometbft::types::v1::SignedMsgType))]
    pub enum SignedMsgType {
        Unknown = 0,
        Prevote = 1,
        Precommit = 2,
        Proposal = 32,
    }
}
