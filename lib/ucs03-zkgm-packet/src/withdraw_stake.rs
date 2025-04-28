use alloy_sol_types::SolType;
use enumorph::Enumorph;
use ucs03_zkgm::com::{INSTR_VERSION_0, OP_WITHDRAW_STAKE};
use unionlabs_primitives::{Bytes, U256};

use crate::{Instruction, Result};

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
pub enum WithdrawStake {
    V0(WithdrawStakeV0),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
pub enum WithdrawStakeShape {
    V0,
}

impl WithdrawStake {
    pub(crate) fn decode(version: u8, operand: impl AsRef<[u8]>) -> Result<Self> {
        match version {
            INSTR_VERSION_0 => WithdrawStakeV0::decode(operand).map(Into::into),
            invalid => Err(format!("invalid withdraw stake version: {invalid}"))?,
        }
    }

    pub(crate) fn shape(&self) -> WithdrawStakeShape {
        match self {
            WithdrawStake::V0(_) => WithdrawStakeShape::V0,
        }
    }

    pub(crate) fn into_instruction(self) -> Instruction {
        match self {
            WithdrawStake::V0(v0) => v0.into_instruction(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
pub struct WithdrawStakeV0 {
    token_id: U256,
    governance_token: Bytes,
    governance_token_wrapped: Bytes,
    sender: Bytes,
    beneficiary: Bytes,
}

impl WithdrawStakeV0 {
    pub(crate) fn decode(operand: impl AsRef<[u8]>) -> Result<Self> {
        let ucs03_zkgm::com::WithdrawStake {
            token_id,
            governance_token,
            governance_token_wrapped,
            sender,
            beneficiary,
        } = ucs03_zkgm::com::WithdrawStake::abi_decode_params_validate(operand.as_ref())?;
        Ok(Self {
            token_id: token_id.into(),
            governance_token: governance_token.into(),
            governance_token_wrapped: governance_token_wrapped.into(),
            sender: sender.into(),
            beneficiary: beneficiary.into(),
        })
    }

    fn into_instruction(self) -> Instruction {
        Instruction::new(
            INSTR_VERSION_0,
            OP_WITHDRAW_STAKE,
            ucs03_zkgm::com::WithdrawStake {
                token_id: self.token_id.into(),
                governance_token: self.governance_token.into(),
                governance_token_wrapped: self.governance_token_wrapped.into(),
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
pub enum WithdrawStakeAck {
    V0(WithdrawStakeV0Ack),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
pub struct WithdrawStakeV0Ack {
    pub amount: U256,
}

impl WithdrawStakeV0Ack {
    fn decode(bz: impl AsRef<[u8]>) -> Result<Self> {
        let ucs03_zkgm::com::WithdrawStakeAck { amount } =
            ucs03_zkgm::com::WithdrawStakeAck::abi_decode_params_validate(bz.as_ref())?;

        Ok(Self {
            amount: amount.into(),
        })
    }
}

impl WithdrawStakeAck {
    pub(crate) fn decode(shape: WithdrawStakeShape, bz: impl AsRef<[u8]>) -> Result<Self> {
        match shape {
            WithdrawStakeShape::V0 => WithdrawStakeV0Ack::decode(bz).map(Self::V0),
        }
    }
}
