use std::time::Duration;

use prost::Message;
use protos::ibc::{
    core::client::v1::QueryClientStateRequest, lightclients::wasm::v1::QueryCodeRequest,
};
use tokio::{task::JoinSet, time::interval};
use tracing::{error, info, info_span, warn, Instrument};
use unionlabs::{
    encoding::{DecodeAs, Proto},
    parse_wasm_client_type, WasmClientType,
};

use crate::{
    indexer::{
        api::IndexerError,
        tm::{postgres::unmapped_client_ids, provider::Provider},
    },
    postgres::insert_client_mapping,
};

pub fn schedule_create_client_checker(
    pg_pool: sqlx::PgPool,
    join_set: &mut JoinSet<Result<(), IndexerError>>,
    provider: Provider,
    internal_chain_id: i32,
) {
    join_set.spawn(
        async move {
            let mut interval = interval(Duration::from_secs(10 * 60));

            loop {
                info!("{}: check", internal_chain_id);

                let tm_clients = match unmapped_client_ids(&pg_pool, internal_chain_id).await {
                    Ok(tm_clients) => tm_clients,
                    Err(err) => {
                        error!("error fetching unmapped clients: {:?} => retry later", err);
                        continue;
                    },
                };

                info!("{}, check: unmapped clients: {}", internal_chain_id, tm_clients.len());

                for client_id in tm_clients {
                    info!("{}: found unmapped client-id: {}", internal_chain_id, client_id);
                    let (provider_id, client_state) = match provider.client_state(QueryClientStateRequest { client_id: client_id.clone()}, None)
                        .await
                        {
                            Ok(result) => (
                                result.provider_id,
                                result.response.into_inner().client_state.expect("client state"),
                            ),
                            Err(err) => {
                                warn!("{}: error fetching client status for {}: {}", internal_chain_id, client_id, err);
                                continue;
                            }
                        };

                    info!("{}: client_state: {:?}", internal_chain_id, client_state);

                    match &*client_state.type_url {
                        "/ibc.lightclients.wasm.v1.ClientState" => {
                            let cs = protos::ibc::lightclients::wasm::v1::ClientState::decode(
                                &*client_state.value,
                            )
                            .unwrap();

                            let wasm_blob = provider.code(QueryCodeRequest { checksum: hex::encode(&*cs.checksum) }, Some(provider_id))
                                .await
                                .unwrap()
                                .response
                                .into_inner()
                                .data;

                            let client_type = parse_wasm_client_type(wasm_blob).unwrap();

                            let counterparty_chain_id = match client_type.unwrap() {
                                WasmClientType::EthereumMinimal
                                | WasmClientType::EthereumMainnet => {
                                    let cs = match ethereum_light_client_types::ClientState::decode_as::<Proto>(&cs.data) {
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
                                    let cs = match cometbls_light_client_types::ClientState::decode_as::<Proto>(&cs.data) {
                                        Ok(cs) => cs,
                                        Err(err) => {
                                            warn!("error while decoding client {client_id}: {:?}. Most likely due to a client state upgrade. This can then be safely ignored", err);
                                            continue
                                        }
                                    };

                                    cs.chain_id.as_str().to_owned()
                                }
                                WasmClientType::Tendermint => {
                                    let cs = match tendermint_light_client_types::ClientState::decode_as::<Proto>(&cs.data) {
                                        Ok(cs) => cs,
                                        Err(err) => {
                                            warn!("error while decoding client {client_id}: {:?}. Most likely due to a client state upgrade. This can then be safely ignored", err);
                                            continue
                                        }
                                    };

                                    cs.chain_id
                                }
                                WasmClientType::Scroll => {
                                    let cs = match scroll_light_client_types::ClientState::decode_as::<Proto>(&cs.data) {
                                        Ok(cs) => cs,
                                        Err(err) => {
                                            warn!("error while decoding client {client_id}: {:?}. Most likely due to a client state upgrade. This can then be safely ignored", err);
                                            continue
                                        }
                                    };

                                    cs.chain_id.to_string()
                                }
                                WasmClientType::Arbitrum => {
                                    let cs = match arbitrum_light_client_types::ClientState::decode_as::<Proto>(&cs.data) {
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
                                    let cs = match berachain_light_client_types::ClientState::decode_as::<Proto>(&cs.data) {
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
                                    let cs = match movement_light_client_types::ClientState::decode_as::<Proto>(&cs.data) {
                                        Ok(cs) => cs,
                                        Err(err) => {
                                            warn!("error while decoding client {client_id}: {:?}. Most likely due to a client state upgrade. This can then be safely ignored", err);
                                            continue;
                                        }
                                    };

                                    cs.chain_id.to_string()
                                }
                            };

                            info!("{}: add mapping {} => {} (ibc.lightclients.wasm.v1.ClientState)", internal_chain_id, client_id, counterparty_chain_id);

                            insert_client_mapping(
                                &pg_pool,
                                internal_chain_id,
                                client_id.clone(),
                                counterparty_chain_id,
                            ).await?;
                        }
                        "/ibc.lightclients.tendermint.v1.ClientState" => {
                            let cs =
                                protos::ibc::lightclients::tendermint::v1::ClientState::decode(
                                    &*client_state.value,
                                )
                                .unwrap();

                            info!("{}: add mapping {} => {} (ibc.lightclients.tendermint.v1.ClientState)", internal_chain_id, client_id, cs.chain_id);

                            insert_client_mapping(
                                &pg_pool,
                                internal_chain_id,
                                client_id.clone(),
                                cs.chain_id,
                            ).await?;
                        }
                        _ => {
                            panic!("unknown client state type {}", client_state.type_url)
                        }
                    };
                }

                interval.tick().await;
            }
        }
        .instrument(info_span!("clients").or_current()),
    );
}
