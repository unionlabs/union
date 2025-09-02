use alloy_sol_types::SolType;
use bytes::Bytes;
use ibc_union_spec::{Packet, Timestamp};
use itertools::Itertools;
use serde_json::{Map, Value};
use time::{macros::format_description, UtcOffset};
use tracing::{debug, error, warn};

pub(crate) mod ucs03_zkgm_0;
mod wrapping;

use super::{
    event::types::Denom,
    handler::types::{TokenName, WrapDirection},
};
use crate::indexer::{
    api::IndexerError,
    enrich::{
        ucs03_zkgm_0::{
            packet::ZkgmPacket,
            packet_ack::{decode, format_flatten},
            PacketHash,
        },
        wrapping::{wrap_direction_chains, IntermediateChannelIds},
    },
    event::types::{BlockHeight, ChannelId, TimeoutTimestamp, UniversalChainId},
    handler::types::{
        bytes_to_value, string_0x_to_bytes, string_base64_to_bytes, AddressCanonical, AddressZkgm,
        Amount, Bond, ChannelMetaData, Fee, Instruction, InstructionHash, InstructionOpcode,
        InstructionPath, InstructionRootPath, InstructionRootSalt, InstructionVersion, Metadata,
        PacketShape, TokenOrderKind, Transfer, Unbond,
    },
    postgres::chain_context::fetch_chain_context_for_universal_chain_id,
    record::{
        change_counter::Changes, channel_meta_data::get_channel_meta_data,
        create_wrapped_token_record::CreateWrappedTokenRecord,
        create_wrapped_token_relation_record::CreateWrappedTokenRelationRecord,
        packet_send_bond_record::PacketSendBondRecord,
        packet_send_decoded_record::PacketSendDecodedRecord,
        packet_send_instructions_search_record::PacketSendInstructionsSearchRecord,
        packet_send_record::PacketSendRecord,
        packet_send_transfers_record::PacketSendTransfersRecord,
        packet_send_unbond_record::PacketSendUnbondRecord, InternalChainId,
    },
};

pub async fn delete_enriched_data_for_block(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    universal_chain_id: &UniversalChainId,
    height: &BlockHeight,
) -> Result<Changes, IndexerError> {
    let chain_context = fetch_chain_context_for_universal_chain_id(tx, universal_chain_id).await?;

    let mut changes = Changes::default();
    changes += PacketSendDecodedRecord::delete_by_chain_and_height(
        tx,
        chain_context.internal_chain_id,
        *height,
    )
    .await?;
    changes += PacketSendTransfersRecord::delete_by_chain_and_height(
        tx,
        chain_context.internal_chain_id,
        *height,
    )
    .await?;
    changes += PacketSendInstructionsSearchRecord::delete_by_chain_and_height(
        tx,
        chain_context.internal_chain_id,
        *height,
    )
    .await?;
    changes += CreateWrappedTokenRelationRecord::delete_by_chain_and_height(
        tx,
        chain_context.internal_chain_id,
        *height,
    )
    .await?;

    Ok(changes)
}

pub async fn enrich_create_wrapped_token_record(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    record: CreateWrappedTokenRecord,
) -> Result<Changes, IndexerError> {
    let mut changes = Changes::default();

    let internal_chain_id: InternalChainId = record.internal_chain_id.into();
    let channel_id: ChannelId = record.channel_id.try_into()?;

    let Some(channel) = get_channel_meta_data(tx, &internal_chain_id, &channel_id).await? else {
        return Ok(Changes::default());
    };

    let internal_unwrapping_chain_id = channel.internal_counterparty_chain_id;

    let create_wrapped_token_relation_record: CreateWrappedTokenRelationRecord =
        (&record, &internal_unwrapping_chain_id).try_into()?;

    changes += create_wrapped_token_relation_record.insert(tx).await?;

    Ok(changes)
}

pub async fn enrich_packet_send_record(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    record: PacketSendRecord,
) -> Result<Changes, IndexerError> {
    let mut changes = Changes::default();

    let internal_chain_id: InternalChainId = record.internal_chain_id.into();
    let channel_id: ChannelId = record.channel_id.try_into()?;
    let packet_hash: crate::indexer::event::types::PacketHash =
        bytes::Bytes::from(record.packet_hash.clone()).into();

    let Some(channel) = get_channel_meta_data(tx, &internal_chain_id, &channel_id).await? else {
        debug!("no channel details for chain {internal_chain_id} and channel {channel_id}");
        return Ok(changes);
    };

    let channel_version = channel.channel_version.clone();

    // currently using the copied code from the pg extension. refactor later to use original types.
    let result = match channel_version.0.as_str() {
        "ucs03-zkgm-0" => match decode(
            &record.data,
            None,
            &PacketHash(record.packet_hash.clone().try_into().unwrap()),
            Some("all"),
        ) {
            Ok(decoded) => decoded,
            Err(error) => {
                warn!("invalid packet data - chain {internal_chain_id} / channel {channel_id} / version {channel_version} with packet-hash: {} and data: {} => {error}", hex::encode(&record.packet_hash), hex::encode(&record.data));
                return Ok(changes);
            }
        },
        _ => {
            debug!("unsupported channel version for chain {internal_chain_id} and channel {channel_id} and version {channel_version}");
            return Ok(changes);
        }
    };

    let Some(Value::Object(tree)) = result.get("tree") else {
        error!("expecting 'tree' Object value when decoding: {result}");
        return Err(IndexerError::ZkgmExpectingTree(
            internal_chain_id,
            channel_id,
            packet_hash,
            result,
        ));
    };

    let Some(Value::Array(flatten)) = result.get("flatten") else {
        error!("expecting 'flatten' Array value when decoding: {result}");
        return Err(IndexerError::ZkgmExpectingFlatten(
            internal_chain_id,
            channel_id,
            packet_hash,
            result,
        ));
    };

    let packet_structure = packet_structure(flatten)?;
    let sort_order = sort_order(&record, &channel)?;

    let packet_send_decoded_record: PacketSendDecodedRecord = (
        &record,
        &channel,
        &Value::Object(tree.clone()),
        &Value::Array(flatten.clone()),
        &packet_structure,
        &sort_order,
    )
        .try_into()?;
    changes += packet_send_decoded_record.insert(tx).await?;

    for transfer in get_transfers(tx, &record, &channel, &packet_structure, flatten).await? {
        let packet_send_transfers_record: PacketSendTransfersRecord = (
            &record,
            &transfer,
            &channel,
            &format!("{sort_order}-{:03}", transfer.transfer_index.0),
        )
            .try_into()?;
        changes += packet_send_transfers_record.insert(tx).await?;
    }

    for bond in get_bonds(tx, &channel, &packet_structure, flatten).await? {
        let packet_send_bond_record: PacketSendBondRecord =
            (&record, &bond, &sort_order).try_into()?;
        changes += packet_send_bond_record.insert(tx).await?;
    }

    for unbond in get_unbonds(&channel, &packet_structure, flatten).await? {
        let packet_send_unbond_record: PacketSendUnbondRecord =
            (&record, &unbond, &channel, &sort_order).try_into()?;
        changes += packet_send_unbond_record.insert(tx).await?;
    }

    // insert packet send transaction
    let instructions = get_instructions(flatten)?
        .into_iter()
        .map(|instruction| {
            (
                &record,
                &instruction,
                &channel,
                &instruction.sort_order(&sort_order)?,
            )
                .try_into()
        })
        .collect::<Result<Vec<PacketSendInstructionsSearchRecord>, IndexerError>>()?;

    changes += PacketSendInstructionsSearchRecord::insert_batch(tx, &instructions).await?;

    Ok(changes)
}

impl Instruction {
    fn sort_order(&self, packet_sort_order: &str) -> Result<String, IndexerError> {
        let indices = self.instruction_path.as_indices()?;

        Ok(match indices.is_empty() {
            true => packet_sort_order.to_string(),
            false => indices
                .iter()
                .map(|index| format!("{packet_sort_order}-{:03}", index))
                .join("."),
        })
    }
}

async fn get_transfers(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    record: &PacketSendRecord,
    channel: &ChannelMetaData,
    packet_structure: &str,
    flatten: &[Value],
) -> Result<Vec<Transfer>, IndexerError> {
    let Some(packet_shape) = packet_shape(packet_structure, flatten)? else {
        return Ok(vec![]);
    };

    let (transfer_data, fee_data) = match packet_shape {
        PacketShape::BatchV0TransferV0Fee
        | PacketShape::BatchV0TransferV1Fee
        | PacketShape::BatchV0TransferV2Fee => (
            InstructionDecoder::from_values_with_index(flatten, 1)?,
            Some(InstructionDecoder::from_values_with_index(flatten, 2)?),
        ),
        PacketShape::BatchV0TransferV1 | PacketShape::BatchV0TransferV2 => (
            InstructionDecoder::from_values_with_index(flatten, 1)?,
            None,
        ),
        PacketShape::TransferV0 | PacketShape::TransferV1 | PacketShape::TransferV2 => (
            InstructionDecoder::from_values_with_index(flatten, 0)?,
            None,
        ),
        _ => {
            // not a transfer
            return Ok(vec![]);
        }
    };

    let transfer_index = 0;
    let sender_zkgm =
        &AddressZkgm::from_string_0x(transfer_data.get_string("sender")?, channel.rpc_type)?;
    let receiver_zkgm = &AddressZkgm::from_string_0x(
        transfer_data.get_string("receiver")?,
        channel.counterparty_rpc_type,
    )?;
    let base_token = transfer_data.get_string("baseToken")?.try_into()?;
    let base_amount: Amount = transfer_data.get_string("baseAmount")?.try_into()?;
    let base_token_name = transfer_data
        .get_string_opt("baseTokenName")?
        .map(|d| d.into());
    let base_token_path = transfer_data
        .get_string_opt("baseTokenPath")?
        .map(|d| d.try_into())
        .transpose()?;
    let base_token_symbol = transfer_data
        .get_string_opt("baseTokenSymbol")?
        .map(|d| d.into());
    let base_token_decimals = transfer_data
        .get_u32_opt("baseTokenDecimals")?
        .map(|d| d.into());
    let quote_token = transfer_data.get_string("quoteToken")?.try_into()?;
    let quote_amount: Amount = transfer_data.get_string("quoteAmount")?.try_into()?;

    let kind: Option<TokenOrderKind> = transfer_data.get_u8_opt("kind")?.map(|d| d.into());
    let metadata: Option<Metadata> = transfer_data
        .get_string_opt("metadata")?
        .map(|d| d.try_into())
        .transpose()?;

    let wrap_direction = match transfer_data.version.0 {
        0..=1 => {
            wrap_direction_chains(
                tx,
                &channel.internal_chain_id,
                &channel.internal_counterparty_chain_id,
                &IntermediateChannelIds::default(), // no support for intermediate channel ids.
                &record.source_channel_id.try_into()?,
                &record.destination_channel_id.try_into()?,
                &base_token,
                &quote_token,
            )
            .await?
        }
        _ => None, // no calculate wrapping on packet-send in version >= 2 messages. wrappings are exposed as new 'create wrapped token' events.
    };

    let fee = calculate_fee(
        fee_data,
        &base_token,
        &base_amount,
        &base_token_name,
        &quote_amount,
        &wrap_direction,
    )?;

    Ok(vec![Transfer {
        transfer_index: transfer_index.into(),
        sender_zkgm: sender_zkgm.clone(),
        sender_canonical: sender_zkgm.try_into().unwrap_or_else(
            |_| // TODO: fallback to be compatible with pg implementation. we should actually not expose this packet as a transfer
                AddressCanonical::from(sender_zkgm.bytes()),
        ),
        sender_display: sender_zkgm.try_into()?,
        receiver_zkgm: receiver_zkgm.clone(),
        receiver_canonical: receiver_zkgm.try_into().unwrap_or_else(
            |_| // TODO: fallback to be compatible with pg implementation. we should actually not expose this packet as a transfer
                AddressCanonical::from(receiver_zkgm.bytes()),
        ),
        receiver_display: receiver_zkgm.try_into()?,
        base_token,
        base_amount,
        base_token_name,
        base_token_path,
        base_token_symbol,
        base_token_decimals,
        quote_token,
        quote_amount,
        kind,
        metadata,
        fee,
        wrap_direction,
        packet_shape,
    }])
}

// ignore bonds that fail to parse until we have robust packet-shape detection
async fn get_bonds(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    channel: &ChannelMetaData,
    packet_structure: &str,
    flatten: &[Value],
) -> Result<Vec<Bond>, IndexerError> {
    Ok(
        match try_get_bonds(tx, channel, packet_structure, flatten).await {
            Ok(bonds) => bonds,
            Err(error) => {
                let packet = Value::Array(flatten.to_vec());
                warn!("error reading bond: {error} => {packet}");
                vec![]
            }
        },
    )
}

async fn try_get_bonds(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    channel: &ChannelMetaData,
    packet_structure: &str,
    flatten: &[Value],
) -> Result<Vec<Bond>, IndexerError> {
    let Some(packet_shape) = packet_shape(packet_structure, flatten)? else {
        return Ok(vec![]);
    };

    let Some((token_order, _bond, _increase_allowance, delivery)) = (match packet_shape {
        PacketShape::BondV2 => Some((
            InstructionDecoder::from_values_with_index(flatten, 1)?,
            InstructionDecoder::from_values_with_index(flatten, 2)?,
            InstructionDecoder::from_values_with_index(flatten, 3)?,
            InstructionDecoder::from_values_with_index(flatten, 4)?,
        )),
        _ => None,
    }) else {
        // not a bond
        return Ok(vec![]);
    };

    // -------------------------------------------------
    // fetch details from call that will deliver the lst
    // -------------------------------------------------

    let delivery_contract_calldata = delivery.get_call_message("contractCalldata")?;

    // delivery channel_id
    let Value::Number(delivery_channel_id) = delivery_contract_calldata
        .get("channel_id")
        .ok_or_else(|| {
            IndexerError::ZkgmExpectingInstructionField(
                "missing 'channel_id' in contractCalldata".to_string(),
                delivery_contract_calldata.to_string(),
            )
        })?
    else {
        return Err(IndexerError::ZkgmExpectingInstructionField(
            "expecting 'channel_id' as Number in contractCalldata".to_string(),
            delivery_contract_calldata.to_string(),
        ));
    };

    let Some(delivery_channel_id) = delivery_channel_id.as_u64() else {
        return Err(IndexerError::ZkgmExpectingInstructionField(
            "expecting 'channel_id' as integer in contractCalldata".to_string(),
            delivery_contract_calldata.to_string(),
        ));
    };

    let delivery_channel_id: ChannelId = u32::try_from(delivery_channel_id)
        .map_err(|_| {
            IndexerError::ZkgmExpectingInstructionField(
                "expecting 'channel_id' as u32 in contractCalldata".to_string(),
                delivery_contract_calldata.to_string(),
            )
        })?
        .into();

    // delivery timeout_timestamp
    let Value::String(delivery_timeout_timestamp) = delivery_contract_calldata
        .get("timeout_timestamp")
        .ok_or_else(|| {
            IndexerError::ZkgmExpectingInstructionField(
                "missing 'timeout_timestamp' in contractCalldata".to_string(),
                delivery_contract_calldata.to_string(),
            )
        })?
    else {
        return Err(IndexerError::ZkgmExpectingInstructionField(
            "expecting 'timeout_timestamp' as String in contractCalldata".to_string(),
            delivery_contract_calldata.to_string(),
        ));
    };

    let delivery_timeout_timestamp =
        TimeoutTimestamp(delivery_timeout_timestamp.parse().map_err(|_| {
            IndexerError::ZkgmExpectingInstructionField(
                "expecting 'timeout_timestamp' as u64 in contractCalldata".to_string(),
                delivery_contract_calldata.to_string(),
            )
        })?);

    // delivery salt
    let Value::String(delivery_salt_string) =
        delivery_contract_calldata.get("salt").ok_or_else(|| {
            IndexerError::ZkgmExpectingInstructionField(
                "missing 'salt' in contractCalldata".to_string(),
                delivery_contract_calldata.to_string(),
            )
        })?
    else {
        return Err(IndexerError::ZkgmExpectingInstructionField(
            "expecting 'salt' as String in contractCalldata".to_string(),
            delivery_contract_calldata.to_string(),
        ));
    };

    let mut delivery_salt = [0u8; 32];
    delivery_salt.copy_from_slice(&string_0x_to_bytes(delivery_salt_string, "salt")?);

    // delivery path
    let Value::String(delivery_path) = delivery_contract_calldata.get("path").ok_or_else(|| {
        IndexerError::ZkgmExpectingInstructionField(
            "missing 'path' in contractCalldata".to_string(),
            delivery_contract_calldata.to_string(),
        )
    })?
    else {
        return Err(IndexerError::ZkgmExpectingInstructionField(
            "expecting 'path' as String in contractCalldata".to_string(),
            delivery_contract_calldata.to_string(),
        ));
    };

    let delivery_path: alloy_sol_types::private::U256 = delivery_path.parse().map_err(|_| {
        IndexerError::ZkgmExpectingInstructionField(
            "expecting 'instruction' as U256 string in contractCalldata".to_string(),
            delivery_contract_calldata.to_string(),
        )
    })?;

    // delivery instruction
    let Value::String(delivery_instruction) = delivery_contract_calldata
        .get("instruction")
        .ok_or_else(|| {
            IndexerError::ZkgmExpectingInstructionField(
                "missing 'instruction' in contractCalldata".to_string(),
                delivery_contract_calldata.to_string(),
            )
        })?
    else {
        return Err(IndexerError::ZkgmExpectingInstructionField(
            "expecting 'instruction' as String in contractCalldata".to_string(),
            delivery_contract_calldata.to_string(),
        ));
    };
    let delivery_instruction = string_0x_to_bytes(delivery_instruction, "contractCalldata")?;

    let delivery_instruction =
        <crate::indexer::enrich::ucs03_zkgm_0::packet::Instruction>::abi_decode_sequence(
            &delivery_instruction,
        )
        .map_err(|_| {
            IndexerError::ZkgmExpectingInstructionField(
                "cannot parse delivery instruction".to_string(),
                delivery_contract_calldata.to_string(),
            )
        })?;

    // delivery channel details
    let delivery_internal_chain_id = channel.internal_counterparty_chain_id;

    let Some(delivery_channel) =
        get_channel_meta_data(tx, &delivery_internal_chain_id, &delivery_channel_id).await?
    else {
        debug!(
            "no delivery channel details for chain {delivery_internal_chain_id} and channel {delivery_channel_id}"
        );
        return Ok(vec![]);
    };

    // ------------------------------------------------------
    // construct delivery packet to calculate the packet-hash
    // ------------------------------------------------------
    let delivery_zkgm_packet = ZkgmPacket {
        salt: alloy_sol_types::private::FixedBytes::from(delivery_salt),
        path: delivery_path,
        instruction: delivery_instruction,
    };

    let delivery_packet = Packet {
        source_channel_id: ibc_union_spec::ChannelId::try_from(delivery_channel.channel_id.0)
            .map_err(|_| {
                IndexerError::ZkgmExpectingInstructionField(
                    "cannot format source_channel_id".to_string(),
                    delivery_channel.channel_id.to_string(),
                )
            })?,
        destination_channel_id: ibc_union_spec::ChannelId::try_from(
            delivery_channel.counterparty_channel_id.0,
        )
        .map_err(|_| {
            IndexerError::ZkgmExpectingInstructionField(
                "cannot format source_channel_id".to_string(),
                delivery_channel.channel_id.to_string(),
            )
        })?,
        data: <ZkgmPacket>::abi_encode_sequence(&delivery_zkgm_packet).into(),
        timeout_height: ibc_union_spec::MustBeZero, //delivery_timeout_height.into(),
        timeout_timestamp: Timestamp::from_nanos(delivery_timeout_timestamp.0),
    };

    let delivery_packet_hash = crate::indexer::event::types::PacketHash(
        delivery_packet.hash().into_bytes().to_vec().into(),
    );

    // use existing logic to transform zkgm packet to json (originates from postgres plugin)
    // so we can extract the token-order details
    let delivery_zkgm_packet_json = serde_json::to_value(delivery_zkgm_packet).map_err(|_| {
        IndexerError::ZkgmExpectingInstructionField(
            "cannot format instruction".to_string(),
            delivery_contract_calldata.to_string(),
        )
    })?;

    let Value::Array(delivery_zkgm_packet_flattened) = format_flatten(&delivery_zkgm_packet_json)
    else {
        return Err(IndexerError::ZkgmExpectingInstructionField(
            "cannot convert delivery instruction".to_string(),
            delivery_zkgm_packet_json.to_string(),
        ));
    };

    let delivery_token_order =
        InstructionDecoder::from_values_with_index(&delivery_zkgm_packet_flattened, 0)?;

    // sender is sender of token-order in the batch
    let sender_zkgm =
        &AddressZkgm::from_string_0x(token_order.get_string("sender")?, channel.rpc_type)?;
    // receiver is receiver of deliver token-order from the call in the batch
    let receiver_zkgm = &AddressZkgm::from_string_0x(
        token_order.get_string("receiver")?,
        delivery_channel.counterparty_rpc_type,
    )?;
    let base_token = token_order.get_string("baseToken")?.try_into()?;
    let base_amount: Amount = token_order.get_string("baseAmount")?.try_into()?;
    let quote_token = delivery_token_order.get_string("quoteToken")?.try_into()?;
    let quote_amount: Amount = delivery_token_order.get_string("quoteAmount")?.try_into()?;
    let remote_base_token = token_order.get_string("quoteToken")?.try_into()?;
    let remote_base_amount: Amount = token_order.get_string("quoteAmount")?.try_into()?;
    let remote_quote_token = delivery_token_order.get_string("baseToken")?.try_into()?;
    let remote_quote_amount: Amount = delivery_token_order.get_string("baseAmount")?.try_into()?;

    Ok(vec![Bond {
        universal_chain_id: channel.universal_chain_id.clone(),
        source_channel_id: channel.channel_id,
        remote_source_channel_id: channel.counterparty_channel_id,
        remote_destination_channel_id: delivery_channel.channel_id,
        destination_channel_id: delivery_channel.counterparty_channel_id,
        source_client_id: channel.client_id,
        remote_source_client_id: channel.counterparty_client_id,
        remote_destination_client_id: delivery_channel.client_id,
        destination_client_id: delivery_channel.counterparty_client_id,
        source_connection_id: channel.connection_id,
        remote_source_connection_id: channel.counterparty_connection_id,
        remote_destination_connection_id: delivery_channel.connection_id,
        destination_connection_id: delivery_channel.counterparty_connection_id,
        source_port_id: channel.port_id.clone(),
        remote_source_port_id: channel.counterparty_port_id.clone(),
        remote_destination_port_id: delivery_channel.port_id,
        destination_port_id: delivery_channel.counterparty_port_id,
        internal_remote_chain_id: channel.internal_counterparty_chain_id,
        internal_destination_chain_id: delivery_channel.internal_counterparty_chain_id,
        remote_universal_chain_id: channel.universal_counterparty_chain_id.clone(),
        destination_universal_chain_id: delivery_channel.universal_counterparty_chain_id,
        source_network: channel.network,
        remote_network: channel.counterparty_network,
        destination_network: delivery_channel.counterparty_network,

        sender_zkgm: sender_zkgm.clone(),
        sender_canonical: sender_zkgm.try_into()?,
        sender_display: sender_zkgm.try_into()?,
        receiver_zkgm: receiver_zkgm.clone(),
        receiver_canonical: receiver_zkgm.try_into()?,
        receiver_display: receiver_zkgm.try_into()?,
        base_token,
        base_amount,
        quote_token,
        quote_amount,
        remote_base_token,
        remote_base_amount,
        remote_quote_token,
        remote_quote_amount,
        delivery_packet_hash,
        packet_shape,
    }])
}

// ignore unbonds that fail to parse until we have robust packet-shape detection
async fn get_unbonds(
    channel: &ChannelMetaData,
    packet_structure: &str,
    flatten: &[Value],
) -> Result<Vec<Unbond>, IndexerError> {
    Ok(
        match try_get_unbonds(channel, packet_structure, flatten).await {
            Ok(bonds) => bonds,
            Err(error) => {
                let packet = Value::Array(flatten.to_vec());
                warn!("error reading unbond: {error} => {packet}");
                vec![]
            }
        },
    )
}

async fn try_get_unbonds(
    channel: &ChannelMetaData,
    packet_structure: &str,
    flatten: &[Value],
) -> Result<Vec<Unbond>, IndexerError> {
    let Some(packet_shape) = packet_shape(packet_structure, flatten)? else {
        return Ok(vec![]);
    };

    let Some((token_order, _unbond)) = (match packet_shape {
        PacketShape::UnbondV2 => Some((
            InstructionDecoder::from_values_with_index(flatten, 1)?,
            InstructionDecoder::from_values_with_index(flatten, 2)?,
        )),
        _ => None,
    }) else {
        // not an unbond
        return Ok(vec![]);
    };

    let sender_zkgm =
        &AddressZkgm::from_string_0x(token_order.get_string("sender")?, channel.rpc_type)?;
    let receiver_zkgm = &AddressZkgm::from_string_0x(
        token_order.get_string("receiver")?,
        channel.counterparty_rpc_type,
    )?;
    let base_token = token_order.get_string("baseToken")?.try_into()?;
    let base_amount: Amount = token_order.get_string("baseAmount")?.try_into()?;
    let unbond_amount: Amount = token_order.get_string("quoteAmount")?.try_into()?;

    Ok(vec![Unbond {
        sender_zkgm: sender_zkgm.clone(),
        sender_canonical: sender_zkgm.try_into()?,
        sender_display: sender_zkgm.try_into()?,
        receiver_zkgm: receiver_zkgm.clone(),
        receiver_canonical: receiver_zkgm.try_into()?,
        receiver_display: receiver_zkgm.try_into()?,
        base_token,
        base_amount,
        unbond_amount,
        packet_shape,
    }])
}

fn get_instructions(flatten: &[Value]) -> Result<Vec<Instruction>, IndexerError> {
    flatten
        .iter()
        .enumerate()
        .map(|(instruction_index, instruction)| {
            let Value::Object(instruction) = instruction else {
                return Err(IndexerError::ZkgmExpectingInstructionField(
                    "instruction is object".to_string(),
                    instruction.to_string(),
                ));
            };

            let decoder = InstructionDecoder::from_map(instruction)?;

            Ok(Instruction {
                instruction_index: instruction_index.try_into()?,
                instruction_hash: decoder.instruction_hash.clone(),
                instruction_type: decoder.get_string("_type")?.clone().into(),
                path: decoder.root_path.clone(),
                salt: decoder.root_salt.clone(),
                instruction_path: decoder.instruction_path.clone(),
                version: decoder.version,
                opcode: decoder.opcode,
                operand_sender: decoder
                    .get_string_opt("sender")?
                    .map(|s| s.try_into())
                    .transpose()?,
                operand_contract_address: decoder
                    .get_string_opt("contract_address")?
                    .map(|s| s.try_into())
                    .transpose()?,
            })
        })
        .collect()
}

fn calculate_fee(
    fee_data: Option<InstructionDecoder<'_>>,
    base_token: &Denom,
    base_amount: &Amount,
    base_token_name: &Option<TokenName>,
    quote_amount: &Amount,
    wrap_direction: &Option<WrapDirection>,
) -> Result<Fee, IndexerError> {
    let fee = match fee_data {
        Some(fee_data) => Fee::Instruction(
            fee_data.get_string("baseToken")?.try_into()?,
            fee_data.get_string("baseAmount")?.try_into()?,
            fee_data.get_string_opt("baseTokenName")?.map(|s| s.into()),
        ),
        None => match *wrap_direction {
            Some(_) => match &base_amount.cmp(quote_amount) {
                std::cmp::Ordering::Greater => {
                    // fee is the difference between base and quote
                    Fee::QuoteDelta(
                        base_token.clone(),
                        base_amount
                            .clone()
                            .checked_sub(quote_amount.clone())
                            .expect("base-amount is greater than quote amount"),
                        base_token_name.clone(),
                    )
                }
                std::cmp::Ordering::Equal => Fee::None,
                std::cmp::Ordering::Less => Fee::QuoteDeltaNegative,
            },
            None => Fee::Swap,
        },
    };
    Ok(fee)
}

struct InstructionDecoder<'a> {
    root_path: InstructionRootPath,
    root_salt: InstructionRootSalt,
    instruction_path: InstructionPath,
    opcode: InstructionOpcode,
    version: InstructionVersion,
    instruction_hash: InstructionHash,
    operand: &'a Map<String, Value>,
}

impl<'a> InstructionDecoder<'a> {
    fn from_value(value: &'a Value) -> Result<Self, IndexerError> {
        let Value::Object(map) = value else {
            return Err(IndexerError::ZkgmExpectingInstructionField(
                "instruction is Object ({value})".to_string(),
                value.to_string(),
            ));
        };

        Self::from_map(map)
    }

    fn from_map(value: &'a Map<String, Value>) -> Result<Self, IndexerError> {
        let Some(Value::Object(root)) = value.get("_root") else {
            return Err(IndexerError::ZkgmExpectingInstructionField(
                "root in value".to_string(),
                Value::Object(value.clone()).to_string(),
            ));
        };

        let Some(Value::Object(operand)) = value.get("operand") else {
            return Err(IndexerError::ZkgmExpectingInstructionField(
                "operand in value".to_string(),
                Value::Object(value.clone()).to_string(),
            ));
        };

        let root_path = Self::get_string_from(root, "path")?.try_into()?;
        let root_salt = Self::get_string_from(root, "salt")?.try_into()?;
        let instruction_path = Self::get_string_from(value, "_index")?.into();
        let opcode = Self::get_u8_from(value, "opcode")?.into();
        let version = Self::get_u8_from(value, "version")?.into();
        let instruction_hash = Self::get_string_from(value, "_instruction_hash")?.try_into()?;

        Ok(Self {
            root_path,
            root_salt,
            instruction_path,
            opcode,
            version,
            instruction_hash,
            operand,
        })
    }

    fn from_values_with_index(
        flatten: &'a [Value],
        transfer_index: usize,
    ) -> Result<Self, IndexerError> {
        let Some(instruction) = flatten.get(transfer_index) else {
            return Err(IndexerError::ZkgmExpectingInstructionField(
                format!("transfer at index {transfer_index}"),
                flatten.iter().join(", "),
            ));
        };

        Self::from_value(instruction)
    }
}

impl<'a> InstructionDecoder<'a> {
    fn get_string(&'a self, key: &str) -> Result<&'a String, IndexerError> {
        Self::get_string_from(self.operand, key)
    }

    fn get_string_opt(&'a self, key: &str) -> Result<Option<&'a String>, IndexerError> {
        Self::get_string_opt_from(self.operand, key)
    }

    fn get_u32_opt(&'a self, key: &str) -> Result<Option<u32>, IndexerError> {
        Self::get_u32_opt_from(self.operand, key)
    }

    fn get_u8_opt(&'a self, key: &str) -> Result<Option<u8>, IndexerError> {
        Self::get_u8_opt_from(self.operand, key)
    }

    fn get_call_message(&'a self, key: &str) -> Result<Value, IndexerError> {
        Self::get_call_message_from(self.operand, key)
    }

    fn get_string_from(
        from: &'a Map<String, Value>,
        key: &str,
    ) -> Result<&'a String, IndexerError> {
        Ok(match from.get(key) {
            Some(Value::String(value)) => value,
            Some(unsupported) => {
                return Err(IndexerError::ZkgmExpectingInstructionField(
                    format!("{key} field in instruction is string ({unsupported})"),
                    Value::Object(from.clone()).to_string(),
                ));
            }
            None => {
                return Err(IndexerError::ZkgmExpectingInstructionField(
                    format!("{key} field in instruction"),
                    Value::Object(from.clone()).to_string(),
                ));
            }
        })
    }

    fn get_string_opt_from(
        from: &'a Map<String, Value>,
        key: &str,
    ) -> Result<Option<&'a String>, IndexerError> {
        Ok(match from.get(key) {
            Some(Value::String(value)) => Some(value),
            Some(unsupported) => {
                return Err(IndexerError::ZkgmExpectingInstructionField(
                    format!("{key} field in instruction is string ({unsupported})"),
                    Value::Object(from.clone()).to_string(),
                ));
            }
            None => None,
        })
    }

    fn get_u32_opt_from(
        from: &'a Map<String, Value>,
        key: &str,
    ) -> Result<Option<u32>, IndexerError> {
        Ok(match from.get(key) {
            Some(node) => {
                let Value::Number(value) = node else {
                    return Err(IndexerError::ZkgmExpectingInstructionField(
                        format!("{key} field in instruction"),
                        Value::Object(from.clone()).to_string(),
                    ));
                };

                let Some(integer) = value.as_i128() else {
                    return Err(IndexerError::ZkgmExpectingInstructionField(
                        format!("{key} field in instruction is integer"),
                        Value::Object(from.clone()).to_string(),
                    ));
                };

                let Ok(result) = u32::try_from(integer) else {
                    return Err(IndexerError::ZkgmExpectingInstructionField(
                        format!("{key} field in instruction is u32 ({value})"),
                        Value::Object(from.clone()).to_string(),
                    ));
                };

                Some(result)
            }
            None => None,
        })
    }

    fn get_u8_opt_from(
        from: &'a Map<String, Value>,
        key: &str,
    ) -> Result<Option<u8>, IndexerError> {
        Ok(match from.get(key) {
            Some(node) => {
                let Value::Number(value) = node else {
                    return Err(IndexerError::ZkgmExpectingInstructionField(
                        format!("{key} field in instruction"),
                        Value::Object(from.clone()).to_string(),
                    ));
                };

                let Some(integer) = value.as_i128() else {
                    return Err(IndexerError::ZkgmExpectingInstructionField(
                        format!("{key} field in instruction is integer"),
                        Value::Object(from.clone()).to_string(),
                    ));
                };

                let Ok(result) = u8::try_from(integer) else {
                    return Err(IndexerError::ZkgmExpectingInstructionField(
                        format!("{key} field in instruction is u8 ({value})"),
                        Value::Object(from.clone()).to_string(),
                    ));
                };

                Some(result)
            }
            None => None,
        })
    }

    fn get_u8_from(from: &'a Map<String, Value>, key: &str) -> Result<u8, IndexerError> {
        Ok(match from.get(key) {
            Some(node) => {
                let Value::Number(value) = node else {
                    return Err(IndexerError::ZkgmExpectingInstructionField(
                        format!("{key} field in instruction is number ({node})"),
                        Value::Object(from.clone()).to_string(),
                    ));
                };

                let Some(integer) = value.as_i128() else {
                    return Err(IndexerError::ZkgmExpectingInstructionField(
                        format!("{key} field in instruction is integer ({value})"),
                        Value::Object(from.clone()).to_string(),
                    ));
                };

                let Ok(result) = u8::try_from(integer) else {
                    return Err(IndexerError::ZkgmExpectingInstructionField(
                        format!("{key} field in instruction is u8 ({integer})"),
                        Value::Object(from.clone()).to_string(),
                    ));
                };

                result
            }
            None => {
                return Err(IndexerError::ZkgmExpectingInstructionField(
                    format!("{key} field in instruction"),
                    Value::Object(from.clone()).to_string(),
                ));
            }
        })
    }

    // decoded contract call data:
    // - fetch key string attribute from map
    // - convert 0x-string to bytes
    // - convert bytes to json
    // - fetch the 'msg' string attribute from json
    // - decode base58-string to bytes
    // - convert bytes to json
    // - return json
    fn get_call_message_from(
        from: &'a Map<String, Value>,
        key: &str,
    ) -> Result<Value, IndexerError> {
        match from.get(key) {
            Some(Value::String(call_envelope_0x)) => {
                let call_envelope_json = string_0x_to_bytes(call_envelope_0x, key)?;
                let call_envelope_json = bytes_to_value(&call_envelope_json, key)?;

                match call_envelope_json.get("msg") {
                    Some(Value::String(msg_base_64)) => {
                        let msg_json = string_base64_to_bytes(msg_base_64, key)?;
                        let msg_json = bytes_to_value(&msg_json, key)?;

                        Ok(msg_json)
                    }
                    Some(unsupported) => Err(IndexerError::ZkgmExpectingInstructionField(
                        format!("{key}.msg field in instruction is string ({unsupported})"),
                        Value::Object(from.clone()).to_string(),
                    )),
                    None => Err(IndexerError::ZkgmExpectingInstructionField(
                        format!("{key}.msg field in instruction"),
                        Value::Object(from.clone()).to_string(),
                    )),
                }
            }
            Some(unsupported) => Err(IndexerError::ZkgmExpectingInstructionField(
                format!("{key} field in instruction is string ({unsupported})"),
                Value::Object(from.clone()).to_string(),
            )),
            None => Err(IndexerError::ZkgmExpectingInstructionField(
                format!("{key} field in instruction"),
                Value::Object(from.clone()).to_string(),
            )),
        }
    }
}

fn packet_shape(
    packet_structure: &str,
    flatten: &[Value],
) -> Result<Option<PacketShape>, IndexerError> {
    Ok(match packet_structure {
        ":2/0,0:3/0,1:3/0" if has_fee(flatten)? => {
            // batch with two transfers with fee
            Some(PacketShape::BatchV0TransferV0Fee)
        }
        ":3/0" => {
            // one transfer (v0)
            Some(PacketShape::TransferV0)
        }
        ":2/0,0:3/1" => {
            // batch with one transfer (v0)
            Some(PacketShape::BatchV0TransferV1)
        }
        ":2/0,0:3/1,1:3/1" if has_fee(flatten)? => {
            // batch with two transfers (v1) with fee
            Some(PacketShape::BatchV0TransferV1Fee)
        }
        ":3/1" => {
            // one transfer (v1)
            Some(PacketShape::TransferV1)
        }
        ":2/0,0:3/2" => {
            // batch with one transfer (v2)
            Some(PacketShape::BatchV0TransferV2)
        }
        ":2/0,0:3/2,1:3/2" if has_fee(flatten)? => {
            // batch with two transfers (v2) with fee
            Some(PacketShape::BatchV0TransferV2Fee)
        }
        ":3/2" => {
            // one transfer (v2)
            Some(PacketShape::TransferV2)
        }
        ":2/0:3/2,1/0,1/0,1/0" if is_bond(flatten)? => {
            // batch with
            // - token-order to transfer token
            // - call to bond and create lst
            // - call to ensure allowance
            // - call to deliver lst
            Some(PacketShape::BondV2)
        }
        ":2/0:3/2,1/0" if is_unbond(flatten)? => {
            // batch with:
            // - token-order to transfer lst
            // - call to unbond lst
            Some(PacketShape::UnbondV2)
        }
        unsupported => {
            debug!("unsupported packet shape: {unsupported}");

            None
        }
    })
}

/// A batch has a fee if the second instruction of the batch (ie the third instruction, because
/// the first one is the batch) has a zero quote amount
fn has_fee(flatten: &[Value]) -> Result<bool, IndexerError> {
    let Some(Value::Object(instruction)) = flatten.get(2) else {
        return Err(IndexerError::ZkgmExpectingInstructionField(
            "three instructions".to_string(),
            Value::Array(flatten.to_vec()).to_string(),
        ));
    };
    let Some(Value::Object(operand)) = instruction.get("operand") else {
        return Err(IndexerError::ZkgmExpectingInstructionField(
            "operand".to_string(),
            Value::Array(flatten.to_vec()).to_string(),
        ));
    };
    let Some(Value::String(quote_amount_hex_0x)) = operand.get("quoteAmount") else {
        return Err(IndexerError::ZkgmExpectingInstructionField(
            "quote amount".to_string(),
            Value::Array(flatten.to_vec()).to_string(),
        ));
    };
    let quote_amount = string_0x_to_bytes(quote_amount_hex_0x, "quote amount")?;

    let is_zero = quote_amount.iter().all(|&b| b == 0);

    Ok(is_zero)
}

const ON_ZKGM_CALL_PROXY: Bytes = Bytes::from_static(&hex_literal::hex!("00"));

fn is_bond(flatten: &[Value]) -> Result<bool, IndexerError> {
    Ok(is_transfer_to(flatten, 0, &ON_ZKGM_CALL_PROXY)?
        && is_call_to(flatten, 1, &ON_ZKGM_CALL_PROXY)?
        && is_call_to(flatten, 2, &ON_ZKGM_CALL_PROXY)?
        && is_call_to(flatten, 3, &ON_ZKGM_CALL_PROXY)?)
}

fn is_unbond(flatten: &[Value]) -> Result<bool, IndexerError> {
    Ok(is_transfer_to(flatten, 0, &ON_ZKGM_CALL_PROXY)?
        && is_call_to(flatten, 1, &ON_ZKGM_CALL_PROXY)?)
}

// check if the instruction at index is a transfer to the specified address
fn is_transfer_to(
    flatten: &[Value],
    index: usize,
    expected_receiver_address: &Bytes,
) -> Result<bool, IndexerError> {
    let Some(Value::Object(instruction)) = flatten.get(index) else {
        return Err(IndexerError::ZkgmExpectingInstructionField(
            format!("instruction with index {index}"),
            Value::Array(flatten.to_vec()).to_string(),
        ));
    };

    if Some(Value::String("TokenOrder".to_string())) != instruction.get("_type").cloned() {
        return Err(IndexerError::ZkgmExpectingInstructionField(
            format!("token-order instruction with index {index}"),
            Value::Array(flatten.to_vec()).to_string(),
        ));
    }

    let Some(Value::String(contract_address_0x)) = instruction.get("receiver") else {
        return Err(IndexerError::ZkgmExpectingInstructionField(
            format!("receiver in token-order instruction with index {index}"),
            Value::Array(flatten.to_vec()).to_string(),
        ));
    };

    let actual_receiver_address = string_0x_to_bytes(
        contract_address_0x,
        format!("receiver is 0x hex in token-order instruction with index {index}").as_str(),
    )?;

    Ok(actual_receiver_address == expected_receiver_address)
}

// check if the instruction at index is a call to the specified address
fn is_call_to(
    flatten: &[Value],
    index: usize,
    expected_contract_address: &Bytes,
) -> Result<bool, IndexerError> {
    let Some(Value::Object(instruction)) = flatten.get(index) else {
        return Err(IndexerError::ZkgmExpectingInstructionField(
            format!("instruction with index {index}"),
            Value::Array(flatten.to_vec()).to_string(),
        ));
    };

    if Some(Value::String("Call".to_string())) != instruction.get("_type").cloned() {
        return Err(IndexerError::ZkgmExpectingInstructionField(
            format!("call instruction with index {index}"),
            Value::Array(flatten.to_vec()).to_string(),
        ));
    }

    let Some(Value::String(contract_address_0x)) = instruction.get("contractAddress") else {
        return Err(IndexerError::ZkgmExpectingInstructionField(
            format!("contract address in call instruction with index {index}"),
            Value::Array(flatten.to_vec()).to_string(),
        ));
    };

    let actual_contract_address = string_0x_to_bytes(
        contract_address_0x,
        format!("contract address is 0x hex in call instruction with index {index}").as_str(),
    )?;

    Ok(actual_contract_address == expected_contract_address)
}

fn packet_structure(flatten: &[Value]) -> Result<String, IndexerError> {
    flatten
        .iter()
        .map(|instruction| {
            let decoder = InstructionDecoder::from_value(instruction)?;

            let instruction_path = decoder.instruction_path;
            let opcode = decoder.opcode;
            let version = decoder.version;

            Ok(format!("{instruction_path}:{opcode}/{version}"))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|strings| strings.join(","))
}

fn sort_order(
    record: &PacketSendRecord,
    channel: &ChannelMetaData,
) -> Result<String, IndexerError> {
    let timestamp = record
        .timestamp
        .to_offset(UtcOffset::UTC)
        .format(&format_description!(
            "[year][month][day][hour][minute][second]"
        ))
        .map_err(|e| {
            IndexerError::InternalCannotMapFromDatabaseDomain(
                "timestamp".to_string(),
                format!("cannot convert to YYYYMMDDHHMMSS UTC: {e}"),
            )
        })?;

    let packet_hash: crate::indexer::event::types::PacketHash =
        bytes::Bytes::from(record.packet_hash.clone()).into();
    let universal_chain_id = channel.universal_chain_id.clone();

    Ok(format!("{timestamp}-{packet_hash}-{universal_chain_id}"))
}
