use alloy_sol_types::SolType;
use enumorph::Enumorph;
use ucs03_zkgm::com::INSTR_VERSION_0;
use unionlabs_primitives::{Bytes, U256};

use crate::Result;

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
pub enum WithdrawRewards {
    V0(WithdrawRewardsV0),
}

impl WithdrawRewards {
    pub(crate) fn decode(version: u8, operand: impl AsRef<[u8]>) -> Result<Self> {
        match version {
            INSTR_VERSION_0 => WithdrawRewardsV0::decode(operand).map(Into::into),
            invalid => Err(format!("invalid withdraw rewards version: {invalid}"))?,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithdrawRewardsV0 {
    token_id: U256,
    governance_token: Bytes,
    governance_token_wrapped: Bytes,
    sender: Bytes,
    validator: Bytes,
    beneficiary: Bytes,
}

impl WithdrawRewardsV0 {
    pub(crate) fn decode(operand: impl AsRef<[u8]>) -> Result<Self> {
        let ucs03_zkgm::com::WithdrawRewards {
            token_id,
            governance_token,
            governance_token_wrapped,
            validator,
            sender,
            beneficiary,
        } = ucs03_zkgm::com::WithdrawRewards::abi_decode_params_validate(operand.as_ref())?;
        Ok(Self {
            token_id: token_id.into(),
            governance_token: governance_token.into(),
            governance_token_wrapped: governance_token_wrapped.into(),
            sender: sender.into(),
            validator: validator.into(),
            beneficiary: beneficiary.into(),
        })
    }
}
