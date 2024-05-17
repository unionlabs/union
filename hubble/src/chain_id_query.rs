use std::str::FromStr;

use ethers::providers::{Http, Middleware, Provider};
use prost::Message;
use protos::ibc::{
    core::client::v1::QueryClientStateRequest, lightclients::wasm::v1::QueryCodeRequest,
};
use sqlx::PgPool;
use tendermint_rpc::{Client, HttpClient};
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
            IndexerConfig::Tm(tm_config) => {
                let client = HttpClient::new(tm_config.url.as_str()).unwrap();

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
                    SELECT
                        cl.client_id, ch.id
                    FROM
                        v0.create_clients cl
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

                for record in tm_clients {
                    let client_state = grpc_client
                        .client_state(QueryClientStateRequest {
                            client_id: record.client_id.clone().unwrap(),
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

                            // dbg!(cs);

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
                                    let cs = unionlabs::ibc::lightclients::ethereum::client_state::ClientState::decode_as::<Proto>(&cs.data).unwrap();

                                    cs.chain_id().to_string()
                                }
                                WasmClientType::Cometbls => {
                                    let cs = unionlabs::ibc::lightclients::cometbls::client_state::ClientState::decode_as::<Proto>(&cs.data).unwrap();

                                    cs.chain_id().to_string()
                                }
                                WasmClientType::Tendermint => {
                                    let cs = unionlabs::ibc::lightclients::tendermint::client_state::ClientState::decode_as::<Proto>(&cs.data).unwrap();

                                    cs.chain_id().to_string()
                                }
                                WasmClientType::Scroll => {
                                    let cs = unionlabs::ibc::lightclients::scroll::client_state::ClientState::decode_as::<Proto>(&cs.data).unwrap();

                                    cs.chain_id().to_string()
                                }
                                WasmClientType::Arbitrum => {
                                    let cs = unionlabs::ibc::lightclients::arbitrum::client_state::ClientState::decode_as::<Proto>(&cs.data).unwrap();

                                    cs.chain_id().to_string()
                                }
                                WasmClientType::Linea => todo!("We still need to add linea"),
                            };

                            datas.push(Data {
                                chain_id: record.id,
                                client_id: record.client_id.clone().unwrap(),
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
                                client_id: record.client_id.clone().unwrap(),
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
                let provider = Provider::<Http>::try_from(eth_config.url.clone().as_str()).unwrap();

                let chain_id = provider.get_chainid().await.unwrap().as_u64().to_string();
                dbg!("hi");

                let eth_clients = sqlx::query!(
                    r#"
                    SELECT
                        cl.transaction_hash, cl.client_id, ch.id
                    FROM
                        v0.evm_client_created cl
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

                    let msg = <contracts::ibc_handler::CreateClientCall as ethers::abi::AbiDecode>::decode(
                        &tx.input,
                    )
                    .unwrap();

                    match &*msg.msg.client_type {
                        "cometbls" => {
                            let cs = unionlabs::ibc::lightclients::cometbls::client_state::ClientState::decode_as::<EthAbi>(&msg.msg.client_state_bytes).unwrap();

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

    let res = sqlx::query!(
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

    println!("rows affected: {}", res.rows_affected());
}
