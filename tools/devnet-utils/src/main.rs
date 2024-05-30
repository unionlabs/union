use base64::{display::Base64Display, engine::general_purpose::STANDARD};
use bip39::Mnemonic;
use clap::{Parser, Subcommand};
use ed25519_compact::{KeyPair, Seed};
use unionlabs::hash::{H160, H256};

#[derive(Parser)]
#[clap(arg_required_else_help = true)]
struct App {
    #[clap(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    #[clap(subcommand)]
    Keygen(KeygenCmd),
    #[command(subcommand)]
    Compute(ComputeCmd),
}

#[derive(Subcommand)]
enum KeygenCmd {
    Key {
        mnemonic: Mnemonic,
        #[arg(long)]
        key_type: KeyType,
    },
    Mnemonic {
        seed: String,
    },
}

#[derive(Subcommand)]
pub enum ComputeCmd {
    Instantiate2Address {
        #[arg(long)]
        creator: H160,
        #[arg(long)]
        checksum: H256,
        #[arg(long)]
        salt: String,
    },
}

#[derive(strum::EnumString, Clone)]
#[strum(serialize_all = "kebab-case")]
enum KeyType {
    Ed25519,
}

fn main() {
    let app = App::parse();

    match app.command {
        Cmd::Keygen(cmd) => match cmd {
            KeygenCmd::Key { mnemonic, key_type } => match key_type {
                KeyType::Ed25519 => {
                    let key_pair =
                        KeyPair::from_seed(Seed::new(mnemonic.to_entropy().try_into().unwrap()));

                    println!("{}", Base64Display::new(&*key_pair.sk, &STANDARD));
                }
            },
            KeygenCmd::Mnemonic { seed } => {
                println!(
                    "{}",
                    Mnemonic::from_entropy(&hex::decode(seed).unwrap()).unwrap()
                );
            }
        },
        Cmd::Compute(cmd) => match cmd {
            ComputeCmd::Instantiate2Address {
                creator,
                checksum,
                salt,
            } => {
                println!(
                    "{}",
                    hex::encode(
                        &*cosmwasm_std::instantiate2_address(
                            &checksum.0,
                            &creator.0.into(),
                            &hex::decode(salt).unwrap(),
                        )
                        .unwrap()
                    )
                );
            }
        },
    }
}
