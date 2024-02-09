use base64::{display::Base64Display, engine::general_purpose::STANDARD};
use bip39::Mnemonic;
use clap::Parser;
use ed25519_compact::{KeyPair, Seed};

#[derive(clap::Parser)]
enum App {
    Key {
        mnemonic: Mnemonic,
        #[arg(long)]
        key_type: KeyType,
    },
    Mnemonic {
        seed: String,
    },
}

#[derive(strum::EnumString, Clone)]
#[strum(serialize_all = "kebab-case")]
enum KeyType {
    Ed25519,
}

fn main() {
    let app = App::parse();

    match app {
        App::Key { mnemonic, key_type } => match key_type {
            KeyType::Ed25519 => {
                let key_pair =
                    KeyPair::from_seed(Seed::new(mnemonic.to_entropy().try_into().unwrap()));

                println!("{}", Base64Display::new(&*key_pair.sk, &STANDARD));
            }
        },
        App::Mnemonic { seed } => {
            println!(
                "{}",
                Mnemonic::from_entropy(&hex::decode(seed).unwrap()).unwrap()
            );
        }
    }
}

#[test]
fn mnemonic() {}
