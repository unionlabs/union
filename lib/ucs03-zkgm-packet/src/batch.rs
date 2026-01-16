use alloy_sol_types::SolValue;
use enumorph::Enumorph;
use ucs03_zkgm::com::{INSTR_VERSION_0, OP_BATCH, OP_CALL, OP_TOKEN_ORDER};
use unionlabs_primitives::Bytes;

use crate::{
    Instruction, Result,
    call::{Call, CallAck, CallShape},
    token_order::{TokenOrder, TokenOrderAck, TokenOrderShape},
};

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case", tag = "@version")
)]
pub enum Batch {
    V0(BatchV0),
}

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case", tag = "@version")
)]
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

    pub(crate) fn into_instruction(self) -> Instruction {
        match self {
            Batch::V0(v0) => v0.into_instruction(),
        }
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
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case", tag = "@version")
)]
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
                        .map(|instructions| {
                            BatchAck::V0(BatchV0Ack {
                                acknowledgements: instructions,
                            })
                        })
                }
            }
        }
    }

    pub(crate) fn encode(&self) -> Bytes {
        match self {
            BatchAck::V0(ack) => ucs03_zkgm::com::BatchAck {
                acknowledgements: ack
                    .acknowledgements
                    .iter()
                    .map(|ack| ack.encode().into())
                    .collect(),
            }
            .abi_encode_params()
            .into(),
        }
    }
}

// TODO: Non-empty
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
pub struct BatchV0 {
    pub instructions: Vec<BatchInstructionV0>,
}

// TODO: Non-empty
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
pub struct BatchV0Ack {
    pub acknowledgements: Vec<BatchInstructionV0Ack>,
}

// TODO: Non-empty
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
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

    fn into_instruction(self) -> Instruction {
        Instruction::new(
            OP_BATCH,
            INSTR_VERSION_0,
            ucs03_zkgm::com::Batch {
                instructions: self
                    .instructions
                    .into_iter()
                    .map(|s| s.into_instruction().into_raw())
                    .collect(),
            },
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case", tag = "@opcode")
)]
pub enum BatchInstructionV0 {
    TokenOrder(TokenOrder),
    Call(Call),
}

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case", tag = "@opcode")
)]
pub enum BatchInstructionV0Ack {
    TokenOrder(TokenOrderAck),
    Call(CallAck),
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
        }
    }

    pub(crate) fn encode(&self) -> Bytes {
        match self {
            BatchInstructionV0Ack::TokenOrder(ack) => ack.encode(),
            BatchInstructionV0Ack::Call(ack) => ack.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case", tag = "@opcode")
)]
pub enum BatchInstructionV0Shape {
    TokenOrder(TokenOrderShape),
    Call(CallShape),
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
            invalid => Err(format!("invalid batch instruction opcode: {invalid}").into()),
        }
    }

    fn shape(&self) -> BatchInstructionV0Shape {
        match self {
            BatchInstructionV0::TokenOrder(token_order) => {
                BatchInstructionV0Shape::TokenOrder(token_order.shape())
            }
            BatchInstructionV0::Call(call) => BatchInstructionV0Shape::Call(call.shape()),
        }
    }

    fn into_instruction(self) -> Instruction {
        match self {
            BatchInstructionV0::TokenOrder(token_order) => token_order.into_instruction(),
            BatchInstructionV0::Call(call) => call.into_instruction(),
        }
    }
}

#[test]
fn batchtest() {
    let bz = hex_literal::hex!(
        "000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000003400000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000026000000000000000000000000000000000000000000000000000000000000000142c96e52fce14baa13868ca8182f8a7903e4e76e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000003f6f736d6f31396d6465776873757977336c6e677a37336b7a386b737236617330617130717130707936706b356d3768353933616c61396379733237336c3438000000000000000000000000000000000000000000000000000000000000000014A1a1d0B9182339e86e80db519218eA03Ec09a1A100000000000000000000000000000000000000000000000000000000000000000000000000000000000000446962632f424332364137413830354543443638323237313934373242434237383432413438454630394446323036313832463846323539423235393345423544323346420000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000003f6f736d6f316174306e6539617977683335706d6c7a3065786c35666c336c6a7770657934676336323079797874687173706477396536306d736c666a786d3300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002800000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000000142c96e52fce14baa13868ca8182f8a7903e4e76e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000003f6f736d6f31396d6465776873757977336c6e677a37336b7a386b737236617330617130717130707936706b356d3768353933616c61396379733237336c34380000000000000000000000000000000000000000000000000000000000000001335b7b22696263223a7b227472616e73666572223a7b226368616e6e656c5f6964223a226368616e6e656c2d3934383134222c22746f5f61646472657373223a7b225f746167223a22436f736d6f73446973706c6179222c2261646472657373223a2261746f6e6531396c6e7063733070767a39687463766d35386a6b7036616b35356d343978356e72307739716a227d2c22616d6f756e74223a7b2264656e6f6d223a226962632f42433236413741383035454344363832323731393437324243423738343241343845463039444632303631383246384632353942323539334542354432334642222c22616d6f756e74223a2231227d2c2274696d656f7574223a7b2274696d657374616d70223a2231373638383434333731323638303030303030227d2c226d656d6f223a22227d7d7d5d00000000000000000000000000"
    );

    let batch = BatchV0::decode(&bz).unwrap();

    dbg!(batch);
}
