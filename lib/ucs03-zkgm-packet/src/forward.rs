use alloc::{boxed::Box, format};

use alloy_sol_types::SolType;
use enumorph::Enumorph;
use unionlabs_primitives::{Bytes, U256};

use crate::{
    Instruction, Result,
    com::{INSTR_VERSION_0, OP_FORWARD},
    root::Root,
};

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case", tag = "@version")
)]
pub enum Forward {
    V0(ForwardV0),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
pub enum ForwardShape {
    V0,
}

impl Forward {
    pub(crate) fn decode(version: u8, operand: impl AsRef<[u8]>) -> Result<Self> {
        match version {
            INSTR_VERSION_0 => ForwardV0::decode(operand).map(Into::into),
            invalid => Err(format!("invalid forward version: {invalid}"))?,
        }
    }

    pub(crate) fn shape(&self) -> ForwardShape {
        match self {
            Forward::V0(_) => ForwardShape::V0,
        }
    }

    pub(crate) fn into_instruction(self) -> Instruction {
        match self {
            Forward::V0(v0) => v0.into_instruction(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
pub struct ForwardV0 {
    path: U256,
    // TODO: Forward v2 to remove this field
    timeout_height: u64,
    timeout_timestamp: u64,
    instruction: Box<Root>,
}

impl ForwardV0 {
    pub(crate) fn decode(operand: impl AsRef<[u8]>) -> Result<Self> {
        let crate::com::Forward {
            path,
            timeout_height,
            timeout_timestamp,
            instruction,
        } = crate::com::Forward::abi_decode_params_validate(operand.as_ref())?;
        Ok(Self {
            path: path.into(),
            timeout_height,
            timeout_timestamp,
            instruction: Box::new(Root::from_raw(instruction)?),
        })
    }

    fn into_instruction(self) -> Instruction {
        Instruction::new(
            OP_FORWARD,
            INSTR_VERSION_0,
            crate::com::Forward {
                path: self.path.into(),
                timeout_height: self.timeout_height,
                timeout_timestamp: self.timeout_timestamp,
                instruction: self.instruction.into_instruction().into_raw(),
            },
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
pub enum ForwardAck {
    V0(ForwardV0Ack),
}

impl ForwardAck {
    pub(crate) fn decode(shape: ForwardShape, bz: impl AsRef<[u8]>) -> Result<Self> {
        match shape {
            ForwardShape::V0 => ForwardV0Ack::decode(bz).map(Self::V0),
        }
    }

    pub(crate) fn encode(&self) -> Bytes {
        match self {
            ForwardAck::V0(ack) => ack.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
pub struct ForwardV0Ack {}

impl ForwardV0Ack {
    fn decode(bz: impl AsRef<[u8]>) -> Result<Self> {
        if bz.as_ref().is_empty() {
            Ok(Self {})
        } else {
            Err("Forward v0 ack must be empty".into())
        }
    }

    pub(crate) fn encode(&self) -> Bytes {
        Bytes::default()
    }
}
