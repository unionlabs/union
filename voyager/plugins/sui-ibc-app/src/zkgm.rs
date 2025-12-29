use std::{
    collections::{HashMap, hash_map::Entry},
    str::FromStr,
    time::Duration,
};

use alloy::sol_types::SolValue;
use hex_literal::hex;
use ibc_union_spec::{ChannelId, datagram::MsgPacketAcknowledgement};
use jsonrpsee::tracing::debug;
use move_core_types::{account_address::AccountAddress, ident_str, language_storage::StructTag};
use sha3::{Digest, Keccak256};
use sui_sdk::{
    rpc_types::{ObjectChange, SuiMoveValue, SuiParsedData, SuiTransactionBlockResponseOptions},
    types::{
        Identifier, TypeTag,
        base_types::ObjectRef,
        crypto::SuiKeyPair,
        dynamic_field::DynamicFieldName,
        transaction::{Argument, CallArg, Command, ObjectArg, SharedObjectMutability},
    },
};
use ucs03_zkgm::com::{Batch, TokenMetadata, TokenOrderV2, ZkgmPacket};
use unionlabs::primitives::{Bytes, H256, encoding::HexPrefixed};
use voyager_sdk::serde_json;

use super::*;

pub const SUI_CALL_ARG_CLOCK: CallArg = CallArg::Object(ObjectArg::SharedObject {
    id: ObjectID::from_single_byte(6),
    initial_shared_version: SequenceNumber::from_u64(1),
    mutability: SharedObjectMutability::Immutable,
});

const TOKEN_BYTECODE: [&[u8]; 2] = [
    hex!("a11ceb0b060000000a01000e020e1e032c27045308055b5607b101d1010882036006e2034b0aad04050cb2042b000a010d020602070212021302140001020001020701000003000c01000103030c0100010504020006050700000b000100010c010601000211030400030808090102040e0b01010c040f0e01010c05100c030001050307040a050d02080007080400020b020108000b030108000105010f010805010b01010900010800070900020a020a020a020b01010805070804020b030109000b02010900010b0201080001090001060804010b03010800020900050c436f696e4d657461646174610e46554e4749424c455f544f4b454e064f7074696f6e0b5472656173757279436170095478436f6e746578740355726c076164647265737304636f696e0f6372656174655f63757272656e63790b64756d6d795f6669656c640e66756e6769626c655f746f6b656e04696e6974046e6f6e65066f7074696f6e137075626c69635f73686172655f6f626a6563740f7075626c69635f7472616e736665720673656e64657207746f5f75323536087472616e736665720a74785f636f6e746578740375726c0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020520").as_slice(),
    hex!("0a0205046d756e6f0a021e1d7a6b676d20746f6b656e206372656174656420627920766f796167657200020109010000000002140b00070011023307010701070238000a0138010c020c030b0238020b030b012e110638030200").as_slice()
];

pub fn begin_recv_call(
    ptb: &mut ProgrammableTransactionBuilder,
    module_info: &ModuleInfo,
    data: MsgPacketRecv,
) -> Argument {
    let (source_channels, dest_channels, packet_data, timeout_heights, timeout_timestamps) = data
        .packets
        .iter()
        .map(|p| {
            (
                p.source_channel_id,
                p.destination_channel_id,
                p.data.clone(),
                0u64,
                p.timeout_timestamp,
            )
        })
        .collect::<(Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>)>();

    let arguments = vec![
        CallArg::Pure(bcs::to_bytes(&source_channels).unwrap()),
        CallArg::Pure(bcs::to_bytes(&dest_channels).unwrap()),
        CallArg::Pure(bcs::to_bytes(&packet_data).unwrap()),
        CallArg::Pure(bcs::to_bytes(&timeout_heights).unwrap()),
        CallArg::Pure(bcs::to_bytes(&timeout_timestamps).unwrap()),
    ]
    .into_iter()
    .map(|a| ptb.input(a))
    .collect::<Result<_, _>>()
    .unwrap();

    ptb.command(Command::move_call(
        module_info.latest_address.into(),
        ident_str!("zkgm").into(),
        ident_str!("begin_recv").into(),
        vec![],
        arguments,
    ))
}

#[allow(clippy::too_many_arguments)]
pub fn recv_packet_call(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    module_info: &ModuleInfo,
    owned_vault_object_id: ObjectID,
    owned_vault_store_initial_seq: SequenceNumber,
    escrow_vault_object_id: ObjectID,
    escrow_vault_store_initial_seq: SequenceNumber,
    coin_t: TypeTag,
    fee_recipient: SuiAddress,
    relayer_msgs: Vec<Bytes>,
    session: Argument,
) -> Argument {
    let arguments = vec![
        CallArg::Object(ObjectArg::SharedObject {
            id: module.ibc_store.into(),
            initial_shared_version: module.ibc_store_initial_seq,
            mutability: SharedObjectMutability::Mutable,
        }),
        CallArg::Object(ObjectArg::SharedObject {
            id: module_info.stores[0].0.into(),
            initial_shared_version: module_info.stores[0].1,
            mutability: SharedObjectMutability::Mutable,
        }),
        CallArg::Object(ObjectArg::SharedObject {
            id: owned_vault_object_id,
            initial_shared_version: owned_vault_store_initial_seq,
            mutability: SharedObjectMutability::Mutable,
        }),
        CallArg::Object(ObjectArg::SharedObject {
            id: escrow_vault_object_id,
            initial_shared_version: escrow_vault_store_initial_seq,
            mutability: SharedObjectMutability::Mutable,
        }),
        SUI_CALL_ARG_CLOCK,
        CallArg::Pure(bcs::to_bytes(&fee_recipient).unwrap()),
        CallArg::Pure(bcs::to_bytes(&relayer_msgs).unwrap()),
    ]
    .into_iter()
    .map(|a| ptb.input(a).unwrap())
    .chain(vec![session])
    .collect();

    ptb.command(Command::move_call(
        module_info.latest_address.into(),
        ident_str!("zkgm").into(),
        ident_str!("recv_packet").into(),
        vec![coin_t],
        arguments,
    ))
}

pub fn end_recv_call(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    module_info: &ModuleInfo,
    fee_recipient: SuiAddress,
    session: Argument,
    data: MsgPacketRecv,
) {
    let arguments = vec![
        CallArg::Object(ObjectArg::SharedObject {
            id: module.ibc_store.into(),
            initial_shared_version: module.ibc_store_initial_seq,
            mutability: SharedObjectMutability::Mutable,
        }),
        CallArg::Object(ObjectArg::SharedObject {
            id: module_info.stores[0].0.into(),
            initial_shared_version: module_info.stores[0].1,
            mutability: SharedObjectMutability::Immutable,
        }),
        SUI_CALL_ARG_CLOCK,
        (&data.proof.into_vec()).into(),
        data.proof_height.into(),
        CallArg::Pure(bcs::to_bytes(&fee_recipient).unwrap()),
        CallArg::Pure(bcs::to_bytes(&data.relayer_msgs).unwrap()),
    ]
    .into_iter()
    .map(|a| ptb.input(a).unwrap())
    .chain(vec![session])
    .collect();

    let _ = ptb.command(Command::move_call(
        module_info.latest_address.into(),
        ident_str!("zkgm").into(),
        ident_str!("end_recv").into(),
        vec![],
        arguments,
    ));
}

pub fn begin_ack_call(
    ptb: &mut ProgrammableTransactionBuilder,
    module_info: &ModuleInfo,
    data: MsgPacketAcknowledgement,
) -> Argument {
    let (source_channels, dest_channels, packet_data, timeout_heights, timeout_timestamps) = data
        .packets
        .iter()
        .map(|p| {
            (
                p.source_channel_id,
                p.destination_channel_id,
                p.data.clone(),
                0u64,
                p.timeout_timestamp,
            )
        })
        .collect::<(Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>)>();

    let arguments = vec![
        CallArg::Pure(bcs::to_bytes(&source_channels).unwrap()),
        CallArg::Pure(bcs::to_bytes(&dest_channels).unwrap()),
        CallArg::Pure(bcs::to_bytes(&packet_data).unwrap()),
        CallArg::Pure(bcs::to_bytes(&timeout_heights).unwrap()),
        CallArg::Pure(bcs::to_bytes(&timeout_timestamps).unwrap()),
        CallArg::Pure(bcs::to_bytes(&data.acknowledgements).unwrap()),
    ]
    .into_iter()
    .map(|a| ptb.input(a))
    .collect::<Result<_, _>>()
    .unwrap();

    ptb.command(Command::move_call(
        module_info.latest_address.into(),
        ident_str!("zkgm").into(),
        ident_str!("begin_ack").into(),
        vec![],
        arguments,
    ))
}

#[allow(clippy::too_many_arguments)]
pub fn acknowledge_packet_call(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    module_info: &ModuleInfo,
    owned_vault_object_id: ObjectID,
    owned_vault_store_initial_seq: SequenceNumber,
    escrow_vault_object_id: ObjectID,
    escrow_vault_store_initial_seq: SequenceNumber,
    coin_t: TypeTag,
    fee_recipient: SuiAddress,
    session: Argument,
) -> Argument {
    let arguments = vec![
        CallArg::Object(ObjectArg::SharedObject {
            id: module.ibc_store.into(),
            initial_shared_version: module.ibc_store_initial_seq,
            mutability: SharedObjectMutability::Mutable,
        }),
        CallArg::Object(ObjectArg::SharedObject {
            id: module_info.stores[0].0.into(),
            initial_shared_version: module_info.stores[0].1,
            mutability: SharedObjectMutability::Mutable,
        }),
        CallArg::Object(ObjectArg::SharedObject {
            id: owned_vault_object_id,
            initial_shared_version: owned_vault_store_initial_seq,
            mutability: SharedObjectMutability::Mutable,
        }),
        CallArg::Object(ObjectArg::SharedObject {
            id: escrow_vault_object_id,
            initial_shared_version: escrow_vault_store_initial_seq,
            mutability: SharedObjectMutability::Mutable,
        }),
        CallArg::Pure(bcs::to_bytes(&fee_recipient).unwrap()),
    ]
    .into_iter()
    .map(|a| ptb.input(a).unwrap())
    .chain(vec![session])
    .collect();

    ptb.command(Command::move_call(
        module_info.latest_address.into(),
        ident_str!("zkgm").into(),
        ident_str!("acknowledge_packet").into(),
        vec![coin_t],
        arguments,
    ))
}

pub fn end_ack_call(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    module_info: &ModuleInfo,
    fee_recipient: SuiAddress,
    session: Argument,
    data: MsgPacketAcknowledgement,
) {
    let arguments = vec![
        CallArg::Object(ObjectArg::SharedObject {
            id: module.ibc_store.into(),
            initial_shared_version: module.ibc_store_initial_seq,
            mutability: SharedObjectMutability::Mutable,
        }),
        CallArg::Object(ObjectArg::SharedObject {
            id: module_info.stores[0].0.into(),
            initial_shared_version: module_info.stores[0].1,
            mutability: SharedObjectMutability::Immutable,
        }),
        (&data.proof.into_vec()).into(),
        data.proof_height.into(),
        CallArg::Pure(bcs::to_bytes(&fee_recipient).unwrap()),
    ]
    .into_iter()
    .map(|a| ptb.input(a).unwrap())
    .chain(vec![session])
    .collect();

    let _ = ptb.command(Command::move_call(
        module_info.latest_address.into(),
        ident_str!("zkgm").into(),
        ident_str!("end_ack").into(),
        vec![],
        arguments,
    ));
}

pub fn begin_timeout_call(
    ptb: &mut ProgrammableTransactionBuilder,
    module_info: &ModuleInfo,
    data: MsgPacketTimeout,
) -> Argument {
    let arguments = vec![
        CallArg::Pure(bcs::to_bytes(&data.packet.source_channel_id).unwrap()),
        CallArg::Pure(bcs::to_bytes(&data.packet.destination_channel_id).unwrap()),
        CallArg::Pure(bcs::to_bytes(&data.packet.data).unwrap()),
        CallArg::Pure(bcs::to_bytes(&data.packet.timeout_height).unwrap()),
        CallArg::Pure(bcs::to_bytes(&data.packet.timeout_timestamp).unwrap()),
    ]
    .into_iter()
    .map(|a| ptb.input(a))
    .collect::<Result<_, _>>()
    .unwrap();

    ptb.command(Command::move_call(
        module_info.latest_address.into(),
        ident_str!("zkgm").into(),
        ident_str!("begin_timeout").into(),
        vec![],
        arguments,
    ))
}

#[allow(clippy::too_many_arguments)]
pub fn timeout_packet_call(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    module_info: &ModuleInfo,
    vault_object_id: ObjectID,
    vault_store_initial_seq: SequenceNumber,
    coin_t: TypeTag,
    session: Argument,
) -> Argument {
    let arguments = vec![
        CallArg::Object(ObjectArg::SharedObject {
            id: module.ibc_store.into(),
            initial_shared_version: module.ibc_store_initial_seq,
            mutability: SharedObjectMutability::Mutable,
        }),
        CallArg::Object(ObjectArg::SharedObject {
            id: module_info.stores[0].0.into(),
            initial_shared_version: module_info.stores[0].1,
            mutability: SharedObjectMutability::Mutable,
        }),
        CallArg::Object(ObjectArg::SharedObject {
            id: vault_object_id,
            initial_shared_version: vault_store_initial_seq,
            mutability: SharedObjectMutability::Mutable,
        }),
    ]
    .into_iter()
    .map(|a| ptb.input(a).unwrap())
    .chain(vec![session])
    .collect();

    ptb.command(Command::move_call(
        module_info.latest_address.into(),
        ident_str!("zkgm").into(),
        ident_str!("timeout_packet").into(),
        vec![coin_t],
        arguments,
    ))
}

pub fn end_timeout_call(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    module_info: &ModuleInfo,
    session: Argument,
    data: MsgPacketTimeout,
) {
    let arguments = vec![
        CallArg::Object(ObjectArg::SharedObject {
            id: module.ibc_store.into(),
            initial_shared_version: module.ibc_store_initial_seq,
            mutability: SharedObjectMutability::Mutable,
        }),
        CallArg::Object(ObjectArg::SharedObject {
            id: module_info.stores[0].0.into(),
            initial_shared_version: module_info.stores[0].1,
            mutability: SharedObjectMutability::Immutable,
        }),
        (&data.proof.into_vec()).into(),
        data.proof_height.into(),
    ]
    .into_iter()
    .map(|a| ptb.input(a).unwrap())
    .chain(vec![session])
    .collect();

    let _ = ptb.command(Command::move_call(
        module_info.latest_address.into(),
        ident_str!("zkgm").into(),
        ident_str!("end_timeout").into(),
        vec![],
        arguments,
    ));
}

pub fn register_capability_call(
    ptb: &mut ProgrammableTransactionBuilder,
    vault_address: SuiAddress,
    vault_object_id: ObjectID,
    initial_seq: SequenceNumber,
    treasury_ref: ObjectRef,
    metadata_ref: ObjectRef,
    coin_t: TypeTag,
) {
    let arguments = [
        ptb.input(CallArg::Object(ObjectArg::SharedObject {
            id: vault_object_id,
            initial_shared_version: initial_seq,
            mutability: SharedObjectMutability::Mutable,
        }))
        .unwrap(),
        ptb.input(CallArg::Object(ObjectArg::ImmOrOwnedObject(treasury_ref)))
            .unwrap(),
        ptb.input(CallArg::Object(ObjectArg::SharedObject {
            id: metadata_ref.0,
            initial_shared_version: metadata_ref.1,
            mutability: SharedObjectMutability::Mutable,
        }))
        .unwrap(),
        // owner is 0x0
        ptb.input(CallArg::Pure(
            H256::<HexPrefixed>::default().into_bytes().to_vec(),
        ))
        .unwrap(),
    ];
    ptb.command(Command::move_call(
        vault_address.into(),
        ident_str!("owned_vault").into(),
        ident_str!("register_capability").into(),
        vec![coin_t.clone()],
        arguments.to_vec(),
    ));
}

pub async fn publish_new_coin(
    module: &Module,
    pk: &SuiKeyPair,
    decimals: u8,
) -> RpcResult<(ObjectRef, ObjectRef, TypeTag)> {
    // There is no wrapped token
    let mut bytecode = TOKEN_BYTECODE[0].to_vec();
    // 31 because it will be followed by a u8 (decimals)
    bytecode.extend_from_slice(&[0; 31]);
    bytecode.extend_from_slice(&decimals.to_be_bytes());
    bytecode.extend_from_slice(TOKEN_BYTECODE[1]);

    let mut ptb = ProgrammableTransactionBuilder::new();

    let res = ptb.command(Command::Publish(
        vec![bytecode],
        vec![
            ObjectID::from_str("0x1").unwrap(),
            ObjectID::from_str("0x2").unwrap(),
        ],
    ));

    let arg = ptb
        .input(CallArg::Pure(
            bcs::to_bytes(&SuiAddress::from(&pk.public())).unwrap(),
        ))
        .unwrap();
    let _ = ptb.command(Command::TransferObjects(vec![res], arg));

    let transaction_response =
        voyager_transaction_plugin_sui::send_transactions(&module.sui_client, pk, ptb.finish())
            .await
            .unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;
    let object_changes = module
        .sui_client
        .read_api()
        .get_transaction_with_options(
            transaction_response.digest,
            SuiTransactionBlockResponseOptions::new().with_object_changes(),
        )
        .await
        .unwrap()
        .object_changes
        .unwrap();
    let (treasury_ref, coin_t) = object_changes
        .iter()
        .find_map(|o| match &o {
            ObjectChange::Created {
                object_type: StructTag {
                    name, type_params, ..
                },
                ..
            } => {
                if name.as_ident_str() == ident_str!("TreasuryCap") {
                    Some((o.object_ref(), type_params[0].clone()))
                } else {
                    None
                }
            }
            _ => None,
        })
        .unwrap();

    let metadata_ref = object_changes
        .iter()
        .find_map(|o| match &o {
            ObjectChange::Created {
                object_type: StructTag { name, .. },
                ..
            } => {
                if name.as_ident_str() == ident_str!("CoinMetadata") {
                    Some(o.object_ref())
                } else {
                    None
                }
            }
            _ => None,
        })
        .unwrap();

    Ok((treasury_ref, metadata_ref, coin_t))
}

pub async fn get_registered_wrapped_token(
    module: &Module,
    wrapped_token: &[u8],
) -> RpcResult<Option<TypeTag>> {
    if let Some(wrapped_token_t) = module
        .sui_client
        .read_api()
        .get_dynamic_field_object(
            module.zkgm_config.wrapped_token_to_t,
            DynamicFieldName {
                type_: TypeTag::Vector(Box::new(TypeTag::U8)),
                value: serde_json::to_value(wrapped_token).expect("serde will work"),
            },
        )
        .await
        .map_err(RpcError::fatal(
            "wrapped_token_to_t is expected to return some data",
        ))?
        .data
    {
        match wrapped_token_t.content.expect("content always exists") {
            SuiParsedData::MoveObject(object) => {
                let SuiMoveValue::String(field_value) = object
                    .fields
                    .field_value("value")
                    .expect("token has a `value` field")
                else {
                    panic!("token must have the type `String`, this voyager might be outdated");
                };

                debug!("the token is already registered");

                let fields: Vec<&str> = field_value.split("::").collect();
                if fields.len() != 3 {
                    panic!(
                        "a registered token must be always in `address::module_name::name` form"
                    );
                }

                Ok(Some(
                    StructTag {
                        address: AccountAddress::from_str(fields[0]).expect("address is valid"),
                        module: Identifier::new(fields[1]).expect("module name is valid"),
                        name: Identifier::new(fields[2]).expect("name is valid"),
                        type_params: vec![],
                    }
                    .into(),
                ))
            }
            SuiParsedData::Package(_) => panic!("this should never be a package"),
        }
    } else {
        Ok(None)
    }
}

pub fn parse_coin_ts(packet_data: Vec<Bytes>) -> RpcResult<Vec<TypeTag>> {
    let parse_type_tag = |base_token: Vec<u8>| -> RpcResult<TypeTag> {
        let quote_token = String::from_utf8(base_token).map_err(RpcError::fatal(
            "in the unwrap case, the quote token must be a utf8 string",
        ))?;
        let fields: Vec<&str> = quote_token.split("::").collect();
        if fields.len() != 3 {
            panic!("a registered token must be always in `address::module_name::name` form");
        }

        Ok(StructTag {
            address: AccountAddress::from_str(fields[0]).expect("address is valid"),
            module: Identifier::new(fields[1]).expect("module name is valid"),
            name: Identifier::new(fields[2]).expect("name is valid"),
            type_params: vec![],
        }
        .into())
    };

    Ok(packet_data
        .into_iter()
        .map(|d| {
            let zkgm_packet = ZkgmPacket::abi_decode_params(&d)
                .map_err(RpcError::fatal("error decoding zkgm packet"))?;
            let mut coin_ts = vec![];
            match zkgm_packet.instruction.opcode {
                OP_BATCH => {
                    let Ok(batch) = Batch::abi_decode_params(&zkgm_packet.instruction.operand)
                    else {
                        panic!("impossible");
                    };

                    for instr in batch.instructions {
                        let Ok(fao) = TokenOrderV2::abi_decode_params(&instr.operand) else {
                            continue;
                        };

                        coin_ts.push(parse_type_tag(fao.base_token.clone().into())?);
                    }
                }
                OP_TOKEN_ORDER => {
                    let Ok(fao) = TokenOrderV2::abi_decode_params(&zkgm_packet.instruction.operand)
                    else {
                        panic!("impossible");
                    };

                    coin_ts.push(parse_type_tag(fao.base_token.clone().into())?);
                }
                _ => {}
            }

            Ok::<_, RpcError>(coin_ts)
        })
        .collect::<Result<Vec<Vec<_>>, _>>()?
        .into_iter()
        .flatten()
        .collect())
}

#[derive(Deserialize)]
struct SuiFungibleAssetMetadata {
    name: String,
    symbol: String,
    decimals: u8,
    owner: H256,
    icon_url: Option<String>,
    description: String,
}

#[allow(clippy::too_many_arguments)]
/// Deploy and register the token if needed in `ZKGM`
pub async fn register_token_if_zkgm(
    module: &Module,
    ptb: &mut ProgrammableTransactionBuilder,
    pk: &SuiKeyPair,
    packet: &ibc_union_spec::Packet,
    zkgm_packet: &ZkgmPacket,
    token_order: TokenOrderV2,
) -> RpcResult<Option<TypeTag>> {
    let (metadata_image, coin_metadata) = match token_order.kind {
        TOKEN_ORDER_KIND_INITIALIZE => {
            // TODO(aeryz): we could drop this packet as well since we know that its gonna fail
            let Ok(metadata) = TokenMetadata::abi_decode_params(&token_order.metadata) else {
                return Ok(None);
            };

            // TODO(aeryz): we can also drop here
            let sui_metadata: SuiFungibleAssetMetadata =
                bcs::from_bytes(&metadata.initializer).unwrap();

            if sui_metadata.owner != H256::<HexPrefixed>::default() {
                return Ok(None);
            }

            (
                Keccak256::new()
                    .chain_update(&token_order.metadata)
                    .finalize()
                    .to_vec(),
                Some(sui_metadata),
            )
        }
        TOKEN_ORDER_KIND_ESCROW => {
            if token_order.metadata.len() != 32 {
                return Err(RpcError::fatal_from_message(format!(
                    "invalid metadata, expected 32 bytes but found {}",
                    token_order.metadata
                )));
            }

            let wrapped_token = predict_wrapped_denom(
                zkgm_packet.path.to_le_bytes().into(),
                packet.destination_channel_id,
                token_order.base_token.to_vec(),
                token_order.metadata.into(),
            );

            // A wrapped token is only registered once, and once it's being received in the SUI side.
            // `wrapped_token` is set to the given coin type. If there's already a coin type with this
            // `wrapped_token`, we have to use that.
            if let Some(wrapped_token_t) =
                get_registered_wrapped_token(module, &wrapped_token).await?
            {
                return Ok(Some(wrapped_token_t));
            } else {
                return Err(RpcError::fatal_from_message(
                    "a token cannot be received for the first time with `ESCROW`, it must be received with `INITIALIZE` first",
                ));
            }
        }
        // If it's an unescrow case, it means that this token is previously sent, so it's already been saved in ZKGM, so we can just parse
        // the quote token as a type tag.
        // If it's  a solve case, we expect the token to be registered previously by a third party. And we can again just parse the quote
        // token as a type tag.
        TOKEN_ORDER_KIND_UNESCROW | TOKEN_ORDER_KIND_SOLVE => {
            // This means the transfer is an unwrap. Hence the `quote_token` must already be in the form `address::module::name`
            // which defines the coin type `T`.
            let quote_token = String::from_utf8(token_order.quote_token.into()).map_err(
                RpcError::fatal("in the unwrap case, the quote token must be a utf8 string"),
            )?;
            let fields: Vec<&str> = quote_token.split("::").collect();
            if fields.len() != 3 {
                return Err(RpcError::fatal_from_message(
                    "a registered token must be always in `address::module_name::name` form",
                ));
            }

            return Ok(Some(
                StructTag {
                    address: AccountAddress::from_str(fields[0]).expect("address is valid"),
                    module: Identifier::new(fields[1]).expect("module name is valid"),
                    name: Identifier::new(fields[2]).expect("name is valid"),
                    type_params: vec![],
                }
                .into(),
            ));
        }
        _ => panic!("tf?"),
    };

    let wrapped_token = predict_wrapped_denom(
        zkgm_packet.path.to_le_bytes().into(),
        packet.destination_channel_id,
        token_order.base_token.to_vec(),
        metadata_image,
    );

    // A token might still be received with `INITIALIZE` although it's already registered. So, we do this
    // additional check and do an early return if we find a registered token.
    if let Some(wrapped_token_t) = get_registered_wrapped_token(module, &wrapped_token).await? {
        return Ok(Some(wrapped_token_t));
    }

    let Some(coin_metadata) = coin_metadata else {
        return Err(RpcError::fatal_from_message(
            "the coin is going to be received for the first time, so the metadata must be provided",
        ));
    };
    let (treasury_ref, metadata_ref, coin_t) =
        publish_new_coin(module, pk, coin_metadata.decimals).await?;

    // let treasury_ref = module
    //     .sui_client
    //     .read_api()
    //     .get_object_with_options(
    //         ObjectID::from_str(
    //             "0x9053c2370c0c751cea3f937339c0ee429dadca72220b51186b6f23fab6d4b2eb",
    //         )
    //         .unwrap(),
    //         SuiObjectDataOptions::default().with_owner(),
    //     )
    //     .await
    //     .map_err(|e| ErrorObject::owned(-1, ErrorReporter(e).to_string(), None::<()>))?
    //     .data
    //     .expect("ibc store object exists on chain")
    //     .object_ref();

    // let metadata_ref = module
    //     .sui_client
    //     .read_api()
    //     .get_object_with_options(
    //         ObjectID::from_str(
    //             "0x3df31bdfc6452c2e72cfc0e3d9ac0dfd92ee375e4f294504e89e2c539847f9b4",
    //         )
    //         .unwrap(),
    //         SuiObjectDataOptions::default().with_owner(),
    //     )
    //     .await
    //     .map_err(|e| ErrorObject::owned(-1, ErrorReporter(e).to_string(), None::<()>))?
    //     .data
    //     .expect("ibc store object exists on chain")
    //     .object_ref();

    // let coin_t =
    //         TypeTag::from_str("0x7ebeecc2437ea672e74b1c554e087a15d8e1f502e20df62f23f1159455c596dc::fungible_token::FUNGIBLE_TOKEN").unwrap();

    // updating name, symbol, icon_url and the description since we don't have these in the published binary right now
    // TODO(aeryz): we should generate the move binary to contain the necessary data and don't do these calls
    coin::update_name(
        ptb,
        treasury_ref,
        metadata_ref,
        coin_t.clone(),
        coin_metadata.name,
    )
    .await;

    coin::update_symbol(
        ptb,
        treasury_ref,
        metadata_ref,
        coin_t.clone(),
        coin_metadata.symbol,
    )
    .await;

    coin::update_description(
        ptb,
        treasury_ref,
        metadata_ref,
        coin_t.clone(),
        coin_metadata.description,
    )
    .await;

    if let Some(icon_url) = coin_metadata.icon_url {
        coin::update_icon_url(ptb, treasury_ref, metadata_ref, coin_t.clone(), icon_url).await;
    }

    let owned_vault_store_initial_seq = module
        .get_initial_seq(module.zkgm_config.owned_vault_object_id)
        .await;

    // We are finally registering the token before calling the recv
    zkgm::register_capability_call(
        ptb,
        module.zkgm_config.owned_vault_package_id,
        module.zkgm_config.owned_vault_object_id,
        owned_vault_store_initial_seq,
        treasury_ref,
        metadata_ref,
        coin_t.clone(),
    );

    Ok(Some(coin_t))
}

pub async fn register_tokens_if_zkgm(
    module: &Module,
    ptb: &mut ProgrammableTransactionBuilder,
    pk: &SuiKeyPair,
    packet: &ibc_union_spec::Packet,
) -> RpcResult<Vec<TypeTag>> {
    let Ok(zkgm_packet) = ZkgmPacket::abi_decode_params(&packet.data) else {
        return Ok(vec![]);
    };

    let mut coin_ts = vec![];

    match zkgm_packet.instruction.opcode {
        OP_BATCH => {
            let Ok(batch) = Batch::abi_decode_params(&zkgm_packet.instruction.operand) else {
                panic!("impossible");
            };

            let mut base_tokens: HashMap<
                (alloy::primitives::Bytes, alloy::primitives::Bytes),
                TypeTag,
            > = HashMap::new();

            for instr in batch.instructions {
                let Ok(fao) = TokenOrderV2::abi_decode_params(&instr.operand) else {
                    continue;
                };

                match base_tokens.entry((fao.base_token.clone(), fao.metadata.clone())) {
                    Entry::Occupied(e) => {
                        coin_ts.push(e.get().clone());
                    }
                    Entry::Vacant(e) => {
                        if let Some(type_tag) =
                            register_token_if_zkgm(module, ptb, pk, packet, &zkgm_packet, fao)
                                .await?
                        {
                            coin_ts.push(type_tag.clone());
                            e.insert(type_tag);
                        }
                    }
                }
            }
        }
        OP_TOKEN_ORDER => {
            let fao = TokenOrderV2::abi_decode_params(&zkgm_packet.instruction.operand)
                .expect("impossible");
            if let Some(type_tag) =
                register_token_if_zkgm(module, ptb, pk, packet, &zkgm_packet, fao).await?
            {
                coin_ts.push(type_tag);
            }
        }
        _ => {}
    }

    Ok(coin_ts)
}

fn predict_wrapped_denom(
    path: H256,
    channel: ChannelId,
    base_token: Vec<u8>,
    metadata_image: Vec<u8>,
) -> Vec<u8> {
    let mut buf = vec![];
    bcs::serialize_into(&mut buf, &path).expect("works");
    bcs::serialize_into(&mut buf, &channel.raw()).expect("works");
    buf.extend_from_slice(&base_token);
    buf.extend_from_slice(&metadata_image);

    Keccak256::new().chain_update(buf).finalize().to_vec()
}
