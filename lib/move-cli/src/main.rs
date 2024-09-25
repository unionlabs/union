// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use anyhow::{Context, Result};
use aptos_crypto::ed25519::PrivateKey;
use aptos_sdk::{
    coin_client::CoinClient,
    rest_client::{Client, FaucetClient},
    types::{AccountKey, LocalAccount},
};
use once_cell::sync::Lazy;
use url::Url;

// :!:>section_1c
static NODE_URL: Lazy<Url> = Lazy::new(|| {
    Url::from_str(
        std::env::var("APTOS_NODE_URL")
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("http://localhost:30731"),
    )
    .unwrap()
});

#[tokio::main]
async fn main() -> Result<()> {
    // :!:>section_1a
    let rest_client = Client::new(NODE_URL.clone());

    // :!:>section_1b
    let coin_client = CoinClient::new(&rest_client); // <:!:section_1b

    // Create two accounts locally, Alice and Bob.
    // :!:>section_2
    let alice = LocalAccount::from_private_key(
        "0x9230ed379bc61c0b1b84ceafad18b55f7b70de7d288c1eabe1c2e9c0650e7d72",
        0,
    )
    .unwrap();

    // Print account addresses.
    println!("\n=== Addresses ===");
    println!("Alice: {}", alice.address().to_hex_literal());

    let balance = coin_client.get_account_balance(&alice.address());

    println!("Balance: {:?}", balance);

    Ok(())
}
