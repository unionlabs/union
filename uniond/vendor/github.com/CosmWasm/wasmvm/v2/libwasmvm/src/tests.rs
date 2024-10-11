#![cfg(test)]

use tempfile::TempDir;

use cosmwasm_vm::testing::{mock_backend, mock_env, mock_info, mock_instance_with_gas_limit};
use cosmwasm_vm::{
    call_execute_raw, call_instantiate_raw, capabilities_from_csv, to_vec, Cache, CacheOptions,
    InstanceOptions, Size,
};

static CYBERPUNK: &[u8] = include_bytes!("../../testdata/cyberpunk.wasm");
const MEMORY_CACHE_SIZE: Size = Size::mebi(200);
const MEMORY_LIMIT: Size = Size::mebi(32);
const GAS_LIMIT: u64 = 200_000_000_000; // ~0.2ms

#[test]
fn handle_cpu_loop_with_cache() {
    let backend = mock_backend(&[]);
    let options = CacheOptions::new(
        TempDir::new().unwrap().path().to_path_buf(),
        capabilities_from_csv("staking"),
        MEMORY_CACHE_SIZE,
        MEMORY_LIMIT,
    );
    let cache = unsafe { Cache::new(options) }.unwrap();

    let options = InstanceOptions {
        gas_limit: GAS_LIMIT,
    };

    // store code
    let checksum = cache.save_wasm(CYBERPUNK).unwrap();

    // instantiate
    let env = mock_env();
    let info = mock_info("creator", &[]);
    let mut instance = cache.get_instance(&checksum, backend, options).unwrap();
    let raw_env = to_vec(&env).unwrap();
    let raw_info = to_vec(&info).unwrap();
    let res = call_instantiate_raw(&mut instance, &raw_env, &raw_info, b"{}");
    let gas_left = instance.get_gas_left();
    let gas_used = options.gas_limit - gas_left;
    println!("Init gas left: {gas_left}, used: {gas_used}");
    assert!(res.is_ok());
    let backend = instance.recycle().unwrap();

    // execute
    let mut instance = cache.get_instance(&checksum, backend, options).unwrap();
    let raw_msg = br#"{"cpu_loop":{}}"#;
    let res = call_execute_raw(&mut instance, &raw_env, &raw_info, raw_msg);
    let gas_left = instance.get_gas_left();
    let gas_used = options.gas_limit - gas_left;
    println!("Handle gas left: {gas_left}, used: {gas_used}");
    assert!(res.is_err());
    assert_eq!(gas_left, 0);
    let _ = instance.recycle();
}

#[test]
fn handle_cpu_loop_no_cache() {
    let gas_limit = GAS_LIMIT;
    let mut instance = mock_instance_with_gas_limit(CYBERPUNK, gas_limit);

    // instantiate
    let env = mock_env();
    let info = mock_info("creator", &[]);
    let raw_env = to_vec(&env).unwrap();
    let raw_info = to_vec(&info).unwrap();
    let res = call_instantiate_raw(&mut instance, &raw_env, &raw_info, b"{}");
    let gas_left = instance.get_gas_left();
    let gas_used = gas_limit - gas_left;
    println!("Init gas left: {gas_left}, used: {gas_used}");
    assert!(res.is_ok());

    // execute
    let raw_msg = br#"{"cpu_loop":{}}"#;
    let res = call_execute_raw(&mut instance, &raw_env, &raw_info, raw_msg);
    let gas_left = instance.get_gas_left();
    let gas_used = gas_limit - gas_left;
    println!("Handle gas left: {gas_left}, used: {gas_used}");
    assert!(res.is_err());
    assert_eq!(gas_left, 0);
}
