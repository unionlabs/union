use std::str::FromStr;

use alloy::{primitives::Keccak256, sol_types::SolValue};
use anyhow::Result;
use clap::{Args, Subcommand};
use serde::Serialize;
use ucs03_zkgm::com::{
    Batch, Instruction, SolverMetadata, TokenMetadata, TokenOrderV1, TokenOrderV2, INSTR_VERSION_0,
    INSTR_VERSION_1, INSTR_VERSION_2, OP_BATCH, OP_TOKEN_ORDER, TOKEN_ORDER_KIND_INITIALIZE,
    TOKEN_ORDER_KIND_SOLVE,
};
use unionlabs::{
    encoding::Decode,
    primitives::{encoding::HexUnprefixed, Bytes, H256, U256},
};

#[derive(Debug, Subcommand)]
pub enum Cmd {
    V1(TokenOrderV1V1Args),
    V2Sui(TokenOrderV2ArgsSui),
}

#[derive(Debug, Serialize, Clone, Args)]
pub struct SuiMetadata {
    #[arg(long)]
    name: String,
    #[arg(long)]
    symbol: String,
    #[arg(long)]
    decimals: u8,
    #[arg(long, default_value_t = Default::default())]
    owner: H256,
    #[arg(long)]
    icon_url: Option<String>,
    #[arg(long)]
    description: String,
}

#[derive(Debug, Args)]
pub struct TokenOrderV2Base {
    #[arg(long)]
    sender: Bytes,
    #[arg(long)]
    receiver: Bytes,
    #[arg(long)]
    base_token: Bytes,
    #[arg(long)]
    base_amount: U256,
    #[arg(long)]
    quote_token: Option<Bytes>,
    #[arg(long)]
    quote_amount: U256,
}

#[derive(Debug, Args)]
pub struct TokenOrderV2ArgsSui {
    #[arg(long)]
    channel: u32,
    #[clap(flatten)]
    base: TokenOrderV2Base,
    #[clap(flatten)]
    metadata: SuiMetadata,
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
            Cmd::V2Sui(fao) => {
                // let metadata = TokenMetadata {
                //     implementation: Default::default(),
                //     initializer: bcs::to_bytes(&fao.metadata).unwrap().into(),
                // }
                // .abi_encode_params()
                // .into();

                // let quote_token = match fao.base.quote_token {
                //     Some(qt) => qt.into(),
                //     None => {
                //         let mut h = Keccak256::new();
                //         h.update(&metadata);
                //         predict_wrapped_token_sui(
                //             U256::ZERO,
                //             fao.channel,
                //             fao.base.base_token.clone().into(),
                //             h.finalize().to_vec(),
                //         )
                //         .into()
                //     }
                // };

                let metadata = SolverMetadata {
                    solverAddress: Bytes::<HexUnprefixed>::from_str(
                        "c113302f2d081c65299ac245ce043c124368af79b5a9ce38d86855841b64d386",
                    )
                    .unwrap()
                    .into(),
                    metadata: Default::default(),
                }
                .abi_encode_params()
                .into();

                let instruction: Bytes = Instruction {
                    version: INSTR_VERSION_2,
                    opcode: OP_TOKEN_ORDER,
                    operand: TokenOrderV2 {
                        sender: fao.base.sender.into(),
                        receiver: fao.base.receiver.into(),
                        base_token: fao.base.base_token.into(),
                        base_amount: fao.base.base_amount.into(),
                        quote_token: fao.base.quote_token.unwrap().into(),
                        quote_amount: fao.base.quote_amount.into(),
                        kind: TOKEN_ORDER_KIND_SOLVE,
                        metadata,
                    }
                    .abi_encode_params()
                    .into(),
                }
                .abi_encode_params()
                .into();

                // let batch: Bytes = Instruction {
                //     version: INSTR_VERSION_0,
                //     opcode: OP_BATCH,
                //     operand: Batch {
                //         instructions: vec![instruction.clone(), instruction],
                //     }
                //     .abi_encode_params()
                //     .into(),
                // }
                // .abi_encode_params()
                // .into();
                //

                println!("{instruction}");
            }
        }

        Ok(())
    }
}

fn predict_wrapped_token_sui(
    path: U256,
    channel: u32,
    base_token: Vec<u8>,
    metadata_image: Vec<u8>,
) -> Vec<u8> {
    let mut h = Keccak256::new();
    h.update(bcs::to_bytes(&path.to_le_bytes()).unwrap());
    h.update(bcs::to_bytes(&channel).unwrap());
    h.update(base_token);
    h.update(metadata_image);
    h.finalize().to_vec()
}
