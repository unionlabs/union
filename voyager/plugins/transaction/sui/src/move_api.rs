use ibc_union_spec::{
    ChannelId,
    datagram::{
        MsgChannelOpenAck, MsgChannelOpenConfirm, MsgChannelOpenInit, MsgChannelOpenTry,
        MsgCommitPacketTimeout, MsgConnectionOpenAck, MsgConnectionOpenConfirm,
        MsgConnectionOpenInit, MsgConnectionOpenTry, MsgCreateClient, MsgUpdateClient,
    },
};
use move_core_types::{ident_str, identifier::IdentStr};
use sui_sdk::types::{
    base_types::{ObjectID, SequenceNumber, SuiAddress},
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::{CallArg, ObjectArg, SharedObjectMutability},
};
use sui_utils::SuiQuery;
use unionlabs::primitives::H256;
use voyager_sdk::rpc::{RpcError, RpcResult};

use crate::{Module, ModuleInfo};

pub const SUI_CALL_ARG_CLOCK: CallArg = CallArg::Object(ObjectArg::SharedObject {
    id: ObjectID::from_single_byte(6),
    initial_shared_version: SequenceNumber::from_u64(1),
    mutability: SharedObjectMutability::Immutable,
});

pub const IBC_IDENT: &IdentStr = ident_str!("ibc");

pub fn create_client(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    data: MsgCreateClient,
) -> RpcResult<()> {
    ptb.move_call(
        module.ibc_contract.into(),
        IBC_IDENT.into(),
        ident_str!("create_client").into(),
        vec![],
        vec![
            CallArg::Object(ObjectArg::SharedObject {
                id: module.ibc_store.into(),
                initial_shared_version: module.ibc_store_initial_seq,
                mutability: SharedObjectMutability::Mutable,
            }),
            CallArg::Pure(bcs::to_bytes(&data.client_type.to_string()).unwrap()),
            (&data.client_state_bytes.into_vec()).into(),
            (&data.consensus_state_bytes.into_vec()).into(),
        ],
    )
    .map_err(RpcError::fatal_from_message)
}

pub fn update_client(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    data: MsgUpdateClient,
) -> RpcResult<()> {
    ptb.move_call(
        module.ibc_contract.into(),
        IBC_IDENT.into(),
        ident_str!("update_client").into(),
        vec![],
        vec![
            CallArg::Object(ObjectArg::SharedObject {
                id: module.ibc_store.into(),
                initial_shared_version: module.ibc_store_initial_seq,
                mutability: SharedObjectMutability::Mutable,
            }),
            SUI_CALL_ARG_CLOCK.clone(),
            data.client_id.raw().into(),
            (&data.client_message.into_vec()).into(),
            CallArg::Pure(<H256>::default().into_bytes().to_vec()),
        ],
    )
    .map_err(RpcError::fatal_from_message)
}

pub fn connection_open_init(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    data: MsgConnectionOpenInit,
) -> RpcResult<()> {
    ptb.move_call(
        module.ibc_contract.into(),
        IBC_IDENT.into(),
        ident_str!("connection_open_init").into(),
        vec![],
        vec![
            CallArg::Object(ObjectArg::SharedObject {
                id: module.ibc_store.into(),
                initial_shared_version: module.ibc_store_initial_seq,
                mutability: SharedObjectMutability::Mutable,
            }),
            data.client_id.raw().into(),
            data.counterparty_client_id.raw().into(),
        ],
    )
    .map_err(RpcError::fatal_from_message)
}

pub fn connection_open_try(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    data: MsgConnectionOpenTry,
) -> RpcResult<()> {
    ptb.move_call(
        module.ibc_contract.into(),
        IBC_IDENT.into(),
        ident_str!("connection_open_try").into(),
        vec![],
        vec![
            CallArg::Object(ObjectArg::SharedObject {
                id: module.ibc_store.into(),
                initial_shared_version: module.ibc_store_initial_seq,
                mutability: SharedObjectMutability::Mutable,
            }),
            data.counterparty_client_id.raw().into(),
            data.counterparty_connection_id.raw().into(),
            data.client_id.raw().into(),
            (&data.proof_init.into_vec()).into(),
            data.proof_height.into(),
        ],
    )
    .map_err(RpcError::fatal_from_message)
}

pub fn connection_open_ack(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    data: MsgConnectionOpenAck,
) -> RpcResult<()> {
    ptb.move_call(
        module.ibc_contract.into(),
        IBC_IDENT.into(),
        ident_str!("connection_open_ack").into(),
        vec![],
        vec![
            CallArg::Object(ObjectArg::SharedObject {
                id: module.ibc_store.into(),
                initial_shared_version: module.ibc_store_initial_seq,
                mutability: SharedObjectMutability::Mutable,
            }),
            data.connection_id.raw().into(),
            data.counterparty_connection_id.raw().into(),
            (&data.proof_try.into_vec()).into(),
            data.proof_height.into(),
        ],
    )
    .map_err(RpcError::fatal_from_message)
}

pub fn connection_open_confirm(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    data: MsgConnectionOpenConfirm,
) -> RpcResult<()> {
    ptb.move_call(
        module.ibc_contract.into(),
        IBC_IDENT.into(),
        ident_str!("connection_open_confirm").into(),
        vec![],
        vec![
            CallArg::Object(ObjectArg::SharedObject {
                id: module.ibc_store.into(),
                initial_shared_version: module.ibc_store_initial_seq,
                mutability: SharedObjectMutability::Mutable,
            }),
            data.connection_id.raw().into(),
            (&data.proof_ack.into_vec()).into(),
            data.proof_height.into(),
        ],
    )
    .map_err(RpcError::fatal_from_message)
}

pub fn channel_open_init(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    module_info: ModuleInfo,
    data: MsgChannelOpenInit,
) -> RpcResult<()> {
    ptb.move_call(
        module_info.latest_address.into(),
        IBC_IDENT.into(),
        ident_str!("channel_open_init").into(),
        vec![],
        vec![
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
            (&data.port_id.into_vec()).into(),
            (&data.counterparty_port_id.into_vec()).into(),
            data.connection_id.raw().into(),
            (&data.version.into_bytes()).into(),
        ],
    )
    .map_err(RpcError::fatal_from_message)
}

pub fn channel_open_try(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    module_info: ModuleInfo,
    data: MsgChannelOpenTry,
) -> RpcResult<()> {
    ptb.move_call(
        module_info.latest_address.into(),
        IBC_IDENT.into(),
        ident_str!("channel_open_try").into(),
        vec![],
        vec![
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
            data.channel.connection_id.raw().into(),
            data.channel.counterparty_channel_id.unwrap().raw().into(),
            (&data.channel.counterparty_port_id.into_vec()).into(),
            (&data.channel.version.into_bytes()).into(),
            (&data.counterparty_version.into_bytes()).into(),
            (&data.proof_init.into_vec()).into(),
            data.proof_height.into(),
        ],
    )
    .map_err(RpcError::fatal_from_message)
}

pub fn channel_open_ack(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    module_info: ModuleInfo,
    data: MsgChannelOpenAck,
) -> RpcResult<()> {
    ptb.move_call(
        module_info.latest_address.into(),
        IBC_IDENT.into(),
        ident_str!("channel_open_ack").into(),
        vec![],
        vec![
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
            data.channel_id.raw().into(),
            (&data.counterparty_version.into_bytes()).into(),
            data.counterparty_channel_id.raw().into(),
            (&data.proof_try.into_vec()).into(),
            data.proof_height.into(),
        ],
    )
    .map_err(RpcError::fatal_from_message)
}

pub fn channel_open_confirm_call(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    module_info: ModuleInfo,
    data: MsgChannelOpenConfirm,
) -> RpcResult<()> {
    ptb.move_call(
        module_info.latest_address.into(),
        IBC_IDENT.into(),
        ident_str!("channel_open_confirm").into(),
        vec![],
        vec![
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
            data.channel_id.raw().into(),
            (&data.proof_ack.into_vec()).into(),
            data.proof_height.into(),
        ],
    )
    .map_err(RpcError::fatal_from_message)
}

pub fn packet_timeout_commitment_call(
    ptb: &mut ProgrammableTransactionBuilder,
    module: &Module,
    data: MsgCommitPacketTimeout,
) -> RpcResult<()> {
    ptb.move_call(
        module.ibc_contract.into(),
        IBC_IDENT.into(),
        ident_str!("commit_packet_timeout").into(),
        vec![],
        vec![
            CallArg::Object(ObjectArg::SharedObject {
                id: module.ibc_store.into(),
                initial_shared_version: module.ibc_store_initial_seq,
                mutability: SharedObjectMutability::Mutable,
            }),
            SUI_CALL_ARG_CLOCK,
            (&data.proof.into_vec()).into(),
            data.proof_height.into(),
        ],
    )
    .map_err(RpcError::fatal_from_message)
}

pub async fn get_port_id(module: &Module, channel_id: ChannelId) -> RpcResult<SuiAddress> {
    let query = SuiQuery::new_with_store(
        &module.sui_client,
        module.ibc_contract.into(),
        module.ibc_store.into(),
        module.ibc_store_initial_seq,
    )
    .await;

    let res = query
        .add_param(channel_id.raw())
        .call(ident_str!("ibc"), ident_str!("get_port_id"))
        .await
        .map_err(RpcError::retryable_from_message)?;

    if res.len() != 1 {
        panic!("expected a single port id")
    }

    let port_id = bcs::from_bytes::<SuiAddress>(&res[0].0)
        .map_err(RpcError::fatal("error decoding sui address from bcs"))?;

    Ok(port_id)
}
