//! This file has some helpers for integration tests.
//! They should be imported via full path to ensure there is no confusion
//! use cosmwasm_vm::testing::X
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};

use cosmwasm_std::{ContractResult, CustomMsg, Env, MessageInfo, QueryResponse, Reply, Response};
#[cfg(feature = "stargate")]
use cosmwasm_std::{
    Ibc3ChannelOpenResponse, IbcBasicResponse, IbcChannelCloseMsg, IbcChannelConnectMsg,
    IbcChannelOpenMsg, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg,
    IbcReceiveResponse,
};

use crate::calls::{
    call_execute, call_instantiate, call_migrate, call_query, call_reply, call_sudo,
};
#[cfg(feature = "stargate")]
use crate::calls::{
    call_ibc_channel_close, call_ibc_channel_connect, call_ibc_channel_open, call_ibc_packet_ack,
    call_ibc_packet_receive, call_ibc_packet_timeout,
};
use crate::instance::Instance;
use crate::serde::to_vec;
use crate::{BackendApi, Querier, Storage};

/// Mimicks the call signature of the smart contracts.
/// Thus it moves env and msg rather than take them as reference.
/// This is inefficient here, but only used in test code.
pub fn instantiate<A, S, Q, M, U>(
    instance: &mut Instance<A, S, Q>,
    env: Env,
    info: MessageInfo,
    msg: M,
) -> ContractResult<Response<U>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    M: Serialize + JsonSchema,
    U: DeserializeOwned + CustomMsg,
{
    let serialized_msg = to_vec(&msg).expect("Testing error: Could not seralize request message");
    call_instantiate(instance, &env, &info, &serialized_msg).expect("VM error")
}

// execute mimicks the call signature of the smart contracts.
// thus it moves env and msg rather than take them as reference.
// this is inefficient here, but only used in test code
pub fn execute<A, S, Q, M, U>(
    instance: &mut Instance<A, S, Q>,
    env: Env,
    info: MessageInfo,
    msg: M,
) -> ContractResult<Response<U>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    M: Serialize + JsonSchema,
    U: DeserializeOwned + CustomMsg,
{
    let serialized_msg = to_vec(&msg).expect("Testing error: Could not seralize request message");
    call_execute(instance, &env, &info, &serialized_msg).expect("VM error")
}

// migrate mimicks the call signature of the smart contracts.
// thus it moves env and msg rather than take them as reference.
// this is inefficient here, but only used in test code
pub fn migrate<A, S, Q, M, U>(
    instance: &mut Instance<A, S, Q>,
    env: Env,
    msg: M,
) -> ContractResult<Response<U>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    M: Serialize + JsonSchema,
    U: DeserializeOwned + CustomMsg,
{
    let serialized_msg = to_vec(&msg).expect("Testing error: Could not seralize request message");
    call_migrate(instance, &env, &serialized_msg).expect("VM error")
}

// sudo mimicks the call signature of the smart contracts.
// thus it moves env and msg rather than take them as reference.
// this is inefficient here, but only used in test code
pub fn sudo<A, S, Q, M, U>(
    instance: &mut Instance<A, S, Q>,
    env: Env,
    msg: M,
) -> ContractResult<Response<U>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    M: Serialize + JsonSchema,
    U: DeserializeOwned + CustomMsg,
{
    let serialized_msg = to_vec(&msg).expect("Testing error: Could not seralize request message");
    call_sudo(instance, &env, &serialized_msg).expect("VM error")
}

// reply mimicks the call signature of the smart contracts.
// thus it moves env and msg rather than take them as reference.
// this is inefficient here, but only used in test code
pub fn reply<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: Env,
    msg: Reply,
) -> ContractResult<Response<U>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    call_reply(instance, &env, &msg).expect("VM error")
}

// query mimicks the call signature of the smart contracts.
// thus it moves env and msg rather than take them as reference.
// this is inefficient here, but only used in test code
pub fn query<A, S, Q, M>(
    instance: &mut Instance<A, S, Q>,
    env: Env,
    msg: M,
) -> ContractResult<QueryResponse>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    M: Serialize + JsonSchema,
{
    let serialized_msg = to_vec(&msg).expect("Testing error: Could not seralize request message");
    call_query(instance, &env, &serialized_msg).expect("VM error")
}

// ibc_channel_open mimicks the call signature of the smart contracts.
// thus it moves env and channel rather than take them as reference.
// this is inefficient here, but only used in test code
#[cfg(feature = "stargate")]
pub fn ibc_channel_open<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: Env,
    msg: IbcChannelOpenMsg,
) -> ContractResult<Option<Ibc3ChannelOpenResponse>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    call_ibc_channel_open(instance, &env, &msg).expect("VM error")
}

// ibc_channel_connect mimicks the call signature of the smart contracts.
// thus it moves env and channel rather than take them as reference.
// this is inefficient here, but only used in test code
#[cfg(feature = "stargate")]
pub fn ibc_channel_connect<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: Env,
    msg: IbcChannelConnectMsg,
) -> ContractResult<IbcBasicResponse<U>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    call_ibc_channel_connect(instance, &env, &msg).expect("VM error")
}

// ibc_channel_close mimicks the call signature of the smart contracts.
// thus it moves env and channel rather than take them as reference.
// this is inefficient here, but only used in test code
#[cfg(feature = "stargate")]
pub fn ibc_channel_close<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: Env,
    msg: IbcChannelCloseMsg,
) -> ContractResult<IbcBasicResponse<U>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    call_ibc_channel_close(instance, &env, &msg).expect("VM error")
}

// ibc_packet_receive mimicks the call signature of the smart contracts.
// thus it moves env and packet rather than take them as reference.
// this is inefficient here, but only used in test code
#[cfg(feature = "stargate")]
pub fn ibc_packet_receive<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: Env,
    msg: IbcPacketReceiveMsg,
) -> ContractResult<IbcReceiveResponse<U>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    call_ibc_packet_receive(instance, &env, &msg).expect("VM error")
}

// ibc_packet_ack mimicks the call signature of the smart contracts.
// thus it moves env and acknowledgement rather than take them as reference.
// this is inefficient here, but only used in test code
#[cfg(feature = "stargate")]
pub fn ibc_packet_ack<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: Env,
    msg: IbcPacketAckMsg,
) -> ContractResult<IbcBasicResponse<U>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    call_ibc_packet_ack(instance, &env, &msg).expect("VM error")
}

// ibc_packet_timeout mimicks the call signature of the smart contracts.
// thus it moves env and packet rather than take them as reference.
// this is inefficient here, but only used in test code
#[cfg(feature = "stargate")]
pub fn ibc_packet_timeout<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: Env,
    msg: IbcPacketTimeoutMsg,
) -> ContractResult<IbcBasicResponse<U>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    call_ibc_packet_timeout(instance, &env, &msg).expect("VM error")
}
