use alloy_sol_types::SolType;
use enumorph::Enumorph;
use ucs03_zkgm::com::{INSTR_VERSION_0, OP_STAKE};
use unionlabs_primitives::{Bytes, U256};

use crate::{Instruction, Result};

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
pub enum Stake {
    V0(StakeV0),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
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

    pub(crate) fn into_instruction(self) -> Instruction {
        match self {
            Stake::V0(v0) => v0.into_instruction(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
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

    fn into_instruction(self) -> Instruction {
        Instruction::new(
            INSTR_VERSION_0,
            OP_STAKE,
            ucs03_zkgm::com::Stake {
                token_id: self.token_id.into(),
                governance_token: self.governance_token.into(),
                governance_token_wrapped: self.governance_token_wrapped.into(),
                sender: self.sender.into(),
                beneficiary: self.beneficiary.into(),
                validator: self.validator.into(),
                amount: self.amount.into(),
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
pub enum StakeAck {
    V0(StakeV0Ack),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
pub struct StakeV0Ack {}

impl StakeV0Ack {
    fn decode(bz: impl AsRef<[u8]>) -> Result<Self> {
        if bz.as_ref().is_empty() {
            Ok(Self {})
        } else {
            Err("stake v0 ack must be empty".into())
        }
    }
}

impl StakeAck {
    pub(crate) fn decode(shape: StakeShape, bz: impl AsRef<[u8]>) -> Result<Self> {
        match shape {
            StakeShape::V0 => StakeV0Ack::decode(bz).map(Self::V0),
        }
    }
}
