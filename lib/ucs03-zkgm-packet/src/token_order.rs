use alloc::{borrow::ToOwned, format, string::String};

use alloy_sol_types::SolValue;
use enumorph::Enumorph;
use unionlabs_primitives::{Bytes, U256};

use crate::{
    Instruction, Result,
    com::{
        FILL_TYPE_MARKETMAKER, FILL_TYPE_PROTOCOL, INSTR_VERSION_1, INSTR_VERSION_2,
        OP_TOKEN_ORDER, TOKEN_ORDER_KIND_ESCROW, TOKEN_ORDER_KIND_INITIALIZE,
        TOKEN_ORDER_KIND_SOLVE, TOKEN_ORDER_KIND_UNESCROW,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
#[repr(u8)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case", tag = "@version")
)]
pub enum TokenOrder {
    #[deprecated(since = "TBD")]
    V1(TokenOrderV1) = INSTR_VERSION_1,
    V2(TokenOrderV2) = INSTR_VERSION_2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case", tag = "@version")
)]
pub enum TokenOrderShape {
    #[deprecated(since = "TBD")]
    V1 = INSTR_VERSION_1,
    V2 = INSTR_VERSION_2,
}

impl TokenOrder {
    pub(crate) fn decode(version: u8, operand: impl AsRef<[u8]>) -> Result<Self> {
        match version {
            INSTR_VERSION_1 => TokenOrderV1::decode(operand).map(Into::into),
            INSTR_VERSION_2 => TokenOrderV2::decode(operand).map(Into::into),
            invalid => Err(format!("invalid token order version: {invalid}"))?,
        }
    }

    pub(crate) fn shape(&self) -> TokenOrderShape {
        match self {
            TokenOrder::V1(_) => TokenOrderShape::V1,
            TokenOrder::V2(_) => TokenOrderShape::V2,
        }
    }

    pub(crate) fn into_instruction(self) -> Instruction {
        match self {
            TokenOrder::V1(v1) => v1.into_instruction(),
            TokenOrder::V2(v2) => v2.into_instruction(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
#[repr(u8)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case", tag = "@version")
)]
pub enum TokenOrderAck {
    #[deprecated(since = "TBD")]
    V1(TokenOrderV1Ack) = INSTR_VERSION_1,
    V2(TokenOrderV2Ack) = INSTR_VERSION_2,
}

impl TokenOrderAck {
    pub(crate) fn decode(shape: TokenOrderShape, ack: impl AsRef<[u8]>) -> Result<Self> {
        match shape {
            TokenOrderShape::V1 => TokenOrderV1Ack::decode(ack).map(TokenOrderAck::V1),
            TokenOrderShape::V2 => TokenOrderV2Ack::decode(ack).map(TokenOrderAck::V2),
        }
    }

    pub(crate) fn encode(&self) -> Bytes {
        match self {
            TokenOrderAck::V1(ack) => ack.encode(),
            TokenOrderAck::V2(ack) => ack.encode(),
        }
    }
}

#[deprecated(note = "token order v1 will be superseded by v2", since = "TBD")]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
pub struct TokenOrderV1 {
    pub sender: Bytes,
    pub receiver: Bytes,
    pub base_token: Bytes,
    pub base_amount: U256,
    pub base_token_symbol: String,
    pub base_token_name: String,
    pub base_token_decimals: u8,
    pub base_token_path: U256,
    pub quote_token: Bytes,
    pub quote_amount: U256,
}

impl TokenOrderV1 {
    pub(crate) fn decode(operand: impl AsRef<[u8]>) -> Result<Self> {
        let crate::com::TokenOrderV1 {
            sender,
            receiver,
            base_token,
            base_amount,
            base_token_symbol,
            base_token_name,
            base_token_decimals,
            base_token_path,
            quote_token,
            quote_amount,
        } = crate::com::TokenOrderV1::abi_decode_params_validate(operand.as_ref())?;
        Ok(Self {
            sender: sender.into(),
            receiver: receiver.into(),
            base_token: base_token.into(),
            base_amount: base_amount.into(),
            base_token_symbol,
            base_token_name,
            base_token_decimals,
            base_token_path: base_token_path.into(),
            quote_token: quote_token.into(),
            quote_amount: quote_amount.into(),
        })
    }

    fn into_instruction(self) -> Instruction {
        Instruction::new(
            OP_TOKEN_ORDER,
            INSTR_VERSION_1,
            crate::com::TokenOrderV1 {
                sender: self.sender.into(),
                receiver: self.receiver.into(),
                base_token: self.base_token.into(),
                base_amount: self.base_amount.into(),
                base_token_symbol: self.base_token_symbol,
                base_token_name: self.base_token_name,
                base_token_decimals: self.base_token_decimals,
                base_token_path: self.base_token_path.into(),
                quote_token: self.quote_token.into(),
                quote_amount: self.quote_amount.into(),
            },
        )
    }
}

#[deprecated(note = "token order v1 will be superseded by v2", since = "TBD")]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
pub enum TokenOrderV1Ack {
    Protocol,
    MarketMaker { market_maker: Bytes },
}

impl TokenOrderV1Ack {
    pub(crate) fn decode(ack: impl AsRef<[u8]>) -> Result<Self> {
        let crate::com::TokenOrderAck {
            fill_type,
            market_maker,
        } = crate::com::TokenOrderAck::abi_decode_params_validate(ack.as_ref())?;

        match U256::from(fill_type) {
            FILL_TYPE_PROTOCOL => {
                if market_maker.is_empty() {
                    Ok(Self::Protocol)
                } else {
                    Err(
                        "invalid token order v1 protocol fill market maker, must be empty"
                            .to_owned(),
                    )?
                }
            }
            FILL_TYPE_MARKETMAKER => Ok(Self::MarketMaker {
                market_maker: market_maker.into(),
            }),
            invalid => Err(format!("invalid token order v1 fill type: {invalid}"))?,
        }
    }

    pub(crate) fn encode(&self) -> Bytes {
        match self {
            TokenOrderV1Ack::Protocol => crate::com::TokenOrderAck {
                fill_type: FILL_TYPE_PROTOCOL.into(),
                market_maker: Default::default(),
            },
            TokenOrderV1Ack::MarketMaker { market_maker } => crate::com::TokenOrderAck {
                fill_type: FILL_TYPE_MARKETMAKER.into(),
                market_maker: market_maker.clone().into(),
            },
        }
        .abi_encode_params()
        .into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
pub enum TokenOrderV2Ack {
    Protocol,
    MarketMaker { market_maker: Bytes },
}

impl TokenOrderV2Ack {
    pub(crate) fn decode(ack: impl AsRef<[u8]>) -> Result<Self> {
        let crate::com::TokenOrderAck {
            fill_type,
            market_maker,
        } = crate::com::TokenOrderAck::abi_decode_params_validate(ack.as_ref())?;

        match U256::from(fill_type) {
            FILL_TYPE_PROTOCOL => {
                if market_maker.is_empty() {
                    Ok(Self::Protocol)
                } else {
                    Err(
                        "invalid token order v2 protocol fill market maker, must be empty"
                            .to_owned(),
                    )?
                }
            }
            FILL_TYPE_MARKETMAKER => Ok(Self::MarketMaker {
                market_maker: market_maker.into(),
            }),
            invalid => Err(format!("invalid token order v2 fill type: {invalid}"))?,
        }
    }

    pub(crate) fn encode(&self) -> Bytes {
        match self {
            TokenOrderV2Ack::Protocol => crate::com::TokenOrderAck {
                fill_type: FILL_TYPE_PROTOCOL.into(),
                market_maker: Default::default(),
            },
            TokenOrderV2Ack::MarketMaker { market_maker } => crate::com::TokenOrderAck {
                fill_type: FILL_TYPE_MARKETMAKER.into(),
                market_maker: market_maker.clone().into(),
            },
        }
        .abi_encode_params()
        .into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
pub struct TokenOrderV2 {
    pub sender: Bytes,
    pub receiver: Bytes,
    pub base_token: Bytes,
    pub base_amount: U256,
    pub quote_token: Bytes,
    pub quote_amount: U256,
    pub metadata: TokenOrderV2Metadata,
}

impl TokenOrderV2 {
    pub(crate) fn decode(operand: impl AsRef<[u8]>) -> Result<Self> {
        let crate::com::TokenOrderV2 {
            sender,
            receiver,
            base_token,
            base_amount,
            quote_token,
            quote_amount,
            kind,
            metadata,
        } = crate::com::TokenOrderV2::abi_decode_params_validate(operand.as_ref())?;
        Ok(Self {
            sender: sender.into(),
            receiver: receiver.into(),
            base_token: base_token.into(),
            base_amount: base_amount.into(),
            quote_token: quote_token.into(),
            quote_amount: quote_amount.into(),
            metadata: TokenOrderV2Metadata::decode(kind, metadata)?,
        })
    }

    pub(crate) fn into_instruction(self) -> Instruction {
        let (kind, metadata) = self.metadata.encode();

        Instruction::new(
            OP_TOKEN_ORDER,
            INSTR_VERSION_2,
            crate::com::TokenOrderV2 {
                sender: self.sender.into(),
                receiver: self.receiver.into(),
                base_token: self.base_token.into(),
                base_amount: self.base_amount.into(),
                quote_token: self.quote_token.into(),
                quote_amount: self.quote_amount.into(),
                kind,
                metadata: metadata.into(),
            },
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case", tag = "@kind")
)]
pub enum TokenOrderV2Metadata {
    Initialize(TokenMetadata),
    Escrow { data: Bytes },
    Unescrow { data: Bytes },
    Solve(SolverMetadata),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
pub struct TokenMetadata {
    pub implementation: Bytes,
    pub initializer: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
pub struct SolverMetadata {
    pub solver_address: Bytes,
    pub metadata: Bytes,
}

impl TokenOrderV2Metadata {
    pub(crate) fn decode(kind: u8, metadata: impl AsRef<[u8]>) -> Result<Self> {
        match kind {
            TOKEN_ORDER_KIND_INITIALIZE => Ok(
                crate::com::TokenMetadata::abi_decode_params_validate(metadata.as_ref()).map(
                    |crate::com::TokenMetadata {
                         implementation,
                         initializer,
                     }| {
                        Self::Initialize(TokenMetadata {
                            implementation: implementation.into(),
                            initializer: initializer.into(),
                        })
                    },
                )?,
            ),
            TOKEN_ORDER_KIND_ESCROW => Ok(Self::Escrow {
                data: metadata.as_ref().into(),
            }),
            TOKEN_ORDER_KIND_UNESCROW => Ok(Self::Unescrow {
                data: metadata.as_ref().into(),
            }),
            TOKEN_ORDER_KIND_SOLVE => Ok(crate::com::SolverMetadata::abi_decode_params_validate(
                metadata.as_ref(),
            )
            .map(
                |crate::com::SolverMetadata {
                     solverAddress,
                     metadata,
                 }| {
                    Self::Solve(SolverMetadata {
                        solver_address: solverAddress.into(),
                        metadata: metadata.into(),
                    })
                },
            )?),
            invalid => Err(format!("invalid token order v2 metadata kind: {invalid}"))?,
        }
    }

    pub(crate) fn encode(self) -> (u8, Bytes) {
        match self {
            TokenOrderV2Metadata::Initialize(TokenMetadata {
                implementation,
                initializer,
            }) => (
                TOKEN_ORDER_KIND_INITIALIZE,
                crate::com::TokenMetadata::abi_encode_params(&crate::com::TokenMetadata {
                    implementation: implementation.into(),
                    initializer: initializer.into(),
                })
                .into(),
            ),
            TokenOrderV2Metadata::Escrow { data } => (TOKEN_ORDER_KIND_ESCROW, data),
            TokenOrderV2Metadata::Unescrow { data } => (TOKEN_ORDER_KIND_UNESCROW, data),
            TokenOrderV2Metadata::Solve(SolverMetadata {
                solver_address,
                metadata,
            }) => (
                TOKEN_ORDER_KIND_SOLVE,
                crate::com::SolverMetadata::abi_encode_params(&crate::com::SolverMetadata {
                    solverAddress: solver_address.into(),
                    metadata: metadata.into(),
                })
                .into(),
            ),
        }
    }
}
