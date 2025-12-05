use enumorph::Enumorph;
use ibc_union_spec::IbcUnion;
use macros::model;
use voyager_sdk::{
    message::data::IbcDatagram,
    rpc::{RpcError, RpcResult},
};

#[model]
#[derive(Enumorph)]
pub enum ModuleCall {
    SubmitTransaction(Vec<IbcMessage>),
}

#[model]
#[derive(Enumorph)]
pub enum IbcMessage {
    IbcUnion(ibc_union_spec::datagram::Datagram),
}

impl IbcMessage {
    pub fn from_raw_datagram(datagram: IbcDatagram) -> RpcResult<Self> {
        datagram
            .decode_datagram::<IbcUnion>()
            .ok_or_else(|| {
                RpcError::fatal_from_message(format!(
                    "unknown IBC version id: {}",
                    datagram.ibc_spec_id
                ))
            })?
            .map_err(RpcError::fatal("unable to decode IBC datagram"))
            .map(Into::into)
    }

    pub fn name(&self) -> &'static str {
        match self {
            IbcMessage::IbcUnion(datagram) => datagram.name(),
        }
    }
}
