use macros::model;

#[model(proto(raw(protos::cometbft::p2p::v1::DefaultNodeInfoOther)))]
pub struct DefaultNodeInfoOther {
    // REVIEW: Is this more strongly typed? So far all i've seen is "on" and "off"
    pub tx_index: String,
    pub rpc_address: String,
}

impl From<protos::cometbft::p2p::v1::DefaultNodeInfoOther> for DefaultNodeInfoOther {
    fn from(value: protos::cometbft::p2p::v1::DefaultNodeInfoOther) -> Self {
        Self {
            tx_index: value.tx_index,
            rpc_address: value.rpc_address,
        }
    }
}
