use alloy_sol_types::SolType;
use enumorph::Enumorph;
use ucs03_zkgm::com::{INSTR_VERSION_0, OP_UNSTAKE};
use unionlabs_primitives::{Bytes, U256};

use crate::{Instruction, Result};

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
pub enum Unstake {
    V0(UnstakeV0),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnstakeShape {
    V0,
}

impl Unstake {
    pub(crate) fn decode(version: u8, operand: impl AsRef<[u8]>) -> Result<Self> {
        match version {
            INSTR_VERSION_0 => UnstakeV0::decode(operand).map(Into::into),
            invalid => Err(format!("invalid unstake version: {invalid}"))?,
        }
    }

    pub(crate) fn shape(&self) -> UnstakeShape {
        match self {
            Unstake::V0(_) => UnstakeShape::V0,
        }
    }

    pub(crate) fn into_instruction(self) -> Instruction {
        match self {
            Unstake::V0(v0) => v0.into_instruction(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnstakeV0 {
    token_id: U256,
    governance_token: Bytes,
    governance_token_wrapped: Bytes,
    sender: Bytes,
    validator: Bytes,
}

impl UnstakeV0 {
    pub(crate) fn decode(operand: impl AsRef<[u8]>) -> Result<Self> {
        let ucs03_zkgm::com::Unstake {
            token_id,
            governance_token,
            governance_token_wrapped,
            sender,
            validator,
        } = ucs03_zkgm::com::Unstake::abi_decode_params_validate(operand.as_ref())?;
        Ok(Self {
            token_id: token_id.into(),
            governance_token: governance_token.into(),
            governance_token_wrapped: governance_token_wrapped.into(),
            sender: sender.into(),
            validator: validator.into(),
        })
    }

    pub(crate) fn into_instruction(self) -> Instruction {
        Instruction::new(
            INSTR_VERSION_0,
            OP_UNSTAKE,
            ucs03_zkgm::com::Unstake {
                token_id: self.token_id.into(),
                governance_token: self.governance_token.into(),
                governance_token_wrapped: self.governance_token_wrapped.into(),
                sender: self.sender.into(),
                validator: self.validator.into(),
            },
        )
    }
}
