use std::str::FromStr;

use ethers::providers::{Http, Middleware, Provider};
use prost::Message;
use protos::ibc::{
    core::client::v1::QueryClientStateRequest, lightclients::wasm::v1::QueryCodeRequest,
};
use sqlx::PgPool;
use tendermint_rpc::{Client, HttpClient};
use tracing::warn;
use unionlabs::{
    encoding::{DecodeAs, EthAbi, Proto},
    parse_wasm_client_type,
    traits::ClientState,
    WasmClientType,
};

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
            IndexerConfig::Arb(_) => {}
            IndexerConfig::Beacon(_) => {}
            IndexerConfig::Bera(_) => {}
            IndexerConfig::EthFork(_) => {}
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
                                    cs.chain_id().to_string()
                                }
                                WasmClientType::Cometbls => {
                                    let cs = match unionlabs::ibc::lightclients::cometbls::client_state::ClientState::decode_as::<Proto>(&cs.data) {
                                        Ok(cs) => cs,
                                        Err(err) => {
                                            warn!("error while decoding client {client_id}: {:?}. Most likely due to a client state upgrade. This can then be safely ignored", err);
                                            continue
                                        }
                                    };

                                    cs.chain_id().to_string()
                                }
                                WasmClientType::Tendermint => {
                                    let cs = match unionlabs::ibc::lightclients::tendermint::client_state::ClientState::decode_as::<Proto>(&cs.data) {
                                        Ok(cs) => cs,
                                        Err(err) => {
                                            warn!("error while decoding client {client_id}: {:?}. Most likely due to a client state upgrade. This can then be safely ignored", err);
                                            continue
                                        }
                                    };

                                    cs.chain_id().to_string()
                                }
                                WasmClientType::Scroll => {
                                    let cs = match unionlabs::ibc::lightclients::scroll::client_state::ClientState::decode_as::<Proto>(&cs.data) {
                                        Ok(cs) => cs,
                                        Err(err) => {
                                            warn!("error while decoding client {client_id}: {:?}. Most likely due to a client state upgrade. This can then be safely ignored", err);
                                            continue
                                        }
                                    };

                                    cs.chain_id().to_string()
                                }
                                WasmClientType::Arbitrum => {
                                    let cs = match unionlabs::ibc::lightclients::arbitrum::client_state::ClientState::decode_as::<Proto>(&cs.data) {
                                        Ok(cs) => cs,
                                        Err(err) => {
                                            warn!("error while decoding client {client_id}: {:?}. Most likely due to a client state upgrade. This can then be safely ignored", err);
                                            continue
                                        }
                                    };

                                    cs.chain_id().to_string()
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

                                    cs.chain_id().to_string()
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
            IndexerConfig::Eth(eth_config) => {
                let provider =
                    Provider::<Http>::try_from(eth_config.urls[0].clone().as_str()).unwrap();

                let chain_id = provider.get_chainid().await.unwrap().as_u64().to_string();
                dbg!("hi");

                let eth_clients = sqlx::query!(
                    r#"
                    SELECT
                        cl.transaction_hash, cl.client_id, ch.id
                    FROM
                        v0_evm.client_created cl
                    JOIN
                        v0.chains ch
                    ON
                        cl.chain_id = ch.id
                    WHERE
                        ch.chain_id = $1
                    "#,
                    chain_id
                )
                .fetch_all(&db)
                .await
                .unwrap();

                for record in eth_clients {
                    let Some(client_id) = record.client_id else {
                        tracing::info!(
                            internal_db_chain_id = record.id,
                            %chain_id,
                            "skipping record"
                        );
                        continue;
                    };

                    let tx = provider
                        .get_transaction(
                            ethers::types::H256::from_str(&record.transaction_hash.unwrap())
                                .unwrap(),
                        )
                        .await
                        .unwrap()
                        .unwrap();

                    let msg = match <contracts::ibc_handler::CreateClientCall as ethers::abi::AbiDecode>::decode(&tx.input) {
                        Ok(msg) => msg,
                        Err(err) => {
                            warn!("could not decode CreateClientCall, most likely due to ABI change: {}", err);
                            continue
                        }
                    };

                    match &*msg.0.client_type {
                        "cometbls" => {
                            let cs = unionlabs::ibc::lightclients::cometbls::client_state::ClientState::decode_as::<EthAbi>(&msg.0.client_state_bytes).unwrap();

                            datas.push(Data {
                                chain_id: record.id,
                                client_id,
                                counterparty_chain_id: cs.chain_id.to_string(),
                            })
                        }
                        ty => panic!("unknown evm client type `{ty}`"),
                    }
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
