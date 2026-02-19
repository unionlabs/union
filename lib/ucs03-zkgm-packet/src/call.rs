use alloc::format;

use alloy_sol_types::SolValue;
use enumorph::Enumorph;
use unionlabs_primitives::Bytes;

use crate::{
    Instruction, Result,
    com::{INSTR_VERSION_0, OP_CALL, TAG_ACK_SUCCESS},
};

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case", tag = "@version")
)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum Call {
    V0(CallV0),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case", tag = "@version")
)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum CallShape {
    V0(CallV0Shape),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
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

    pub(crate) fn into_instruction(self) -> Instruction {
        match self {
            Call::V0(v0) => v0.into_instruction(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct CallV0 {
    pub sender: Bytes,
    pub eureka: bool,
    pub contract_address: Bytes,
    pub contract_calldata: Bytes,
}

impl CallV0 {
    pub(crate) fn decode(operand: impl AsRef<[u8]>) -> Result<Self> {
        let crate::com::Call {
            sender,
            eureka,
            contract_address,
            contract_calldata,
        } = crate::com::Call::abi_decode_params_validate(operand.as_ref())?;
        Ok(Self {
            sender: sender.into(),
            eureka,
            contract_address: contract_address.into(),
            contract_calldata: contract_calldata.into(),
        })
    }

    fn into_instruction(self) -> Instruction {
        Instruction::new(
            OP_CALL,
            INSTR_VERSION_0,
            crate::com::Call {
                sender: self.sender.into(),
                eureka: self.eureka,
                contract_address: self.contract_address.into(),
                contract_calldata: self.contract_calldata.into(),
            },
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case", tag = "@version")
)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum CallAck {
    V0(CallV0Ack),
}

impl CallAck {
    pub(crate) fn decode(shape: CallShape, ack: impl AsRef<[u8]>) -> Result<Self> {
        match shape {
            CallShape::V0(shape) => CallV0Ack::decode(shape, ack).map(CallAck::V0),
        }
    }

    pub(crate) fn encode(&self) -> Bytes {
        match self {
            CallAck::V0(ack) => ack.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum CallV0Ack {
    NonEureka,
    Eureka(Bytes),
}

impl CallV0Ack {
    pub(crate) fn decode(shape: CallV0Shape, ack: impl AsRef<[u8]>) -> Result<Self> {
        if shape.eureka {
            Ok(Self::Eureka(ack.as_ref().into()))
        } else if ack.as_ref() == TAG_ACK_SUCCESS.to_be_bytes() {
            Ok(Self::NonEureka)
        } else {
            Err("invalid call v1 eureka ack, expected bytes32(TAG_ACK_SUCCESS)")?
        }
    }

    pub(crate) fn encode(&self) -> Bytes {
        match self {
            CallV0Ack::NonEureka => TAG_ACK_SUCCESS.to_be_bytes().into(),
            CallV0Ack::Eureka(bytes) => bytes.clone(),
        }
    }
}
