#![expect(deprecated)]

use alloy_sol_types::SolType;
use enumorph::Enumorph;
use ucs03_zkgm::com::{
    FILL_TYPE_MARKETMAKER, FILL_TYPE_PROTOCOL, INSTR_VERSION_1, INSTR_VERSION_2,
    TOKEN_ORDER_KIND_ESCROW, TOKEN_ORDER_KIND_INITIALIZE, TOKEN_ORDER_KIND_SOLVE,
    TOKEN_ORDER_KIND_UNESCROW,
};
use unionlabs_primitives::{Bytes, U256};

use crate::Result;

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
#[repr(u8)]
pub enum TokenOrder {
    #[deprecated(since = "TBD")]
    V1(TokenOrderV1) = INSTR_VERSION_1,
    V2(TokenOrderV2) = INSTR_VERSION_2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
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
}

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
#[repr(u8)]
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
}

#[deprecated(note = "token order v1 will be superceded by v2", since = "TBD")]
#[derive(Debug, Clone, PartialEq, Eq)]
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
        let ucs03_zkgm::com::TokenOrderV1 {
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
        } = ucs03_zkgm::com::TokenOrderV1::abi_decode_params_validate(operand.as_ref())?;
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
}

#[deprecated(note = "token order v1 will be superceded by v2", since = "TBD")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenOrderV1Ack {
    Protocol,
    MarketMaker { market_maker: Bytes },
}

impl TokenOrderV1Ack {
    pub(crate) fn decode(ack: impl AsRef<[u8]>) -> Result<Self> {
        let ucs03_zkgm::com::TokenOrderAck {
            fill_type,
            market_maker,
        } = ucs03_zkgm::com::TokenOrderAck::abi_decode_params_validate(ack.as_ref())?;

        match fill_type {
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenOrderV2Ack {
    Protocol,
    MarketMaker { market_maker: Bytes },
}

impl TokenOrderV2Ack {
    pub(crate) fn decode(ack: impl AsRef<[u8]>) -> Result<Self> {
        let ucs03_zkgm::com::TokenOrderAck {
            fill_type,
            market_maker,
        } = ucs03_zkgm::com::TokenOrderAck::abi_decode_params_validate(ack.as_ref())?;

        match fill_type {
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
        let ucs03_zkgm::com::TokenOrderV2 {
            sender,
            receiver,
            base_token,
            base_amount,
            quote_token,
            quote_amount,
            kind,
            metadata,
        } = ucs03_zkgm::com::TokenOrderV2::abi_decode_params_validate(operand.as_ref())?;
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenOrderV2Metadata {
    Initialize(TokenMetadata),
    Escrow(Bytes),
    Unescrow(Bytes),
    Solve(SolveMetadata),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenMetadata {
    pub implementation: Bytes,
    pub initializer: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolveMetadata {
    pub solver_address: Bytes,
    pub metadata: Bytes,
}

impl TokenOrderV2Metadata {
    pub(crate) fn decode(kind: u8, metadata: impl AsRef<[u8]>) -> Result<Self> {
        match kind {
            TOKEN_ORDER_KIND_INITIALIZE => Ok(
                ucs03_zkgm::com::TokenMetadata::abi_decode_params_validate(metadata.as_ref()).map(
                    |ucs03_zkgm::com::TokenMetadata {
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
            TOKEN_ORDER_KIND_ESCROW => Ok(Self::Escrow(metadata.as_ref().into())),
            TOKEN_ORDER_KIND_UNESCROW => Ok(Self::Unescrow(metadata.as_ref().into())),
            TOKEN_ORDER_KIND_SOLVE => Ok(
                ucs03_zkgm::com::SolverMetadata::abi_decode_params_validate(metadata.as_ref())
                    .map(
                        |ucs03_zkgm::com::SolverMetadata {
                             solverAddress,
                             metadata,
                         }| {
                            Self::Solve(SolveMetadata {
                                solver_address: solverAddress.into(),
                                metadata: metadata.into(),
                            })
                        },
                    )?,
            ),
            invalid => Err(format!("invalid token order v2 metadata kind: {invalid}"))?,
        }
    }
}
