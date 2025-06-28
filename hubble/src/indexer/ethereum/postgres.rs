use std::collections::HashMap;

use alloy::primitives::Address;
use sqlx::{Postgres, Transaction};

use crate::indexer::{
    api::IndexerError,
    ethereum::abi::{Abi, AbiRegistration},
    record::{InternalChainId, PgValue},
};

pub async fn get_abi_registration(
    tx: &mut Transaction<'_, Postgres>,
    internal_chain_id: InternalChainId,
    height: crate::indexer::event::types::BlockHeight,
) -> Result<AbiRegistration, IndexerError> {
    let result = sqlx::query!(
        r#"
        SELECT    address, abi
        FROM      v2_evm.contracts
        WHERE     internal_chain_id = $1
        AND       $2 between start_height and end_height
        AND       abi IS NOT NULL
        "#,
        internal_chain_id.pg_value()?,
        height.pg_value()?,
    )
    .fetch_all(tx.as_mut())
    .await?
    .into_iter()
    .map(|record| {
        (
            record
                .address
                .parse::<Address>()
                .expect("abi can be parsed"),
            Abi {
                definition: record.abi.expect("abi not null"),
            },
        )
    })
    .collect::<HashMap<Address, Abi>>();

    Ok(AbiRegistration {
        administration: result,
    })
}
