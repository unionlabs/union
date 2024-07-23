use std::{str::FromStr, time};

use chain_utils::{
    cosmos_sdk::{CosmosSdkChain, CosmosSdkChainExt, GasConfig},
    private_key::PrivateKey,
};
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::{
    action::{Action, FunctionCallAction},
    hash::CryptoHash,
    transaction::Transaction,
    types::{AccountId, BlockId, BlockReference},
    views::QueryRequest,
};
use serde::Deserialize;
use tendermint_rpc::WebSocketClientUrl;
use unionlabs::{
    encoding::{EncodeAs, Proto},
    google::protobuf::any::{mk_any, IntoAny},
    hash::H256,
    ibc::{
        core::{client::height::Height, commitment::merkle_root::MerkleRoot},
        lightclients::{
            cometbls::{client_state::ClientState, consensus_state::ConsensusState},
            near, wasm,
        },
    },
};

#[derive(serde::Serialize)]
pub struct CreateClient {
    pub client_type: String,
    pub client_state: Vec<u8>,
    pub consensus_state: Vec<u8>,
}

#[derive(serde::Serialize)]
pub struct RegisterClient {
    pub client_type: String,
    pub account: String,
}

struct Near {
    rpc: JsonRpcClient,
    light_client_id: AccountId,
    ibc_id: AccountId,
    signer: near_crypto::InMemorySigner,
}

struct Union {
    union: chain_utils::union::Union,
}

impl Union {
    async fn new() -> Self {
        Union {
            union: chain_utils::union::Union::new(chain_utils::union::Config {
                signers: vec![
                    serde_json::from_str(
                        r#"{ "raw": "0xaa820fa947beb242032a41b6dc9a8b9c37d8f5fbcda0966b1ec80335b10a7d6f" }"#,
                    )
                    .unwrap(),
                ],
                ws_url: WebSocketClientUrl::from_str("ws://localhost:26657/websocket").unwrap(),
                prover_endpoint: "http://localhost:9999".to_string(),
                grpc_url: "http://localhost:9090".to_string(),
                gas_config: GasConfig {
                    gas_price: 1.0,
                    gas_denom: "muno".to_string(),
                    gas_multiplier: 1.1,
                    max_gas: 400000,
                },
            })
            .await
            .unwrap(),
        }
    }

    async fn create_client<A: IntoAny, B: IntoAny>(&self, client_state: A, consensus_state: B) {
        self.union
            .signers()
            .with(|signer| async {
                let msg = protos::ibc::core::client::v1::MsgCreateClient {
                    client_state: Some(client_state.into_any().into()),
                    consensus_state: Some(consensus_state.into_any().into()),
                    signer: signer.to_string(),
                };

                let tx_hash = self
                    .union
                    .broadcast_tx_commit(signer, [mk_any(&msg)])
                    .await
                    .unwrap();

                println!("[ + ] Near client created: {tx_hash}");
            })
            .await;
    }
}

impl Near {
    fn new() -> Self {
        let rpc = JsonRpcClient::connect("http://localhost:3030");
        let light_client_id: AccountId = "cometbls-light-client.node0".parse().unwrap();
        let ibc_id: AccountId = "ibc-union.node0".parse().unwrap();
        let secret_key = "ed25519:3D4YudUQRE39Lc4JHghuB5WM8kbgDDa34mnrEP5DdTApVH81af7e2dWgNPEaiQfdJnZq1CNPp5im4Rg5b733oiMP".parse().unwrap();
        let account_id = "node0".parse().unwrap();
        let signer = near_crypto::InMemorySigner::from_secret_key(account_id, secret_key);

        Near {
            rpc,
            light_client_id,
            ibc_id,
            signer,
        }
    }

    async fn self_consensus_state(&self, height: u64) -> near::consensus_state::ConsensusState {
        let block = self
            .rpc
            .call(methods::block::RpcBlockRequest {
                block_reference: BlockReference::BlockId(BlockId::Height(height)),
            })
            .await
            .unwrap();
        let chunk_prev_state_root = block.header.prev_state_root;
        let timestamp = block.header.timestamp_nanosec;
        near::consensus_state::ConsensusState {
            state: block_header_to_inner_lite(block.header),
            chunk_prev_state_root,
            timestamp,
        }
    }

    async fn self_client_state(&self) -> near::client_state::ClientState {
        let chain_id = self
            .rpc
            .call(methods::status::RpcStatusRequest)
            .await
            .unwrap()
            .chain_id;

        let block = self
            .rpc
            .call(methods::block::RpcBlockRequest {
                block_reference: BlockReference::Finality(near_primitives::types::Finality::Final),
            })
            .await
            .unwrap();

        let validators = self
            .rpc
            .call(
                methods::EXPERIMENTAL_validators_ordered::RpcValidatorsOrderedRequest {
                    block_id: Some(BlockId::Height(block.header.height)),
                },
            )
            .await
            .unwrap();
        near::client_state::ClientState {
            chain_id,
            latest_height: block.header.height - 1,
            ibc_account_id: "acc.near".to_string().try_into().unwrap(),
            // TODO(aeryz): this is only valid in this sandboxed environment where the validator set is not changing. For a real environment,
            // the relayer must read the block producers using another endpoint.
            initial_block_producers: convert_block_producers(validators),
            frozen_height: 0,
        }
    }

    async fn send_tx<T: serde::Serialize>(&self, method_name: &str, args: T) {
        let access_key_query_response = self
            .rpc
            .call(methods::query::RpcQueryRequest {
                block_reference: BlockReference::latest(),
                request: near_primitives::views::QueryRequest::ViewAccessKey {
                    account_id: self.signer.account_id.clone(),
                    public_key: self.signer.public_key.clone(),
                },
            })
            .await
            .unwrap();

        println!("access key {:?}", access_key_query_response);

        let current_nonce = match access_key_query_response.kind {
            QueryResponseKind::AccessKey(access_key) => access_key.nonce,
            _ => panic!("failed to extract current nonce"),
        };

        let transaction = Transaction {
            signer_id: self.signer.account_id.clone(),
            public_key: self.signer.public_key.clone(),
            nonce: current_nonce + 1,
            receiver_id: self.ibc_id.clone(),
            block_hash: access_key_query_response.block_hash,
            actions: vec![Action::FunctionCall(Box::new(FunctionCallAction {
                method_name: method_name.to_string(),
                args: serde_json::to_string(&args).unwrap().into_bytes(),
                gas: 100_000_000_000_000, // 100 TeraGas
                deposit: 0,
            }))],
        };

        let request = methods::send_tx::RpcSendTransactionRequest {
            signed_transaction: transaction.sign(&self.signer),
            wait_until: near_primitives::views::TxExecutionStatus::Final,
        };

        let response = self.rpc.call(request).await.unwrap();

        for out in response
            .final_execution_outcome
            .unwrap()
            .into_outcome()
            .receipts_outcome
        {
            if !out.outcome.logs.is_empty() {
                println!("[ i ] Logs: {:?}", out.outcome.logs);
            }
        }
    }

    async fn register_client(&self) {
        let register_client = RegisterClient {
            client_type: "cometbls".to_string(),
            account: self.light_client_id.to_string(),
        };
        self.send_tx("register_client", register_client).await;
    }

    async fn create_client(&self) {
        let block = self
            .rpc
            .call(methods::block::RpcBlockRequest {
                block_reference: BlockReference::Finality(near_primitives::types::Finality::Final),
            })
            .await
            .unwrap();

        let timestamp = block.header.timestamp_nanosec;

        let create_client = CreateClient {
            client_type: "cometbls".to_string(),
            client_state: ClientState {
                chain_id: "hello".to_string(),
                trusting_period: 1000,
                unbonding_period: 1000,
                max_clock_drift: 1000,
                frozen_height: Height {
                    revision_number: 0,
                    revision_height: 0,
                },
                latest_height: Height {
                    revision_number: 0,
                    revision_height: 8,
                },
            }
            .encode_as::<Proto>(),
            consensus_state: ConsensusState {
                timestamp,
                app_hash: MerkleRoot {
                    hash: H256::default(),
                },
                next_validators_hash: H256::default(),
            }
            .encode_as::<Proto>(),
        };

        self.send_tx("create_client", create_client).await;

        println!("[ + ] Client created!");
    }
}

#[tokio::main]
async fn main() {
    let near = Near::new();
    // near.register_client().await;
    // near.create_client().await;

    let union = Union::new().await;
    let cs = near.self_client_state().await;
    union
        .create_client(
            wasm::client_state::ClientState {
                data: cs.clone(),
                checksum: hex::decode(
                    "b6a4778f785d67f90fd85d96e2c1447be115f2211e8ec9fbd209c84bd2517191",
                )
                .unwrap()
                .try_into()
                .unwrap(),
                latest_height: Height {
                    revision_number: 0,
                    revision_height: cs.latest_height,
                },
            },
            wasm::consensus_state::ConsensusState {
                data: near.self_consensus_state(cs.latest_height + 1).await,
            },
        )
        .await;
}

pub fn convert_block_producers(
    bps: Vec<near_primitives::views::validator_stake_view::ValidatorStakeView>,
) -> Vec<near::validator_stake_view::ValidatorStakeView> {
    bps.into_iter()
        .map(|stake| {
            let near_primitives::views::validator_stake_view::ValidatorStakeView::V1(stake) = stake;
            let stake = near::validator_stake_view::ValidatorStakeView::V1(
                near::validator_stake_view::ValidatorStakeViewV1 {
                    account_id: stake.account_id,
                    public_key: unionlabs::near::types::PublicKey::Ed25519(
                        stake.public_key.key_data().try_into().unwrap(),
                    ),
                    stake: stake.stake,
                },
            );
            stake
        })
        .collect()
}

pub fn block_header_to_inner_lite(
    header: near_primitives::views::BlockHeaderView,
) -> near::block_header_inner::BlockHeaderInnerLiteView {
    use near_primitives_core::hash::CryptoHash;
    near::block_header_inner::BlockHeaderInnerLiteView {
        height: header.height,
        epoch_id: CryptoHash(header.epoch_id.0),
        next_epoch_id: CryptoHash(header.next_epoch_id.0),
        prev_state_root: CryptoHash(header.prev_state_root.0),
        outcome_root: CryptoHash(header.outcome_root.0),
        timestamp: header.timestamp,
        timestamp_nanosec: header.timestamp_nanosec,
        next_bp_hash: CryptoHash(header.next_bp_hash.0),
        block_merkle_root: CryptoHash(header.block_merkle_root.0),
    }
}
