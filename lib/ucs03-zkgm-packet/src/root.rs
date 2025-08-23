use alloy_sol_types::SolType;
use enumorph::Enumorph;
use ucs03_zkgm::com::{
    OP_BATCH, OP_CALL, OP_FORWARD, OP_STAKE, OP_TOKEN_ORDER, OP_UNSTAKE, OP_WITHDRAW_REWARDS,
    OP_WITHDRAW_STAKE,
};

use crate::{
    batch::{Batch, BatchShape},
    call::{Call, CallShape},
    forward::{Forward, ForwardShape},
    stake::{Stake, StakeShape},
    token_order::{TokenOrder, TokenOrderShape},
    unstake::{Unstake, UnstakeShape},
    withdraw_rewards::{WithdrawRewards, WithdrawRewardsShape},
    withdraw_stake::{WithdrawStake, WithdrawStakeShape},
    Result,
};

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
pub enum Root {
    Batch(Batch),
    TokenOrder(TokenOrder),
    Call(Call),
    Forward(Forward),
    Stake(Stake),
    Unstake(Unstake),
    WithdrawStake(WithdrawStake),
    WithdrawRewards(WithdrawRewards),
}

impl Root {
    pub fn decode(bz: &[u8]) -> Result<Self> {
        let instruction = ucs03_zkgm::com::Instruction::abi_decode_params_validate(bz)?;

        Self::from_raw(instruction)
    }

    pub fn shape(&self) -> RootShape {
        match self {
            Root::Batch(batch) => RootShape::Batch(batch.shape()),
            Root::TokenOrder(token_order) => RootShape::TokenOrder(token_order.shape()),
            Root::Call(call) => RootShape::Call(call.shape()),
            Root::Forward(forward) => RootShape::Forward(forward.shape()),
            Root::Stake(stake) => RootShape::Stake(stake.shape()),
            Root::Unstake(unstake) => RootShape::Unstake(unstake.shape()),
            Root::WithdrawStake(withdraw_stake) => RootShape::WithdrawStake(withdraw_stake.shape()),
            Root::WithdrawRewards(withdraw_rewards) => {
                RootShape::WithdrawRewards(withdraw_rewards.shape())
            }
        }
    }

    pub(crate) fn from_raw(instruction: ucs03_zkgm::com::Instruction) -> Result<Root> {
        match instruction.opcode {
            OP_FORWARD => Forward::decode(instruction.version, instruction.operand).map(Into::into),
            OP_CALL => Call::decode(instruction.version, instruction.operand).map(Into::into),
            OP_BATCH => Batch::decode(instruction.version, instruction.operand).map(Into::into),
            OP_TOKEN_ORDER => {
                TokenOrder::decode(instruction.version, instruction.operand).map(Into::into)
            }
            OP_STAKE => Stake::decode(instruction.version, instruction.operand).map(Into::into),
            OP_UNSTAKE => Unstake::decode(instruction.version, instruction.operand).map(Into::into),
            OP_WITHDRAW_STAKE => {
                WithdrawStake::decode(instruction.version, instruction.operand).map(Into::into)
            }
            OP_WITHDRAW_REWARDS => {
                WithdrawRewards::decode(instruction.version, instruction.operand).map(Into::into)
            }
            invalid => Err(format!("invalid opcode: {invalid}").into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
pub enum RootShape {
    Batch(BatchShape),
    TokenOrder(TokenOrderShape),
    Call(CallShape),
    Forward(ForwardShape),
    Stake(StakeShape),
    Unstake(UnstakeShape),
    WithdrawStake(WithdrawStakeShape),
    WithdrawRewards(WithdrawRewardsShape),
}
