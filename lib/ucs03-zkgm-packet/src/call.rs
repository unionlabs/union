use alloy_sol_types::SolType;
use enumorph::Enumorph;
use ucs03_zkgm::com::{INSTR_VERSION_0, TAG_ACK_SUCCESS};
use unionlabs_primitives::Bytes;

use crate::Result;

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
pub enum Call {
    V0(CallV0),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallShape {
    V0(CallV0Shape),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallV0Shape {
    pub eureka: bool,
}

impl Call {
    pub(crate) fn decode(version: u8, operand: impl AsRef<[u8]>) -> Result<Self> {
        match version {
            INSTR_VERSION_0 => CallV0::decode(operand).map(Into::into),
            invalid => Err(format!("invalid call version: {invalid}"))?,
        }
    }

    pub(crate) fn shape(&self) -> CallShape {
        match self {
            Call::V0(CallV0 { eureka, .. }) => CallShape::V0(CallV0Shape { eureka: *eureka }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallV0 {
    pub sender: Bytes,
    pub eureka: bool,
    pub contract_address: Bytes,
    pub contract_calldata: Bytes,
}

impl CallV0 {
    pub(crate) fn decode(operand: impl AsRef<[u8]>) -> Result<Self> {
        let ucs03_zkgm::com::Call {
            sender,
            eureka,
            contract_address,
            contract_calldata,
        } = ucs03_zkgm::com::Call::abi_decode_params_validate(operand.as_ref())?;
        Ok(Self {
            sender: sender.into(),
            eureka,
            contract_address: contract_address.into(),
            contract_calldata: contract_calldata.into(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
pub enum CallAck {
    V0(CallAckV0),
}

impl CallAck {
    pub(crate) fn decode(shape: CallShape, ack: impl AsRef<[u8]>) -> Result<Self> {
        match shape {
            CallShape::V0(shape) => CallAckV0::decode(shape, ack).map(CallAck::V0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallAckV0 {
    NonEureka,
    Eureka(Bytes),
}

impl CallAckV0 {
    pub(crate) fn decode(shape: CallV0Shape, ack: impl AsRef<[u8]>) -> Result<Self> {
        if shape.eureka {
            Ok(Self::Eureka(ack.as_ref().into()))
        } else if ack.as_ref() == TAG_ACK_SUCCESS.to_be_bytes::<32>() {
            Ok(Self::NonEureka)
        } else {
            Err("invalid call v1 eureka ack, expected bytes32(TAG_ACK_SUCCESS)")?
        }
    }
}
