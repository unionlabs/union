use sqlx::Postgres;

use crate::indexer::{
    api::IndexerError,
    event::types::ChannelId,
    handler::types::ChannelMetaData,
    record::{InternalChainId, PgValue},
};

pub async fn get_channel_meta_data(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    internal_chain_id: &InternalChainId,
    channel_id: &ChannelId,
) -> Result<Option<ChannelMetaData>, IndexerError> {
    sqlx::query!(
        "
        SELECT 
            chain.chain_id,
            chain.family,
            counterparty_chain.chain_id AS counterparty_chain_id,
            counterparty_chain.family   AS counterparty_family,
            connection.client_id,
            connection.counterparty_client_id,
            connection.connection_id,
            connection.counterparty_connection_id,
            channel.port_id,
            channel.counterparty_port_id,
            channel.counterparty_channel_id,
            channel.version             AS channel_version,
            client.client_type,
            chain.id                    AS internal_chain_id,
            counterparty_chain.id       AS internal_counterparty_chain_id,
            chain.rpc_type,
            counterparty_chain.rpc_type AS counterparty_rpc_type,
            chain.testnet               AS chain_testnet,
            counterparty_chain.testnet  AS counterparty_chain_testnet
        FROM config.chains chain
                JOIN (SELECT channel_open_init.internal_chain_id,
                            channel_open_init.port_id,
                            channel_open_init.channel_id,
                            channel_open_ack.counterparty_channel_id,
                            channel_open_init.connection_id,
                            channel_open_init.counterparty_port_id,
                            channel_open_init.version
                    FROM v2_sync.channel_open_init_sync channel_open_init
                    JOIN v2_sync.channel_open_ack_sync channel_open_ack 
                        ON channel_open_init.internal_chain_id = channel_open_ack.internal_chain_id 
                        AND channel_open_init.channel_id = channel_open_ack.channel_id 
                    UNION ALL
                    SELECT channel_open_try.internal_chain_id,
                            channel_open_try.port_id,
                            channel_open_try.channel_id,
                            channel_open_try.counterparty_channel_id,
                            channel_open_try.connection_id,
                            channel_open_try.counterparty_port_id,
                            channel_open_try.counterparty_version
                    FROM v2_sync.channel_open_try_sync channel_open_try) channel
                    ON channel.internal_chain_id = $1 AND channel.channel_id = $2
                JOIN (SELECT connection_open_ack.internal_chain_id,
                            connection_open_ack.connection_id,
                            connection_open_ack.client_id,
                            connection_open_ack.counterparty_client_id,
                            connection_open_ack.counterparty_connection_id
                    FROM v2_sync.connection_open_ack_sync connection_open_ack
                    UNION ALL
                    SELECT connection_open_try.internal_chain_id,
                            connection_open_try.connection_id,
                            connection_open_try.client_id,
                            connection_open_try.counterparty_client_id,
                            connection_open_try.counterparty_connection_id
                    FROM v2_sync.connection_open_try_sync connection_open_try) connection
                    ON connection.internal_chain_id = $1 AND channel.connection_id = connection.connection_id
                JOIN v2_sync.create_client_sync client
                    ON client.internal_chain_id = $1 AND connection.client_id = client.client_id
                JOIN config.client_type client_type ON client.client_type = client_type.client_type
                JOIN config.chain_representations counterparty_chain_representation
                    ON client.counterparty_chain_id = counterparty_chain_representation.chain_id AND
                        client_type.ibc_interface = counterparty_chain_representation.ibc_interface
                JOIN config.chains counterparty_chain
                    ON counterparty_chain_representation.internal_chain_id = counterparty_chain.id
        WHERE chain.id = $1
        ",
        internal_chain_id.pg_value()?,
        channel_id.pg_value()?,
    )
    .fetch_optional(tx.as_mut())
    .await?
    .map(|record| Ok(ChannelMetaData {
        rpc_type: record.rpc_type.try_into()?,
        counterparty_rpc_type: record.counterparty_rpc_type.try_into()?,
        internal_chain_id: record.internal_chain_id.into(),
        internal_counterparty_chain_id: record.internal_counterparty_chain_id.into(),
        universal_chain_id: (record.family, Some(record.chain_id.clone())).try_into()?,
        universal_counterparty_chain_id: (record.counterparty_family, Some(record.counterparty_chain_id.clone())).try_into()?,
        canonical_chain_id: record.chain_id.into(),
        canonical_counterparty_chain_id: record.counterparty_chain_id.into(),
        network: record.chain_testnet.try_into()?,
        counterparty_network: record.counterparty_chain_testnet.try_into()?,
        client_type: record.client_type.try_into()?,
        client_id: record.client_id.try_into()?,
        counterparty_client_id: record.counterparty_client_id.try_into()?,
        connection_id: record.connection_id.try_into()?,
        counterparty_connection_id: record.counterparty_connection_id.try_into()?,
        channel_id: *channel_id,
        port_id: record.port_id.try_into()?,
        counterparty_port_id: record.counterparty_port_id.try_into()?,
        counterparty_channel_id: record.counterparty_channel_id.try_into()?,
        channel_version: record.channel_version.try_into()?,
    }))
    .transpose()
}
