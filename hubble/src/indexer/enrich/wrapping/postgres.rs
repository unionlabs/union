use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    enrich::wrapping::{ContractAddressDisplay, IbcInterface, Minter},
    record::{InternalChainId, PgValue},
};

pub async fn get_ibc_interface_and_ucs03(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    internal_chain_id: &InternalChainId,
) -> Result<(IbcInterface, ContractAddressDisplay, Option<Minter>), IndexerError> {
    trace!("get_ibc_interface_and_ucs03: {internal_chain_id}");

    let ibc_interface = sqlx::query!(
        "
            SELECT c.ibc_interface
            FROM config.chains c
            WHERE id = $1;
        ",
        internal_chain_id.pg_value()?,
    )
    .fetch_optional(tx.as_mut())
    .await?
    .map(|record| match record.ibc_interface.as_deref() {
        Some("ibc-cosmwasm") => Ok(IbcInterface::IbcCosmwasm),
        Some("ibc-solidity") => Ok(IbcInterface::IbcSolidity),
        unsupported => Err(IndexerError::InternalCannotMapFromDatabaseDomain(
            "ibc-interface".to_string(),
            unsupported.unwrap_or("-").to_string(),
        )),
    })
    .transpose()?
    .ok_or_else(|| {
        IndexerError::InternalCannotMapFromDatabaseDomain(
            "ibc-interface".to_string(),
            format!("no chain: {internal_chain_id}"),
        )
    })?;

    let (contract_address_display, minter) = sqlx::query!(
        "
            SELECT contract_address_display, minter_type, minter_address_display
            FROM config.ucs03
            WHERE internal_chain_id = $1;
        ",
        internal_chain_id.pg_value()?,
    )
    .fetch_optional(tx.as_mut())
    .await?
    .map(|record| {
        let contract_address_display = record.contract_address_display.into();
        let minter = match record.minter_type.as_deref() {
            Some("cw20") => Ok(Some(Minter::Cw20(
                record
                    .minter_address_display
                    .ok_or_else(|| {
                        IndexerError::InternalCannotMapFromDatabaseDomain(
                            "minter-address_display (cw20)".to_string(),
                            "-".to_string(),
                        )
                    })?
                    .into(),
            ))),
            Some("osmosis_tokenfactory") => Ok(Some(Minter::OsmosisTokenfactory(
                record
                    .minter_address_display
                    .ok_or_else(|| {
                        IndexerError::InternalCannotMapFromDatabaseDomain(
                            "minter-address_display (osmosis)".to_string(),
                            "-".to_string(),
                        )
                    })?
                    .into(),
            ))),
            Some(unsupported) => Err(IndexerError::InternalCannotMapFromDatabaseDomain(
                "minter-type".to_string(),
                unsupported.to_string(),
            )),
            None => Ok(None),
        }?;

        Ok::<_, IndexerError>((contract_address_display, minter))
    })
    .transpose()?
    .ok_or_else(|| {
        IndexerError::InternalCannotMapFromDatabaseDomain(
            "ucs03".to_string(),
            format!("no chain: {internal_chain_id}"),
        )
    })?;

    Ok((ibc_interface, contract_address_display, minter))
}
