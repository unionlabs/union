use alloy_sol_types::SolType;
use enumorph::Enumorph;
use ucs03_zkgm::com::INSTR_VERSION_0;
use unionlabs_primitives::{Bytes, U256};

use crate::Result;

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
pub enum Stake {
    V0(StakeV0),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StakeShape {
    V0,
}

impl Stake {
    pub(crate) fn decode(version: u8, operand: impl AsRef<[u8]>) -> Result<Self> {
        match version {
            INSTR_VERSION_0 => StakeV0::decode(operand).map(Into::into),
            invalid => Err(format!("invalid stake version: {invalid}"))?,
        }
    }

    pub(crate) fn shape(&self) -> StakeShape {
        match self {
            Stake::V0(_) => StakeShape::V0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StakeV0 {
    token_id: U256,
    governance_token: Bytes,
    governance_token_wrapped: Bytes,
    sender: Bytes,
    beneficiary: Bytes,
    validator: Bytes,
    amount: U256,
}

impl StakeV0 {
    pub(crate) fn decode(operand: impl AsRef<[u8]>) -> Result<Self> {
        let ucs03_zkgm::com::Stake {
            token_id,
            governance_token,
            governance_token_wrapped,
            sender,
            beneficiary,
            validator,
            amount,
        } = ucs03_zkgm::com::Stake::abi_decode_params_validate(operand.as_ref())?;
        Ok(Self {
            token_id: token_id.into(),
            governance_token: governance_token.into(),
            governance_token_wrapped: governance_token_wrapped.into(),
            sender: sender.into(),
            beneficiary: beneficiary.into(),
            validator: validator.into(),
            amount: amount.into(),
        })
    }
}
