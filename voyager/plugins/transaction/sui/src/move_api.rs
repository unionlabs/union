use ibc_union_spec::datagram::{
    MsgChannelOpenInit, MsgChannelOpenTry, MsgConnectionOpenAck, MsgConnectionOpenConfirm,
    MsgConnectionOpenInit, MsgConnectionOpenTry, MsgCreateClient, MsgUpdateClient,
};
use move_core_types_sui::{ident_str, identifier::IdentStr};
use sui_sdk::types::{
    base_types::{ObjectID, SequenceNumber},
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::{CallArg, ObjectArg},
};
use unionlabs::primitives::{encoding::HexPrefixed, H256};
use voyager_sdk::anyhow;

use crate::{Module, ModuleInfo};

pub const SUI_CALL_ARG_CLOCK: CallArg = CallArg::Object(ObjectArg::SharedObject {
    id: ObjectID::from_single_byte(6),
    initial_shared_version: SequenceNumber::from_u64(1),
    mutable: false,
});

pub const IBC_IDENT: &IdentStr = ident_str!("ibc");

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
