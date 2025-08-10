use alloy_sol_types::SolType;
use enumorph::Enumorph;
use ucs03_zkgm::com::{
    INSTR_VERSION_1, INSTR_VERSION_2, TOKEN_ORDER_KIND_ESCROW, TOKEN_ORDER_KIND_INITIALIZE,
    TOKEN_ORDER_KIND_UNESCROW,
};
use unionlabs_primitives::{Bytes, U256};

use super::Result;

#[derive(Debug, Clone, PartialEq, Eq, Enumorph)]
pub enum TokenOrder {
    V1(TokenOrderV1),
    V2(TokenOrderV2),
}

impl TokenOrder {
    pub(crate) fn decode(version: u8, operand: impl AsRef<[u8]>) -> Result<Self> {
        match version {
            INSTR_VERSION_1 => TokenOrderV1::decode(operand).map(Into::into),
            INSTR_VERSION_2 => TokenOrderV2::decode(operand).map(Into::into),
            invalid => Err(format!("invalid token order version: {invalid}"))?,
        }
    }
}

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenMetadata {
    pub implementation: Bytes,
    pub initializer: Bytes,
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
            invalid => Err(format!("invalid token order v2 metadata kind: {invalid}"))?,
        }
    }
}
