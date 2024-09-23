use base64::{display::Base64Display, engine::general_purpose::STANDARD};
use bip39::Mnemonic;
use clap::{Parser, Subcommand};
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
        #[arg(long, default_value_t = OutputFormat::Base64)]
        output: OutputFormat,
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
    Secp256k1,
}

#[derive(strum::EnumString, strum::Display, Clone)]
#[strum(serialize_all = "kebab-case")]
pub enum OutputFormat {
    Base64,
    Hex,
}

fn main() {
    let app = App::parse();

    match app.command {
        Cmd::Keygen(cmd) => match cmd {
            KeygenCmd::Key {
                mnemonic,
                key_type,
                output,
            } => {
                let bz = match key_type {
                    KeyType::Ed25519 => ed25519_compact::KeyPair::from_seed(
                        ed25519_compact::Seed::new(mnemonic.to_entropy().try_into().unwrap()),
                    )
                    .sk
                    .to_vec(),
                    KeyType::Secp256k1 => {
                        tiny_hderive::bip32::ExtendedPrivKey::derive(
                            &mnemonic.to_seed(""),
                            // this is the default cosmossdk hd path
                            "m/44'/118'/0'/0/0",
                        )
                        .unwrap()
                        .secret()
                        .to_vec()
                    }
                };

                match output {
                    OutputFormat::Base64 => {
                        println!("{}", Base64Display::new(&bz, &STANDARD));
                    }
                    OutputFormat::Hex => {
                        println!("{}", hex::encode(bz));
                    }
                }
            }
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
                            checksum.as_ref(),
                            &creator.get().into(),
                            &hex::decode(salt).unwrap(),
                        )
                        .unwrap()
                    )
                );
            }
        },
    }
}
