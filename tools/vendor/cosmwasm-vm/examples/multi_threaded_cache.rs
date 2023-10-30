use std::sync::Arc;
use std::thread;
use tempfile::TempDir;

use cosmwasm_std::{coins, Empty};
use cosmwasm_vm::testing::{mock_backend, mock_env, mock_info, MockApi, MockQuerier, MockStorage};
use cosmwasm_vm::{
    call_execute, call_instantiate, capabilities_from_csv, Cache, CacheOptions, InstanceOptions,
    Size,
};

// Instance
const DEFAULT_MEMORY_LIMIT: Size = Size::mebi(64);
const DEFAULT_GAS_LIMIT: u64 = 400_000 * 150_000;
const DEFAULT_INSTANCE_OPTIONS: InstanceOptions = InstanceOptions {
    gas_limit: DEFAULT_GAS_LIMIT,
    print_debug: false,
};
// Cache
const MEMORY_CACHE_SIZE: Size = Size::mebi(200);

static CONTRACT: &[u8] = include_bytes!("../testdata/hackatom.wasm");

const SAVE_WASM_THREADS: usize = 32;
const INSTANTIATION_THREADS: usize = 2048;
const THREADS: usize = SAVE_WASM_THREADS + INSTANTIATION_THREADS;

pub fn main() {
    let options = CacheOptions {
        base_dir: TempDir::new().unwrap().into_path(),
        available_capabilities: capabilities_from_csv("iterator,staking"),
        memory_cache_size: MEMORY_CACHE_SIZE,
        instance_memory_limit: DEFAULT_MEMORY_LIMIT,
    };

    let cache: Cache<MockApi, MockStorage, MockQuerier> = unsafe { Cache::new(options).unwrap() };
    let cache = Arc::new(cache);

    let checksum = cache.save_wasm(CONTRACT).unwrap();

    let mut threads = Vec::with_capacity(THREADS);
    for _ in 0..SAVE_WASM_THREADS {
        let cache = Arc::clone(&cache);

        threads.push(thread::spawn(move || {
            let checksum = cache.save_wasm(CONTRACT).unwrap();
            println!("Done saving Wasm {}", checksum);
        }));
    }
    for _ in 0..INSTANTIATION_THREADS {
        let cache = Arc::clone(&cache);

        threads.push(thread::spawn(move || {
            let checksum = checksum;
            let mut instance = cache
                .get_instance(&checksum, mock_backend(&[]), DEFAULT_INSTANCE_OPTIONS)
                .unwrap();
            println!("Done instantiating contract");

            let info = mock_info("creator", &coins(1000, "earth"));
            let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
            let contract_result =
                call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
            assert!(contract_result.into_result().is_ok());

            let info = mock_info("verifies", &coins(15, "earth"));
            let msg = br#"{"release":{}}"#;
            let contract_result =
                call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
            assert!(contract_result.into_result().is_ok());
        }));
    }

    threads.into_iter().for_each(|thread| {
        thread
            .join()
            .expect("The thread creating or execution failed !")
    });

    assert_eq!(cache.stats().misses, 0);
    assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
    assert_eq!(
        cache.stats().hits_memory_cache,
        INSTANTIATION_THREADS as u32 - 1
    );
    assert_eq!(cache.stats().hits_fs_cache, 1);
}
