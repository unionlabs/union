use std::{
    collections::HashMap,
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
use num_bigint::BigUint;
use protos::union::galois::api::v3::union_prover_api_client;
use serde::{Deserialize, Serialize};
use tendermint_rpc::{Client as _, WebSocketClientUrl};
use tokio::time::sleep;
use unionlabs::{
    bounded::BoundedI64,
    cometbls::types::canonical_vote::CanonicalVote,
    encoding::{DecodeAs, Encode, EncodeAs, Proto},
    google::protobuf::any::{mk_any, Any, IntoAny},
    hash::H256,
    ibc::{
        core::{
            channel::{self, packet::Packet},
            client::height::{Height, IsHeight as _},
            commitment::{merkle_prefix::MerklePrefix, merkle_root::MerkleRoot},
            connection::version::Version,
        },
        lightclients::{
            cometbls::{self, client_state::ClientState, consensus_state::ConsensusState},
            near, wasm,
        },
    },
    ics24::ClientConsensusStatePath,
    id::{ChannelId, ConnectionId, PortId},
    near::{
        raw_state_proof::RawStateProof,
        types::{self, MerklePathItem},
    },
    tendermint::{
        crypto::public_key::PublicKey,
        types::{
            commit_sig::CommitSig, signed_header::SignedHeader, simple_validator::SimpleValidator,
            validator::Validator,
        },
    },
    traits::Chain,
    union::galois::{
        poll_request::PollRequest,
        poll_response::{PollResponse, ProveRequestDone, ProveRequestFailed},
        prove_request::ProveRequest,
        prove_response::ProveResponse,
        validator_set_commit::ValidatorSetCommit,
    },
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

#[derive(Clone, serde::Serialize)]
pub struct UpdateClient {
    pub client_id: String,
    pub client_msg: Vec<u8>,
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

    async fn fetch_validators(&self, height: Height) -> Vec<Validator> {
        self.union
            .tm_client()
            .validators(
                TryInto::<::tendermint::block::Height>::try_into(height.revision_height()).unwrap(),
                tendermint_rpc::Paging::All,
            )
            .await
            .unwrap()
            .validators
            .into_iter()
            .map(tendermint_helpers::tendermint_validator_info_to_validator)
            .collect()
    }

    async fn fetch_commit(&self, height: Height) -> SignedHeader {
        let commit = self
            .union
            .tm_client()
            .commit(
                TryInto::<::tendermint::block::Height>::try_into(height.revision_height()).unwrap(),
            )
            .await
            .unwrap();

        tendermint_helpers::tendermint_commit_to_signed_header(commit)
    }

    async fn fetch_prove_request(&self, request: ProveRequest) -> ProveResponse {
        let response = union_prover_api_client::UnionProverApiClient::connect(
            self.union.prover_endpoint.clone(),
        )
        .await
        .unwrap()
        .prove(protos::union::galois::api::v3::ProveRequest::from(request))
        .await
        .unwrap();
        // .poll(protos::union::galois::api::v3::PollRequest::from(
        //     PollRequest {
        //         request: request.clone(),
        //     },
        // ))
        // .await
        // .map(|x| x.into_inner().try_into().unwrap());

        // match response {
        //     Err(err) => panic!("prove request failed: {:?}", err),
        //     Ok(PollResponse::Failed(ProveRequestFailed { message })) => {
        //         panic!("panic with: {message}");
        //     }
        //     Ok(PollResponse::Done(ProveRequestDone { response })) => response,
        //     _ => panic!("other"),
        // }
        response.into_inner().try_into().unwrap()
    }

    async fn next_light_header(
        &self,
        trusted_height: Height,
        new_height: Height,
    ) -> cometbls::header::Header {
        let signed_header = self.fetch_commit(new_height).await;
        let untrusted_validators = self.fetch_validators(new_height).await;
        let trusted_validators = self.fetch_validators(trusted_height).await;

        let make_validators_commit =
            |mut validators: Vec<unionlabs::tendermint::types::validator::Validator>| {
                // Validators must be sorted to match the root, by token then address
                validators.sort_by(|a, b| {
                    // TODO: Double check how these comparisons are supposed to work
                    #[allow(clippy::collapsible_else_if)]
                    if a.voting_power == b.voting_power {
                        if a.address < b.address {
                            std::cmp::Ordering::Less
                        } else {
                            std::cmp::Ordering::Greater
                        }
                    } else {
                        if a.voting_power > b.voting_power {
                            std::cmp::Ordering::Less
                        } else {
                            std::cmp::Ordering::Greater
                        }
                    }
                });

                // The bitmap is a public input of the circuit, it must fit in Fr (scalar field) bn254
                let mut bitmap = BigUint::default();
                // REVIEW: This will over-allocate for the trusted validators; should be benchmarked
                let mut signatures = Vec::<Vec<u8>>::with_capacity(validators.len());

                let validators_map = validators
                    .iter()
                    .enumerate()
                    .map(|(i, v)| (v.address, i))
                    .collect::<HashMap<_, _>>();

                // For each validator signature, we search for the actual validator
                // in the set and set it's signed bit to 1. We then push the
                // signature only if the validator signed. It's possible that we
                // don't find a validator for a given signature as the validator set
                // may have drifted (trusted validator set).
                for sig in signed_header.commit.signatures.iter() {
                    match sig {
                        CommitSig::Absent => {}
                        CommitSig::Commit {
                            validator_address,
                            timestamp: _,
                            signature,
                        } => {
                            if let Some(validator_index) = validators_map.get(validator_address) {
                                bitmap.set_bit(*validator_index as u64, true);
                                signatures.push(signature.clone());
                            } else {
                            }
                        }
                        CommitSig::Nil { .. } => {}
                    }
                }

                let simple_validators = validators
                    .iter()
                    .map(|v| {
                        let PublicKey::Bn254(ref key) = v.pub_key else {
                            panic!("must be bn254")
                        };
                        SimpleValidator {
                            pub_key: PublicKey::Bn254(key.to_vec()),
                            voting_power: v.voting_power.into(),
                        }
                    })
                    .collect::<Vec<_>>();

                ValidatorSetCommit {
                    validators: simple_validators,
                    signatures,
                    bitmap: bitmap.to_bytes_be(),
                }
            };

        let trusted_validators_commit = make_validators_commit(trusted_validators);
        let untrusted_validators_commit = make_validators_commit(untrusted_validators);
        let proof = self.fetch_prove_request(ProveRequest {
            vote: CanonicalVote {
                // REVIEW: Should this be hardcoded to precommit?
                ty: unionlabs::tendermint::types::signed_msg_type::SignedMsgType::Precommit,
                height: signed_header.commit.height,
                round: BoundedI64::new(signed_header.commit.round.inner().into())
                    .expect("0..=i32::MAX can be converted to 0..=i64::MAX safely"),
                block_id: unionlabs::tendermint::types::canonical_block_id::CanonicalBlockId {
                    hash: signed_header.commit.block_id.hash.unwrap_or_default(),
                    part_set_header: unionlabs::tendermint::types::canonical_block_header::CanonicalPartSetHeader {
                        total: signed_header.commit.block_id.part_set_header.total,
                        hash: signed_header
                            .commit
                            .block_id
                            .part_set_header
                            .hash
                            .unwrap_or_default(),
                    },
                },
                chain_id: signed_header.header.chain_id.clone(),
            },
            untrusted_header: signed_header.header.clone(),
            trusted_commit: trusted_validators_commit,
            untrusted_commit: untrusted_validators_commit,
        })
        .await;

        cometbls::header::Header {
            signed_header: signed_header.into(),
            trusted_height,
            zero_knowledge_proof: proof.proof.evm_proof,
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

    async fn state_proof(&self, block_height: u64, commitment_key: &str) -> (Vec<u8>, Vec<u8>) {
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
                let value = res.values[0].value.to_vec();
                let state_proof = res
                    .proof
                    .clone()
                    .into_iter()
                    .map(|item| item.to_vec())
                    .collect();
                (
                    serde_json::to_vec(&RawStateProof { state_proof }).unwrap(),
                    borsh::from_slice::<Vec<u8>>(&value).unwrap(),
                )
            }
            _ => panic!("invalid response"),
        }
    }

    async fn update_client<T: Encode<Proto>>(&self, client_id: &str, header: T) {
        let update = UpdateClient {
            client_id: client_id.to_string(),
            client_msg: header.encode(),
        };

        self.send_tx("update_client", update).await;
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

        println!("response: {:?}", response);

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
    let near_lc = "08-wasm-1";
    let cometbls_lc = "cometbls-1";
    let near_connection = "connection-1";
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
    // union
    //     .create_client(
    //         wasm::client_state::ClientState {
    //             data: cs.clone(),
    //             checksum: hex::decode(
    //                 "88ec41b1142e2895fe8b1260897ed7734670412381322694e32b81348a441a94",
    //             )
    //             .unwrap()
    //             .try_into()
    //             .unwrap(),
    //             latest_height: Height {
    //                 revision_number: 0,
    //                 revision_height: cs.latest_height,
    //             },
    //         },
    //         wasm::consensus_state::ConsensusState {
    //             data: near.self_consensus_state(cs.latest_height + 1).await,
    //         },
    //     )
    //     .await;

    // near.connection_open_init(cometbls_lc, near_lc).await;
    let header = near.next_light_header(cs.latest_height).await;
    // union.update_client(near_lc, header.clone()).await;

    sleep(Duration::from_secs(3)).await;

    let latest_height = union.union.query_latest_height().await.unwrap();
    let light_header = union
        .next_light_header(union_client_state.latest_height, latest_height)
        .await;

    near.update_client(cometbls_lc, light_header.clone()).await;
    panic!();

    sleep(Duration::from_secs(2)).await;

    // let latest_height = union.union.query_latest_height().await.unwrap();
    // let client_state =
    //     Any::<wasm::client_state::ClientState<near::client_state::ClientState>>::decode_as::<Proto>(
    //         &union
    //             .fetch_abci_query(
    //                 latest_height.revision_height,
    //                 AbciQueryType::State,
    //                 format!("clients/{near_lc}/clientState").as_str(),
    //             )
    //             .await,
    //     ).unwrap();

    let (proof_init, _) = near
        .state_proof(
            header.new_state.inner_lite.height - 1,
            &format!("connections/{}", near_connection),
        )
        .await;

    let (proof_client, client_state) = near
        .state_proof(
            header.new_state.inner_lite.height - 1,
            &format!("clients/{cometbls_lc}/clientState"),
        )
        .await;

    let client_state = ClientState::decode_as::<Proto>(&client_state)
        .unwrap()
        .into_any();
    println!("uinon latest height: {}", union_client_state.latest_height);
    let (proof_consensus, _) = near
        .state_proof(
            header.new_state.inner_lite.height - 1,
            &ClientConsensusStatePath {
                client_id: cometbls_lc.to_string(),
                height: union_client_state.latest_height,
            }
            .to_string(),
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

    let latest_height = union.union.query_latest_height().await.unwrap();
    let light_header = union
        .next_light_header(union_client_state.latest_height, latest_height)
        .await;

    near.update_client(cometbls_lc, light_header.clone()).await;
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

pub mod tendermint_helpers {
    use unionlabs::{
        bounded::BoundedI64,
        google::protobuf::timestamp::Timestamp,
        hash::H256,
        tendermint::{
            crypto::public_key::PublicKey,
            types::{
                block_id::BlockId, commit::Commit, commit_sig::CommitSig,
                part_set_header::PartSetHeader, signed_header::SignedHeader, validator::Validator,
            },
        },
    };

    pub fn tendermint_commit_to_signed_header(
        commit: tendermint_rpc::endpoint::commit::Response,
    ) -> SignedHeader {
        let header_timestamp =
            tendermint_proto::google::protobuf::Timestamp::from(commit.signed_header.header.time);

        SignedHeader {
            header: unionlabs::tendermint::types::header::Header {
                version: unionlabs::tendermint::version::consensus::Consensus {
                    block: commit.signed_header.header.version.block,
                    app: commit.signed_header.header.version.app,
                },
                chain_id: commit.signed_header.header.chain_id.into(),
                height: tendermint_height_to_bounded_i64(commit.signed_header.header.height),
                time: Timestamp {
                    seconds: header_timestamp.seconds.try_into().unwrap(),
                    nanos: header_timestamp.nanos.try_into().unwrap(),
                },
                last_block_id: BlockId {
                    hash: Some(tendermint_hash_to_h256(
                        commit.signed_header.header.last_block_id.unwrap().hash,
                    )),
                    part_set_header: PartSetHeader {
                        total: commit
                            .signed_header
                            .header
                            .last_block_id
                            .unwrap()
                            .part_set_header
                            .total,
                        hash: Some(tendermint_hash_to_h256(
                            commit
                                .signed_header
                                .header
                                .last_block_id
                                .unwrap()
                                .part_set_header
                                .hash,
                        )),
                    },
                },
                last_commit_hash: tendermint_hash_to_h256(
                    commit.signed_header.header.last_commit_hash.unwrap(),
                ),
                data_hash: tendermint_hash_to_h256(commit.signed_header.header.data_hash.unwrap()),
                validators_hash: tendermint_hash_to_h256(
                    commit.signed_header.header.validators_hash,
                ),
                next_validators_hash: tendermint_hash_to_h256(
                    commit.signed_header.header.next_validators_hash,
                ),
                consensus_hash: tendermint_hash_to_h256(commit.signed_header.header.consensus_hash),
                app_hash: commit
                    .signed_header
                    .header
                    .app_hash
                    .as_bytes()
                    .try_into()
                    .unwrap(),
                last_results_hash: tendermint_hash_to_h256(
                    commit.signed_header.header.last_results_hash.unwrap(),
                ),
                evidence_hash: tendermint_hash_to_h256(
                    commit.signed_header.header.evidence_hash.unwrap(),
                ),
                proposer_address: commit
                    .signed_header
                    .header
                    .proposer_address
                    .as_bytes()
                    .try_into()
                    .expect("value is a [u8; 20] internally, this should not fail; qed;"),
            },
            commit: Commit {
                height: tendermint_height_to_bounded_i64(commit.signed_header.commit.height),
                round: i32::from(commit.signed_header.commit.round)
                    .try_into()
                    .unwrap(),
                block_id: BlockId {
                    hash: Some(tendermint_hash_to_h256(
                        commit.signed_header.commit.block_id.hash,
                    )),
                    part_set_header: PartSetHeader {
                        total: commit.signed_header.commit.block_id.part_set_header.total,
                        hash: Some(tendermint_hash_to_h256(
                            commit.signed_header.commit.block_id.part_set_header.hash,
                        )),
                    },
                },
                signatures: commit
                    .signed_header
                    .commit
                    .signatures
                    .into_iter()
                    .map(tendermint_commit_sig_to_commit_sig)
                    .collect(),
            },
        }
    }

    fn tendermint_commit_sig_to_commit_sig(sig: tendermint::block::CommitSig) -> CommitSig {
        match sig {
            ::tendermint::block::CommitSig::BlockIdFlagAbsent => CommitSig::Absent,
            ::tendermint::block::CommitSig::BlockIdFlagCommit {
                validator_address,
                timestamp,
                signature,
            } => CommitSig::Commit {
                validator_address: Vec::from(validator_address).try_into().unwrap(),
                timestamp: {
                    let ts = tendermint_proto::google::protobuf::Timestamp::from(timestamp);

                    Timestamp {
                        seconds: ts.seconds.try_into().unwrap(),
                        nanos: ts.nanos.try_into().unwrap(),
                    }
                },
                signature: signature.unwrap().into_bytes(),
            },
            ::tendermint::block::CommitSig::BlockIdFlagNil {
                validator_address,
                timestamp,
                signature,
            } => CommitSig::Nil {
                validator_address: Vec::from(validator_address).try_into().unwrap(),
                timestamp: {
                    let ts = tendermint_proto::google::protobuf::Timestamp::from(timestamp);

                    Timestamp {
                        seconds: ts.seconds.try_into().unwrap(),
                        nanos: ts.nanos.try_into().unwrap(),
                    }
                },
                signature: signature.unwrap().into_bytes(),
            },
        }
    }

    pub fn tendermint_validator_info_to_validator(val: ::tendermint::validator::Info) -> Validator {
        Validator {
            address: val
                .address
                .as_bytes()
                .try_into()
                .expect("value is 20 bytes internally; should not fail; qed"),
            pub_key: match val.pub_key {
                ::tendermint::PublicKey::Ed25519(key) => PublicKey::Ed25519(key.as_bytes().into()),
                ::tendermint::PublicKey::Bn254(key) => PublicKey::Bn254(key.to_vec()),
                _ => todo!(),
            },
            voting_power: BoundedI64::new(val.power.value().try_into().unwrap()).unwrap(),
            proposer_priority: val.proposer_priority.value(),
        }
    }

    fn tendermint_hash_to_h256(hash: tendermint::Hash) -> H256 {
        match hash {
            tendermint::Hash::Sha256(hash) => hash.into(),
            tendermint::Hash::None => panic!("empty hash???"),
        }
    }

    pub fn tendermint_height_to_bounded_i64(
        height: ::tendermint::block::Height,
    ) -> BoundedI64<0, { i64::MAX }> {
        i64::from(height).try_into().unwrap()
    }
}
