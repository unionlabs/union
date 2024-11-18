use enumorph::Enumorph;
use jsonrpsee::{core::RpcResult, types::ErrorObject};
use macros::model;
use unionlabs::ErrorReporter;
use voyager_message::{
    data::IbcDatagram, ibc_union::IbcUnion, ibc_v1::IbcV1, FATAL_JSONRPC_ERROR_CODE,
};

#[model]
#[derive(Enumorph)]
pub enum ModuleCall {
    SubmitTransaction(Vec<IbcMessage>),
}

#[model]
#[derive(Enumorph)]
pub enum IbcMessage {
    IbcV1(voyager_message::ibc_v1::IbcMessage),
    IbcUnion(voyager_message::ibc_union::IbcMsg),
}

impl IbcMessage {
    pub fn from_raw_datagram(datagram: IbcDatagram) -> RpcResult<Self> {
        match datagram.decode_datagram::<IbcV1>() {
            Some(Ok(ok)) => Ok(ok.into()),
            Some(Err(err)) => Err(ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                format!("unable to decode IBC datagram: {}", ErrorReporter(err)),
                None::<()>,
            )),
            None => match datagram.decode_datagram::<IbcUnion>() {
                Some(Ok(ok)) => Ok(ok.into()),
                Some(Err(err)) => Err(ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to decode IBC datagram: {}", ErrorReporter(err)),
                    None::<()>,
                )),
                None => Err(ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unknown IBC version id: {}", datagram.ibc_version_id),
                    None::<()>,
                )),
            },
        }
    }
}
