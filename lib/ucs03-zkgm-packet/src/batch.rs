use alloy_sol_types::SolType;
use enumorph::Enumorph;
use ucs03_zkgm::com::{INSTR_VERSION_0, OP_CALL, OP_FORWARD, OP_TOKEN_ORDER};

use crate::{call::Call, forward::Forward, token_order::TokenOrder, Result};

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
pub enum Batch {
    V0(BatchV0),
}

impl Batch {
    pub(crate) fn decode(version: u8, operand: impl AsRef<[u8]>) -> Result<Self> {
        match version {
            INSTR_VERSION_0 => BatchV0::decode(operand).map(Into::into),
            invalid => Err(format!("invalid batch version: {invalid}"))?,
        }
    }
}

// TODO: Non-empty
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchV0 {
    instructions: Vec<BatchableInstructionV0>,
}

impl BatchV0 {
    pub(crate) fn decode(operand: impl AsRef<[u8]>) -> Result<Self> {
        let instruction = ucs03_zkgm::com::Batch::abi_decode_params_validate(operand.as_ref())?;
        Ok(Self {
            instructions: instruction
                .instructions
                .into_iter()
                .map(BatchableInstructionV0::from_raw)
                .collect::<Result<_>>()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
pub enum BatchableInstructionV0 {
    TokenOrder(TokenOrder),
    Call(Call),
    Forward(Forward),
}

impl BatchableInstructionV0 {
    pub fn decode(bz: &[u8]) -> Result<Self> {
        let instruction = ucs03_zkgm::com::Instruction::abi_decode_params_validate(bz)?;

        Self::from_raw(instruction)
    }

    fn from_raw(instruction: ucs03_zkgm::com::Instruction) -> Result<BatchableInstructionV0> {
        match instruction.opcode {
            OP_TOKEN_ORDER => {
                TokenOrder::decode(instruction.version, instruction.operand).map(Into::into)
            }
            OP_CALL => Call::decode(instruction.version, instruction.operand).map(Into::into),
            OP_FORWARD => Forward::decode(instruction.version, instruction.operand).map(Into::into),
            invalid => Err(format!("invalid batch instruction opcode: {invalid}").into()),
        }
    }
}
