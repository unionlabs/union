use enumorph::Enumorph;
use ibc_classic_spec::IbcClassic;
use ibc_union_spec::IbcUnion;
use jsonrpsee::{core::RpcResult, types::ErrorObject};
use macros::model;
use unionlabs::ErrorReporter;
use voyager_message::{data::IbcDatagram, FATAL_JSONRPC_ERROR_CODE};

#[model]
#[derive(Enumorph)]
pub enum ModuleCall {
    SubmitTransaction(Vec<IbcMessage>),
}

#[model]
#[derive(Enumorph)]
pub enum IbcMessage {
    IbcV1(ibc_classic_spec::Datagram),
    IbcUnion(ibc_union_spec::Datagram),
}

impl IbcMessage {
    pub fn from_raw_datagram(datagram: IbcDatagram) -> RpcResult<Self> {
        match datagram.decode_datagram::<IbcClassic>() {
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
                    format!("unknown IBC version id: {}", datagram.ibc_spec_id),
                    None::<()>,
                )),
            },
        }
    }
}
