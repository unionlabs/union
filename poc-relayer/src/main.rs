use std::{
    str::FromStr,
    time::{self, Duration},
};

use chain_utils::{
    cosmos_sdk::{CosmosSdkChain, CosmosSdkChainExt, CosmosSdkChainRpcs as _, GasConfig},
    private_key::PrivateKey,
};
use ibc_vm_rs::{
    states::connection_handshake::{self, ConnectionEnd},
    IbcEvent, DEFAULT_IBC_VERSION,
};
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::{
    action::{Action, FunctionCallAction},
    hash::CryptoHash,
    transaction::Transaction,
    types::{AccountId, BlockId, BlockReference, Finality},
    views::{BlockHeaderInnerLiteView, LightClientBlockView, QueryRequest},
};
use serde::{Deserialize, Serialize};
use tendermint_rpc::{Client as _, WebSocketClientUrl};
use tokio::time::sleep;
use unionlabs::{
    encoding::{DecodeAs, Encode, EncodeAs, Proto},
    google::protobuf::any::{mk_any, Any, IntoAny},
    hash::H256,
    ibc::{
        core::{
            channel::{self, packet::Packet},
            client::height::Height,
            commitment::{merkle_prefix::MerklePrefix, merkle_root::MerkleRoot},
            connection::version::Version,
        },
        lightclients::{
            cometbls::{client_state::ClientState, consensus_state::ConsensusState},
            near, wasm,
        },
    },
    id::{ChannelId, ConnectionId, PortId},
    near::{
        raw_state_proof::RawStateProof,
        types::{self, MerklePathItem},
    },
    traits::Chain,
    validated::ValidateT as _,
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

#[derive(serde::Serialize)]
pub struct ConnectionOpenInit {
    pub client_id: String,
    pub counterparty: connection_handshake::Counterparty,
    pub version: Version,
    pub delay_period: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    bound(serialize = "", deserialize = ""),
    deny_unknown_fields,
    rename_all = "snake_case"
)]
pub enum AbciQueryType {
    State,
    Proof,
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

    async fn fetch_abci_query(&self, height: u64, ty: AbciQueryType, path: &str) -> Vec<u8> {
        const IBC_STORE_PATH: &str = "store/ibc/key";

        let mut client =
            protos::cosmos::base::tendermint::v1beta1::service_client::ServiceClient::connect(
                self.union.grpc_url().clone(),
            )
            .await
            .unwrap();

        let query_result = client
            .abci_query(
                protos::cosmos::base::tendermint::v1beta1::AbciQueryRequest {
                    data: path.as_bytes().to_vec(),
                    path: IBC_STORE_PATH.to_string(),
                    height: i64::try_from(height).unwrap() - 1_i64,
                    prove: matches!(ty, AbciQueryType::Proof),
                },
            )
            .await
            .unwrap()
            .into_inner();

        query_result.value
    }

    async fn connection_open_try<S: Into<protos::google::protobuf::Any>>(
        &self,
        client_id: &str,
        client_state: S,
        counterparty_client_id: &str,
        counterparty_connection_id: &str,
        proof_height: u64,
        proof_init: Vec<u8>,
        proof_client: Vec<u8>,
        proof_consensus: Vec<u8>,
        consensus_height: Height,
    ) {
        self.union
            .signers()
            .with(|signer| async {
                let msg = protos::ibc::core::connection::v1::MsgConnectionOpenTry {
                    client_id: client_id.to_string(),
                    client_state: Some(client_state.into()),
                    counterparty: Some(
                        connection_handshake::Counterparty {
                            client_id: counterparty_client_id.to_string().validate().unwrap(),
                            connection_id: counterparty_connection_id.to_string(),
                            prefix: MerklePrefix {
                                key_prefix: b"ibc".into(),
                            },
                        }
                        .into(),
                    ),
                    delay_period: 0,
                    counterparty_versions: DEFAULT_IBC_VERSION
                        .clone()
                        .into_iter()
                        .map(Into::into)
                        .collect(),
                    proof_height: Some(
                        Height {
                            revision_number: 0,
                            revision_height: proof_height,
                        }
                        .into(),
                    ),
                    proof_init,
                    proof_client,
                    proof_consensus,
                    consensus_height: Some(consensus_height.into()),
                    signer: signer.to_string(),
                    host_consensus_state_proof: vec![],
                    previous_connection_id: "".to_string(),
                };

                let tx_hash = self
                    .union
                    .broadcast_tx_commit(signer, [mk_any(&msg)])
                    .await
                    .unwrap();

                println!("[ + ] ConnectionOpenTry on Union: {tx_hash}");
            })
            .await;
    }

    async fn update_client<T: Encode<Proto>>(&self, client_id: &str, header: T) {
        self.union
            .signers()
            .with(|signer| async {
                let msg = protos::ibc::lightclients::wasm::v1::ClientMessage {
                    data: header.encode(),
                };

                let msg = protos::ibc::core::client::v1::MsgUpdateClient {
                    client_id: client_id.to_string(),
                    client_message: Some(mk_any(&msg)),
                    signer: signer.to_string(),
                };

                let tx_hash = self
                    .union
                    .broadcast_tx_commit(signer, [mk_any(&msg)])
                    .await
                    .unwrap();

                println!("[ + ] Near client updated: {tx_hash}");
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

    async fn state_proof(&self, block_height: u64, commitment_key: &str) -> Vec<u8> {
        #[derive(serde::Serialize)]
        pub struct RawStateProof {
            state_proof: Vec<Vec<u8>>,
        }

        let mut proof_key = b"commitments".to_vec();
        proof_key.extend(borsh::to_vec(commitment_key).unwrap());

        match self
            .rpc
            .call(methods::query::RpcQueryRequest {
                block_reference: BlockReference::BlockId(BlockId::Height(block_height)),
                request: QueryRequest::ViewState {
                    account_id: self.ibc_id.clone(),
                    prefix: proof_key.into(),
                    include_proof: true,
                },
            })
            .await
            .unwrap()
            .kind
        {
            QueryResponseKind::ViewState(res) => {
                let state_proof = res
                    .proof
                    .clone()
                    .into_iter()
                    .map(|item| item.to_vec())
                    .collect();
                serde_json::to_vec(&RawStateProof { state_proof }).unwrap()
            }
            _ => panic!("invalid response"),
        }
    }

    async fn connection_open_init(&self, client_id: &str, counterparty_client_id: &str) {
        let init = ConnectionOpenInit {
            client_id: client_id.to_string(),
            counterparty: connection_handshake::Counterparty {
                client_id: counterparty_client_id.to_string().validate().unwrap(),
                connection_id: "".into(),
                prefix: MerklePrefix {
                    key_prefix: b"ibc".into(),
                },
            },
            version: DEFAULT_IBC_VERSION[0].clone(),
            delay_period: 0,
        };

        self.send_tx("connection_open_init", init).await;
    }

    async fn next_light_header(&self, trusted_height: u64) -> near::header::Header {
        let block = self
            .rpc
            .call(methods::block::RpcBlockRequest {
                block_reference: BlockReference::Finality(Finality::Final),
            })
            .await
            .unwrap();

        sleep(Duration::from_secs(1)).await;

        let lc_block = self
            .rpc
            .call(
                methods::next_light_client_block::RpcLightClientNextBlockRequest {
                    last_block_hash: block.header.hash,
                },
            )
            .await
            .unwrap()
            .unwrap();

        let (prev_state_root, prev_state_root_proof) =
            self.chunk_proof(lc_block.inner_lite.height).await;

        near::header::Header {
            new_state: Self::convert_light_client_block_view(lc_block),
            trusted_height,
            prev_state_root_proof,
            prev_state_root,
        }
    }

    pub async fn chunk_proof(&self, block_height: u64) -> (CryptoHash, types::MerklePath) {
        let chunks = self
            .rpc
            .call(methods::block::RpcBlockRequest {
                block_reference: BlockReference::BlockId(BlockId::Height(block_height)),
            })
            .await
            .unwrap()
            .chunks;

        let prev_state_root = CryptoHash(chunks[0].prev_state_root.0);

        let (_, merkle_path) = near_primitives::merkle::merklize(
            &chunks
                .into_iter()
                .map(|chunk| CryptoHash(chunk.prev_state_root.0))
                .collect::<Vec<CryptoHash>>(),
        );

        let prev_state_root_proof = merkle_path[0]
            .clone()
            .into_iter()
            .map(|item| MerklePathItem {
                hash: near_primitives_core::hash::CryptoHash(item.hash.0),
                direction: match item.direction {
                    near_primitives::merkle::Direction::Left => types::Direction::Left,
                    near_primitives::merkle::Direction::Right => types::Direction::Right,
                },
            })
            .collect();

        (prev_state_root, prev_state_root_proof)
    }

    pub fn convert_block_header_inner(
        block_view: BlockHeaderInnerLiteView,
    ) -> near::block_header_inner::BlockHeaderInnerLiteView {
        near::block_header_inner::BlockHeaderInnerLiteView {
            height: block_view.height,
            epoch_id: CryptoHash(block_view.epoch_id.0),
            next_epoch_id: CryptoHash(block_view.next_epoch_id.0),
            prev_state_root: CryptoHash(block_view.prev_state_root.0),
            outcome_root: CryptoHash(block_view.outcome_root.0),
            timestamp: block_view.timestamp,
            timestamp_nanosec: block_view.timestamp_nanosec,
            next_bp_hash: CryptoHash(block_view.next_bp_hash.0),
            block_merkle_root: CryptoHash(block_view.block_merkle_root.0),
        }
    }

    pub fn convert_light_client_block_view(
        light_client_block: LightClientBlockView,
    ) -> near::light_client_block::LightClientBlockView {
        near::light_client_block::LightClientBlockView {
            inner_lite: Self::convert_block_header_inner(light_client_block.inner_lite),
            prev_block_hash: near_primitives_core::hash::CryptoHash(
                light_client_block.prev_block_hash.0,
            ),
            next_block_inner_hash: near_primitives_core::hash::CryptoHash(
                light_client_block.next_block_inner_hash.0,
            ),
            inner_rest_hash: near_primitives_core::hash::CryptoHash(
                light_client_block.inner_rest_hash.0,
            ),
            next_bps: light_client_block.next_bps.map(|bps| {
                bps.into_iter()
                    .map(|stake| {
                        let near_primitives::views::validator_stake_view::ValidatorStakeView::V1(
                            stake,
                        ) = stake;
                        near::validator_stake_view::ValidatorStakeView::V1(
                            near::validator_stake_view::ValidatorStakeViewV1 {
                                account_id: stake.account_id,
                                public_key: types::PublicKey::Ed25519(
                                    stake.public_key.key_data().try_into().unwrap(),
                                ),
                                stake: stake.stake,
                            },
                        )
                    })
                    .collect()
            }),
            approvals_after_next: light_client_block
                .approvals_after_next
                .into_iter()
                .map(|sig| {
                    sig.map(|s| match s.as_ref() {
                        near_crypto::Signature::ED25519(sig) => {
                            Box::new(types::Signature::Ed25519(sig.to_bytes().to_vec()))
                        }
                        near_crypto::Signature::SECP256K1(_) => {
                            Box::new(types::Signature::Secp256k1(Vec::new()))
                        }
                    })
                })
                .collect(),
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
            ibc_account_id: self.ibc_id.clone(),
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

    async fn create_client<A: Encode<Proto>, B: Encode<Proto>>(
        &self,
        client_state: A,
        consensus_state: B,
    ) {
        let block = self
            .rpc
            .call(methods::block::RpcBlockRequest {
                block_reference: BlockReference::Finality(near_primitives::types::Finality::Final),
            })
            .await
            .unwrap();

        let create_client = CreateClient {
            client_type: "cometbls".to_string(),
            client_state: client_state.encode(),
            consensus_state: consensus_state.encode(),
        };

        self.send_tx("create_client", create_client).await;

        println!("[ + ] Client created!");
    }
}

#[tokio::main]
async fn main() {
    let near_lc = "08-wasm-2";
    let cometbls_lc = "cometbls-3";
    let near_connection = "connection-2";
    let near = Near::new();
    let union = Union::new().await;

    near.register_client().await;

    let latest_height = union.union.query_latest_height().await.unwrap();
    let union_client_state = union.union.self_client_state(latest_height).await;
    near.create_client(
        union_client_state.clone(),
        union.union.self_consensus_state(latest_height).await,
    )
    .await;

    let cs = near.self_client_state().await;
    union
        .create_client(
            wasm::client_state::ClientState {
                data: cs.clone(),
                checksum: hex::decode(
                    "799fbe760eb51eef4645acc0aa4fc7546503f942c7f9dc414a8202c3b87abc29",
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

    near.connection_open_init(cometbls_lc, near_lc).await;
    let header = near.next_light_header(cs.latest_height).await;
    union.update_client(near_lc, header.clone()).await;

    sleep(Duration::from_secs(2)).await;

    let latest_height = union.union.query_latest_height().await.unwrap();
    let client_state =
        Any::<wasm::client_state::ClientState<near::client_state::ClientState>>::decode_as::<Proto>(
            &union
                .fetch_abci_query(
                    latest_height.revision_height,
                    AbciQueryType::State,
                    format!("clients/{near_lc}/clientState").as_str(),
                )
                .await,
        ).unwrap();

    let proof_init = near
        .state_proof(
            header.new_state.inner_lite.height - 1,
            &format!("connections/{}", near_connection),
        )
        .await;

    let proof_client = near
        .state_proof(
            header.new_state.inner_lite.height - 1,
            &format!("clients/{cometbls_lc}/clientState"),
        )
        .await;

    let proof_consensus = near
        .state_proof(
            header.new_state.inner_lite.height - 1,
            format!(
                "clients/{cometbls_lc}/consensusStates/1-{}",
                cs.latest_height
            )
            .as_str(),
        )
        .await;

    union
        .connection_open_try(
            near_lc,
            client_state,
            cometbls_lc,
            near_connection,
            header.new_state.inner_lite.height - 1,
            proof_init,
            proof_client,
            proof_consensus,
            union_client_state.latest_height,
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

fn key_from_path(path: &str) -> Vec<u8> {
    let mut commitments: Vec<u8> = Vec::new();
    commitments.extend(b"commitments");
    commitments.extend(borsh::to_vec(path).unwrap());
    commitments
}
