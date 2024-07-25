use std::{env, str::FromStr, thread::sleep, time::Duration};

use hex_literal::hex;
use near_primitives_core::hash::CryptoHash;
use near_workspaces::{
    network::Sandbox,
    sandbox, testnet,
    types::{Gas, KeyType, NearToken, SecretKey},
    Account, AccountId, Contract, Worker,
};
use unionlabs::{
    encoding::{EncodeAs, Proto},
    google::protobuf::timestamp::Timestamp,
    hash::H256,
    ibc::lightclients::cometbls::{client_state::ClientState, light_header::LightHeader},
};

const VERIFIER_ENV: &str = "VERIFIER";

mod alice {
    pub const CLIENT_TYPE: &str = "near-alice";
}
mod bob {
    pub const CLIENT_TYPE: &str = "near-bob";
}

#[derive(serde::Serialize)]
pub struct CreateClient {
    pub client_id: String,
    pub client_state: Vec<u8>,
    pub consensus_state: Vec<u8>,
}

#[derive(serde::Serialize)]
pub struct TestCircuit {
    chain_id: String,
    trusted_validators_hash: H256,
    header: LightHeader,
    zkp: Vec<u8>,
}

pub async fn deploy_contract(
    sandbox: &Worker<Sandbox>,
    account_id: &str,
    env_key: &'static str,
) -> Contract {
    let wasm_path = env::var(env_key).unwrap();
    let wasm_blob = std::fs::read(wasm_path).unwrap();
    let account_id = account_id.to_string().try_into().unwrap();
    let secret_key = SecretKey::from_seed(KeyType::ED25519, "testificate");
    sandbox
        .create_tla_and_deploy(account_id, secret_key, &wasm_blob)
        .await
        .unwrap()
        .unwrap()
}

#[tokio::main]
async fn main() {
    let sandbox = sandbox().await.unwrap();
    let verifier_contract = deploy_contract(&sandbox, "verifier.test.near", VERIFIER_ENV).await;
    let owner = sandbox.root_account().unwrap();
    let user = owner
        .create_subaccount("user")
        .initial_balance(NearToken::from_near(30))
        .transact()
        .await
        .unwrap()
        .into_result()
        .unwrap();
    let res = user
        .call(verifier_contract.id(), "initialize")
        .args_json(CreateClient {
            client_id: String::from("08-wasm-0"),
            client_state: ClientState::default().encode_as::<Proto>(),
            consensus_state: Vec::new(),
        })
        .transact()
        .await
        .unwrap();
    println!("res1: {res:?}");

    let res = user
        .call(verifier_contract.id(), "test_circuit")
        .gas(Gas::from_gas(300000000000000))
        .args_json(TestCircuit {
            chain_id: "union-testnet-8".into(),
            trusted_validators_hash: hex!(
                "1deda64b1cc1319718f168b5aa8ed904b7d5b0ab932acdf6deae0ad9bd565a53"
            )
            .into(),
            header: LightHeader {
                height: 969001.try_into().unwrap(),
                time: Timestamp::from_str("2024-06-18T13:20:56.784169335Z").unwrap(),
                validators_hash: hex!(
                    "1deda64b1cc1319718f168b5aa8ed904b7d5b0ab932acdf6deae0ad9bd565a53"
                )
                .into(),
                next_validators_hash: hex!(
                    "01a84dca649aa2df8de2f65a84c9092bbd5296b4bc54d818f844b28573d8e0be"
                )
                .into(),
                app_hash: hex!("1818da4a8b1c430557a3018adc2bf9a06e56c3b530e5cce7709232e0f03bd9ab")
                    .into(),
            },
            zkp: hex!("086541c22b53d509d8369492d32683188f0b379950ea3c5da84aca2b331d911c163bc6e30c7610b6903832184d284399d140b316134202cfa53b695ed17db64e271a8ab10b015cc4562730180cc7af7d7509b64de00b5864ccef3ab6b5c187da1511c4af3392d5e4465cebeb3c92cad546ab6b5b7de08923ae756d4a49d972920ed4f1b33bde26016e753fe00e9ee8b37873e4df4696cce84baa34e444d6f9dc0021b25644dc22fd9414197dd9e094180eac33a5e6fc6d2e04e12df5baaae92815173080dedcafeb2789245e75f1c38ddaa4611273fa5eed1cb77f75aabace770186385a3a373190a9091147de95b3f11050152bc4376573ed454cfd703f1e7106edb33921b12717708fe03861534c812a5ea6c7e0ec428c02292f1e7dafb45901e8b29e0b18ba7cbfad2a7aef7db558f3eb49a943a379a03b1b976df912a0c329b66224da89f94e29c49b3c5070b86b23d9d23424246235088ea858a21340cc2d1120ac3dc25febd188abf16774ea49564f34bc769b6abd9295128c391dad18").to_vec(),
        })
        .transact()
        .await
        .unwrap();
    println!("res: {res:?}");
}
