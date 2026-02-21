use alloc::format;

use alloy_sol_types::SolValue;
use unionlabs_primitives::Bytes;

use crate::{
    Instruction, Result,
    batch::{Batch, BatchAck, BatchShape},
    call::{Call, CallAck, CallShape},
    com::{OP_BATCH, OP_CALL, OP_FORWARD, OP_TOKEN_ORDER},
    forward::{Forward, ForwardAck, ForwardShape},
    token_order::{TokenOrder, TokenOrderAck, TokenOrderShape},
};

attrs! {
    #[tag("@opcode")]
    #[enumorph]
    pub enum Root {
        Batch(Batch),
        TokenOrder(TokenOrder),
        Call(Call),
        Forward(Forward),
    }
}

impl Root {
    pub fn decode(bz: &[u8]) -> Result<Self> {
        let instruction = crate::com::Instruction::abi_decode_params_validate(bz)?;

        Self::from_raw(instruction)
    }

    pub fn shape(&self) -> RootShape {
        match self {
            Root::Batch(batch) => RootShape::Batch(batch.shape()),
            Root::TokenOrder(token_order) => RootShape::TokenOrder(token_order.shape()),
            Root::Call(call) => RootShape::Call(call.shape()),
            Root::Forward(forward) => RootShape::Forward(forward.shape()),
        }
    }

    pub(crate) fn from_raw(instruction: crate::com::Instruction) -> Result<Root> {
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

    pub fn into_instruction(self) -> Instruction {
        match self {
            Root::Batch(batch) => batch.into_instruction(),
            Root::TokenOrder(token_order) => token_order.into_instruction(),
            Root::Call(call) => call.into_instruction(),
            Root::Forward(forward) => forward.into_instruction(),
        }
    }

    pub fn encode(self) -> Bytes {
        self.into_instruction()
            .into_raw()
            .abi_encode_params()
            .into()
    }
}

attrs! {
    #[tag("@opcode")]
    #[enumorph]
    pub enum RootShape {
        Batch(BatchShape),
        TokenOrder(TokenOrderShape),
        Call(CallShape),
        Forward(ForwardShape),
    }
}

attrs! {
    #[tag("@opcode")]
    #[enumorph]
    pub enum RootAck {
        Batch(BatchAck),
        TokenOrder(TokenOrderAck),
        Call(CallAck),
        Forward(ForwardAck),
    }
}

impl RootAck {
    pub(crate) fn decode(shape: RootShape, inner_ack: &[u8]) -> Result<RootAck> {
        match shape {
            RootShape::Batch(shape) => BatchAck::decode(shape, inner_ack).map(RootAck::Batch),
            RootShape::TokenOrder(shape) => {
                TokenOrderAck::decode(shape, inner_ack).map(RootAck::TokenOrder)
            }
            RootShape::Call(shape) => CallAck::decode(shape, inner_ack).map(RootAck::Call),
            RootShape::Forward(shape) => ForwardAck::decode(shape, inner_ack).map(RootAck::Forward),
        }
    }

    pub(crate) fn encode(&self) -> Bytes {
        match self {
            RootAck::Batch(ack) => ack.encode(),
            RootAck::TokenOrder(ack) => ack.encode(),
            RootAck::Call(ack) => ack.encode(),
            RootAck::Forward(ack) => ack.encode(),
        }
    }
}
