//! exports exposes the public wasm API
//!
//! interface_version_8, allocate and deallocate turn into Wasm exports
//! as soon as cosmwasm_std is `use`d in the contract, even privately.
//!
//! `do_execute`, `do_instantiate`, `do_migrate`, `do_query`, `do_reply`
//! and `do_sudo` should be wrapped with a extern "C" entry point including
//! the contract-specific function pointer. This is done via the `#[entry_point]`
//! macro attribute from cosmwasm-derive.
use alloc::vec::Vec;
use core::marker::PhantomData;

use serde::de::DeserializeOwned;

use crate::deps::OwnedDeps;
#[cfg(feature = "stargate")]
use crate::ibc::{
    IbcBasicResponse, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg,
    IbcChannelOpenResponse, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg,
    IbcReceiveResponse,
};
use crate::imports::{ExternalApi, ExternalQuerier, ExternalStorage};
use crate::memory::{alloc, consume_region, release_buffer, Region};
#[cfg(feature = "abort")]
use crate::panic::install_panic_handler;
use crate::query::CustomQuery;
use crate::results::{ContractResult, QueryResponse, Reply, Response};
use crate::serde::{from_json, to_json_vec};
use crate::types::Env;
use crate::{CustomMsg, Deps, DepsMut, MessageInfo};

#[cfg(feature = "iterator")]
#[no_mangle]
extern "C" fn requires_iterator() -> () {}

#[cfg(feature = "staking")]
#[no_mangle]
extern "C" fn requires_staking() -> () {}

#[cfg(feature = "stargate")]
#[no_mangle]
extern "C" fn requires_stargate() -> () {}

#[cfg(feature = "cosmwasm_1_1")]
#[no_mangle]
extern "C" fn requires_cosmwasm_1_1() -> () {}

#[cfg(feature = "cosmwasm_1_2")]
#[no_mangle]
extern "C" fn requires_cosmwasm_1_2() -> () {}

#[cfg(feature = "cosmwasm_1_3")]
#[no_mangle]
extern "C" fn requires_cosmwasm_1_3() -> () {}

#[cfg(feature = "cosmwasm_1_4")]
#[no_mangle]
extern "C" fn requires_cosmwasm_1_4() -> () {}

/// interface_version_* exports mark which Wasm VM interface level this contract is compiled for.
/// They can be checked by cosmwasm_vm.
/// Update this whenever the Wasm VM interface breaks.
#[no_mangle]
extern "C" fn interface_version_8() -> () {}

/// allocate reserves the given number of bytes in wasm memory and returns a pointer
/// to a Region defining this data. This space is managed by the calling process
/// and should be accompanied by a corresponding deallocate
#[no_mangle]
extern "C" fn allocate(size: usize) -> u32 {
    alloc(size) as u32
}

/// deallocate expects a pointer to a Region created with allocate.
/// It will free both the Region and the memory referenced by the Region.
#[no_mangle]
extern "C" fn deallocate(pointer: u32) {
    // auto-drop Region on function end
    let _ = unsafe { consume_region(pointer as *mut Region) };
}

// TODO: replace with https://doc.rust-lang.org/std/ops/trait.Try.html once stabilized
macro_rules! r#try_into_contract_result {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => {
                return ContractResult::Err(err.to_string());
            }
        }
    };
    ($expr:expr,) => {
        $crate::try_into_contract_result!($expr)
    };
}

/// This should be wrapped in an external "C" export, containing a contract-specific function as an argument.
///
/// - `Q`: custom query type (see QueryRequest)
/// - `M`: message type for request
/// - `C`: custom response message type (see CosmosMsg)
/// - `E`: error type for responses
pub fn do_instantiate<Q, M, C, E>(
    instantiate_fn: &dyn Fn(DepsMut<Q>, Env, MessageInfo, M) -> Result<Response<C>, E>,
    env_ptr: u32,
    info_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    M: DeserializeOwned,
    C: CustomMsg,
    E: ToString,
{
    #[cfg(feature = "abort")]
    install_panic_handler();
    let res = _do_instantiate(
        instantiate_fn,
        env_ptr as *mut Region,
        info_ptr as *mut Region,
        msg_ptr as *mut Region,
    );
    let v = to_json_vec(&res).unwrap();
    release_buffer(v) as u32
}

/// do_execute should be wrapped in an external "C" export, containing a contract-specific function as arg
///
/// - `Q`: custom query type (see QueryRequest)
/// - `M`: message type for request
/// - `C`: custom response message type (see CosmosMsg)
/// - `E`: error type for responses
pub fn do_execute<Q, M, C, E>(
    execute_fn: &dyn Fn(DepsMut<Q>, Env, MessageInfo, M) -> Result<Response<C>, E>,
    env_ptr: u32,
    info_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    M: DeserializeOwned,
    C: CustomMsg,
    E: ToString,
{
    #[cfg(feature = "abort")]
    install_panic_handler();
    let res = _do_execute(
        execute_fn,
        env_ptr as *mut Region,
        info_ptr as *mut Region,
        msg_ptr as *mut Region,
    );
    let v = to_json_vec(&res).unwrap();
    release_buffer(v) as u32
}

/// do_migrate should be wrapped in an external "C" export, containing a contract-specific function as arg
///
/// - `Q`: custom query type (see QueryRequest)
/// - `M`: message type for request
/// - `C`: custom response message type (see CosmosMsg)
/// - `E`: error type for responses
pub fn do_migrate<Q, M, C, E>(
    migrate_fn: &dyn Fn(DepsMut<Q>, Env, M) -> Result<Response<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    M: DeserializeOwned,
    C: CustomMsg,
    E: ToString,
{
    #[cfg(feature = "abort")]
    install_panic_handler();
    let res = _do_migrate(migrate_fn, env_ptr as *mut Region, msg_ptr as *mut Region);
    let v = to_json_vec(&res).unwrap();
    release_buffer(v) as u32
}

/// do_sudo should be wrapped in an external "C" export, containing a contract-specific function as arg
///
/// - `Q`: custom query type (see QueryRequest)
/// - `M`: message type for request
/// - `C`: custom response message type (see CosmosMsg)
/// - `E`: error type for responses
pub fn do_sudo<Q, M, C, E>(
    sudo_fn: &dyn Fn(DepsMut<Q>, Env, M) -> Result<Response<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    M: DeserializeOwned,
    C: CustomMsg,
    E: ToString,
{
    #[cfg(feature = "abort")]
    install_panic_handler();
    let res = _do_sudo(sudo_fn, env_ptr as *mut Region, msg_ptr as *mut Region);
    let v = to_json_vec(&res).unwrap();
    release_buffer(v) as u32
}

/// do_reply should be wrapped in an external "C" export, containing a contract-specific function as arg
/// message body is always `SubcallResult`
///
/// - `Q`: custom query type (see QueryRequest)
/// - `C`: custom response message type (see CosmosMsg)
/// - `E`: error type for responses
pub fn do_reply<Q, C, E>(
    reply_fn: &dyn Fn(DepsMut<Q>, Env, Reply) -> Result<Response<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    #[cfg(feature = "abort")]
    install_panic_handler();
    let res = _do_reply(reply_fn, env_ptr as *mut Region, msg_ptr as *mut Region);
    let v = to_json_vec(&res).unwrap();
    release_buffer(v) as u32
}

/// do_query should be wrapped in an external "C" export, containing a contract-specific function as arg
///
/// - `Q`: custom query type (see QueryRequest)
/// - `M`: message type for request
/// - `E`: error type for responses
pub fn do_query<Q, M, E>(
    query_fn: &dyn Fn(Deps<Q>, Env, M) -> Result<QueryResponse, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    M: DeserializeOwned,
    E: ToString,
{
    #[cfg(feature = "abort")]
    install_panic_handler();
    let res = _do_query(query_fn, env_ptr as *mut Region, msg_ptr as *mut Region);
    let v = to_json_vec(&res).unwrap();
    release_buffer(v) as u32
}

/// do_ibc_channel_open is designed for use with #[entry_point] to make a "C" extern
///
/// contract_fn does the protocol version negotiation during channel handshake phase
///
/// - `Q`: custom query type (see QueryRequest)
/// - `E`: error type for responses
#[cfg(feature = "stargate")]
pub fn do_ibc_channel_open<Q, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, IbcChannelOpenMsg) -> Result<IbcChannelOpenResponse, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    E: ToString,
{
    #[cfg(feature = "abort")]
    install_panic_handler();
    let res = _do_ibc_channel_open(contract_fn, env_ptr as *mut Region, msg_ptr as *mut Region);
    let v = to_json_vec(&res).unwrap();
    release_buffer(v) as u32
}

/// do_ibc_channel_connect is designed for use with #[entry_point] to make a "C" extern
///
/// contract_fn is a callback when a IBC channel is established (after both sides agree in open)
///
/// - `Q`: custom query type (see QueryRequest)
/// - `C`: custom response message type (see CosmosMsg)
/// - `E`: error type for responses
#[cfg(feature = "stargate")]
pub fn do_ibc_channel_connect<Q, C, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, IbcChannelConnectMsg) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    #[cfg(feature = "abort")]
    install_panic_handler();
    let res = _do_ibc_channel_connect(contract_fn, env_ptr as *mut Region, msg_ptr as *mut Region);
    let v = to_json_vec(&res).unwrap();
    release_buffer(v) as u32
}

/// do_ibc_channel_close is designed for use with #[entry_point] to make a "C" extern
///
/// contract_fn is a callback when a IBC channel belonging to this contract is closed
///
/// - `Q`: custom query type (see QueryRequest)
/// - `C`: custom response message type (see CosmosMsg)
/// - `E`: error type for responses
#[cfg(feature = "stargate")]
pub fn do_ibc_channel_close<Q, C, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, IbcChannelCloseMsg) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    #[cfg(feature = "abort")]
    install_panic_handler();
    let res = _do_ibc_channel_close(contract_fn, env_ptr as *mut Region, msg_ptr as *mut Region);
    let v = to_json_vec(&res).unwrap();
    release_buffer(v) as u32
}

/// do_ibc_packet_receive is designed for use with #[entry_point] to make a "C" extern
///
/// contract_fn is called when this chain receives an IBC Packet on a channel belonging
/// to this contract
///
/// - `Q`: custom query type (see QueryRequest)
/// - `C`: custom response message type (see CosmosMsg)
/// - `E`: error type for responses
#[cfg(feature = "stargate")]
pub fn do_ibc_packet_receive<Q, C, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, IbcPacketReceiveMsg) -> Result<IbcReceiveResponse<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    #[cfg(feature = "abort")]
    install_panic_handler();
    let res = _do_ibc_packet_receive(contract_fn, env_ptr as *mut Region, msg_ptr as *mut Region);
    let v = to_json_vec(&res).unwrap();
    release_buffer(v) as u32
}

/// do_ibc_packet_ack is designed for use with #[entry_point] to make a "C" extern
///
/// contract_fn is called when this chain receives an IBC Acknowledgement for a packet
/// that this contract previously sent
///
/// - `Q`: custom query type (see QueryRequest)
/// - `C`: custom response message type (see CosmosMsg)
/// - `E`: error type for responses
#[cfg(feature = "stargate")]
pub fn do_ibc_packet_ack<Q, C, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, IbcPacketAckMsg) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    #[cfg(feature = "abort")]
    install_panic_handler();
    let res = _do_ibc_packet_ack(contract_fn, env_ptr as *mut Region, msg_ptr as *mut Region);
    let v = to_json_vec(&res).unwrap();
    release_buffer(v) as u32
}

/// do_ibc_packet_timeout is designed for use with #[entry_point] to make a "C" extern
///
/// contract_fn is called when a packet that this contract previously sent has provably
/// timedout and will never be relayed to the calling chain. This generally behaves
/// like ick_ack_fn upon an acknowledgement containing an error.
///
/// - `Q`: custom query type (see QueryRequest)
/// - `C`: custom response message type (see CosmosMsg)
/// - `E`: error type for responses
#[cfg(feature = "stargate")]
pub fn do_ibc_packet_timeout<Q, C, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, IbcPacketTimeoutMsg) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: u32,
    msg_ptr: u32,
) -> u32
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    #[cfg(feature = "abort")]
    install_panic_handler();
    let res = _do_ibc_packet_timeout(contract_fn, env_ptr as *mut Region, msg_ptr as *mut Region);
    let v = to_json_vec(&res).unwrap();
    release_buffer(v) as u32
}

fn _do_instantiate<Q, M, C, E>(
    instantiate_fn: &dyn Fn(DepsMut<Q>, Env, MessageInfo, M) -> Result<Response<C>, E>,
    env_ptr: *mut Region,
    info_ptr: *mut Region,
    msg_ptr: *mut Region,
) -> ContractResult<Response<C>>
where
    Q: CustomQuery,
    M: DeserializeOwned,
    C: CustomMsg,
    E: ToString,
{
    let env: Vec<u8> = unsafe { consume_region(env_ptr) };
    let info: Vec<u8> = unsafe { consume_region(info_ptr) };
    let msg: Vec<u8> = unsafe { consume_region(msg_ptr) };

    let env: Env = try_into_contract_result!(from_json(env));
    let info: MessageInfo = try_into_contract_result!(from_json(info));
    let msg: M = try_into_contract_result!(from_json(msg));

    let mut deps = make_dependencies();
    instantiate_fn(deps.as_mut(), env, info, msg).into()
}

fn _do_execute<Q, M, C, E>(
    execute_fn: &dyn Fn(DepsMut<Q>, Env, MessageInfo, M) -> Result<Response<C>, E>,
    env_ptr: *mut Region,
    info_ptr: *mut Region,
    msg_ptr: *mut Region,
) -> ContractResult<Response<C>>
where
    Q: CustomQuery,
    M: DeserializeOwned,
    C: CustomMsg,
    E: ToString,
{
    let env: Vec<u8> = unsafe { consume_region(env_ptr) };
    let info: Vec<u8> = unsafe { consume_region(info_ptr) };
    let msg: Vec<u8> = unsafe { consume_region(msg_ptr) };

    let env: Env = try_into_contract_result!(from_json(env));
    let info: MessageInfo = try_into_contract_result!(from_json(info));
    let msg: M = try_into_contract_result!(from_json(msg));

    let mut deps = make_dependencies();
    execute_fn(deps.as_mut(), env, info, msg).into()
}

fn _do_migrate<Q, M, C, E>(
    migrate_fn: &dyn Fn(DepsMut<Q>, Env, M) -> Result<Response<C>, E>,
    env_ptr: *mut Region,
    msg_ptr: *mut Region,
) -> ContractResult<Response<C>>
where
    Q: CustomQuery,
    M: DeserializeOwned,
    C: CustomMsg,
    E: ToString,
{
    let env: Vec<u8> = unsafe { consume_region(env_ptr) };
    let msg: Vec<u8> = unsafe { consume_region(msg_ptr) };

    let env: Env = try_into_contract_result!(from_json(env));
    let msg: M = try_into_contract_result!(from_json(msg));

    let mut deps = make_dependencies();
    migrate_fn(deps.as_mut(), env, msg).into()
}

fn _do_sudo<Q, M, C, E>(
    sudo_fn: &dyn Fn(DepsMut<Q>, Env, M) -> Result<Response<C>, E>,
    env_ptr: *mut Region,
    msg_ptr: *mut Region,
) -> ContractResult<Response<C>>
where
    Q: CustomQuery,
    M: DeserializeOwned,
    C: CustomMsg,
    E: ToString,
{
    let env: Vec<u8> = unsafe { consume_region(env_ptr) };
    let msg: Vec<u8> = unsafe { consume_region(msg_ptr) };

    let env: Env = try_into_contract_result!(from_json(env));
    let msg: M = try_into_contract_result!(from_json(msg));

    let mut deps = make_dependencies();
    sudo_fn(deps.as_mut(), env, msg).into()
}

fn _do_reply<Q, C, E>(
    reply_fn: &dyn Fn(DepsMut<Q>, Env, Reply) -> Result<Response<C>, E>,
    env_ptr: *mut Region,
    msg_ptr: *mut Region,
) -> ContractResult<Response<C>>
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    let env: Vec<u8> = unsafe { consume_region(env_ptr) };
    let msg: Vec<u8> = unsafe { consume_region(msg_ptr) };

    let env: Env = try_into_contract_result!(from_json(env));
    let msg: Reply = try_into_contract_result!(from_json(msg));

    let mut deps = make_dependencies();
    reply_fn(deps.as_mut(), env, msg).into()
}

fn _do_query<Q, M, E>(
    query_fn: &dyn Fn(Deps<Q>, Env, M) -> Result<QueryResponse, E>,
    env_ptr: *mut Region,
    msg_ptr: *mut Region,
) -> ContractResult<QueryResponse>
where
    Q: CustomQuery,
    M: DeserializeOwned,
    E: ToString,
{
    let env: Vec<u8> = unsafe { consume_region(env_ptr) };
    let msg: Vec<u8> = unsafe { consume_region(msg_ptr) };

    let env: Env = try_into_contract_result!(from_json(env));
    let msg: M = try_into_contract_result!(from_json(msg));

    let deps = make_dependencies();
    query_fn(deps.as_ref(), env, msg).into()
}

#[cfg(feature = "stargate")]
fn _do_ibc_channel_open<Q, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, IbcChannelOpenMsg) -> Result<IbcChannelOpenResponse, E>,
    env_ptr: *mut Region,
    msg_ptr: *mut Region,
) -> ContractResult<IbcChannelOpenResponse>
where
    Q: CustomQuery,
    E: ToString,
{
    let env: Vec<u8> = unsafe { consume_region(env_ptr) };
    let msg: Vec<u8> = unsafe { consume_region(msg_ptr) };

    let env: Env = try_into_contract_result!(from_json(env));
    let msg: IbcChannelOpenMsg = try_into_contract_result!(from_json(msg));

    let mut deps = make_dependencies();
    contract_fn(deps.as_mut(), env, msg).into()
}

#[cfg(feature = "stargate")]
fn _do_ibc_channel_connect<Q, C, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, IbcChannelConnectMsg) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: *mut Region,
    msg_ptr: *mut Region,
) -> ContractResult<IbcBasicResponse<C>>
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    let env: Vec<u8> = unsafe { consume_region(env_ptr) };
    let msg: Vec<u8> = unsafe { consume_region(msg_ptr) };

    let env: Env = try_into_contract_result!(from_json(env));
    let msg: IbcChannelConnectMsg = try_into_contract_result!(from_json(msg));

    let mut deps = make_dependencies();
    contract_fn(deps.as_mut(), env, msg).into()
}

#[cfg(feature = "stargate")]
fn _do_ibc_channel_close<Q, C, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, IbcChannelCloseMsg) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: *mut Region,
    msg_ptr: *mut Region,
) -> ContractResult<IbcBasicResponse<C>>
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    let env: Vec<u8> = unsafe { consume_region(env_ptr) };
    let msg: Vec<u8> = unsafe { consume_region(msg_ptr) };

    let env: Env = try_into_contract_result!(from_json(env));
    let msg: IbcChannelCloseMsg = try_into_contract_result!(from_json(msg));

    let mut deps = make_dependencies();
    contract_fn(deps.as_mut(), env, msg).into()
}

#[cfg(feature = "stargate")]
fn _do_ibc_packet_receive<Q, C, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, IbcPacketReceiveMsg) -> Result<IbcReceiveResponse<C>, E>,
    env_ptr: *mut Region,
    msg_ptr: *mut Region,
) -> ContractResult<IbcReceiveResponse<C>>
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    let env: Vec<u8> = unsafe { consume_region(env_ptr) };
    let msg: Vec<u8> = unsafe { consume_region(msg_ptr) };

    let env: Env = try_into_contract_result!(from_json(env));
    let msg: IbcPacketReceiveMsg = try_into_contract_result!(from_json(msg));

    let mut deps = make_dependencies();
    contract_fn(deps.as_mut(), env, msg).into()
}

#[cfg(feature = "stargate")]
fn _do_ibc_packet_ack<Q, C, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, IbcPacketAckMsg) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: *mut Region,
    msg_ptr: *mut Region,
) -> ContractResult<IbcBasicResponse<C>>
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    let env: Vec<u8> = unsafe { consume_region(env_ptr) };
    let msg: Vec<u8> = unsafe { consume_region(msg_ptr) };

    let env: Env = try_into_contract_result!(from_json(env));
    let msg: IbcPacketAckMsg = try_into_contract_result!(from_json(msg));

    let mut deps = make_dependencies();
    contract_fn(deps.as_mut(), env, msg).into()
}

#[cfg(feature = "stargate")]
fn _do_ibc_packet_timeout<Q, C, E>(
    contract_fn: &dyn Fn(DepsMut<Q>, Env, IbcPacketTimeoutMsg) -> Result<IbcBasicResponse<C>, E>,
    env_ptr: *mut Region,
    msg_ptr: *mut Region,
) -> ContractResult<IbcBasicResponse<C>>
where
    Q: CustomQuery,
    C: CustomMsg,
    E: ToString,
{
    let env: Vec<u8> = unsafe { consume_region(env_ptr) };
    let msg: Vec<u8> = unsafe { consume_region(msg_ptr) };

    let env: Env = try_into_contract_result!(from_json(env));
    let msg: IbcPacketTimeoutMsg = try_into_contract_result!(from_json(msg));

    let mut deps = make_dependencies();
    contract_fn(deps.as_mut(), env, msg).into()
}

/// Makes all bridges to external dependencies (i.e. Wasm imports) that are injected by the VM
pub(crate) fn make_dependencies<Q>() -> OwnedDeps<ExternalStorage, ExternalApi, ExternalQuerier, Q>
where
    Q: CustomQuery,
{
    OwnedDeps {
        storage: ExternalStorage::new(),
        api: ExternalApi::new(),
        querier: ExternalQuerier::new(),
        custom_query_type: PhantomData,
    }
}
