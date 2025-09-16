use alloy_sol_types::SolType;
use enumorph::Enumorph;
use ucs03_zkgm::com::{INSTR_VERSION_0, OP_BATCH, OP_CALL, OP_TOKEN_ORDER};

use crate::{
    call::{Call, CallAck, CallShape},
    token_order::{TokenOrder, TokenOrderAck, TokenOrderShape},
    Instruction, Result,
};

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case", tag = "@version")
)]
pub enum Batch {
    V0(BatchV0),
}

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case", tag = "@version")
)]
pub enum BatchShape {
    V0(BatchV0Shape),
}

impl Batch {
    pub(crate) fn decode(version: u8, operand: impl AsRef<[u8]>) -> Result<Self> {
        match version {
            INSTR_VERSION_0 => BatchV0::decode(operand).map(Into::into),
            invalid => Err(format!("invalid batch version: {invalid}"))?,
        }
    }

    pub(crate) fn into_instruction(self) -> Instruction {
        match self {
            Batch::V0(v0) => v0.into_instruction(),
        }
    }

    pub(crate) fn shape(&self) -> BatchShape {
        match self {
            Batch::V0(BatchV0 { instructions }) => BatchShape::V0(BatchV0Shape {
                instructions: instructions.iter().map(|b| b.shape()).collect(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case", tag = "@version")
)]
pub enum BatchAck {
    V0(BatchV0Ack),
}

impl BatchAck {
    pub(crate) fn decode(shape: BatchShape, ack: impl AsRef<[u8]>) -> Result<Self> {
        match shape {
            BatchShape::V0(BatchV0Shape { instructions }) => {
                let ucs03_zkgm::com::BatchAck { acknowledgements } =
                    ucs03_zkgm::com::BatchAck::abi_decode_params_validate(ack.as_ref())?;

                if instructions.len() != acknowledgements.len() {
                    Err(format!(
                        "invalid batch v0 shape, expected {} instructions but decoded {}",
                        instructions.len(),
                        acknowledgements.len()
                    ))?
                } else {
                    acknowledgements
                        .into_iter()
                        .zip(instructions)
                        .map(|(ack, shape)| BatchInstructionV0Ack::decode(shape, ack))
                        .collect::<Result<Vec<_>>>()
                        .map(|instructions| BatchAck::V0(BatchV0Ack { instructions }))
                }
            }
        }
    }
}

// TODO: Non-empty
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
pub struct BatchV0 {
    pub instructions: Vec<BatchInstructionV0>,
}

// TODO: Non-empty
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
pub struct BatchV0Ack {
    pub instructions: Vec<BatchInstructionV0Ack>,
}

// TODO: Non-empty
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
pub struct BatchV0Shape {
    pub instructions: Vec<BatchInstructionV0Shape>,
}

impl BatchV0 {
    pub(crate) fn decode(operand: impl AsRef<[u8]>) -> Result<Self> {
        let ucs03_zkgm::com::Batch { instructions } =
            ucs03_zkgm::com::Batch::abi_decode_params_validate(operand.as_ref())?;
        Ok(Self {
            instructions: instructions
                .into_iter()
                .map(BatchInstructionV0::from_raw)
                .collect::<Result<_>>()?,
        })
    }

    fn into_instruction(self) -> Instruction {
        Instruction::new(
            OP_BATCH,
            INSTR_VERSION_0,
            ucs03_zkgm::com::Batch {
                instructions: self
                    .instructions
                    .into_iter()
                    .map(|s| s.into_instruction().into_raw())
                    .collect(),
            },
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case", tag = "@opcode")
)]
pub enum BatchInstructionV0 {
    TokenOrder(TokenOrder),
    Call(Call),
}

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case", tag = "@opcode")
)]
pub enum BatchInstructionV0Ack {
    TokenOrder(TokenOrderAck),
    Call(CallAck),
}

impl BatchInstructionV0Ack {
    fn decode(
        shape: BatchInstructionV0Shape,
        ack: impl AsRef<[u8]>,
    ) -> Result<BatchInstructionV0Ack> {
        match shape {
            BatchInstructionV0Shape::TokenOrder(shape) => {
                TokenOrderAck::decode(shape, ack).map(BatchInstructionV0Ack::TokenOrder)
            }
            BatchInstructionV0Shape::Call(shape) => {
                CallAck::decode(shape, ack).map(BatchInstructionV0Ack::Call)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case", tag = "@opcode")
)]
pub enum BatchInstructionV0Shape {
    TokenOrder(TokenOrderShape),
    Call(CallShape),
}

impl BatchInstructionV0 {
    pub fn decode(bz: &[u8]) -> Result<Self> {
        let instruction = ucs03_zkgm::com::Instruction::abi_decode_params_validate(bz)?;

        Self::from_raw(instruction)
    }

    fn from_raw(instruction: ucs03_zkgm::com::Instruction) -> Result<BatchInstructionV0> {
        match instruction.opcode {
            OP_TOKEN_ORDER => {
                TokenOrder::decode(instruction.version, instruction.operand).map(Into::into)
            }
            OP_CALL => Call::decode(instruction.version, instruction.operand).map(Into::into),
            invalid => Err(format!("invalid batch instruction opcode: {invalid}").into()),
        }
    }

    fn shape(&self) -> BatchInstructionV0Shape {
        match self {
            BatchInstructionV0::TokenOrder(token_order) => {
                BatchInstructionV0Shape::TokenOrder(token_order.shape())
            }
            BatchInstructionV0::Call(call) => BatchInstructionV0Shape::Call(call.shape()),
        }
    }

    fn into_instruction(self) -> Instruction {
        match self {
            BatchInstructionV0::TokenOrder(token_order) => token_order.into_instruction(),
            BatchInstructionV0::Call(call) => call.into_instruction(),
        }
    }
}
