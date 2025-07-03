use std::fmt;

use bytes::Bytes;
use hex::encode;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::indexer::{
    api::IndexerError,
    enrich::wrapping::{
        create3::create3_0_1, instantiate2::instantiate2_0_1, osmosis::predict_osmosis_wrapper_0_1,
        postgres::get_ibc_interface_and_ucs03,
    },
    event::types::{ChannelId, ContractAddress, Denom},
    handler::types::WrapDirection,
    record::InternalChainId,
};

mod aptos;
mod cosmos;
mod create3;
mod erc55;
mod instantiate2;
mod osmosis;
mod postgres;

#[allow(clippy::too_many_arguments)] // migrating postgres code; will refactor later
pub async fn wrap_direction_chains(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    internal_source_chain_id: &InternalChainId,
    internal_destination_chain_id: &InternalChainId,
    intermediate_channel_ids: &IntermediateChannelIds,
    source_channel_id: &ChannelId,
    destination_channel_id: &ChannelId,
    base_denom: &Denom,
    quote_denom: &Denom,
) -> Result<Option<WrapDirection>, IndexerError> {
    let (source_ibc_interface, source_contract_address_display, source_minter) =
        &get_ibc_interface_and_ucs03(tx, internal_source_chain_id).await?;

    let (destination_ibc_interface, destination_contract_address_display, destination_minter) =
        &get_ibc_interface_and_ucs03(tx, internal_destination_chain_id).await?;

    wrap_direction_pure(
        source_ibc_interface,
        destination_ibc_interface,
        intermediate_channel_ids,
        source_channel_id,
        destination_channel_id,
        source_contract_address_display,
        source_minter,
        destination_contract_address_display,
        destination_minter,
        base_denom,
        quote_denom,
    )
}

#[allow(clippy::too_many_arguments)] // migrating postgres code; will refactor later
fn wrap_direction_pure(
    source_ibc_interface: &IbcInterface,
    destination_ibc_interface: &IbcInterface,
    intermediate_channel_ids: &IntermediateChannelIds,
    source_channel_id: &ChannelId,
    destination_channel_id: &ChannelId,
    source_contract_address_display: &ContractAddressDisplay,
    source_minter: &Option<Minter>,
    destination_contract_address_display: &ContractAddressDisplay,
    destination_minter: &Option<Minter>,
    base_denom: &Denom,
    quote_denom: &Denom,
) -> Result<Option<WrapDirection>, IndexerError> {
    Ok(
        if wrap_is_same_as_predict_wrapped_address(
            destination_ibc_interface,
            intermediate_channel_ids,
            destination_channel_id,
            base_denom,
            quote_denom,
            destination_contract_address_display,
            destination_minter,
        )? {
            Some(WrapDirection::Wrapping)
        } else if wrap_is_same_as_predict_wrapped_address(
            source_ibc_interface,
            intermediate_channel_ids,
            source_channel_id,
            quote_denom,
            base_denom,
            source_contract_address_display,
            source_minter,
        )? {
            Some(WrapDirection::Unwrapping)
        } else {
            None
        },
    )
}

fn wrap_is_same_as_predict_wrapped_address(
    destination_ibc_interface: &IbcInterface,
    intermediate_channel_ids: &IntermediateChannelIds,
    receiver_channel_id: &ChannelId,
    base_denom: &Denom,
    quote_denom: &Denom,
    contract_address_display: &ContractAddressDisplay,
    minter: &Option<Minter>,
) -> Result<bool, IndexerError> {
    Ok(match (destination_ibc_interface, minter) {
        (IbcInterface::IbcSolidity, _) => {
            wrap_evm(
                intermediate_channel_ids,
                receiver_channel_id,
                base_denom,
                &contract_address_display.to_contract_address_assume_hex()?,
            )? == *quote_denom
        }
        (IbcInterface::IbcCosmwasm, Some(minter)) => {
            wrap_cosmos(
                intermediate_channel_ids,
                receiver_channel_id,
                base_denom,
                minter,
            )? == quote_denom.to_bech32_decoded()?
        }
        (IbcInterface::IbcCosmwasm, None) => {
            debug!("no minter configured for {contract_address_display}");
            false
        }
    })
}

fn wrap_evm(
    intermediate_channel_ids: &IntermediateChannelIds,
    receiver_channel_id: &ChannelId,
    base_denom: &Denom,
    deployer: &ContractAddress,
) -> Result<Denom, IndexerError> {
    let result = create3_0_1(
        &intermediate_channel_ids.0,
        receiver_channel_id.0.into(),
        base_denom.0.as_ref(),
        deployer.0.as_ref(),
    ).map_err(|e| IndexerError::WrapperPredictionError(
        "create3".to_string(),
        e.to_string(),
    ))?;
    
    Ok(bytes::Bytes::from(result).into())
}

fn wrap_cosmos(
    intermediate_channel_ids: &IntermediateChannelIds,
    receiver_channel_id: &ChannelId,
    base_denom: &Denom,
    minter: &Minter,
) -> Result<Denom, IndexerError> {
    let result = match minter {
        Minter::Cw20(minter_address_display) => {
            instantiate2_0_1(
                &intermediate_channel_ids.0,
                receiver_channel_id.0.into(),
                base_denom.0.as_ref(),
                minter_address_display
                    .to_contract_address_assume_bech32()?
                    .0
                    .as_ref(),
            ).map_err(|e| IndexerError::WrapperPredictionError(
                "instantiate2".to_string(),
                e.to_string(),
            ))?
        },
        Minter::OsmosisTokenfactory(minter_address_display) => {
            predict_osmosis_wrapper_0_1(
                &intermediate_channel_ids.0,
                receiver_channel_id.0.into(),
                base_denom.0.as_ref(),
                &minter_address_display.0,
            ).map_err(|e| IndexerError::WrapperPredictionError(
                "osmosis".to_string(),
                e.to_string(),
            ))?
        }
    };

    Ok(bytes::Bytes::from(result).into())
}

pub struct IntermediateChannelIds(Bytes);

impl Default for IntermediateChannelIds {
    fn default() -> Self {
        Self(Bytes::from_static(&[0x00])) // todo: currently always '0x00'.
    }
}

impl Denom {
    fn to_bech32_decoded(&self) -> Result<Denom, IndexerError> {
        bech32::decode(std::str::from_utf8(&self.0).map_err(|_| {
            IndexerError::Bech32DecodeErrorInvalidBech32("denom".to_string(), encode(&self.0))
        })?)
        .map(|(_, data)| Bytes::from(data).into())
        .map_err(|_| {
            IndexerError::Bech32DecodeErrorInvalidBech32("denom".to_string(), encode(&self.0))
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IbcInterface {
    #[serde(rename = "ibc-solidity")]
    IbcSolidity,
    #[serde(rename = "ibc-cosmwasm")]
    IbcCosmwasm,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Minter {
    #[serde(rename = "cw20")]
    Cw20(ContractAddressDisplay),
    #[serde(rename = "osmosis_tokenfactory")]
    OsmosisTokenfactory(ContractAddressDisplay),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractAddressDisplay(pub String);

impl ContractAddressDisplay {
    fn to_contract_address_assume_hex(&self) -> Result<ContractAddress, IndexerError> {
        let hex_str = &self.0.strip_prefix("0x").ok_or_else(|| {
            IndexerError::HexDecodeErrorExpecting0x(
                "contract-address-display".to_string(),
                self.0.clone(),
            )
        })?;
        let vec = hex::decode(hex_str).map_err(|_| {
            IndexerError::HexDecodeErrorInvalidHex(
                "contract-address-display".to_string(),
                self.0.clone(),
            )
        })?;
        Ok(Bytes::from(vec).into())
    }
    fn to_contract_address_assume_bech32(&self) -> Result<ContractAddress, IndexerError> {
        bech32::decode(&self.0)
            .map(|(_, data)| Bytes::from(data).into())
            .map_err(|_| {
                IndexerError::Bech32DecodeErrorInvalidBech32(
                    "contract-address".to_string(),
                    self.0.clone(),
                )
            })
    }
}

impl fmt::Display for ContractAddressDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl From<String> for ContractAddressDisplay {
    fn from(value: String) -> Self {
        Self(value)
    }
}
