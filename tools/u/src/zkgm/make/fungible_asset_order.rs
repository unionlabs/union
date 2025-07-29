use alloy::sol_types::SolValue;
use anyhow::Result;
use clap::{Args, Subcommand};
use ucs03_zkgm::com::{Instruction, TokenOrderV1, INSTR_VERSION_1, OP_TOKEN_ORDER};
use unionlabs::primitives::{Bytes, U256};

#[derive(Debug, Subcommand)]
pub enum Cmd {
    V1(TokenOrderV1V1Args),
}

#[derive(Debug, Args)]
pub struct TokenOrderV1V1Args {
    #[arg(long)]
    sender: Bytes,
    #[arg(long)]
    receiver: Bytes,
    #[arg(long)]
    base_token: Bytes,
    #[arg(long)]
    base_amount: U256,
    #[arg(long)]
    base_token_symbol: String,
    #[arg(long)]
    base_token_name: String,
    #[arg(long)]
    base_token_decimals: u8,
    #[arg(long)]
    base_token_path: U256,
    #[arg(long)]
    quote_token: Bytes,
    #[arg(long)]
    quote_amount: U256,
}

impl Cmd {
    pub fn run(self) -> Result<()> {
        match self {
            Cmd::V1(fungible_asset_order_v1_args) => {
                let instruction: Bytes = Instruction {
                    version: INSTR_VERSION_1,
                    opcode: OP_TOKEN_ORDER,
                    operand: TokenOrderV1 {
                        sender: fungible_asset_order_v1_args.sender.into(),
                        receiver: fungible_asset_order_v1_args.receiver.into(),
                        base_token: fungible_asset_order_v1_args.base_token.into(),
                        base_amount: fungible_asset_order_v1_args.base_amount.into(),
                        base_token_symbol: fungible_asset_order_v1_args.base_token_symbol,
                        base_token_name: fungible_asset_order_v1_args.base_token_name,
                        base_token_decimals: fungible_asset_order_v1_args.base_token_decimals,
                        base_token_path: fungible_asset_order_v1_args.base_token_path.into(),
                        quote_token: fungible_asset_order_v1_args.quote_token.into(),
                        quote_amount: fungible_asset_order_v1_args.quote_amount.into(),
                    }
                    .abi_encode_params()
                    .into(),
                }
                .abi_encode_params()
                .into();

                println!("{instruction}");
            }
        }

        Ok(())
    }
}
