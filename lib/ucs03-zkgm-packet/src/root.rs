use alloy_sol_types::SolType;
use enumorph::Enumorph;
use ucs03_zkgm::com::{OP_BATCH, OP_CALL, OP_FORWARD, OP_TOKEN_ORDER};

use crate::{batch::Batch, call::Call, forward::Forward, token_order::TokenOrder, Result};

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
pub enum Root {
    Batch(Batch),
    TokenOrder(TokenOrder),
    Call(Call),
    Forward(Forward),
}

impl Root {
    pub fn decode(bz: &[u8]) -> Result<Self> {
        let instruction = ucs03_zkgm::com::Instruction::abi_decode_params_validate(bz)?;

        Self::from_raw(instruction)
    }

    pub(crate) fn from_raw(instruction: ucs03_zkgm::com::Instruction) -> Result<Root> {
        match instruction.opcode {
            OP_FORWARD => Forward::decode(instruction.version, instruction.operand).map(Into::into),
            OP_CALL => Call::decode(instruction.version, instruction.operand).map(Into::into),
            OP_BATCH => Batch::decode(instruction.version, instruction.operand).map(Into::into),
            OP_TOKEN_ORDER => {
                TokenOrder::decode(instruction.version, instruction.operand).map(Into::into)
            }
            invalid => Err(format!("invalid opcode: {invalid}").into()),
        }
    }
}
