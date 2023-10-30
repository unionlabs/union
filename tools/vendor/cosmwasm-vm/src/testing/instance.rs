//! This file has some helpers for integration tests.
//! They should be imported via full path to ensure there is no confusion
//! use cosmwasm_vm::testing::X
use cosmwasm_std::Coin;
use std::collections::HashSet;

use crate::capabilities::capabilities_from_csv;
use crate::compatibility::check_wasm;
use crate::instance::{Instance, InstanceOptions};
use crate::size::Size;
use crate::{Backend, BackendApi, Querier, Storage};

use super::mock::{MockApi, MOCK_CONTRACT_ADDR};
use super::querier::MockQuerier;
use super::storage::MockStorage;

/// This gas limit is used in integration tests and should be high enough to allow a reasonable
/// number of contract executions and queries on one instance. For this reason it is significatly
/// higher than the limit for a single execution that we have in the production setup.
const DEFAULT_GAS_LIMIT: u64 = 500_000_000_000; // ~0.5ms
const DEFAULT_MEMORY_LIMIT: Option<Size> = Some(Size::mebi(16));
const DEFAULT_PRINT_DEBUG: bool = true;

pub fn mock_instance(
    wasm: &[u8],
    contract_balance: &[Coin],
) -> Instance<MockApi, MockStorage, MockQuerier> {
    mock_instance_with_options(
        wasm,
        MockInstanceOptions {
            contract_balance: Some(contract_balance),
            ..Default::default()
        },
    )
}

pub fn mock_instance_with_failing_api(
    wasm: &[u8],
    contract_balance: &[Coin],
    backend_error: &'static str,
) -> Instance<MockApi, MockStorage, MockQuerier> {
    mock_instance_with_options(
        wasm,
        MockInstanceOptions {
            contract_balance: Some(contract_balance),
            backend_error: Some(backend_error),
            ..Default::default()
        },
    )
}

pub fn mock_instance_with_balances(
    wasm: &[u8],
    balances: &[(&str, &[Coin])],
) -> Instance<MockApi, MockStorage, MockQuerier> {
    mock_instance_with_options(
        wasm,
        MockInstanceOptions {
            balances,
            ..Default::default()
        },
    )
}

/// Creates an instance from the given Wasm bytecode.
/// The gas limit is measured in [CosmWasm gas](https://github.com/CosmWasm/cosmwasm/blob/main/docs/GAS.md).
pub fn mock_instance_with_gas_limit(
    wasm: &[u8],
    gas_limit: u64,
) -> Instance<MockApi, MockStorage, MockQuerier> {
    mock_instance_with_options(
        wasm,
        MockInstanceOptions {
            gas_limit,
            ..Default::default()
        },
    )
}

#[derive(Debug)]
pub struct MockInstanceOptions<'a> {
    // dependencies
    pub balances: &'a [(&'a str, &'a [Coin])],
    /// This option is merged into balances and might override an existing value
    pub contract_balance: Option<&'a [Coin]>,
    /// When set, all calls to the API fail with BackendError::Unknown containing this message
    pub backend_error: Option<&'static str>,

    // instance
    pub available_capabilities: HashSet<String>,
    /// Gas limit measured in [CosmWasm gas](https://github.com/CosmWasm/cosmwasm/blob/main/docs/GAS.md).
    pub gas_limit: u64,
    pub print_debug: bool,
    /// Memory limit in bytes. Use a value that is divisible by the Wasm page size 65536, e.g. full MiBs.
    pub memory_limit: Option<Size>,
}

impl MockInstanceOptions<'_> {
    fn default_capabilities() -> HashSet<String> {
        #[allow(unused_mut)]
        let mut out =
            capabilities_from_csv("iterator,staking,cosmwasm_1_1,cosmwasm_1_2,cosmwasm_1_3");
        #[cfg(feature = "stargate")]
        out.insert("stargate".to_string());
        out
    }
}

impl Default for MockInstanceOptions<'_> {
    fn default() -> Self {
        Self {
            // dependencies
            balances: Default::default(),
            contract_balance: Default::default(),
            backend_error: None,

            // instance
            available_capabilities: Self::default_capabilities(),
            gas_limit: DEFAULT_GAS_LIMIT,
            print_debug: DEFAULT_PRINT_DEBUG,
            memory_limit: DEFAULT_MEMORY_LIMIT,
        }
    }
}

pub fn mock_instance_with_options(
    wasm: &[u8],
    options: MockInstanceOptions,
) -> Instance<MockApi, MockStorage, MockQuerier> {
    check_wasm(wasm, &options.available_capabilities).unwrap();
    let contract_address = MOCK_CONTRACT_ADDR;

    // merge balances
    let mut balances = options.balances.to_vec();
    if let Some(contract_balance) = options.contract_balance {
        // Remove old entry if exists
        if let Some(pos) = balances.iter().position(|item| item.0 == contract_address) {
            balances.remove(pos);
        }
        balances.push((contract_address, contract_balance));
    }

    let api = if let Some(backend_error) = options.backend_error {
        MockApi::new_failing(backend_error)
    } else {
        MockApi::default()
    };

    let backend = Backend {
        api,
        storage: MockStorage::default(),
        querier: MockQuerier::new(&balances),
    };
    let memory_limit = options.memory_limit;
    let options = InstanceOptions {
        gas_limit: options.gas_limit,
        print_debug: options.print_debug,
    };
    Instance::from_code(wasm, backend, options, memory_limit).unwrap()
}

/// Creates InstanceOptions for testing
pub fn mock_instance_options() -> (InstanceOptions, Option<Size>) {
    (
        InstanceOptions {
            gas_limit: DEFAULT_GAS_LIMIT,
            print_debug: DEFAULT_PRINT_DEBUG,
        },
        DEFAULT_MEMORY_LIMIT,
    )
}

/// Runs a series of IO tests, hammering especially on allocate and deallocate.
/// This could be especially useful when run with some kind of leak detector.
pub fn test_io<A, S, Q>(instance: &mut Instance<A, S, Q>)
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    let sizes: Vec<usize> = vec![0, 1, 3, 10, 200, 2000, 5 * 1024];
    let bytes: Vec<u8> = vec![0x00, 0xA5, 0xFF];

    for size in sizes.into_iter() {
        for byte in bytes.iter() {
            let original = vec![*byte; size];
            let wasm_ptr = instance
                .allocate(original.len())
                .expect("Could not allocate memory");
            instance
                .write_memory(wasm_ptr, &original)
                .expect("Could not write data");
            let wasm_data = instance.read_memory(wasm_ptr, size).expect("error reading");
            assert_eq!(
                original, wasm_data,
                "failed for size {}; expected: {:?}; actual: {:?}",
                size, original, wasm_data
            );
            instance
                .deallocate(wasm_ptr)
                .expect("Could not deallocate memory");
        }
    }
}
