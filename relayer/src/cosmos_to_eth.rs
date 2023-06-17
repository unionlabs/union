use std::{str::FromStr, sync::Arc, time::Duration};

use contracts::{glue::*, ibc_handler::*, shared_types::IbcCoreClientV1HeightData};
use ethers::{
    abi::AbiEncode,
    prelude::{decode_logs, k256::ecdsa, SignerMiddleware},
    providers::{Http, Middleware, Provider, StreamExt},
    signers::{LocalWallet, Signer, Wallet},
    types::{Address, U256},
};
use num_bigint::BigUint;
use prost::Message;
use protos::{
    cosmos::staking::{
        self,
        v1beta1::{QueryValidatorRequest, QueryValidatorsRequest},
    },
    tendermint::{
        crypto::{public_key, PublicKey},
        types::{
            CanonicalBlockId, CanonicalPartSetHeader, CanonicalVote, SignedMsgType, SimpleValidator,
        },
    },
    union::prover::api::v1::{union_prover_api_client, ProveRequest, ValidatorSetCommit},
};
use sha2::{Digest, Sha256};
use tendermint_rpc::{
    endpoint::commit, query::EventType, Client, HttpClient, SubscriptionClient, WebSocketClient,
    WebSocketClientUrl,
};

use crate::{Args, ETH_RPC_API};

pub const CHAIN_ID: &str = "union-devnet-1";
pub const COMETBLS_CLIENT_TYPE: &str = "cometbls";
const COSMOS_CLIENT_ID: &str = "evm";
const COSMOS_CONNECTION_ID: &str = "union";
const COSMOS_CHANNEL_ID: &str = "default";
const MERKLE_PREFIX: &str = "ibc";
pub const PORT_ID: &str = "transfer";

type ConnectionId = String;
type ChannelId = String;
type ClientId = String;
type PortId = String;

struct Connections {
    pub evm: ConnectionId,
    pub cosmos: ConnectionId,
}

struct Channels {
    pub evm: ChannelId,
    pub cosmos: ChannelId,
}

// The story behind this is too dark to be explained, you must personally ask hussein.aitlahcen@gmail.com
fn encode_dynamic_singleton_tuple(bytes: Vec<u8>) -> Vec<u8> {
    U256::from(32)
        .encode()
        .clone()
        .into_iter()
        .chain(bytes.into_iter())
        .collect::<Vec<_>>()
}

pub async fn create_client<M>(
    handler: &IBCHandler<M>,
    commit: &commit::Response,
    params: &staking::v1beta1::Params,
) -> ClientId
where
    M: Middleware,
{
    let height = commit.signed_header.header.height;

    let unbonding_period = std::time::Duration::new(
        params
            .unbonding_time
            .clone()
            .unwrap()
            .seconds
            .try_into()
            .unwrap(),
        params
            .unbonding_time
            .clone()
            .unwrap()
            .nanos
            .try_into()
            .unwrap(),
    );

    let client_state = UnionIbcLightclientsCometblsV1ClientStateData {
        chain_id: CHAIN_ID.into(),
        // https://github.com/cometbft/cometbft/blob/da0e55604b075bac9e1d5866cb2e62eaae386dd9/light/verifier.go#L16
        trust_level: UnionIbcLightclientsCometblsV1FractionData {
            numerator: 1,
            denominator: 3,
        },
        // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
        trusting_period: GoogleProtobufDurationData {
            seconds: (unbonding_period * 85 / 100).as_secs().try_into().unwrap(),
            nanos: (unbonding_period * 85 / 100)
                .subsec_nanos()
                .try_into()
                .unwrap(),
        },
        unbonding_period: GoogleProtobufDurationData {
            seconds: unbonding_period.as_secs().try_into().unwrap(),
            nanos: unbonding_period.subsec_nanos().try_into().unwrap(),
        },
        // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
        max_clock_drift: GoogleProtobufDurationData {
            seconds: 60 * 10,
            nanos: 0,
        },
        frozen_height: IbcCoreClientV1HeightData {
            revision_number: 0,
            revision_height: 0,
        },
    };

    println!("{:?}", client_state);

    let consensus_state = UnionIbcLightclientsCometblsV1ConsensusStateData {
        root: IbcCoreCommitmentV1MerkleRootData {
            hash: commit
                .signed_header
                .header
                .app_hash
                .as_bytes()
                .to_vec()
                .into(),
        },
        next_validators_hash: commit
            .signed_header
            .header
            .next_validators_hash
            .as_bytes()
            .to_vec()
            .into(),
    };

    println!("{:?}", consensus_state);

    let normalized_client_state_bytes = encode_dynamic_singleton_tuple(client_state.encode());
    let normalized_consensus_state_bytes = encode_dynamic_singleton_tuple(consensus_state.encode());

    let msg_create_client = MsgCreateClient {
        client_type: COMETBLS_CLIENT_TYPE.to_string(),
        client_state_bytes: normalized_client_state_bytes.into(),
        consensus_state_bytes: normalized_consensus_state_bytes.into(),
    };

    let rcp = handler
        .create_client(msg_create_client)
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();

    // TODO(benluelo): get logs this way
    let client_id = decode_logs::<IBCHandlerEvents>(
        rcp.logs
            .into_iter()
            .map(|l| l.into())
            .collect::<Vec<_>>()
            .as_ref(),
    )
    .unwrap()
    .into_iter()
    .find_map(|l| match l {
        IBCHandlerEvents::GeneratedClientIdentifierFilter(client_id) => Some(client_id.0),
        _ => None,
    })
    .unwrap();

    client_id
}

async fn connection_handshake<M>(handler: &IBCHandler<M>, client_id: ClientId) -> Connections
where
    M: Middleware,
{
    let rcp = handler
        .connection_open_init(MsgConnectionOpenInit {
            client_id,
            counterparty: IbcCoreConnectionV1CounterpartyData {
                client_id: COSMOS_CLIENT_ID.into(),
                connection_id: COSMOS_CONNECTION_ID.into(),
                prefix: IbcCoreCommitmentV1MerklePrefixData {
                    key_prefix: MERKLE_PREFIX.as_bytes().to_vec().into(),
                },
            },
            // TODO
            delay_period: 6,
        })
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();

    let evm_connection_id = decode_logs::<IBCHandlerEvents>(
        rcp.logs
            .into_iter()
            .map(|l| l.into())
            .collect::<Vec<_>>()
            .as_ref(),
    )
    .unwrap()
    .into_iter()
    .find_map(|l| match l {
        IBCHandlerEvents::GeneratedConnectionIdentifierFilter(connection_id) => {
            Some(connection_id.0)
        }
        _ => None,
    })
    .unwrap();

    // TODO: query geth `eth_getProof` to generate evm membership proofs
    // TODO: cosmos connection_open_try

    // TODO: query cosmos `abci_query` to generate membership proof
    handler
        .connection_open_ack(MsgConnectionOpenAck {
            connection_id: evm_connection_id.clone(),
            // TODO
            client_state_bytes: Default::default(),
            version: IbcCoreConnectionV1VersionData {
                identifier: "1".into(),
                features: vec!["ORDER_UNORDERED".into(), "ORDER_ORDERED".into()],
            },
            counterparty_connection_id: COSMOS_CONNECTION_ID.into(),
            // TODO
            proof_try: Default::default(),
            // TODO
            proof_client: Default::default(),
            // TODO
            proof_consensus: Default::default(),
            // TODO
            proof_height: IbcCoreClientV1HeightData {
                revision_number: 0,
                revision_height: 0,
            },
            // TODO
            consensus_height: IbcCoreClientV1HeightData {
                revision_number: 0,
                revision_height: 0,
            },
        })
        .send()
        .await
        .unwrap()
        .await
        .unwrap();

    // TODO: query geth `eth_getProof` to generate evm membership proofs
    // TODO: cosmos open_confirm

    return Connections {
        evm: evm_connection_id,
        cosmos: COSMOS_CONNECTION_ID.into(),
    };
}

async fn channel_handshake<M>(
    handler: &IBCHandler<M>,
    connections: &Connections,
    port_id: PortId,
) -> Channels
where
    M: Middleware,
{
    // port/channel and connection ids will highly likely be different for the counterparty
    let rcp = handler
        .channel_open_init(MsgChannelOpenInit {
            port_id: port_id.clone(),
            channel: IbcCoreChannelV1ChannelData {
                state: 1,
                ordering: 1,
                counterparty: IbcCoreChannelV1CounterpartyData {
                    port_id: port_id.clone(),
                    channel_id: COSMOS_CHANNEL_ID.into(),
                },
                connection_hops: vec![connections.evm.clone()],
                version: "1".into(),
            },
        })
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();

    let evm_channel_id = decode_logs::<IBCHandlerEvents>(
        rcp.logs
            .into_iter()
            .map(|l| l.into())
            .collect::<Vec<_>>()
            .as_ref(),
    )
    .unwrap()
    .into_iter()
    .find_map(|l| match l {
        IBCHandlerEvents::GeneratedChannelIdentifierFilter(channel_id) => Some(channel_id.0),
        _ => None,
    })
    .unwrap();

    // TODO: query geth `eth_getProof` to generate evm membership proofs
    // TODO: cosmos open_try

    // TODO: query cosmos `abci_query` to generate membership proof
    handler
        .channel_open_ack(MsgChannelOpenAck {
            port_id,
            channel_id: evm_channel_id.clone(),
            counterparty_version: "1".into(),
            counterparty_channel_id: COSMOS_CHANNEL_ID.into(),
            // TODO
            proof_try: Default::default(),
            // TODO
            proof_height: IbcCoreClientV1HeightData {
                revision_number: 0,
                revision_height: 0,
            },
        })
        .send()
        .await
        .unwrap()
        .await
        .unwrap();

    // TODO: query geth `eth_getProof` to generate evm membership proofs
    // TODO: cosmos open_confirm

    Channels {
        evm: evm_channel_id,
        cosmos: COSMOS_CHANNEL_ID.into(),
    }
}

/*

1. Register the client
2. Create an instance of the client
3. Connection handshake
4. Channel handshake

5. UpdateClient
6. 5.

NOTE: 1/2/3/4 must be done only once
 */
// pub async fn update_contract(args: &Args) {
//     let mut staking_client =
//         staking::v1beta1::query_client::QueryClient::connect("http://0.0.0.0:9090")
//             .await
//             .unwrap();
//     let staking_params = staking_client
//         .params(staking::v1beta1::QueryParamsRequest {})
//         .await
//         .unwrap()
//         .into_inner()
//         .params
//         .unwrap();

//     let (tm_client, tm_driver) = WebSocketClient::builder(
//         WebSocketClientUrl::from_str("ws://0.0.0.0:26657/websocket").unwrap(),
//     )
//     .compat_mode(tendermint_rpc::client::CompatMode::V0_37)
//     .build()
//     .await
//     .unwrap();
//     let _ = tokio::spawn(async move { tm_driver.run().await });

//     let commit: commit::Response = tm_client.latest_commit().await.unwrap();

//     let handler = create_ibc_handler_client(args).await;

//     handler
//         .register_client(COMETBLS_CLIENT_TYPE.into(), args.cometbls_client_address)
//         .send()
//         .await
//         .unwrap()
//         .await
//         .unwrap();

//     println!("Creating client...");

//     let client_id = create_client(&handler, &commit, &staking_params).await;

//     println!("Created client - {}", client_id);

//     println!("Binding ICS20 bank...");
//     handler
//         .bind_port(PORT_ID.into(), args.ics20_module_address)
//         .send()
//         .await
//         .unwrap()
//         .await
//         .unwrap();

//     // NOTE: the following is possible if we register a module to a port, i.e. handler.bind_port with a contract that act as an IBC module
//     println!("Creating connections...");
//     let connections = connection_handshake(&handler, client_id.clone()).await;
//     println!("Creating channels...");
//     let _ = channel_handshake(&handler, &connections, PORT_ID.into()).await;

//     let mut previous_height = commit.signed_header.header.height;
//     loop {
//         let commit = loop {
//             let r = tm_client.commit(previous_height.increment()).await;
//             match r {
//                 Ok(commit) => break commit,
//                 Err(_) => std::thread::sleep(Duration::from_millis(500)),
//             }
//         };

//         println!("New block {:?}", commit.signed_header.header.height);

//         println!("Query validators...");
//         let mut validators = staking_client
//             .validators(QueryValidatorsRequest {
//                 // How to use BondStatus???
//                 status: "BOND_STATUS_BONDED".into(),
//                 pagination: None,
//             })
//             .await
//             .unwrap()
//             .into_inner()
//             .validators;

//         // Validators must be sorted to match the root, by token then address
//         validators.sort_by(|a, b| {
//             let a_tokens = str::parse::<u128>(&a.tokens).unwrap();
//             let b_tokens = str::parse::<u128>(&b.tokens).unwrap();
//             if a_tokens == b_tokens {
//                 let a_key = protos::cosmos::crypto::bn254::PubKey::decode::<&[u8]>(
//                     &a.consensus_pubkey.clone().unwrap().value,
//                 )
//                 .unwrap()
//                 .key;
//                 let b_key = protos::cosmos::crypto::bn254::PubKey::decode::<&[u8]>(
//                     &b.consensus_pubkey.clone().unwrap().value,
//                 )
//                 .unwrap()
//                 .key;
//                 // Tendermint address are sha256(pubkey)[0:20]
//                 let a_address = Sha256::new()
//                     .chain_update(a_key)
//                     .finalize()
//                     .into_iter()
//                     .take(20)
//                     .collect::<Vec<_>>();
//                 let b_address = Sha256::new()
//                     .chain_update(b_key)
//                     .finalize()
//                     .into_iter()
//                     .take(20)
//                     .collect::<Vec<_>>();
//                 a_address.cmp(&b_address)
//             } else {
//                 a_tokens.cmp(&b_tokens)
//             }
//         });

//         let simple_validators = validators
//             .iter()
//             .map(|v| {
//                 SimpleValidator {
//                     // Couldn't find a less ugly way
//                     pub_key: v.consensus_pubkey.as_ref().map(|pk| PublicKey {
//                         sum: Some(public_key::Sum::Bn254(
//                             protos::cosmos::crypto::bn254::PubKey::decode::<&[u8]>(
//                                 &pk.value.clone(),
//                             )
//                             .unwrap()
//                             .key,
//                         )),
//                     }),
//                     // Equivalent of sdk.TokensToConsensusPower(sdk.NewIntFromBigInt(tokens), sdk.DefaultPowerReduction)
//                     voting_power: (str::parse::<u128>(&v.tokens).unwrap() / 1000000u128) as _,
//                 }
//             })
//             .collect::<Vec<_>>();

//         let mut bitmap = BigUint::default();
//         let mut signatures =
//             Vec::<Vec<u8>>::with_capacity(commit.signed_header.commit.signatures.len());
//         // NOTE: we assume that the signatures are correctly ordered. i.e. they follow the validator set order as the index is used to aggregate validator pks.
//         for (i, sig) in commit.signed_header.commit.signatures.iter().enumerate() {
//             match sig {
//                 tendermint::block::CommitSig::BlockIdFlagAbsent => {}
//                 tendermint::block::CommitSig::BlockIdFlagCommit {
//                     signature,
//                     validator_address,
//                     ..
//                 } => {
//                     bitmap.set_bit(i as _, true);
//                     signatures.push(signature.clone().unwrap().into_bytes());
//                     println!("Validator {:?} signed", validator_address);
//                 }
//                 // TODO: not sure about this case
//                 tendermint::block::CommitSig::BlockIdFlagNil { .. } => {
//                     println!("Nul flag???");
//                 }
//             }
//         }

//         let trusted_commit = Some(ValidatorSetCommit {
//             validators: simple_validators,
//             signatures,
//             bitmap: bitmap.to_bytes_be(),
//         });

//         // The untrusted commit is the same as we only deal with adjacent verification for now.
//         let untrusted_commit = trusted_commit.clone();

//         println!("Generate ZKP...");
//         let mut prover_client = union_prover_api_client::UnionProverApiClient::connect(
//             "https://prover.cryptware.io:443",
//         )
//         .await
//         .unwrap();
//         let prove_res = prover_client
//             .prove(ProveRequest {
//                 vote: Some(CanonicalVote {
//                     r#type: SignedMsgType::Precommit.into(),
//                     height: commit.signed_header.commit.height.into(),
//                     round: u32::from(commit.signed_header.commit.round) as _,
//                     block_id: Some(CanonicalBlockId {
//                         hash: commit
//                             .signed_header
//                             .commit
//                             .block_id
//                             .hash
//                             .as_bytes()
//                             .to_vec(),
//                         part_set_header: Some(CanonicalPartSetHeader {
//                             total: commit.signed_header.commit.block_id.part_set_header.total,
//                             hash: commit
//                                 .signed_header
//                                 .commit
//                                 .block_id
//                                 .part_set_header
//                                 .hash
//                                 .as_bytes()
//                                 .to_vec(),
//                         }),
//                     }),
//                     chain_id: commit.signed_header.header.chain_id.clone().into(),
//                 }),
//                 trusted_commit,
//                 untrusted_commit,
//             })
//             .await
//             .unwrap()
//             .into_inner();

//         let header_timestamp =
//             tendermint_proto::google::protobuf::Timestamp::from(commit.signed_header.header.time);
//         let client_message = UnionIbcLightclientsCometblsV1HeaderData {
//             signed_header: TendermintTypesSignedHeaderData {
//                 header: TendermintTypesHeaderData {
//                     version: TendermintVersionConsensusData {
//                         block: commit.signed_header.header.version.block,
//                         app: commit.signed_header.header.version.app,
//                     },
//                     chain_id: commit.signed_header.header.chain_id.into(),
//                     height: commit.signed_header.header.height.into(),
//                     time: GoogleProtobufTimestampData {
//                         secs: header_timestamp.seconds,
//                         nanos: header_timestamp.nanos.into(),
//                     },
//                     last_block_id: TendermintTypesBlockIDData {
//                         hash: commit
//                             .signed_header
//                             .header
//                             .last_block_id
//                             .unwrap()
//                             .hash
//                             .as_bytes()
//                             .to_vec()
//                             .into(),
//                         part_set_header: TendermintTypesPartSetHeaderData {
//                             total: commit
//                                 .signed_header
//                                 .header
//                                 .last_block_id
//                                 .unwrap()
//                                 .part_set_header
//                                 .total,
//                             hash: commit
//                                 .signed_header
//                                 .header
//                                 .last_block_id
//                                 .unwrap()
//                                 .part_set_header
//                                 .hash
//                                 .as_bytes()
//                                 .to_vec()
//                                 .into(),
//                         },
//                     },
//                     last_commit_hash: commit
//                         .signed_header
//                         .header
//                         .last_commit_hash
//                         .unwrap()
//                         .as_bytes()
//                         .to_vec()
//                         .into(),
//                     data_hash: commit
//                         .signed_header
//                         .header
//                         .data_hash
//                         .unwrap()
//                         .as_bytes()
//                         .to_vec()
//                         .into(),
//                     validators_hash: commit
//                         .signed_header
//                         .header
//                         .validators_hash
//                         .as_bytes()
//                         .to_vec()
//                         .into(),
//                     next_validators_hash: commit
//                         .signed_header
//                         .header
//                         .next_validators_hash
//                         .as_bytes()
//                         .to_vec()
//                         .into(),
//                     consensus_hash: commit
//                         .signed_header
//                         .header
//                         .consensus_hash
//                         .as_bytes()
//                         .to_vec()
//                         .into(),
//                     app_hash: commit
//                         .signed_header
//                         .header
//                         .app_hash
//                         .as_bytes()
//                         .to_vec()
//                         .into(),
//                     last_results_hash: commit
//                         .signed_header
//                         .header
//                         .last_results_hash
//                         .unwrap()
//                         .as_bytes()
//                         .to_vec()
//                         .into(),
//                     evidence_hash: commit
//                         .signed_header
//                         .header
//                         .evidence_hash
//                         .unwrap()
//                         .as_bytes()
//                         .to_vec()
//                         .into(),
//                     proposer_address: commit
//                         .signed_header
//                         .header
//                         .proposer_address
//                         .as_bytes()
//                         .to_vec()
//                         .into(),
//                 },
//                 commit: TendermintTypesCommitData {
//                     height: commit.signed_header.commit.height.into(),
//                     round: commit.signed_header.commit.round.into(),
//                     block_id: TendermintTypesBlockIDData {
//                         hash: commit
//                             .signed_header
//                             .commit
//                             .block_id
//                             .hash
//                             .as_bytes()
//                             .to_vec()
//                             .into(),
//                         part_set_header: TendermintTypesPartSetHeaderData {
//                             total: commit.signed_header.commit.block_id.part_set_header.total,
//                             hash: commit
//                                 .signed_header
//                                 .commit
//                                 .block_id
//                                 .part_set_header
//                                 .hash
//                                 .as_bytes()
//                                 .to_vec()
//                                 .into(),
//                         },
//                     },
//                     // NOTE: We don't need the signatures are they are part of the ZKP
//                     signatures: vec![],
//                 },
//             },
//             untrusted_validator_set_root: prove_res.untrusted_validator_set_root.into(),
//             trusted_height: IbcCoreClientV1HeightData {
//                 revision_number: 0,
//                 revision_height: previous_height.into(),
//             },
//             zero_knowledge_proof: prove_res.proof.unwrap().evm_proof.into(),
//         };

//         println!("Client message {:?}", client_message);

//         let normalized_client_message_bytes =
//             encode_dynamic_singleton_tuple(client_message.encode());

//         println!("Updating client...");
//         handler
//             .update_client(MsgUpdateClient {
//                 client_id: client_id.clone(),
//                 client_message: normalized_client_message_bytes.into(),
//             })
//             .send()
//             .await
//             .unwrap()
//             .await
//             .unwrap();

//         // Next
//         previous_height = previous_height.increment();
//     }
// }

// pub async fn create_ibc_handler_client(
//     args: &Args,
// ) -> IBCHandler<SignerMiddleware<Provider<Http>, Wallet<ecdsa::SigningKey>>> {
//     let provider = Arc::new({
//         let provider = Provider::<Http>::try_from(ETH_RPC_API).unwrap();
//         let chain_id = provider.get_chainid().await.unwrap();
//         let wallet = "4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77"
//             .parse::<LocalWallet>()
//             .unwrap()
//             .with_chain_id(chain_id.as_u64());
//         SignerMiddleware::new(provider, wallet)
//     });

//     contracts::ibc_handler::IBCHandler::new(args.ibc_handler_address, provider)
// }

// async fn event_listener() -> StreamExt {
//     //
// }
