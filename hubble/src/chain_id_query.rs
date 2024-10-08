use alloy::sol;
use prost::Message;
use protos::ibc::{
    core::client::v1::QueryClientStateRequest, lightclients::wasm::v1::QueryCodeRequest,
};
use sqlx::PgPool;
use tendermint_rpc::{Client, HttpClient};
use tracing::warn;
use unionlabs::{
    encoding::{DecodeAs, Proto},
    parse_wasm_client_type, WasmClientType,
};
sol! {
    contract IbcHandler {
        function CreateClient(MsgCreateClient calldata) returns (string memory);
    }

    struct MsgCreateClient {
        string client_type;
        bytes client_state_bytes;
        bytes consensus_state_bytes;
        address relayer;
    }
}
use crate::cli::{IndexerConfig, Indexers};

#[derive(Debug)]
struct Data {
    chain_id: i32,
    client_id: String,
    counterparty_chain_id: String,
}

pub async fn tx(db: PgPool, indexers: Indexers) {
    let mut datas = vec![];

    for indexer in indexers {
        match indexer {
            IndexerConfig::DummyFetcher(_) => {}
            IndexerConfig::EthFetcher(_) => {}
            IndexerConfig::TmFetcher(_) => {}
            IndexerConfig::AptosFetcher(_) => {}
            IndexerConfig::Scroll(_) => {}
            IndexerConfig::Arb(_) => {}
            IndexerConfig::Beacon(_) => {}
            IndexerConfig::Bera(_) => {}
            IndexerConfig::Tm(tm_config) => {
                let client = HttpClient::new(tm_config.urls[0].as_str()).unwrap();

                let grpc_url = tm_config.grpc_url.clone().unwrap();

                let mut grpc_client =
                    protos::ibc::core::client::v1::query_client::QueryClient::connect(
                        grpc_url.clone(),
                    )
                    .await
                    .unwrap();

                let chain_id = client.status().await.unwrap().node_info.network.to_string();

                let tm_clients = sqlx::query!(
                    r#"
                    select cc.client_id, ch.id
                    from v0_cosmos.create_client cc
                    join v0.chains ch on cc.chain_id = ch.id
                    left join v0.clients cl on 
                        cl.chain_id = ch.id and 
                        cl.client_id = cc.client_id
                    where
                        ch.chain_id = $1 and 
                        cc.client_id is not null 
                        and cl.chain_id is null
                    "#,
                    chain_id
                )
                .fetch_all(&db)
                .await
                .unwrap();

                for record in tm_clients {
                    let client_id = record.client_id.unwrap();
                    let client_state = grpc_client
                        .client_state(QueryClientStateRequest {
                            client_id: client_id.clone(),
                        })
                        .await
                        .unwrap()
                        .into_inner()
                        .client_state
                        .unwrap();

                    match &*client_state.type_url {
                        "/ibc.lightclients.wasm.v1.ClientState" => {
                            let cs = protos::ibc::lightclients::wasm::v1::ClientState::decode(
                                &*client_state.value,
                            )
                            .unwrap();

                            let mut client =
                                protos::ibc::lightclients::wasm::v1::query_client::QueryClient::connect(
                                    grpc_url.clone(),
                                )
                                .await
                                .unwrap();

                            let wasm_blob = client
                                .code(QueryCodeRequest {
                                    checksum: hex::encode(&*cs.checksum),
                                })
                                .await
                                .unwrap()
                                .into_inner()
                                .data;

                            let client_type = parse_wasm_client_type(wasm_blob).unwrap();

                            let counterparty_chain_id = match client_type.unwrap() {
                                WasmClientType::EthereumMinimal
                                | WasmClientType::EthereumMainnet => {
                                    let cs = match unionlabs::ibc::lightclients::ethereum::client_state::ClientState::decode_as::<Proto>(&cs.data) {
                                        Ok(cs) => cs,
                                        // We changed the format of berachain client states, but union-testnet-8 still contains an old configuration which we need to ignore.
                                        Err(err) => {
                                            warn!("error while decoding client {client_id}: {:?}. Most likely due to a client state upgrade. This can then be safely ignored", err);
                                            continue;
                                        }
                                    };
                                    cs.chain_id.to_string()
                                }
                                WasmClientType::Cometbls => {
                                    let cs = match unionlabs::ibc::lightclients::cometbls::client_state::ClientState::decode_as::<Proto>(&cs.data) {
                                        Ok(cs) => cs,
                                        Err(err) => {
                                            warn!("error while decoding client {client_id}: {:?}. Most likely due to a client state upgrade. This can then be safely ignored", err);
                                            continue
                                        }
                                    };

                                    cs.chain_id
                                }
                                WasmClientType::Tendermint => {
                                    let cs = match unionlabs::ibc::lightclients::tendermint::client_state::ClientState::decode_as::<Proto>(&cs.data) {
                                        Ok(cs) => cs,
                                        Err(err) => {
                                            warn!("error while decoding client {client_id}: {:?}. Most likely due to a client state upgrade. This can then be safely ignored", err);
                                            continue
                                        }
                                    };

                                    cs.chain_id
                                }
                                WasmClientType::Scroll => {
                                    let cs = match unionlabs::ibc::lightclients::scroll::client_state::ClientState::decode_as::<Proto>(&cs.data) {
                                        Ok(cs) => cs,
                                        Err(err) => {
                                            warn!("error while decoding client {client_id}: {:?}. Most likely due to a client state upgrade. This can then be safely ignored", err);
                                            continue
                                        }
                                    };

                                    cs.chain_id.to_string()
                                }
                                WasmClientType::Arbitrum => {
                                    let cs = match unionlabs::ibc::lightclients::arbitrum::client_state::ClientState::decode_as::<Proto>(&cs.data) {
                                        Ok(cs) => cs,
                                        Err(err) => {
                                            warn!("error while decoding client {client_id}: {:?}. Most likely due to a client state upgrade. This can then be safely ignored", err);
                                            continue
                                        }
                                    };

                                    cs.chain_id.to_string()
                                }
                                WasmClientType::Linea => todo!("We still need to add linea"),
                                WasmClientType::Berachain => {
                                    let cs = match unionlabs::ibc::lightclients::berachain::client_state::ClientState::decode_as::<Proto>(&cs.data) {
                                        Ok(cs) => cs,
                                        // We changed the format of berachain client states, but union-testnet-8 still contains an old configuration which we need to ignore.
                                        Err(err) => {
                                            warn!("error while decoding client {client_id}: {:?}. Most likely due to a client state upgrade. This can then be safely ignored", err);
                                            continue;
                                        }
                                    };

                                    cs.execution_chain_id.to_string()
                                }
                                WasmClientType::EvmInCosmos => {
                                    todo!("We still need to add evm-in-cosmos")
                                }
                                WasmClientType::Movement => {
                                    let cs = match unionlabs::ibc::lightclients::movement::client_state::ClientState::decode_as::<Proto>(&cs.data) {
                                        Ok(cs) => cs,
                                        Err(err) => {
                                            warn!("error while decoding client {client_id}: {:?}. Most likely due to a client state upgrade. This can then be safely ignored", err);
                                            continue;
                                        }
                                    };

                                    cs.chain_id.to_string()
                                }
                            };

                            datas.push(Data {
                                chain_id: record.id,
                                client_id: client_id.clone(),
                                counterparty_chain_id,
                            })
                        }
                        "/ibc.lightclients.tendermint.v1.ClientState" => {
                            let cs =
                                protos::ibc::lightclients::tendermint::v1::ClientState::decode(
                                    &*client_state.value,
                                )
                                .unwrap();

                            datas.push(Data {
                                chain_id: record.id,
                                client_id: client_id.clone(),
                                counterparty_chain_id: cs.chain_id,
                            })
                        }
                        _ => {
                            panic!("unknown client state type {}", client_state.type_url)
                        }
                    };
                }
            }
        }
    }

    sqlx::query!(
        r#"
        INSERT INTO
            v0.clients (chain_id, client_id, counterparty_chain_id)
        SELECT
            *
        FROM
            UNNEST($1::integer[], $2::text[], $3::text[])
        ON CONFLICT DO NOTHING
        "#,
        &datas.iter().map(|x| x.chain_id).collect::<Vec<_>>()[..],
        &datas
            .iter()
            .map(|x| x.client_id.clone())
            .collect::<Vec<_>>()[..],
        &datas
            .iter()
            .map(|x| x.counterparty_chain_id.clone())
            .collect::<Vec<_>>()[..],
    )
    .execute(&db)
    .await
    .unwrap();
}
