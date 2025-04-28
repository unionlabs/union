use std::error::Error;

use alloy_sol_types::SolType;
use ucs03_zkgm::com::{INSTR_VERSION_0, OP_FUNGIBLE_ASSET_ORDER};

pub enum Instruction {
    FungibleAssetOrder(FungibleAssetOrderInstruction),
    Batch(BatchInstruction),
    Multiplex(MultiplexInstruction),
}

impl Instruction {
    pub fn decode(bz: &[u8]) -> Result<Self> {
        let instruction = ucs03_zkgm::com::Instruction::abi_decode_params(bz, true)?;

        Ok(match (instruction.version, instruction.opcode) {
            (INSTR_VERSION_0, OP_FUNGIBLE_ASSET_ORDER) => Self::FungibleAssetOrder(
                FungibleAssetOrderInstruction::decode(&instruction.operand)?,
            ),
            invalid => Err(format!("invalid version and opcode: {invalid:?}"))?,
        })
    }
}

pub struct FungibleAssetOrderInstruction {
    sender: Bytes,
    receiver: Bytes,
    base_token: Bytes,
    base_amount: U256,
    base_token_symbol: String,
    base_token_name: String,
    base_token_decimals: u8,
    base_token_path: U256,
    quote_token: Bytes,
    quote_amount: U256,
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;

impl FungibleAssetOrderInstruction {
    pub fn decode(bz: &[u8]) -> Result<Self> {
        let fao = ucs03_zkgm::com::FungibleAssetOrder::abi_decode_params(bz, true)?;

        Ok(Self {})
    }
}
