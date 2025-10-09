use ibc_union_spec::{
    ChannelId,
    datagram::{
        MsgChannelOpenAck, MsgChannelOpenConfirm, MsgChannelOpenInit, MsgChannelOpenTry,
        MsgConnectionOpenAck, MsgConnectionOpenConfirm, MsgConnectionOpenInit,
        MsgConnectionOpenTry, MsgCreateClient, MsgPacketRecv, MsgUpdateClient,
    },
};
use move_core_types_sui::{ident_str, identifier::IdentStr};
use sui_sdk::{
    SuiClient,
    rpc_types::{SuiObjectDataOptions, SuiTypeTag},
    types::{
        Identifier, TypeTag,
        base_types::{ObjectID, ObjectRef, SequenceNumber, SuiAddress},
        programmable_transaction_builder::ProgrammableTransactionBuilder,
        transaction::{Argument, CallArg, Command, ObjectArg, TransactionKind},
    },
};
use unionlabs::primitives::{Bytes, H256, encoding::HexPrefixed};
use voyager_sdk::anyhow;

use crate::{Module, ModuleInfo};

pub const SUI_CALL_ARG_CLOCK: CallArg = CallArg::Object(ObjectArg::SharedObject {
    id: ObjectID::from_single_byte(6),
    initial_shared_version: SequenceNumber::from_u64(1),
    mutable: false,
});

pub const IBC_IDENT: &IdentStr = ident_str!("ibc");

pub struct SuiQuery<'a> {
    client: &'a SuiClient,
    params: Vec<CallArg>,
}

impl<'a> SuiQuery<'a> {
    pub async fn new(client: &'a SuiClient, ibc_store_id: ObjectID) -> Self {
        let object_ref = client
            .read_api()
            .get_object_with_options(ibc_store_id, SuiObjectDataOptions::new())
            .await
            .unwrap()
            .object_ref_if_exists()
            .unwrap();
        Self {
            client,
            params: vec![CallArg::Object(ObjectArg::ImmOrOwnedObject(object_ref))],
        }
    }

    pub fn add_param<T>(mut self, param: T) -> Self
    where
        T: serde::Serialize,
    {
        self.params
            .push(CallArg::Pure(bcs::to_bytes(&param).unwrap()));
        self
    }

    pub async fn call(
        self,
        package: ObjectID,
        function: &str,
    ) -> Result<Vec<(Vec<u8>, SuiTypeTag)>, String> {
        let mut ptb = ProgrammableTransactionBuilder::new();
        ptb.command(Command::move_call(
            package,
            IBC_IDENT.into(),
            Identifier::new(function).unwrap(),
            vec![],
            self.params
                .iter()
                .enumerate()
                .map(|(i, _)| Argument::Input(i as u16))
                .collect(),
        ));

        for arg in self.params {
            ptb.input(arg).unwrap();
        }

        let res = self
            .client
            .read_api()
            .dev_inspect_transaction_block(
                SuiAddress::ZERO,
                TransactionKind::ProgrammableTransaction(ptb.finish()),
                None,
                None,
                None,
            )
            .await
            .unwrap();

        match (res.results, res.error) {
            (Some(res), _) => Ok(res[0].clone().return_values),
            (_, Some(err)) => Err(err),
            _ => panic!("invalid"),
        }
    }
}

pub fn create_client(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    data: MsgCreateClient,
) -> anyhow::Result<()> {
    ptb.move_call(
        module.ibc_handler_address.into(),
        IBC_IDENT.into(),
        ident_str!("create_client").into(),
        vec![],
        vec![
            CallArg::Object(ObjectArg::SharedObject {
                id: module.ibc_store.into(),
                initial_shared_version: module.ibc_store_initial_seq,
                mutable: true,
            }),
            CallArg::Pure(bcs::to_bytes(&data.client_type.to_string()).unwrap()),
            (&data.client_state_bytes.into_vec()).into(),
            (&data.consensus_state_bytes.into_vec()).into(),
        ],
    )
}

pub fn update_client(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    data: MsgUpdateClient,
) -> anyhow::Result<()> {
    ptb.move_call(
        module.ibc_handler_address.into(),
        IBC_IDENT.into(),
        ident_str!("update_client").into(),
        vec![],
        vec![
            CallArg::Object(ObjectArg::SharedObject {
                id: module.ibc_store.into(),
                initial_shared_version: module.ibc_store_initial_seq,
                mutable: true,
            }),
            SUI_CALL_ARG_CLOCK.clone(),
            data.client_id.raw().into(),
            (&data.client_message.into_vec()).into(),
            CallArg::Pure(H256::<HexPrefixed>::default().into_bytes().to_vec()),
        ],
    )
}

pub fn connection_open_init(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    data: MsgConnectionOpenInit,
) -> anyhow::Result<()> {
    ptb.move_call(
        module.ibc_handler_address.into(),
        IBC_IDENT.into(),
        ident_str!("connection_open_init").into(),
        vec![],
        vec![
            CallArg::Object(ObjectArg::SharedObject {
                id: module.ibc_store.into(),
                initial_shared_version: module.ibc_store_initial_seq,
                mutable: true,
            }),
            data.client_id.raw().into(),
            data.counterparty_client_id.raw().into(),
        ],
    )
}

pub fn connection_open_try(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    data: MsgConnectionOpenTry,
) -> anyhow::Result<()> {
    ptb.move_call(
        module.ibc_handler_address.into(),
        IBC_IDENT.into(),
        ident_str!("connection_open_try").into(),
        vec![],
        vec![
            CallArg::Object(ObjectArg::SharedObject {
                id: module.ibc_store.into(),
                initial_shared_version: module.ibc_store_initial_seq,
                mutable: true,
            }),
            data.counterparty_client_id.raw().into(),
            data.counterparty_connection_id.raw().into(),
            data.client_id.raw().into(),
            (&data.proof_init.into_vec()).into(),
            data.proof_height.into(),
        ],
    )
}

pub fn connection_open_ack(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    data: MsgConnectionOpenAck,
) -> anyhow::Result<()> {
    ptb.move_call(
        module.ibc_handler_address.into(),
        IBC_IDENT.into(),
        ident_str!("connection_open_ack").into(),
        vec![],
        vec![
            CallArg::Object(ObjectArg::SharedObject {
                id: module.ibc_store.into(),
                initial_shared_version: module.ibc_store_initial_seq,
                mutable: true,
            }),
            data.connection_id.raw().into(),
            data.counterparty_connection_id.raw().into(),
            (&data.proof_try.into_vec()).into(),
            data.proof_height.into(),
        ],
    )
}

pub fn connection_open_confirm(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    data: MsgConnectionOpenConfirm,
) -> anyhow::Result<()> {
    ptb.move_call(
        module.ibc_handler_address.into(),
        IBC_IDENT.into(),
        ident_str!("connection_open_confirm").into(),
        vec![],
        vec![
            CallArg::Object(ObjectArg::SharedObject {
                id: module.ibc_store.into(),
                initial_shared_version: module.ibc_store_initial_seq,
                mutable: true,
            }),
            data.connection_id.raw().into(),
            (&data.proof_ack.into_vec()).into(),
            data.proof_height.into(),
        ],
    )
}

pub fn channel_open_init(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    module_info: ModuleInfo,
    data: MsgChannelOpenInit,
) -> anyhow::Result<()> {
    ptb.move_call(
        module_info.latest_address.into(),
        module_info.module_name,
        ident_str!("channel_open_init").into(),
        vec![],
        vec![
            CallArg::Object(ObjectArg::SharedObject {
                id: module.ibc_store.into(),
                initial_shared_version: module.ibc_store_initial_seq,
                mutable: true,
            }),
            (&data.port_id.into_vec()).into(),
            (&data.counterparty_port_id.into_vec()).into(),
            data.connection_id.raw().into(),
            (&data.version.into_bytes()).into(),
        ],
    )
}

pub fn channel_open_try(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    module_info: ModuleInfo,
    data: MsgChannelOpenTry,
) -> anyhow::Result<()> {
    ptb.move_call(
        module_info.latest_address.into(),
        module_info.module_name,
        ident_str!("channel_open_try").into(),
        vec![],
        vec![
            CallArg::Object(ObjectArg::SharedObject {
                id: module.ibc_store.into(),
                initial_shared_version: module.ibc_store_initial_seq,
                mutable: true,
            }),
            (&data.port_id.into_vec()).into(),
            data.channel.connection_id.raw().into(),
            CallArg::Pure(bcs::to_bytes(&data.channel.counterparty_channel_id).unwrap()),
            (&data.channel.counterparty_port_id.into_vec()).into(),
            (&data.channel.version.into_bytes()).into(),
            (&data.counterparty_version.into_bytes()).into(),
            (&data.proof_init.into_vec()).into(),
            data.proof_height.into(),
        ],
    )
}

pub fn channel_open_ack(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    module_info: ModuleInfo,
    port_id: String,
    data: MsgChannelOpenAck,
) -> anyhow::Result<()> {
    ptb.move_call(
        module_info.latest_address.into(),
        module_info.module_name,
        ident_str!("channel_open_ack").into(),
        vec![],
        vec![
            CallArg::Object(ObjectArg::SharedObject {
                id: module.ibc_store.into(),
                initial_shared_version: module.ibc_store_initial_seq,
                mutable: true,
            }),
            (&port_id.into_bytes()).into(),
            data.channel_id.raw().into(),
            (&data.counterparty_version.into_bytes()).into(),
            data.counterparty_channel_id.raw().into(),
            (&data.proof_try.into_vec()).into(),
            data.proof_height.into(),
        ],
    )
}

pub fn channel_open_confirm_call(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    module_info: ModuleInfo,
    data: MsgChannelOpenConfirm,
) -> anyhow::Result<()> {
    ptb.move_call(
        module_info.latest_address.into(),
        module_info.module_name,
        ident_str!("channel_open_confirm").into(),
        vec![],
        vec![
            CallArg::Object(ObjectArg::SharedObject {
                id: module.ibc_store.into(),
                initial_shared_version: module.ibc_store_initial_seq,
                mutable: true,
            }),
            data.channel_id.raw().into(),
            (&data.proof_ack.into_vec()).into(),
            data.proof_height.into(),
        ],
    )
}

pub mod zkgm {
    use ibc_union_spec::datagram::MsgPacketAcknowledgement;

    use super::*;

    pub fn begin_recv_call(
        ptb: &mut ProgrammableTransactionBuilder,
        module_info: &ModuleInfo,
        data: MsgPacketRecv,
    ) -> Argument {
        let (source_channels, dest_channels, packet_data, timeout_heights, timeout_timestamps) =
            data.packets
                .iter()
                .map(|p| {
                    (
                        p.source_channel_id,
                        p.destination_channel_id,
                        p.data.clone(),
                        0,
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
            module_info.module_name.clone(),
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
        store_initial_seq: SequenceNumber,
        coin_t: TypeTag,
        fee_recipient: SuiAddress,
        relayer_msgs: Vec<Bytes>,
        session: Argument,
    ) -> Argument {
        let arguments = vec![
            CallArg::Object(ObjectArg::SharedObject {
                id: module.ibc_store.into(),
                initial_shared_version: module.ibc_store_initial_seq,
                mutable: true,
            }),
            CallArg::Object(ObjectArg::SharedObject {
                id: module_info.stores[0].into(),
                initial_shared_version: store_initial_seq,
                mutable: true,
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
            module_info.module_name.clone(),
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
    ) -> anyhow::Result<()> {
        let arguments = vec![
            CallArg::Object(ObjectArg::SharedObject {
                id: module.ibc_store.into(),
                initial_shared_version: module.ibc_store_initial_seq,
                mutable: true,
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
            module_info.module_name.clone(),
            ident_str!("end_recv").into(),
            vec![],
            arguments,
        ));

        Ok(())
    }

    pub fn begin_ack_call(
        ptb: &mut ProgrammableTransactionBuilder,
        module_info: &ModuleInfo,
        data: MsgPacketAcknowledgement,
    ) -> Argument {
        let (source_channels, dest_channels, packet_data, timeout_heights, timeout_timestamps) =
            data.packets
                .iter()
                .map(|p| {
                    (
                        p.source_channel_id,
                        p.destination_channel_id,
                        p.data.clone(),
                        0,
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
            module_info.module_name.clone(),
            ident_str!("begin_ack").into(),
            vec![],
            arguments,
        ))
    }

    pub fn acknowledge_packet_call(
        ptb: &mut ProgrammableTransactionBuilder,
        module: &Module,
        module_info: &ModuleInfo,
        store_initial_seq: SequenceNumber,
        coin_t: TypeTag,
        fee_recipient: SuiAddress,
        session: Argument,
    ) -> Argument {
        let arguments = vec![
            CallArg::Object(ObjectArg::SharedObject {
                id: module.ibc_store.into(),
                initial_shared_version: module.ibc_store_initial_seq,
                mutable: true,
            }),
            CallArg::Object(ObjectArg::SharedObject {
                id: module_info.stores[0].into(),
                initial_shared_version: store_initial_seq,
                mutable: true,
            }),
            CallArg::Pure(bcs::to_bytes(&fee_recipient).unwrap()),
        ]
        .into_iter()
        .map(|a| ptb.input(a).unwrap())
        .chain(vec![session])
        .collect();

        ptb.command(Command::move_call(
            module_info.latest_address.into(),
            module_info.module_name.clone(),
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
    ) -> anyhow::Result<()> {
        let arguments = vec![
            CallArg::Object(ObjectArg::SharedObject {
                id: module.ibc_store.into(),
                initial_shared_version: module.ibc_store_initial_seq,
                mutable: true,
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
            module_info.module_name.clone(),
            ident_str!("end_ack").into(),
            vec![],
            arguments,
        ));

        Ok(())
    }

    pub fn register_capability_call(
        ptb: &mut ProgrammableTransactionBuilder,
        module_info: &ModuleInfo,
        initial_seq: SequenceNumber,
        treasury_ref: ObjectRef,
        metadata_ref: ObjectRef,
        coin_t: TypeTag,
    ) -> anyhow::Result<()> {
        let arguments = [
            ptb.input(CallArg::Object(ObjectArg::SharedObject {
                id: module_info.stores[0].into(),
                initial_shared_version: initial_seq,
                mutable: true,
            }))
            .unwrap(),
            ptb.input(CallArg::Object(ObjectArg::ImmOrOwnedObject(treasury_ref)))
                .unwrap(),
            ptb.input(CallArg::Object(ObjectArg::SharedObject {
                id: metadata_ref.0,
                initial_shared_version: metadata_ref.1,
                mutable: true,
            }))
            .unwrap(),
            // owner is 0x0
            ptb.input(CallArg::Pure(
                H256::<HexPrefixed>::default().into_bytes().to_vec(),
            ))
            .unwrap(),
        ];
        ptb.command(Command::move_call(
            module_info.latest_address.into(),
            module_info.module_name.clone(),
            ident_str!("register_capability").into(),
            vec![coin_t.clone()],
            arguments.to_vec(),
        ));

        Ok(())
    }
}

pub async fn get_port_id(module: &Module, channel_id: ChannelId) -> anyhow::Result<String> {
    let query = SuiQuery::new(&module.sui_client, module.ibc_store.into()).await;

    let res = query
        .add_param(channel_id.raw())
        .call(module.ibc_handler_address.into(), "get_port_id")
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    if res.len() != 1 {
        panic!("expected a single encoded connection end")
    }

    let port_id = bcs::from_bytes::<String>(&res[0].0)?;

    Ok(port_id)
}
