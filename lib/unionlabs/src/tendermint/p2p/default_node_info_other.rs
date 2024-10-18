use macros::model;

#[model(proto(raw(protos::tendermint::p2p::DefaultNodeInfoOther)))]
pub struct DefaultNodeInfoOther {
    // REVIEW: Is this more strongly typed? So far all i've seen is "on" and "off"
    pub tx_index: String,
    pub rpc_address: String,
}

#[cfg(feature = "proto")]
impl From<protos::tendermint::p2p::DefaultNodeInfoOther> for DefaultNodeInfoOther {
    fn from(value: protos::tendermint::p2p::DefaultNodeInfoOther) -> Self {
        Self {
            tx_index: value.tx_index,
            rpc_address: value.rpc_address,
        }
    }
}
