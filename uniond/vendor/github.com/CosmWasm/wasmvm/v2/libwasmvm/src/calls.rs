//! A module containing calls into smart contracts via Cache and Instance.

use std::convert::TryInto;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::SystemTime;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use cosmwasm_std::Checksum;
use cosmwasm_vm::{
    call_execute_raw, call_ibc_channel_close_raw, call_ibc_channel_connect_raw,
    call_ibc_channel_open_raw, call_ibc_destination_callback_raw, call_ibc_packet_ack_raw,
    call_ibc_packet_receive_raw, call_ibc_packet_timeout_raw, call_ibc_source_callback_raw,
    call_instantiate_raw, call_migrate_raw, call_query_raw, call_reply_raw, call_sudo_raw, Backend,
    Cache, Instance, InstanceOptions, VmResult,
};

use crate::api::GoApi;
use crate::args::{ARG1, ARG2, ARG3, CACHE_ARG, CHECKSUM_ARG, GAS_REPORT_ARG};
use crate::cache::{cache_t, to_cache};
use crate::db::Db;
use crate::error::{handle_c_error_binary, Error};
use crate::memory::{ByteSliceView, UnmanagedVector};
use crate::querier::GoQuerier;
use crate::storage::GoStorage;
use crate::GasReport;

fn into_backend(db: Db, api: GoApi, querier: GoQuerier) -> Backend<GoApi, GoStorage, GoQuerier> {
    Backend {
        api,
        storage: GoStorage::new(db),
        querier,
    }
}

#[no_mangle]
pub extern "C" fn instantiate(
    cache: *mut cache_t,
    checksum: ByteSliceView,
    env: ByteSliceView,
    info: ByteSliceView,
    msg: ByteSliceView,
    db: Db,
    api: GoApi,
    querier: GoQuerier,
    gas_limit: u64,
    print_debug: bool,
    gas_report: Option<&mut GasReport>,
    error_msg: Option<&mut UnmanagedVector>,
) -> UnmanagedVector {
    call_3_args(
        call_instantiate_raw,
        cache,
        checksum,
        env,
        info,
        msg,
        db,
        api,
        querier,
        gas_limit,
        print_debug,
        gas_report,
        error_msg,
    )
}

#[no_mangle]
pub extern "C" fn execute(
    cache: *mut cache_t,
    checksum: ByteSliceView,
    env: ByteSliceView,
    info: ByteSliceView,
    msg: ByteSliceView,
    db: Db,
    api: GoApi,
    querier: GoQuerier,
    gas_limit: u64,
    print_debug: bool,
    gas_report: Option<&mut GasReport>,
    error_msg: Option<&mut UnmanagedVector>,
) -> UnmanagedVector {
    call_3_args(
        call_execute_raw,
        cache,
        checksum,
        env,
        info,
        msg,
        db,
        api,
        querier,
        gas_limit,
        print_debug,
        gas_report,
        error_msg,
    )
}

#[no_mangle]
pub extern "C" fn migrate(
    cache: *mut cache_t,
    checksum: ByteSliceView,
    env: ByteSliceView,
    msg: ByteSliceView,
    db: Db,
    api: GoApi,
    querier: GoQuerier,
    gas_limit: u64,
    print_debug: bool,
    gas_report: Option<&mut GasReport>,
    error_msg: Option<&mut UnmanagedVector>,
) -> UnmanagedVector {
    call_2_args(
        call_migrate_raw,
        cache,
        checksum,
        env,
        msg,
        db,
        api,
        querier,
        gas_limit,
        print_debug,
        gas_report,
        error_msg,
    )
}

#[no_mangle]
pub extern "C" fn sudo(
    cache: *mut cache_t,
    checksum: ByteSliceView,
    env: ByteSliceView,
    msg: ByteSliceView,
    db: Db,
    api: GoApi,
    querier: GoQuerier,
    gas_limit: u64,
    print_debug: bool,
    gas_report: Option<&mut GasReport>,
    error_msg: Option<&mut UnmanagedVector>,
) -> UnmanagedVector {
    call_2_args(
        call_sudo_raw,
        cache,
        checksum,
        env,
        msg,
        db,
        api,
        querier,
        gas_limit,
        print_debug,
        gas_report,
        error_msg,
    )
}

#[no_mangle]
pub extern "C" fn reply(
    cache: *mut cache_t,
    checksum: ByteSliceView,
    env: ByteSliceView,
    msg: ByteSliceView,
    db: Db,
    api: GoApi,
    querier: GoQuerier,
    gas_limit: u64,
    print_debug: bool,
    gas_report: Option<&mut GasReport>,
    error_msg: Option<&mut UnmanagedVector>,
) -> UnmanagedVector {
    call_2_args(
        call_reply_raw,
        cache,
        checksum,
        env,
        msg,
        db,
        api,
        querier,
        gas_limit,
        print_debug,
        gas_report,
        error_msg,
    )
}

#[no_mangle]
pub extern "C" fn query(
    cache: *mut cache_t,
    checksum: ByteSliceView,
    env: ByteSliceView,
    msg: ByteSliceView,
    db: Db,
    api: GoApi,
    querier: GoQuerier,
    gas_limit: u64,
    print_debug: bool,
    gas_report: Option<&mut GasReport>,
    error_msg: Option<&mut UnmanagedVector>,
) -> UnmanagedVector {
    call_2_args(
        call_query_raw,
        cache,
        checksum,
        env,
        msg,
        db,
        api,
        querier,
        gas_limit,
        print_debug,
        gas_report,
        error_msg,
    )
}

#[no_mangle]
pub extern "C" fn ibc_channel_open(
    cache: *mut cache_t,
    checksum: ByteSliceView,
    env: ByteSliceView,
    msg: ByteSliceView,
    db: Db,
    api: GoApi,
    querier: GoQuerier,
    gas_limit: u64,
    print_debug: bool,
    gas_report: Option<&mut GasReport>,
    error_msg: Option<&mut UnmanagedVector>,
) -> UnmanagedVector {
    call_2_args(
        call_ibc_channel_open_raw,
        cache,
        checksum,
        env,
        msg,
        db,
        api,
        querier,
        gas_limit,
        print_debug,
        gas_report,
        error_msg,
    )
}

#[no_mangle]
pub extern "C" fn ibc_channel_connect(
    cache: *mut cache_t,
    checksum: ByteSliceView,
    env: ByteSliceView,
    msg: ByteSliceView,
    db: Db,
    api: GoApi,
    querier: GoQuerier,
    gas_limit: u64,
    print_debug: bool,
    gas_report: Option<&mut GasReport>,
    error_msg: Option<&mut UnmanagedVector>,
) -> UnmanagedVector {
    call_2_args(
        call_ibc_channel_connect_raw,
        cache,
        checksum,
        env,
        msg,
        db,
        api,
        querier,
        gas_limit,
        print_debug,
        gas_report,
        error_msg,
    )
}

#[no_mangle]
pub extern "C" fn ibc_channel_close(
    cache: *mut cache_t,
    checksum: ByteSliceView,
    env: ByteSliceView,
    msg: ByteSliceView,
    db: Db,
    api: GoApi,
    querier: GoQuerier,
    gas_limit: u64,
    print_debug: bool,
    gas_report: Option<&mut GasReport>,
    error_msg: Option<&mut UnmanagedVector>,
) -> UnmanagedVector {
    call_2_args(
        call_ibc_channel_close_raw,
        cache,
        checksum,
        env,
        msg,
        db,
        api,
        querier,
        gas_limit,
        print_debug,
        gas_report,
        error_msg,
    )
}

#[no_mangle]
pub extern "C" fn ibc_packet_receive(
    cache: *mut cache_t,
    checksum: ByteSliceView,
    env: ByteSliceView,
    msg: ByteSliceView,
    db: Db,
    api: GoApi,
    querier: GoQuerier,
    gas_limit: u64,
    print_debug: bool,
    gas_report: Option<&mut GasReport>,
    error_msg: Option<&mut UnmanagedVector>,
) -> UnmanagedVector {
    call_2_args(
        call_ibc_packet_receive_raw,
        cache,
        checksum,
        env,
        msg,
        db,
        api,
        querier,
        gas_limit,
        print_debug,
        gas_report,
        error_msg,
    )
}

#[no_mangle]
pub extern "C" fn ibc_packet_ack(
    cache: *mut cache_t,
    checksum: ByteSliceView,
    env: ByteSliceView,
    msg: ByteSliceView,
    db: Db,
    api: GoApi,
    querier: GoQuerier,
    gas_limit: u64,
    print_debug: bool,
    gas_report: Option<&mut GasReport>,
    error_msg: Option<&mut UnmanagedVector>,
) -> UnmanagedVector {
    call_2_args(
        call_ibc_packet_ack_raw,
        cache,
        checksum,
        env,
        msg,
        db,
        api,
        querier,
        gas_limit,
        print_debug,
        gas_report,
        error_msg,
    )
}

#[no_mangle]
pub extern "C" fn ibc_packet_timeout(
    cache: *mut cache_t,
    checksum: ByteSliceView,
    env: ByteSliceView,
    msg: ByteSliceView,
    db: Db,
    api: GoApi,
    querier: GoQuerier,
    gas_limit: u64,
    print_debug: bool,
    gas_report: Option<&mut GasReport>,
    error_msg: Option<&mut UnmanagedVector>,
) -> UnmanagedVector {
    call_2_args(
        call_ibc_packet_timeout_raw,
        cache,
        checksum,
        env,
        msg,
        db,
        api,
        querier,
        gas_limit,
        print_debug,
        gas_report,
        error_msg,
    )
}

#[no_mangle]
pub extern "C" fn ibc_source_callback(
    cache: *mut cache_t,
    checksum: ByteSliceView,
    env: ByteSliceView,
    msg: ByteSliceView,
    db: Db,
    api: GoApi,
    querier: GoQuerier,
    gas_limit: u64,
    print_debug: bool,
    gas_report: Option<&mut GasReport>,
    error_msg: Option<&mut UnmanagedVector>,
) -> UnmanagedVector {
    call_2_args(
        call_ibc_source_callback_raw,
        cache,
        checksum,
        env,
        msg,
        db,
        api,
        querier,
        gas_limit,
        print_debug,
        gas_report,
        error_msg,
    )
}

#[no_mangle]
pub extern "C" fn ibc_destination_callback(
    cache: *mut cache_t,
    checksum: ByteSliceView,
    env: ByteSliceView,
    msg: ByteSliceView,
    db: Db,
    api: GoApi,
    querier: GoQuerier,
    gas_limit: u64,
    print_debug: bool,
    gas_report: Option<&mut GasReport>,
    error_msg: Option<&mut UnmanagedVector>,
) -> UnmanagedVector {
    call_2_args(
        call_ibc_destination_callback_raw,
        cache,
        checksum,
        env,
        msg,
        db,
        api,
        querier,
        gas_limit,
        print_debug,
        gas_report,
        error_msg,
    )
}

type VmFn2Args = fn(
    instance: &mut Instance<GoApi, GoStorage, GoQuerier>,
    arg1: &[u8],
    arg2: &[u8],
) -> VmResult<Vec<u8>>;

// this wraps all error handling and ffi for the 6 ibc entry points and query.
// (all of which take env and one "msg" argument).
// the only difference is which low-level function they dispatch to.
fn call_2_args(
    vm_fn: VmFn2Args,
    cache: *mut cache_t,
    checksum: ByteSliceView,
    arg1: ByteSliceView,
    arg2: ByteSliceView,
    db: Db,
    api: GoApi,
    querier: GoQuerier,
    gas_limit: u64,
    print_debug: bool,
    gas_report: Option<&mut GasReport>,
    error_msg: Option<&mut UnmanagedVector>,
) -> UnmanagedVector {
    let r = match to_cache(cache) {
        Some(c) => catch_unwind(AssertUnwindSafe(move || {
            do_call_2_args(
                vm_fn,
                c,
                checksum,
                arg1,
                arg2,
                db,
                api,
                querier,
                gas_limit,
                print_debug,
                gas_report,
            )
        }))
        .unwrap_or_else(|err| {
            eprintln!("Panic in do_call_2_args: {err:?}");
            Err(Error::panic())
        }),
        None => Err(Error::unset_arg(CACHE_ARG)),
    };
    let data = handle_c_error_binary(r, error_msg);
    UnmanagedVector::new(Some(data))
}

// this is internal processing, same for all the 6 ibc entry points
fn do_call_2_args(
    vm_fn: VmFn2Args,
    cache: &mut Cache<GoApi, GoStorage, GoQuerier>,
    checksum: ByteSliceView,
    arg1: ByteSliceView,
    arg2: ByteSliceView,
    db: Db,
    api: GoApi,
    querier: GoQuerier,
    gas_limit: u64,
    print_debug: bool,
    gas_report: Option<&mut GasReport>,
) -> Result<Vec<u8>, Error> {
    let gas_report = gas_report.ok_or_else(|| Error::empty_arg(GAS_REPORT_ARG))?;
    let checksum: Checksum = checksum
        .read()
        .ok_or_else(|| Error::unset_arg(CHECKSUM_ARG))?
        .try_into()?;
    let arg1 = arg1.read().ok_or_else(|| Error::unset_arg(ARG1))?;
    let arg2 = arg2.read().ok_or_else(|| Error::unset_arg(ARG2))?;

    let backend = into_backend(db, api, querier);
    let options = InstanceOptions { gas_limit };
    let mut instance: Instance<GoApi, GoStorage, GoQuerier> =
        cache.get_instance(&checksum, backend, options)?;

    // If print_debug = false, use default debug handler from cosmwasm-vm, which discards messages
    if print_debug {
        instance.set_debug_handler(|msg, info| {
            let t = now_rfc3339();
            let gas = info.gas_remaining;
            eprintln!("[{t}]: {msg} (gas remaining: {gas})");
        });
    }

    // We only check this result after reporting gas usage and returning the instance into the cache.
    let res = vm_fn(&mut instance, arg1, arg2);
    *gas_report = instance.create_gas_report().into();
    Ok(res?)
}

type VmFn3Args = fn(
    instance: &mut Instance<GoApi, GoStorage, GoQuerier>,
    arg1: &[u8],
    arg2: &[u8],
    arg3: &[u8],
) -> VmResult<Vec<u8>>;

// This wraps all error handling and ffi for instantiate, execute and migrate
// (and anything else that takes env, info and msg arguments).
// The only difference is which low-level function they dispatch to.
fn call_3_args(
    vm_fn: VmFn3Args,
    cache: *mut cache_t,
    checksum: ByteSliceView,
    arg1: ByteSliceView,
    arg2: ByteSliceView,
    arg3: ByteSliceView,
    db: Db,
    api: GoApi,
    querier: GoQuerier,
    gas_limit: u64,
    print_debug: bool,
    gas_report: Option<&mut GasReport>,
    error_msg: Option<&mut UnmanagedVector>,
) -> UnmanagedVector {
    let r = match to_cache(cache) {
        Some(c) => catch_unwind(AssertUnwindSafe(move || {
            do_call_3_args(
                vm_fn,
                c,
                checksum,
                arg1,
                arg2,
                arg3,
                db,
                api,
                querier,
                gas_limit,
                print_debug,
                gas_report,
            )
        }))
        .unwrap_or_else(|err| {
            eprintln!("Panic in do_call_3_args: {err:?}");
            Err(Error::panic())
        }),
        None => Err(Error::unset_arg(CACHE_ARG)),
    };
    let data = handle_c_error_binary(r, error_msg);
    UnmanagedVector::new(Some(data))
}

fn do_call_3_args(
    vm_fn: VmFn3Args,
    cache: &mut Cache<GoApi, GoStorage, GoQuerier>,
    checksum: ByteSliceView,
    arg1: ByteSliceView,
    arg2: ByteSliceView,
    arg3: ByteSliceView,
    db: Db,
    api: GoApi,
    querier: GoQuerier,
    gas_limit: u64,
    print_debug: bool,
    gas_report: Option<&mut GasReport>,
) -> Result<Vec<u8>, Error> {
    let gas_report = gas_report.ok_or_else(|| Error::empty_arg(GAS_REPORT_ARG))?;
    let checksum: Checksum = checksum
        .read()
        .ok_or_else(|| Error::unset_arg(CHECKSUM_ARG))?
        .try_into()?;
    let arg1 = arg1.read().ok_or_else(|| Error::unset_arg(ARG1))?;
    let arg2 = arg2.read().ok_or_else(|| Error::unset_arg(ARG2))?;
    let arg3 = arg3.read().ok_or_else(|| Error::unset_arg(ARG3))?;

    let backend = into_backend(db, api, querier);
    let options = InstanceOptions { gas_limit };
    let mut instance = cache.get_instance(&checksum, backend, options)?;

    // If print_debug = false, use default debug handler from cosmwasm-vm, which discards messages
    if print_debug {
        instance.set_debug_handler(|msg, info| {
            let t = now_rfc3339();
            let gas = info.gas_remaining;
            eprintln!("[{t}]: {msg} (gas remaining: {gas})");
        });
    }

    // We only check this result after reporting gas usage and returning the instance into the cache.
    let res = vm_fn(&mut instance, arg1, arg2, arg3);
    *gas_report = instance.create_gas_report().into();
    Ok(res?)
}

fn now_rfc3339() -> String {
    let dt = OffsetDateTime::from(SystemTime::now());
    dt.format(&Rfc3339).unwrap_or_default()
}
