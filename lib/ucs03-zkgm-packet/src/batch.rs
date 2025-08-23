use alloy_sol_types::SolType;
use enumorph::Enumorph;
use ucs03_zkgm::com::{
    INSTR_VERSION_0, OP_CALL, OP_STAKE, OP_TOKEN_ORDER, OP_UNSTAKE, OP_WITHDRAW_REWARDS,
    OP_WITHDRAW_STAKE,
};
use unionlabs_primitives::Bytes;

use crate::{
    call::{Call, CallAck, CallShape},
    stake::{Stake, StakeShape},
    token_order::{TokenOrder, TokenOrderAck, TokenOrderShape},
    unstake::{Unstake, UnstakeShape},
    withdraw_rewards::{WithdrawRewards, WithdrawRewardsShape},
    withdraw_stake::{WithdrawStake, WithdrawStakeShape},
    Result,
};

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
pub enum Batch {
    V0(BatchV0),
}

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
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

    pub(crate) fn encode(&self) -> (u8, u8, Bytes) {
        // match self {
        //     Batch::V0(BatchV0 { instructions }) => (
        //         INSTR_VERSION_0,
        //         OP_BATCH,
        //         ucs03_zkgm::com::Batch {
        //             instructions: instructions,
        //         },
        //     ),
        // }
        todo!()
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
pub struct BatchV0 {
    pub instructions: Vec<BatchInstructionV0>,
}

// TODO: Non-empty
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchV0Ack {
    pub instructions: Vec<BatchInstructionV0Ack>,
}

// TODO: Non-empty
#[derive(Debug, Clone, PartialEq, Eq)]
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
}

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
pub enum BatchInstructionV0 {
    TokenOrder(TokenOrder),
    Call(Call),
    Stake(Stake),
    Unstake(Unstake),
    WithdrawStake(WithdrawStake),
    WithdrawRewards(WithdrawRewards),
}

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
pub enum BatchInstructionV0Ack {
    TokenOrder(TokenOrderAck),
    Call(CallAck),
    // Stake(StakeAck),
    // Unstake(UnstakeAck),
    // WithdrawStake(WithdrawStakeAck),
    // WithdrawRewards(WithdrawRewardsAck),
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
            BatchInstructionV0Shape::Stake(_shape) => todo!(),
            BatchInstructionV0Shape::Unstake(_shape) => todo!(),
            BatchInstructionV0Shape::WithdrawStake(_shape) => {
                todo!()
            }
            BatchInstructionV0Shape::WithdrawRewards(_shape) => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
pub enum BatchInstructionV0Shape {
    TokenOrder(TokenOrderShape),
    Call(CallShape),
    Stake(StakeShape),
    Unstake(UnstakeShape),
    WithdrawStake(WithdrawStakeShape),
    WithdrawRewards(WithdrawRewardsShape),
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
            OP_STAKE => Stake::decode(instruction.version, instruction.operand).map(Into::into),
            OP_UNSTAKE => Unstake::decode(instruction.version, instruction.operand).map(Into::into),
            OP_WITHDRAW_STAKE => {
                WithdrawStake::decode(instruction.version, instruction.operand).map(Into::into)
            }
            OP_WITHDRAW_REWARDS => {
                WithdrawRewards::decode(instruction.version, instruction.operand).map(Into::into)
            }
            invalid => Err(format!("invalid batch instruction opcode: {invalid}").into()),
        }
    }

    fn shape(&self) -> BatchInstructionV0Shape {
        match self {
            BatchInstructionV0::TokenOrder(token_order) => {
                BatchInstructionV0Shape::TokenOrder(token_order.shape())
            }
            BatchInstructionV0::Call(call) => BatchInstructionV0Shape::Call(call.shape()),
            BatchInstructionV0::Stake(stake) => BatchInstructionV0Shape::Stake(stake.shape()),
            BatchInstructionV0::Unstake(unstake) => {
                BatchInstructionV0Shape::Unstake(unstake.shape())
            }
            BatchInstructionV0::WithdrawStake(withdraw_stake) => {
                BatchInstructionV0Shape::WithdrawStake(withdraw_stake.shape())
            }
            BatchInstructionV0::WithdrawRewards(withdraw_rewards) => {
                BatchInstructionV0Shape::WithdrawRewards(withdraw_rewards.shape())
            }
        }
    }
}
