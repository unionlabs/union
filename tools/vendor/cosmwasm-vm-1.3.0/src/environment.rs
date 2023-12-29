//! Internal details to be used by instance.rs only
use std::borrow::{Borrow, BorrowMut};
use std::ptr::NonNull;
use std::sync::{Arc, RwLock};

use wasmer::{HostEnvInitError, Instance as WasmerInstance, Memory, Val, WasmerEnv};
use wasmer_middlewares::metering::{get_remaining_points, set_remaining_points, MeteringPoints};

use crate::backend::{BackendApi, GasInfo, Querier, Storage};
use crate::errors::{VmError, VmResult};

/// Keep this as low as necessary to avoid deepy nested errors like this:
///
/// ```plain
/// RuntimeErr { msg: "Wasmer runtime error: RuntimeError: Error executing Wasm: Wasmer runtime error: RuntimeError: Error executing Wasm: Wasmer runtime error: RuntimeError: Error executing Wasm: Wasmer runtime error: RuntimeError: Error executing Wasm: Wasmer runtime error: RuntimeError: Maximum call depth exceeded." }
/// ```
const MAX_CALL_DEPTH: usize = 2;

/// Never can never be instantiated.
/// Replace this with the [never primitive type](https://doc.rust-lang.org/std/primitive.never.html) when stable.
#[derive(Debug)]
pub enum Never {}

/** gas config data */

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GasConfig {
    /// Gas costs of VM (not Backend) provided functionality
    /// secp256k1 signature verification cost
    pub secp256k1_verify_cost: u64,
    /// secp256k1 public key recovery cost
    pub secp256k1_recover_pubkey_cost: u64,
    /// ed25519 signature verification cost
    pub ed25519_verify_cost: u64,
    /// ed25519 batch signature verification cost
    pub ed25519_batch_verify_cost: u64,
    /// ed25519 batch signature verification cost (single public key)
    pub ed25519_batch_verify_one_pubkey_cost: u64,
}

impl Default for GasConfig {
    fn default() -> Self {
        // Target is 10^12 per millisecond (see GAS.md), i.e. 10^9 gas per µ second.
        const GAS_PER_US: u64 = 1_000_000_000;
        Self {
            // ~154 us in crypto benchmarks
            secp256k1_verify_cost: 154 * GAS_PER_US,
            // ~162 us in crypto benchmarks
            secp256k1_recover_pubkey_cost: 162 * GAS_PER_US,
            // ~63 us in crypto benchmarks
            ed25519_verify_cost: 63 * GAS_PER_US,
            // Gas cost factors, relative to ed25519_verify cost
            // From https://docs.rs/ed25519-zebra/2.2.0/ed25519_zebra/batch/index.html
            ed25519_batch_verify_cost: 63 * GAS_PER_US / 2,
            ed25519_batch_verify_one_pubkey_cost: 63 * GAS_PER_US / 4,
        }
    }
}

/** context data **/

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct GasState {
    /// Gas limit for the computation, including internally and externally used gas.
    /// This is set when the Environment is created and never mutated.
    ///
    /// Measured in [CosmWasm gas](https://github.com/CosmWasm/cosmwasm/blob/main/docs/GAS.md).
    pub gas_limit: u64,
    /// Tracking the gas used in the Cosmos SDK, in CosmWasm gas units.
    pub externally_used_gas: u64,
}

impl GasState {
    fn with_limit(gas_limit: u64) -> Self {
        Self {
            gas_limit,
            externally_used_gas: 0,
        }
    }
}

/// A environment that provides access to the ContextData.
/// The environment is clonable but clones access the same underlying data.
pub struct Environment<A: BackendApi, S: Storage, Q: Querier> {
    pub api: A,
    pub print_debug: bool,
    pub gas_config: GasConfig,
    data: Arc<RwLock<ContextData<S, Q>>>,
}

unsafe impl<A: BackendApi, S: Storage, Q: Querier> Send for Environment<A, S, Q> {}

unsafe impl<A: BackendApi, S: Storage, Q: Querier> Sync for Environment<A, S, Q> {}

impl<A: BackendApi, S: Storage, Q: Querier> Clone for Environment<A, S, Q> {
    fn clone(&self) -> Self {
        Environment {
            api: self.api,
            print_debug: self.print_debug,
            gas_config: self.gas_config.clone(),
            data: self.data.clone(),
        }
    }
}

impl<A: BackendApi, S: Storage, Q: Querier> WasmerEnv for Environment<A, S, Q> {
    fn init_with_instance(&mut self, _instance: &WasmerInstance) -> Result<(), HostEnvInitError> {
        Ok(())
    }
}

impl<A: BackendApi, S: Storage, Q: Querier> Environment<A, S, Q> {
    pub fn new(api: A, gas_limit: u64, print_debug: bool) -> Self {
        Environment {
            api,
            print_debug,
            gas_config: GasConfig::default(),
            data: Arc::new(RwLock::new(ContextData::new(gas_limit))),
        }
    }

    fn with_context_data_mut<C, R>(&self, callback: C) -> R
    where
        C: FnOnce(&mut ContextData<S, Q>) -> R,
    {
        let mut guard = self.data.as_ref().write().unwrap();
        let context_data = guard.borrow_mut();
        callback(context_data)
    }

    fn with_context_data<C, R>(&self, callback: C) -> R
    where
        C: FnOnce(&ContextData<S, Q>) -> R,
    {
        let guard = self.data.as_ref().read().unwrap();
        let context_data = guard.borrow();
        callback(context_data)
    }

    pub fn with_gas_state<C, R>(&self, callback: C) -> R
    where
        C: FnOnce(&GasState) -> R,
    {
        self.with_context_data(|context_data| callback(&context_data.gas_state))
    }

    pub fn with_gas_state_mut<C, R>(&self, callback: C) -> R
    where
        C: FnOnce(&mut GasState) -> R,
    {
        self.with_context_data_mut(|context_data| callback(&mut context_data.gas_state))
    }

    pub fn with_wasmer_instance<C, R>(&self, callback: C) -> VmResult<R>
    where
        C: FnOnce(&WasmerInstance) -> VmResult<R>,
    {
        self.with_context_data(|context_data| match context_data.wasmer_instance {
            Some(instance_ptr) => {
                let instance_ref = unsafe { instance_ptr.as_ref() };
                callback(instance_ref)
            }
            None => Err(VmError::uninitialized_context_data("wasmer_instance")),
        })
    }

    /// Calls a function with the given name and arguments.
    /// The number of return values is variable and controlled by the guest.
    /// Usually we expect 0 or 1 return values. Use [`Self::call_function0`]
    /// or [`Self::call_function1`] to ensure the number of return values is checked.
    fn call_function(&self, name: &str, args: &[Val]) -> VmResult<Box<[Val]>> {
        // Clone function before calling it to avoid dead locks
        let func = self.with_wasmer_instance(|instance| {
            let func = instance.exports.get_function(name)?;
            Ok(func.clone())
        })?;
        self.increment_call_depth()?;
        let res = func.call(args).map_err(|runtime_err| -> VmError {
            self.with_wasmer_instance::<_, Never>(|instance| {
                let err: VmError = match get_remaining_points(instance) {
                    MeteringPoints::Remaining(_) => VmError::from(runtime_err),
                    MeteringPoints::Exhausted => VmError::gas_depletion(),
                };
                Err(err)
            })
            .unwrap_err() // with_wasmer_instance can only succeed if the callback succeeds
        });
        self.decrement_call_depth();
        res
    }

    pub fn call_function0(&self, name: &str, args: &[Val]) -> VmResult<()> {
        let result = self.call_function(name, args)?;
        let expected = 0;
        let actual = result.len();
        if actual != expected {
            return Err(VmError::result_mismatch(name, expected, actual));
        }
        Ok(())
    }

    pub fn call_function1(&self, name: &str, args: &[Val]) -> VmResult<Val> {
        let result = self.call_function(name, args)?;
        let expected = 1;
        let actual = result.len();
        if actual != expected {
            return Err(VmError::result_mismatch(name, expected, actual));
        }
        Ok(result[0].clone())
    }

    pub fn with_storage_from_context<C, T>(&self, callback: C) -> VmResult<T>
    where
        C: FnOnce(&mut S) -> VmResult<T>,
    {
        self.with_context_data_mut(|context_data| match context_data.storage.as_mut() {
            Some(data) => callback(data),
            None => Err(VmError::uninitialized_context_data("storage")),
        })
    }

    pub fn with_querier_from_context<C, T>(&self, callback: C) -> VmResult<T>
    where
        C: FnOnce(&mut Q) -> VmResult<T>,
    {
        self.with_context_data_mut(|context_data| match context_data.querier.as_mut() {
            Some(querier) => callback(querier),
            None => Err(VmError::uninitialized_context_data("querier")),
        })
    }

    /// Creates a back reference from a contact to its partent instance
    pub fn set_wasmer_instance(&self, wasmer_instance: Option<NonNull<WasmerInstance>>) {
        self.with_context_data_mut(|context_data| {
            context_data.wasmer_instance = wasmer_instance;
        });
    }

    /// Returns true iff the storage is set to readonly mode
    pub fn is_storage_readonly(&self) -> bool {
        self.with_context_data(|context_data| context_data.storage_readonly)
    }

    pub fn set_storage_readonly(&self, new_value: bool) {
        self.with_context_data_mut(|context_data| {
            context_data.storage_readonly = new_value;
        })
    }

    /// Increments the call depth by 1 and returns the new value
    pub fn increment_call_depth(&self) -> VmResult<usize> {
        let new = self.with_context_data_mut(|context_data| {
            let new = context_data.call_depth + 1;
            context_data.call_depth = new;
            new
        });
        if new > MAX_CALL_DEPTH {
            return Err(VmError::max_call_depth_exceeded());
        }
        Ok(new)
    }

    /// Decrements the call depth by 1 and returns the new value
    pub fn decrement_call_depth(&self) -> usize {
        self.with_context_data_mut(|context_data| {
            let new = context_data
                .call_depth
                .checked_sub(1)
                .expect("Call depth < 0. This is a bug.");
            context_data.call_depth = new;
            new
        })
    }

    pub fn get_gas_left(&self) -> u64 {
        self.with_wasmer_instance(|instance| {
            Ok(match get_remaining_points(instance) {
                MeteringPoints::Remaining(count) => count,
                MeteringPoints::Exhausted => 0,
            })
        })
        .expect("Wasmer instance is not set. This is a bug in the lifecycle.")
    }

    pub fn set_gas_left(&self, new_value: u64) {
        self.with_wasmer_instance(|instance| {
            set_remaining_points(instance, new_value);
            Ok(())
        })
        .expect("Wasmer instance is not set. This is a bug in the lifecycle.")
    }

    /// Decreases gas left by the given amount.
    /// If the amount exceeds the available gas, the remaining gas is set to 0 and
    /// an VmError::GasDepletion error is returned.
    #[allow(unused)] // used in tests
    pub fn decrease_gas_left(&self, amount: u64) -> VmResult<()> {
        self.with_wasmer_instance(|instance| {
            let remaining = match get_remaining_points(instance) {
                MeteringPoints::Remaining(count) => count,
                MeteringPoints::Exhausted => 0,
            };
            if amount > remaining {
                set_remaining_points(instance, 0);
                Err(VmError::gas_depletion())
            } else {
                set_remaining_points(instance, remaining - amount);
                Ok(())
            }
        })
    }

    pub fn memory(&self) -> Memory {
        self.with_wasmer_instance(|instance| {
            let first: Option<Memory> = instance
                .exports
                .iter()
                .memories()
                .next()
                .map(|pair| pair.1.clone());
            // Every contract in CosmWasm must have exactly one exported memory.
            // This is ensured by `check_wasm`/`check_wasm_memories`, which is called for every
            // contract added to the Cache as well as in integration tests.
            // It is possible to bypass this check when using `Instance::from_code` but then you
            // learn the hard way when this panics, or when trying to upload the contract to chain.
            let memory = first.expect("A contract must have exactly one exported memory.");
            Ok(memory)
        })
        .expect("Wasmer instance is not set. This is a bug in the lifecycle.")
    }

    /// Moves owned instances of storage and querier into the env.
    /// Should be followed by exactly one call to move_out when the instance is finished.
    pub fn move_in(&self, storage: S, querier: Q) {
        self.with_context_data_mut(|context_data| {
            context_data.storage = Some(storage);
            context_data.querier = Some(querier);
        });
    }

    /// Returns the original storage and querier as owned instances, and closes any remaining
    /// iterators. This is meant to be called when recycling the instance.
    pub fn move_out(&self) -> (Option<S>, Option<Q>) {
        self.with_context_data_mut(|context_data| {
            (context_data.storage.take(), context_data.querier.take())
        })
    }
}

pub struct ContextData<S: Storage, Q: Querier> {
    gas_state: GasState,
    storage: Option<S>,
    storage_readonly: bool,
    call_depth: usize,
    querier: Option<Q>,
    /// A non-owning link to the wasmer instance
    wasmer_instance: Option<NonNull<WasmerInstance>>,
}

impl<S: Storage, Q: Querier> ContextData<S, Q> {
    pub fn new(gas_limit: u64) -> Self {
        ContextData::<S, Q> {
            gas_state: GasState::with_limit(gas_limit),
            storage: None,
            storage_readonly: true,
            call_depth: 0,
            querier: None,
            wasmer_instance: None,
        }
    }
}

pub fn process_gas_info<A: BackendApi, S: Storage, Q: Querier>(
    env: &Environment<A, S, Q>,
    info: GasInfo,
) -> VmResult<()> {
    let gas_left = env.get_gas_left();

    let new_limit = env.with_gas_state_mut(|gas_state| {
        gas_state.externally_used_gas += info.externally_used;
        // These lines reduce the amount of gas available to wasmer
        // so it can not consume gas that was consumed externally.
        gas_left
            .saturating_sub(info.externally_used)
            .saturating_sub(info.cost)
    });

    // This tells wasmer how much more gas it can consume from this point in time.
    env.set_gas_left(new_limit);

    if info.externally_used + info.cost > gas_left {
        Err(VmError::gas_depletion())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::Storage;
    use crate::conversion::ref_to_u32;
    use crate::errors::VmError;
    use crate::size::Size;
    use crate::testing::{MockApi, MockQuerier, MockStorage};
    use crate::wasm_backend::compile;
    use cosmwasm_std::{
        coins, from_binary, to_vec, AllBalanceResponse, BankQuery, Empty, QueryRequest,
    };
    use wasmer::{imports, Function, Instance as WasmerInstance};

    static CONTRACT: &[u8] = include_bytes!("../testdata/hackatom.wasm");

    // prepared data
    const INIT_KEY: &[u8] = b"foo";
    const INIT_VALUE: &[u8] = b"bar";
    // this account has some coins
    const INIT_ADDR: &str = "someone";
    const INIT_AMOUNT: u128 = 500;
    const INIT_DENOM: &str = "TOKEN";

    const TESTING_GAS_LIMIT: u64 = 500_000_000_000; // ~0.5ms
    const DEFAULT_QUERY_GAS_LIMIT: u64 = 300_000;
    const TESTING_MEMORY_LIMIT: Option<Size> = Some(Size::mebi(16));

    fn make_instance(
        gas_limit: u64,
    ) -> (
        Environment<MockApi, MockStorage, MockQuerier>,
        Box<WasmerInstance>,
    ) {
        let env = Environment::new(MockApi::default(), gas_limit, false);

        let module = compile(CONTRACT, TESTING_MEMORY_LIMIT, &[]).unwrap();
        let store = module.store();
        // we need stubs for all required imports
        let import_obj = imports! {
            "env" => {
                "db_read" => Function::new_native(store, |_a: u32| -> u32 { 0 }),
                "db_write" => Function::new_native(store, |_a: u32, _b: u32| {}),
                "db_remove" => Function::new_native(store, |_a: u32| {}),
                "db_scan" => Function::new_native(store, |_a: u32, _b: u32, _c: i32| -> u32 { 0 }),
                "db_next" => Function::new_native(store, |_a: u32| -> u32 { 0 }),
                "query_chain" => Function::new_native(store, |_a: u32| -> u32 { 0 }),
                "addr_validate" => Function::new_native(store, |_a: u32| -> u32 { 0 }),
                "addr_canonicalize" => Function::new_native(store, |_a: u32, _b: u32| -> u32 { 0 }),
                "addr_humanize" => Function::new_native(store, |_a: u32, _b: u32| -> u32 { 0 }),
                "secp256k1_verify" => Function::new_native(store, |_a: u32, _b: u32, _c: u32| -> u32 { 0 }),
                "secp256k1_recover_pubkey" => Function::new_native(store, |_a: u32, _b: u32, _c: u32| -> u64 { 0 }),
                "ed25519_verify" => Function::new_native(store, |_a: u32, _b: u32, _c: u32| -> u32 { 0 }),
                "ed25519_batch_verify" => Function::new_native(store, |_a: u32, _b: u32, _c: u32| -> u32 { 0 }),
                "debug" => Function::new_native(store, |_a: u32| {}),
                "abort" => Function::new_native(store, |_a: u32| {}),
            },
        };
        let instance = Box::from(WasmerInstance::new(&module, &import_obj).unwrap());

        let instance_ptr = NonNull::from(instance.as_ref());
        env.set_wasmer_instance(Some(instance_ptr));
        env.set_gas_left(gas_limit);

        (env, instance)
    }

    fn leave_default_data(env: &Environment<MockApi, MockStorage, MockQuerier>) {
        // create some mock data
        let mut storage = MockStorage::new();
        storage
            .set(INIT_KEY, INIT_VALUE)
            .0
            .expect("error setting value");
        let querier: MockQuerier<Empty> =
            MockQuerier::new(&[(INIT_ADDR, &coins(INIT_AMOUNT, INIT_DENOM))]);
        env.move_in(storage, querier);
    }

    #[test]
    fn move_out_works() {
        let (env, _instance) = make_instance(TESTING_GAS_LIMIT);

        // empty data on start
        let (inits, initq) = env.move_out();
        assert!(inits.is_none());
        assert!(initq.is_none());

        // store it on the instance
        leave_default_data(&env);
        let (s, q) = env.move_out();
        assert!(s.is_some());
        assert!(q.is_some());
        assert_eq!(
            s.unwrap().get(INIT_KEY).0.unwrap(),
            Some(INIT_VALUE.to_vec())
        );

        // now is empty again
        let (ends, endq) = env.move_out();
        assert!(ends.is_none());
        assert!(endq.is_none());
    }

    #[test]
    fn process_gas_info_works_for_cost() {
        let (env, _instance) = make_instance(100);
        assert_eq!(env.get_gas_left(), 100);

        // Consume all the Gas that we allocated
        process_gas_info(&env, GasInfo::with_cost(70)).unwrap();
        assert_eq!(env.get_gas_left(), 30);
        process_gas_info(&env, GasInfo::with_cost(4)).unwrap();
        assert_eq!(env.get_gas_left(), 26);
        process_gas_info(&env, GasInfo::with_cost(6)).unwrap();
        assert_eq!(env.get_gas_left(), 20);
        process_gas_info(&env, GasInfo::with_cost(20)).unwrap();
        assert_eq!(env.get_gas_left(), 0);

        // Using one more unit of gas triggers a failure
        match process_gas_info(&env, GasInfo::with_cost(1)).unwrap_err() {
            VmError::GasDepletion { .. } => {}
            err => panic!("unexpected error: {:?}", err),
        }
    }

    #[test]
    fn process_gas_info_works_for_externally_used() {
        let (env, _instance) = make_instance(100);
        assert_eq!(env.get_gas_left(), 100);

        // Consume all the Gas that we allocated
        process_gas_info(&env, GasInfo::with_externally_used(70)).unwrap();
        assert_eq!(env.get_gas_left(), 30);
        process_gas_info(&env, GasInfo::with_externally_used(4)).unwrap();
        assert_eq!(env.get_gas_left(), 26);
        process_gas_info(&env, GasInfo::with_externally_used(6)).unwrap();
        assert_eq!(env.get_gas_left(), 20);
        process_gas_info(&env, GasInfo::with_externally_used(20)).unwrap();
        assert_eq!(env.get_gas_left(), 0);

        // Using one more unit of gas triggers a failure
        match process_gas_info(&env, GasInfo::with_externally_used(1)).unwrap_err() {
            VmError::GasDepletion { .. } => {}
            err => panic!("unexpected error: {:?}", err),
        }
    }

    #[test]
    fn process_gas_info_works_for_cost_and_externally_used() {
        let (env, _instance) = make_instance(100);
        assert_eq!(env.get_gas_left(), 100);
        let gas_state = env.with_gas_state(|gas_state| gas_state.clone());
        assert_eq!(gas_state.gas_limit, 100);
        assert_eq!(gas_state.externally_used_gas, 0);

        process_gas_info(&env, GasInfo::new(17, 4)).unwrap();
        assert_eq!(env.get_gas_left(), 79);
        let gas_state = env.with_gas_state(|gas_state| gas_state.clone());
        assert_eq!(gas_state.gas_limit, 100);
        assert_eq!(gas_state.externally_used_gas, 4);

        process_gas_info(&env, GasInfo::new(9, 0)).unwrap();
        assert_eq!(env.get_gas_left(), 70);
        let gas_state = env.with_gas_state(|gas_state| gas_state.clone());
        assert_eq!(gas_state.gas_limit, 100);
        assert_eq!(gas_state.externally_used_gas, 4);

        process_gas_info(&env, GasInfo::new(0, 70)).unwrap();
        assert_eq!(env.get_gas_left(), 0);
        let gas_state = env.with_gas_state(|gas_state| gas_state.clone());
        assert_eq!(gas_state.gas_limit, 100);
        assert_eq!(gas_state.externally_used_gas, 74);

        // More cost fail but do not change stats
        match process_gas_info(&env, GasInfo::new(1, 0)).unwrap_err() {
            VmError::GasDepletion { .. } => {}
            err => panic!("unexpected error: {:?}", err),
        }
        assert_eq!(env.get_gas_left(), 0);
        let gas_state = env.with_gas_state(|gas_state| gas_state.clone());
        assert_eq!(gas_state.gas_limit, 100);
        assert_eq!(gas_state.externally_used_gas, 74);

        // More externally used fails and changes stats
        match process_gas_info(&env, GasInfo::new(0, 1)).unwrap_err() {
            VmError::GasDepletion { .. } => {}
            err => panic!("unexpected error: {:?}", err),
        }
        assert_eq!(env.get_gas_left(), 0);
        let gas_state = env.with_gas_state(|gas_state| gas_state.clone());
        assert_eq!(gas_state.gas_limit, 100);
        assert_eq!(gas_state.externally_used_gas, 75);
    }

    #[test]
    fn process_gas_info_zeros_gas_left_when_exceeded() {
        // with_externally_used
        {
            let (env, _instance) = make_instance(100);
            let result = process_gas_info(&env, GasInfo::with_externally_used(120));
            match result.unwrap_err() {
                VmError::GasDepletion { .. } => {}
                err => panic!("unexpected error: {:?}", err),
            }
            assert_eq!(env.get_gas_left(), 0);
            let gas_state = env.with_gas_state(|gas_state| gas_state.clone());
            assert_eq!(gas_state.gas_limit, 100);
            assert_eq!(gas_state.externally_used_gas, 120);
        }

        // with_cost
        {
            let (env, _instance) = make_instance(100);
            let result = process_gas_info(&env, GasInfo::with_cost(120));
            match result.unwrap_err() {
                VmError::GasDepletion { .. } => {}
                err => panic!("unexpected error: {:?}", err),
            }
            assert_eq!(env.get_gas_left(), 0);
            let gas_state = env.with_gas_state(|gas_state| gas_state.clone());
            assert_eq!(gas_state.gas_limit, 100);
            assert_eq!(gas_state.externally_used_gas, 0);
        }
    }

    #[test]
    fn process_gas_info_works_correctly_with_gas_consumption_in_wasmer() {
        let (env, _instance) = make_instance(100);
        assert_eq!(env.get_gas_left(), 100);

        // Some gas was consumed externally
        process_gas_info(&env, GasInfo::with_externally_used(50)).unwrap();
        assert_eq!(env.get_gas_left(), 50);
        process_gas_info(&env, GasInfo::with_externally_used(4)).unwrap();
        assert_eq!(env.get_gas_left(), 46);

        // Consume 20 gas directly in wasmer
        env.decrease_gas_left(20).unwrap();
        assert_eq!(env.get_gas_left(), 26);

        process_gas_info(&env, GasInfo::with_externally_used(6)).unwrap();
        assert_eq!(env.get_gas_left(), 20);
        process_gas_info(&env, GasInfo::with_externally_used(20)).unwrap();
        assert_eq!(env.get_gas_left(), 0);

        // Using one more unit of gas triggers a failure
        match process_gas_info(&env, GasInfo::with_externally_used(1)).unwrap_err() {
            VmError::GasDepletion { .. } => {}
            err => panic!("unexpected error: {:?}", err),
        }
    }

    #[test]
    fn is_storage_readonly_defaults_to_true() {
        let (env, _instance) = make_instance(TESTING_GAS_LIMIT);
        leave_default_data(&env);

        assert!(env.is_storage_readonly());
    }

    #[test]
    fn set_storage_readonly_can_change_flag() {
        let (env, _instance) = make_instance(TESTING_GAS_LIMIT);
        leave_default_data(&env);

        // change
        env.set_storage_readonly(false);
        assert!(!env.is_storage_readonly());

        // still false
        env.set_storage_readonly(false);
        assert!(!env.is_storage_readonly());

        // change back
        env.set_storage_readonly(true);
        assert!(env.is_storage_readonly());
    }

    #[test]
    fn call_function_works() {
        let (env, _instance) = make_instance(TESTING_GAS_LIMIT);
        leave_default_data(&env);

        let result = env.call_function("allocate", &[10u32.into()]).unwrap();
        let ptr = ref_to_u32(&result[0]).unwrap();
        assert!(ptr > 0);
    }

    #[test]
    fn call_function_fails_for_missing_instance() {
        let (env, _instance) = make_instance(TESTING_GAS_LIMIT);
        leave_default_data(&env);

        // Clear context's wasmer_instance
        env.set_wasmer_instance(None);

        let res = env.call_function("allocate", &[]);
        match res.unwrap_err() {
            VmError::UninitializedContextData { kind, .. } => assert_eq!(kind, "wasmer_instance"),
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn call_function_fails_for_missing_function() {
        let (env, _instance) = make_instance(TESTING_GAS_LIMIT);
        leave_default_data(&env);

        let res = env.call_function("doesnt_exist", &[]);
        match res.unwrap_err() {
            VmError::ResolveErr { msg, .. } => {
                assert_eq!(msg, "Could not get export: Missing export doesnt_exist");
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn call_function0_works() {
        let (env, _instance) = make_instance(TESTING_GAS_LIMIT);
        leave_default_data(&env);

        env.call_function0("interface_version_8", &[]).unwrap();
    }

    #[test]
    fn call_function0_errors_for_wrong_result_count() {
        let (env, _instance) = make_instance(TESTING_GAS_LIMIT);
        leave_default_data(&env);

        let result = env.call_function0("allocate", &[10u32.into()]);
        match result.unwrap_err() {
            VmError::ResultMismatch {
                function_name,
                expected,
                actual,
                ..
            } => {
                assert_eq!(function_name, "allocate");
                assert_eq!(expected, 0);
                assert_eq!(actual, 1);
            }
            err => panic!("unexpected error: {:?}", err),
        }
    }

    #[test]
    fn call_function1_works() {
        let (env, _instance) = make_instance(TESTING_GAS_LIMIT);
        leave_default_data(&env);

        let result = env.call_function1("allocate", &[10u32.into()]).unwrap();
        let ptr = ref_to_u32(&result).unwrap();
        assert!(ptr > 0);
    }

    #[test]
    fn call_function1_errors_for_wrong_result_count() {
        let (env, _instance) = make_instance(TESTING_GAS_LIMIT);
        leave_default_data(&env);

        let result = env.call_function1("allocate", &[10u32.into()]).unwrap();
        let ptr = ref_to_u32(&result).unwrap();
        assert!(ptr > 0);

        let result = env.call_function1("deallocate", &[ptr.into()]);
        match result.unwrap_err() {
            VmError::ResultMismatch {
                function_name,
                expected,
                actual,
                ..
            } => {
                assert_eq!(function_name, "deallocate");
                assert_eq!(expected, 1);
                assert_eq!(actual, 0);
            }
            err => panic!("unexpected error: {:?}", err),
        }
    }

    #[test]
    fn with_storage_from_context_set_get() {
        let (env, _instance) = make_instance(TESTING_GAS_LIMIT);
        leave_default_data(&env);

        let val = env
            .with_storage_from_context::<_, _>(|store| {
                Ok(store.get(INIT_KEY).0.expect("error getting value"))
            })
            .unwrap();
        assert_eq!(val, Some(INIT_VALUE.to_vec()));

        let set_key: &[u8] = b"more";
        let set_value: &[u8] = b"data";

        env.with_storage_from_context::<_, _>(|store| {
            store
                .set(set_key, set_value)
                .0
                .expect("error setting value");
            Ok(())
        })
        .unwrap();

        env.with_storage_from_context::<_, _>(|store| {
            assert_eq!(store.get(INIT_KEY).0.unwrap(), Some(INIT_VALUE.to_vec()));
            assert_eq!(store.get(set_key).0.unwrap(), Some(set_value.to_vec()));
            Ok(())
        })
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "A panic occurred in the callback.")]
    fn with_storage_from_context_handles_panics() {
        let (env, _instance) = make_instance(TESTING_GAS_LIMIT);
        leave_default_data(&env);

        env.with_storage_from_context::<_, ()>(|_store| {
            panic!("A panic occurred in the callback.")
        })
        .unwrap();
    }

    #[test]
    fn with_querier_from_context_works() {
        let (env, _instance) = make_instance(TESTING_GAS_LIMIT);
        leave_default_data(&env);

        let res = env
            .with_querier_from_context::<_, _>(|querier| {
                let req: QueryRequest<Empty> = QueryRequest::Bank(BankQuery::AllBalances {
                    address: INIT_ADDR.to_string(),
                });
                let (result, _gas_info) =
                    querier.query_raw(&to_vec(&req).unwrap(), DEFAULT_QUERY_GAS_LIMIT);
                Ok(result.unwrap())
            })
            .unwrap()
            .unwrap()
            .unwrap();
        let balance: AllBalanceResponse = from_binary(&res).unwrap();

        assert_eq!(balance.amount, coins(INIT_AMOUNT, INIT_DENOM));
    }

    #[test]
    #[should_panic(expected = "A panic occurred in the callback.")]
    fn with_querier_from_context_handles_panics() {
        let (env, _instance) = make_instance(TESTING_GAS_LIMIT);
        leave_default_data(&env);

        env.with_querier_from_context::<_, ()>(|_querier| {
            panic!("A panic occurred in the callback.")
        })
        .unwrap();
    }
}
