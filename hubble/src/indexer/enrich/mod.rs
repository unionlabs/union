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
        ucs03_zkgm_0::{packet_ack::decode, PacketHash},
        wrapping::{wrap_direction_chains, IntermediateChannelIds},
    },
    event::types::{BlockHeight, ChannelId, UniversalChainId},
    handler::types::{
        string_0x_to_bytes, AddressCanonical, AddressZkgm, Amount, ChannelMetaData, Fee,
        Instruction, InstructionHash, InstructionOpcode, InstructionPath, InstructionRootPath,
        InstructionRootSalt, InstructionVersion, Metadata, PacketShape, TokenOrderKind, Transfer,
    },
    postgres::chain_context::fetch_chain_context_for_universal_chain_id,
    record::{
        change_counter::Changes, channel_meta_data::get_channel_meta_data,
        create_wrapped_token_record::CreateWrappedTokenRecord,
        create_wrapped_token_relation_record::CreateWrappedTokenRelationRecord,
        packet_send_decoded_record::PacketSendDecodedRecord,
        packet_send_instructions_search_record::PacketSendInstructionsSearchRecord,
        packet_send_record::PacketSendRecord,
        packet_send_transfers_record::PacketSendTransfersRecord, InternalChainId,
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
    };

    let transfer_index = 0;
    let sender_zkgm = &AddressZkgm::from_string_0x(
        transfer_data.get_string("sender")?,
        channel.rpc_type.clone(),
    )?;
    let receiver_zkgm = &AddressZkgm::from_string_0x(
        transfer_data.get_string("receiver")?,
        channel.counterparty_rpc_type.clone(),
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
