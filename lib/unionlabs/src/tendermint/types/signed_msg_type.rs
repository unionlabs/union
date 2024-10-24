use crate::macros::wrapper_enum;

wrapper_enum! {
    #[model(proto(protos::tendermint::types::SignedMsgType))]
    pub enum SignedMsgType {
        Unknown = 0,
        Prevote = 1,
        Precommit = 2,
        Proposal = 32,
    }
}
