use alloy_sol_types::SolType;
use enumorph::Enumorph;
use ucs03_zkgm::com::{INSTR_VERSION_1, INSTR_VERSION_2};
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
        let instruction =
            ucs03_zkgm::com::TokenOrderV1::abi_decode_params_validate(operand.as_ref())?;
        Ok(Self {
            sender: instruction.sender.into(),
            receiver: instruction.receiver.into(),
            base_token: instruction.base_token.into(),
            base_amount: instruction.base_amount.into(),
            base_token_symbol: instruction.base_token_symbol,
            base_token_name: instruction.base_token_name,
            base_token_decimals: instruction.base_token_decimals,
            base_token_path: instruction.base_token_path.into(),
            quote_token: instruction.quote_token.into(),
            quote_amount: instruction.quote_amount.into(),
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
    pub kind: u8,
    pub metadata: Bytes,
}

impl TokenOrderV2 {
    pub(crate) fn decode(operand: impl AsRef<[u8]>) -> Result<Self> {
        let instruction =
            ucs03_zkgm::com::TokenOrderV2::abi_decode_params_validate(operand.as_ref())?;
        Ok(Self {
            sender: instruction.sender.into(),
            receiver: instruction.receiver.into(),
            base_token: instruction.base_token.into(),
            base_amount: instruction.base_amount.into(),
            quote_token: instruction.quote_token.into(),
            quote_amount: instruction.quote_amount.into(),
            kind: instruction.kind,
            metadata: instruction.metadata.into(),
        })
    }
}
