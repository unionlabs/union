use alloy_sol_types::SolType;
use enumorph::Enumorph;
use ucs03_zkgm::com::{INSTR_VERSION_0, OP_WITHDRAW_REWARDS};
use unionlabs_primitives::{Bytes, U256};

use crate::{Instruction, Result};

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
pub enum WithdrawRewards {
    V0(WithdrawRewardsV0),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
pub enum WithdrawRewardsShape {
    V0,
}

impl WithdrawRewards {
    pub(crate) fn decode(version: u8, operand: impl AsRef<[u8]>) -> Result<Self> {
        match version {
            INSTR_VERSION_0 => WithdrawRewardsV0::decode(operand).map(Into::into),
            invalid => Err(format!("invalid withdraw rewards version: {invalid}"))?,
        }
    }

    pub(crate) fn shape(&self) -> WithdrawRewardsShape {
        match self {
            WithdrawRewards::V0(_) => WithdrawRewardsShape::V0,
        }
    }

    pub(crate) fn into_instruction(self) -> Instruction {
        match self {
            WithdrawRewards::V0(v0) => v0.into_instruction(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
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

    fn into_instruction(self) -> Instruction {
        Instruction::new(
            INSTR_VERSION_0,
            OP_WITHDRAW_REWARDS,
            ucs03_zkgm::com::WithdrawRewards {
                token_id: self.token_id.into(),
                governance_token: self.governance_token.into(),
                governance_token_wrapped: self.governance_token_wrapped.into(),
                validator: self.validator.into(),
                sender: self.sender.into(),
                beneficiary: self.beneficiary.into(),
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
pub enum WithdrawRewardsAck {
    V0(WithdrawRewardsV0Ack),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
pub struct WithdrawRewardsV0Ack {
    pub amount: U256,
}

impl WithdrawRewardsV0Ack {
    fn decode(bz: impl AsRef<[u8]>) -> Result<Self> {
        let ucs03_zkgm::com::WithdrawRewardsAck { amount } =
            ucs03_zkgm::com::WithdrawRewardsAck::abi_decode_params_validate(bz.as_ref())?;

        Ok(Self {
            amount: amount.into(),
        })
    }
}

impl WithdrawRewardsAck {
    pub(crate) fn decode(shape: WithdrawRewardsShape, bz: impl AsRef<[u8]>) -> Result<Self> {
        match shape {
            WithdrawRewardsShape::V0 => WithdrawRewardsV0Ack::decode(bz).map(Self::V0),
        }
    }
}
