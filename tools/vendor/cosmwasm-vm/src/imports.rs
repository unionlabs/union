//! Import implementations

use std::cmp::max;
use std::marker::PhantomData;

use cosmwasm_crypto::{
    ed25519_batch_verify, ed25519_verify, secp256k1_recover_pubkey, secp256k1_verify, CryptoError,
};
use cosmwasm_crypto::{
    ECDSA_PUBKEY_MAX_LEN, ECDSA_SIGNATURE_LEN, EDDSA_PUBKEY_LEN, MESSAGE_HASH_MAX_LEN,
};

#[cfg(feature = "iterator")]
use cosmwasm_std::Order;
use wasmer::{AsStoreMut, FunctionEnvMut};

use crate::backend::{BackendApi, BackendError, Querier, Storage};
use crate::conversion::{ref_to_u32, to_u32};
use crate::environment::{process_gas_info, DebugInfo, Environment};
use crate::errors::{CommunicationError, VmError, VmResult};
#[cfg(feature = "iterator")]
use crate::memory::maybe_read_region;
use crate::memory::{read_region, write_region};
use crate::sections::decode_sections;
#[allow(unused_imports)]
use crate::sections::encode_sections;
use crate::serde::to_vec;
use crate::GasInfo;

/// A kibi (kilo binary)
const KI: usize = 1024;
/// A mibi (mega binary)
const MI: usize = 1024 * 1024;
/// Max key length for db_write/db_read/db_remove/db_scan (when VM reads the key argument from Wasm memory)
const MAX_LENGTH_DB_KEY: usize = 64 * KI;
/// Max value length for db_write (when VM reads the value argument from Wasm memory)
const MAX_LENGTH_DB_VALUE: usize = 128 * KI;
/// Typically 20 (Cosmos SDK, Ethereum), 32 (Nano, Substrate) or 54 (MockApi)
const MAX_LENGTH_CANONICAL_ADDRESS: usize = 64;
/// The max length of human address inputs (in bytes).
/// The maximum allowed size for [bech32](https://github.com/bitcoin/bips/blob/master/bip-0173.mediawiki#bech32)
/// is 90 characters and we're adding some safety margin around that for other formats.
const MAX_LENGTH_HUMAN_ADDRESS: usize = 256;
const MAX_LENGTH_QUERY_CHAIN_REQUEST: usize = 64 * KI;
/// Length of a serialized Ed25519  signature
const MAX_LENGTH_ED25519_SIGNATURE: usize = 64;
/// Max length of a Ed25519 message in bytes.
/// This is an arbitrary value, for performance / memory constraints. If you need to verify larger
/// messages, let us know.
const MAX_LENGTH_ED25519_MESSAGE: usize = 128 * 1024;
/// Max number of batch Ed25519 messages / signatures / public_keys.
/// This is an arbitrary value, for performance / memory constraints. If you need to batch-verify a
/// larger number of signatures, let us know.
const MAX_COUNT_ED25519_BATCH: usize = 256;

/// Max length for a debug message
const MAX_LENGTH_DEBUG: usize = 2 * MI;

/// Max length for an abort message
const MAX_LENGTH_ABORT: usize = 2 * MI;

// Import implementations
//
// This block of do_* prefixed functions is tailored for Wasmer's
// Function::new_native_with_env interface. Those require an env in the first
// argument and cannot capture other variables. Thus everything is accessed
// through the env.

/// Reads a storage entry from the VM's storage into Wasm memory
pub fn do_db_read<A: BackendApi + 'static, S: Storage + 'static, Q: Querier + 'static>(
    mut env: FunctionEnvMut<Environment<A, S, Q>>,
    key_ptr: u32,
) -> VmResult<u32> {
    let (data, mut store) = env.data_and_store_mut();

    let key = read_region(&data.memory(&store), key_ptr, MAX_LENGTH_DB_KEY)?;

    let (result, gas_info) = data.with_storage_from_context::<_, _>(|store| Ok(store.get(&key)))?;
    process_gas_info(data, &mut store, gas_info)?;
    let value = result?;

    let out_data = match value {
        Some(data) => data,
        None => return Ok(0),
    };
    write_to_contract(data, &mut store, &out_data)
}

/// Writes a storage entry from Wasm memory into the VM's storage
pub fn do_db_write<A: BackendApi + 'static, S: Storage + 'static, Q: Querier + 'static>(
    mut env: FunctionEnvMut<Environment<A, S, Q>>,
    key_ptr: u32,
    value_ptr: u32,
) -> VmResult<()> {
    let (data, mut store) = env.data_and_store_mut();

    if data.is_storage_readonly() {
        return Err(VmError::write_access_denied());
    }

    /// Converts a region length error to a different variant for better understandability
    fn convert_error(e: VmError, kind: &'static str) -> VmError {
        if let VmError::CommunicationErr {
            source: CommunicationError::RegionLengthTooBig { length, max_length },
            ..
        } = e
        {
            VmError::generic_err(format!(
                "{kind} too big. Tried to write {length} bytes to storage, limit is {max_length}."
            ))
        } else {
            e
        }
    }

    let key = read_region(&data.memory(&store), key_ptr, MAX_LENGTH_DB_KEY)
        .map_err(|e| convert_error(e, "Key"))?;
    let value = read_region(&data.memory(&store), value_ptr, MAX_LENGTH_DB_VALUE)
        .map_err(|e| convert_error(e, "Value"))?;

    let (result, gas_info) =
        data.with_storage_from_context::<_, _>(|store| Ok(store.set(&key, &value)))?;
    process_gas_info(data, &mut store, gas_info)?;
    result?;

    Ok(())
}

pub fn do_db_remove<A: BackendApi + 'static, S: Storage + 'static, Q: Querier + 'static>(
    mut env: FunctionEnvMut<Environment<A, S, Q>>,
    key_ptr: u32,
) -> VmResult<()> {
    let (data, mut store) = env.data_and_store_mut();

    if data.is_storage_readonly() {
        return Err(VmError::write_access_denied());
    }

    let key = read_region(&data.memory(&store), key_ptr, MAX_LENGTH_DB_KEY)?;

    let (result, gas_info) =
        data.with_storage_from_context::<_, _>(|store| Ok(store.remove(&key)))?;
    process_gas_info(data, &mut store, gas_info)?;
    result?;

    Ok(())
}

pub fn do_addr_validate<A: BackendApi + 'static, S: Storage + 'static, Q: Querier + 'static>(
    mut env: FunctionEnvMut<Environment<A, S, Q>>,
    source_ptr: u32,
) -> VmResult<u32> {
    let (data, mut store) = env.data_and_store_mut();

    let source_data = read_region(&data.memory(&store), source_ptr, MAX_LENGTH_HUMAN_ADDRESS)?;
    if source_data.is_empty() {
        return write_to_contract(data, &mut store, b"Input is empty");
    }

    let source_string = match String::from_utf8(source_data) {
        Ok(s) => s,
        Err(_) => return write_to_contract(data, &mut store, b"Input is not valid UTF-8"),
    };

    let (result, gas_info) = data.api.canonical_address(&source_string);
    process_gas_info(data, &mut store, gas_info)?;
    let canonical = match result {
        Ok(data) => data,
        Err(BackendError::UserErr { msg, .. }) => {
            return write_to_contract(data, &mut store, msg.as_bytes())
        }
        Err(err) => return Err(VmError::from(err)),
    };

    let (result, gas_info) = data.api.human_address(&canonical);
    process_gas_info(data, &mut store, gas_info)?;
    let normalized = match result {
        Ok(addr) => addr,
        Err(BackendError::UserErr { msg, .. }) => {
            return write_to_contract(data, &mut store, msg.as_bytes())
        }
        Err(err) => return Err(VmError::from(err)),
    };

    if normalized != source_string {
        return write_to_contract(data, &mut store, b"Address is not normalized");
    }

    Ok(0)
}

pub fn do_addr_canonicalize<A: BackendApi + 'static, S: Storage + 'static, Q: Querier + 'static>(
    mut env: FunctionEnvMut<Environment<A, S, Q>>,
    source_ptr: u32,
    destination_ptr: u32,
) -> VmResult<u32> {
    let (data, mut store) = env.data_and_store_mut();

    let source_data = read_region(&data.memory(&store), source_ptr, MAX_LENGTH_HUMAN_ADDRESS)?;
    if source_data.is_empty() {
        return write_to_contract(data, &mut store, b"Input is empty");
    }

    let source_string = match String::from_utf8(source_data) {
        Ok(s) => s,
        Err(_) => return write_to_contract(data, &mut store, b"Input is not valid UTF-8"),
    };

    let (result, gas_info) = data.api.canonical_address(&source_string);
    process_gas_info(data, &mut store, gas_info)?;
    match result {
        Ok(canonical) => {
            write_region(&data.memory(&store), destination_ptr, canonical.as_slice())?;
            Ok(0)
        }
        Err(BackendError::UserErr { msg, .. }) => {
            Ok(write_to_contract(data, &mut store, msg.as_bytes())?)
        }
        Err(err) => Err(VmError::from(err)),
    }
}

pub fn do_addr_humanize<A: BackendApi + 'static, S: Storage + 'static, Q: Querier + 'static>(
    mut env: FunctionEnvMut<Environment<A, S, Q>>,
    source_ptr: u32,
    destination_ptr: u32,
) -> VmResult<u32> {
    let (data, mut store) = env.data_and_store_mut();

    let canonical = read_region(
        &data.memory(&store),
        source_ptr,
        MAX_LENGTH_CANONICAL_ADDRESS,
    )?;

    let (result, gas_info) = data.api.human_address(&canonical);
    process_gas_info(data, &mut store, gas_info)?;
    match result {
        Ok(human) => {
            write_region(&data.memory(&store), destination_ptr, human.as_bytes())?;
            Ok(0)
        }
        Err(BackendError::UserErr { msg, .. }) => {
            Ok(write_to_contract(data, &mut store, msg.as_bytes())?)
        }
        Err(err) => Err(VmError::from(err)),
    }
}

/// Return code (error code) for a valid signature
const SECP256K1_VERIFY_CODE_VALID: u32 = 0;

/// Return code (error code) for an invalid signature
const SECP256K1_VERIFY_CODE_INVALID: u32 = 1;

pub fn do_secp256k1_verify<A: BackendApi + 'static, S: Storage + 'static, Q: Querier + 'static>(
    mut env: FunctionEnvMut<Environment<A, S, Q>>,
    hash_ptr: u32,
    signature_ptr: u32,
    pubkey_ptr: u32,
) -> VmResult<u32> {
    let (data, mut store) = env.data_and_store_mut();

    let hash = read_region(&data.memory(&store), hash_ptr, MESSAGE_HASH_MAX_LEN)?;
    let signature = read_region(&data.memory(&store), signature_ptr, ECDSA_SIGNATURE_LEN)?;
    let pubkey = read_region(&data.memory(&store), pubkey_ptr, ECDSA_PUBKEY_MAX_LEN)?;

    let gas_info = GasInfo::with_cost(data.gas_config.secp256k1_verify_cost);
    process_gas_info(data, &mut store, gas_info)?;
    let result = secp256k1_verify(&hash, &signature, &pubkey);
    let code = match result {
        Ok(valid) => {
            if valid {
                SECP256K1_VERIFY_CODE_VALID
            } else {
                SECP256K1_VERIFY_CODE_INVALID
            }
        }
        Err(err) => match err {
            CryptoError::InvalidHashFormat { .. }
            | CryptoError::InvalidPubkeyFormat { .. }
            | CryptoError::InvalidSignatureFormat { .. }
            | CryptoError::GenericErr { .. } => err.code(),
            CryptoError::BatchErr { .. } | CryptoError::InvalidRecoveryParam { .. } => {
                panic!("Error must not happen for this call")
            }
        },
    };
    Ok(code)
}

pub fn do_secp256k1_recover_pubkey<
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
>(
    mut env: FunctionEnvMut<Environment<A, S, Q>>,
    hash_ptr: u32,
    signature_ptr: u32,
    recover_param: u32,
) -> VmResult<u64> {
    let (data, mut store) = env.data_and_store_mut();

    let hash = read_region(&data.memory(&store), hash_ptr, MESSAGE_HASH_MAX_LEN)?;
    let signature = read_region(&data.memory(&store), signature_ptr, ECDSA_SIGNATURE_LEN)?;
    let recover_param: u8 = match recover_param.try_into() {
        Ok(rp) => rp,
        Err(_) => return Ok((CryptoError::invalid_recovery_param().code() as u64) << 32),
    };

    let gas_info = GasInfo::with_cost(data.gas_config.secp256k1_recover_pubkey_cost);
    process_gas_info(data, &mut store, gas_info)?;
    let result = secp256k1_recover_pubkey(&hash, &signature, recover_param);
    match result {
        Ok(pubkey) => {
            let pubkey_ptr = write_to_contract(data, &mut store, pubkey.as_ref())?;
            Ok(to_low_half(pubkey_ptr))
        }
        Err(err) => match err {
            CryptoError::InvalidHashFormat { .. }
            | CryptoError::InvalidSignatureFormat { .. }
            | CryptoError::InvalidRecoveryParam { .. }
            | CryptoError::GenericErr { .. } => Ok(to_high_half(err.code())),
            CryptoError::BatchErr { .. } | CryptoError::InvalidPubkeyFormat { .. } => {
                panic!("Error must not happen for this call")
            }
        },
    }
}

/// Return code (error code) for a valid signature
const ED25519_VERIFY_CODE_VALID: u32 = 0;

/// Return code (error code) for an invalid signature
const ED25519_VERIFY_CODE_INVALID: u32 = 1;

pub fn do_ed25519_verify<A: BackendApi + 'static, S: Storage + 'static, Q: Querier + 'static>(
    mut env: FunctionEnvMut<Environment<A, S, Q>>,
    message_ptr: u32,
    signature_ptr: u32,
    pubkey_ptr: u32,
) -> VmResult<u32> {
    let (data, mut store) = env.data_and_store_mut();

    let message = read_region(
        &data.memory(&store),
        message_ptr,
        MAX_LENGTH_ED25519_MESSAGE,
    )?;
    let signature = read_region(
        &data.memory(&store),
        signature_ptr,
        MAX_LENGTH_ED25519_SIGNATURE,
    )?;
    let pubkey = read_region(&data.memory(&store), pubkey_ptr, EDDSA_PUBKEY_LEN)?;

    let gas_info = GasInfo::with_cost(data.gas_config.ed25519_verify_cost);
    process_gas_info(data, &mut store, gas_info)?;
    let result = ed25519_verify(&message, &signature, &pubkey);
    let code = match result {
        Ok(valid) => {
            if valid {
                ED25519_VERIFY_CODE_VALID
            } else {
                ED25519_VERIFY_CODE_INVALID
            }
        }
        Err(err) => match err {
            CryptoError::InvalidPubkeyFormat { .. }
            | CryptoError::InvalidSignatureFormat { .. }
            | CryptoError::GenericErr { .. } => err.code(),
            CryptoError::BatchErr { .. }
            | CryptoError::InvalidHashFormat { .. }
            | CryptoError::InvalidRecoveryParam { .. } => {
                panic!("Error must not happen for this call")
            }
        },
    };
    Ok(code)
}

pub fn do_ed25519_batch_verify<
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
>(
    mut env: FunctionEnvMut<Environment<A, S, Q>>,
    messages_ptr: u32,
    signatures_ptr: u32,
    public_keys_ptr: u32,
) -> VmResult<u32> {
    let (data, mut store) = env.data_and_store_mut();

    let messages = read_region(
        &data.memory(&store),
        messages_ptr,
        (MAX_LENGTH_ED25519_MESSAGE + 4) * MAX_COUNT_ED25519_BATCH,
    )?;
    let signatures = read_region(
        &data.memory(&store),
        signatures_ptr,
        (MAX_LENGTH_ED25519_SIGNATURE + 4) * MAX_COUNT_ED25519_BATCH,
    )?;
    let public_keys = read_region(
        &data.memory(&store),
        public_keys_ptr,
        (EDDSA_PUBKEY_LEN + 4) * MAX_COUNT_ED25519_BATCH,
    )?;

    let messages = decode_sections(&messages);
    let signatures = decode_sections(&signatures);
    let public_keys = decode_sections(&public_keys);

    let gas_cost = if public_keys.len() == 1 {
        data.gas_config.ed25519_batch_verify_one_pubkey_cost
    } else {
        data.gas_config.ed25519_batch_verify_cost
    } * signatures.len() as u64;
    let gas_info = GasInfo::with_cost(max(gas_cost, data.gas_config.ed25519_verify_cost));
    process_gas_info(data, &mut store, gas_info)?;
    let result = ed25519_batch_verify(&messages, &signatures, &public_keys);
    let code = match result {
        Ok(valid) => {
            if valid {
                ED25519_VERIFY_CODE_VALID
            } else {
                ED25519_VERIFY_CODE_INVALID
            }
        }
        Err(err) => match err {
            CryptoError::BatchErr { .. }
            | CryptoError::InvalidPubkeyFormat { .. }
            | CryptoError::InvalidSignatureFormat { .. }
            | CryptoError::GenericErr { .. } => err.code(),
            CryptoError::InvalidHashFormat { .. } | CryptoError::InvalidRecoveryParam { .. } => {
                panic!("Error must not happen for this call")
            }
        },
    };
    Ok(code)
}

/// Prints a debug message to console.
/// This does not charge gas, so debug printing should be disabled when used in a blockchain module.
pub fn do_debug<A: BackendApi + 'static, S: Storage + 'static, Q: Querier + 'static>(
    mut env: FunctionEnvMut<Environment<A, S, Q>>,
    message_ptr: u32,
) -> VmResult<()> {
    let (data, mut store) = env.data_and_store_mut();

    if let Some(debug_handler) = data.debug_handler() {
        let message_data = read_region(&data.memory(&store), message_ptr, MAX_LENGTH_DEBUG)?;
        let msg = String::from_utf8_lossy(&message_data);
        let gas_remaining = data.get_gas_left(&mut store);
        debug_handler.borrow_mut()(
            &msg,
            DebugInfo {
                gas_remaining,
                __lifetime: PhantomData,
            },
        );
    }
    Ok(())
}

/// Aborts the contract and shows the given error message
pub fn do_abort<A: BackendApi + 'static, S: Storage + 'static, Q: Querier + 'static>(
    mut env: FunctionEnvMut<Environment<A, S, Q>>,
    message_ptr: u32,
) -> VmResult<()> {
    let (data, store) = env.data_and_store_mut();

    let message_data = read_region(&data.memory(&store), message_ptr, MAX_LENGTH_ABORT)?;
    let msg = String::from_utf8_lossy(&message_data);
    Err(VmError::aborted(msg))
}

pub fn do_query_chain<A: BackendApi + 'static, S: Storage + 'static, Q: Querier + 'static>(
    mut env: FunctionEnvMut<Environment<A, S, Q>>,
    request_ptr: u32,
) -> VmResult<u32> {
    let (data, mut store) = env.data_and_store_mut();

    let request = read_region(
        &data.memory(&store),
        request_ptr,
        MAX_LENGTH_QUERY_CHAIN_REQUEST,
    )?;

    let gas_remaining = data.get_gas_left(&mut store);
    let (result, gas_info) = data.with_querier_from_context::<_, _>(|querier| {
        Ok(querier.query_raw(&request, gas_remaining))
    })?;
    process_gas_info(data, &mut store, gas_info)?;
    let serialized = to_vec(&result?)?;
    write_to_contract(data, &mut store, &serialized)
}

#[cfg(feature = "iterator")]
pub fn do_db_scan<A: BackendApi + 'static, S: Storage + 'static, Q: Querier + 'static>(
    mut env: FunctionEnvMut<Environment<A, S, Q>>,
    start_ptr: u32,
    end_ptr: u32,
    order: i32,
) -> VmResult<u32> {
    let (data, mut store) = env.data_and_store_mut();

    let start = maybe_read_region(&data.memory(&store), start_ptr, MAX_LENGTH_DB_KEY)?;
    let end = maybe_read_region(&data.memory(&store), end_ptr, MAX_LENGTH_DB_KEY)?;
    let order: Order = order
        .try_into()
        .map_err(|_| CommunicationError::invalid_order(order))?;

    let (result, gas_info) = data.with_storage_from_context::<_, _>(|store| {
        Ok(store.scan(start.as_deref(), end.as_deref(), order))
    })?;
    process_gas_info(data, &mut store, gas_info)?;
    let iterator_id = result?;
    Ok(iterator_id)
}

#[cfg(feature = "iterator")]
pub fn do_db_next<A: BackendApi + 'static, S: Storage + 'static, Q: Querier + 'static>(
    mut env: FunctionEnvMut<Environment<A, S, Q>>,
    iterator_id: u32,
) -> VmResult<u32> {
    let (data, mut store) = env.data_and_store_mut();

    let (result, gas_info) =
        data.with_storage_from_context::<_, _>(|store| Ok(store.next(iterator_id)))?;

    process_gas_info(data, &mut store, gas_info)?;

    // Empty key will later be treated as _no more element_.
    let (key, value) = result?.unwrap_or_else(|| (Vec::<u8>::new(), Vec::<u8>::new()));

    let out_data = encode_sections(&[key, value])?;
    write_to_contract(data, &mut store, &out_data)
}

#[cfg(feature = "iterator")]
pub fn do_db_next_key<A: BackendApi + 'static, S: Storage + 'static, Q: Querier + 'static>(
    mut env: FunctionEnvMut<Environment<A, S, Q>>,
    iterator_id: u32,
) -> VmResult<u32> {
    let (data, mut store) = env.data_and_store_mut();

    let (result, gas_info) =
        data.with_storage_from_context::<_, _>(|store| Ok(store.next_key(iterator_id)))?;

    process_gas_info(data, &mut store, gas_info)?;

    let key = match result? {
        Some(key) => key,
        None => return Ok(0),
    };

    write_to_contract(data, &mut store, &key)
}

#[cfg(feature = "iterator")]
pub fn do_db_next_value<A: BackendApi + 'static, S: Storage + 'static, Q: Querier + 'static>(
    mut env: FunctionEnvMut<Environment<A, S, Q>>,
    iterator_id: u32,
) -> VmResult<u32> {
    let (data, mut store) = env.data_and_store_mut();

    let (result, gas_info) =
        data.with_storage_from_context::<_, _>(|store| Ok(store.next_value(iterator_id)))?;

    process_gas_info(data, &mut store, gas_info)?;

    let value = match result? {
        Some(value) => value,
        None => return Ok(0),
    };

    write_to_contract(data, &mut store, &value)
}

/// Creates a Region in the contract, writes the given data to it and returns the memory location
fn write_to_contract<A: BackendApi + 'static, S: Storage + 'static, Q: Querier + 'static>(
    data: &Environment<A, S, Q>,
    store: &mut impl AsStoreMut,
    input: &[u8],
) -> VmResult<u32> {
    let out_size = to_u32(input.len())?;
    let result = data.call_function1(store, "allocate", &[out_size.into()])?;
    let target_ptr = ref_to_u32(&result)?;
    if target_ptr == 0 {
        return Err(CommunicationError::zero_address().into());
    }
    write_region(&data.memory(store), target_ptr, input)?;
    Ok(target_ptr)
}

/// Returns the data shifted by 32 bits towards the most significant bit.
///
/// This is independent of endianness. But to get the idea, it would be
/// `data || 0x00000000` in big endian representation.
#[inline]
fn to_high_half(data: u32) -> u64 {
    // See https://stackoverflow.com/a/58956419/2013738 to understand
    // why this is endianness agnostic.
    (data as u64) << 32
}

/// Returns the data copied to the 4 least significant bytes.
///
/// This is independent of endianness. But to get the idea, it would be
/// `0x00000000 || data` in big endian representation.
#[inline]
fn to_low_half(data: u32) -> u64 {
    data.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{
        coins, from_json, AllBalanceResponse, BankQuery, Binary, Empty, QueryRequest, SystemError,
        SystemResult, WasmQuery,
    };
    use hex_literal::hex;
    use std::ptr::NonNull;
    use wasmer::{imports, Function, FunctionEnv, Instance as WasmerInstance, Store};

    use crate::backend::{BackendError, Storage};
    use crate::size::Size;
    use crate::testing::{MockApi, MockQuerier, MockStorage};
    use crate::wasm_backend::{compile, make_compiling_engine};

    static CONTRACT: &[u8] = include_bytes!("../testdata/hackatom.wasm");

    // prepared data
    const KEY1: &[u8] = b"ant";
    const VALUE1: &[u8] = b"insect";
    const KEY2: &[u8] = b"tree";
    const VALUE2: &[u8] = b"plant";

    // this account has some coins
    const INIT_ADDR: &str = "someone";
    const INIT_AMOUNT: u128 = 500;
    const INIT_DENOM: &str = "TOKEN";

    const TESTING_GAS_LIMIT: u64 = 500_000_000_000; // ~0.5ms
    const TESTING_MEMORY_LIMIT: Option<Size> = Some(Size::mebi(16));

    const ECDSA_HASH_HEX: &str = "5ae8317d34d1e595e3fa7247db80c0af4320cce1116de187f8f7e2e099c0d8d0";
    const ECDSA_SIG_HEX: &str = "207082eb2c3dfa0b454e0906051270ba4074ac93760ba9e7110cd9471475111151eb0dbbc9920e72146fb564f99d039802bf6ef2561446eb126ef364d21ee9c4";
    const ECDSA_PUBKEY_HEX: &str = "04051c1ee2190ecfb174bfe4f90763f2b4ff7517b70a2aec1876ebcfd644c4633fb03f3cfbd94b1f376e34592d9d41ccaf640bb751b00a1fadeb0c01157769eb73";

    const EDDSA_MSG_HEX: &str = "";
    const EDDSA_SIG_HEX: &str = "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b";
    const EDDSA_PUBKEY_HEX: &str =
        "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";

    fn make_instance(
        api: MockApi,
    ) -> (
        FunctionEnv<Environment<MockApi, MockStorage, MockQuerier>>,
        Store,
        Box<WasmerInstance>,
    ) {
        let gas_limit = TESTING_GAS_LIMIT;
        let env = Environment::new(api, gas_limit);

        let engine = make_compiling_engine(TESTING_MEMORY_LIMIT);
        let module = compile(&engine, CONTRACT).unwrap();
        let mut store = Store::new(engine);

        let fe = FunctionEnv::new(&mut store, env);

        // we need stubs for all required imports
        let import_obj = imports! {
            "env" => {
                "db_read" => Function::new_typed(&mut store, |_a: u32| -> u32 { 0 }),
                "db_write" => Function::new_typed(&mut store, |_a: u32, _b: u32| {}),
                "db_remove" => Function::new_typed(&mut store, |_a: u32| {}),
                "db_scan" => Function::new_typed(&mut store, |_a: u32, _b: u32, _c: i32| -> u32 { 0 }),
                "db_next" => Function::new_typed(&mut store, |_a: u32| -> u32 { 0 }),
                "db_next_key" => Function::new_typed(&mut store, |_a: u32| -> u32 { 0 }),
                "db_next_value" => Function::new_typed(&mut store, |_a: u32| -> u32 { 0 }),
                "query_chain" => Function::new_typed(&mut store, |_a: u32| -> u32 { 0 }),
                "addr_validate" => Function::new_typed(&mut store, |_a: u32| -> u32 { 0 }),
                "addr_canonicalize" => Function::new_typed(&mut store, |_a: u32, _b: u32| -> u32 { 0 }),
                "addr_humanize" => Function::new_typed(&mut store, |_a: u32, _b: u32| -> u32 { 0 }),
                "secp256k1_verify" => Function::new_typed(&mut store, |_a: u32, _b: u32, _c: u32| -> u32 { 0 }),
                "secp256k1_recover_pubkey" => Function::new_typed(&mut store, |_a: u32, _b: u32, _c: u32| -> u64 { 0 }),
                "ed25519_verify" => Function::new_typed(&mut store, |_a: u32, _b: u32, _c: u32| -> u32 { 0 }),
                "ed25519_batch_verify" => Function::new_typed(&mut store, |_a: u32, _b: u32, _c: u32| -> u32 { 0 }),
                "debug" => Function::new_typed(&mut store, |_a: u32| {}),
                "abort" => Function::new_typed(&mut store, |_a: u32| {}),
            },
        };
        let wasmer_instance =
            Box::from(WasmerInstance::new(&mut store, &module, &import_obj).unwrap());
        let memory = wasmer_instance
            .exports
            .get_memory("memory")
            .unwrap()
            .clone();

        fe.as_mut(&mut store).memory = Some(memory);

        let instance_ptr = NonNull::from(wasmer_instance.as_ref());

        {
            let mut fe_mut = fe.clone().into_mut(&mut store);
            let (env, mut store) = fe_mut.data_and_store_mut();

            env.set_wasmer_instance(Some(instance_ptr));
            env.set_gas_left(&mut store, gas_limit);
            env.set_storage_readonly(false);
        }

        (fe, store, wasmer_instance)
    }

    fn leave_default_data(
        fe_mut: &mut FunctionEnvMut<Environment<MockApi, MockStorage, MockQuerier>>,
    ) {
        let (env, _store) = fe_mut.data_and_store_mut();

        // create some mock data
        let mut storage = MockStorage::new();
        storage.set(KEY1, VALUE1).0.expect("error setting");
        storage.set(KEY2, VALUE2).0.expect("error setting");
        let querier: MockQuerier<Empty> =
            MockQuerier::new(&[(INIT_ADDR, &coins(INIT_AMOUNT, INIT_DENOM))]);
        env.move_in(storage, querier);
    }

    fn write_data(
        fe_mut: &mut FunctionEnvMut<Environment<MockApi, MockStorage, MockQuerier>>,
        data: &[u8],
    ) -> u32 {
        let (env, mut store) = fe_mut.data_and_store_mut();

        let result = env
            .call_function1(&mut store, "allocate", &[(data.len() as u32).into()])
            .unwrap();
        let region_ptr = ref_to_u32(&result).unwrap();
        write_region(&env.memory(&store), region_ptr, data).expect("error writing");
        region_ptr
    }

    fn create_empty(
        wasmer_instance: &WasmerInstance,
        fe_mut: &mut FunctionEnvMut<Environment<MockApi, MockStorage, MockQuerier>>,
        capacity: u32,
    ) -> u32 {
        let (_, mut store) = fe_mut.data_and_store_mut();
        let allocate = wasmer_instance
            .exports
            .get_function("allocate")
            .expect("error getting function");
        let result = allocate
            .call(&mut store, &[capacity.into()])
            .expect("error calling allocate");
        ref_to_u32(&result[0]).expect("error converting result")
    }

    /// A Region reader that is just good enough for the tests in this file
    fn force_read(
        fe_mut: &mut FunctionEnvMut<Environment<MockApi, MockStorage, MockQuerier>>,
        region_ptr: u32,
    ) -> Vec<u8> {
        let (env, store) = fe_mut.data_and_store_mut();

        read_region(&env.memory(&store), region_ptr, 5000).unwrap()
    }

    #[test]
    fn do_db_read_works() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);
        leave_default_data(&mut fe_mut);

        let key_ptr = write_data(&mut fe_mut, KEY1);
        let result = do_db_read(fe_mut.as_mut(), key_ptr);
        let value_ptr = result.unwrap();
        assert!(value_ptr > 0);
        leave_default_data(&mut fe_mut);
        assert_eq!(force_read(&mut fe_mut, value_ptr), VALUE1);
    }

    #[test]
    fn do_db_read_works_for_non_existent_key() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);
        leave_default_data(&mut fe_mut);

        let key_ptr = write_data(&mut fe_mut, b"I do not exist in storage");
        let result = do_db_read(fe_mut, key_ptr);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn do_db_read_fails_for_large_key() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);
        leave_default_data(&mut fe_mut);

        let key_ptr = write_data(&mut fe_mut, &vec![7u8; 300 * 1024]);
        let result = do_db_read(fe_mut, key_ptr);
        match result.unwrap_err() {
            VmError::CommunicationErr {
                source: CommunicationError::RegionLengthTooBig { length, .. },
                ..
            } => assert_eq!(length, 300 * 1024),
            e => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn do_db_write_works() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let key_ptr = write_data(&mut fe_mut, b"new storage key");
        let value_ptr = write_data(&mut fe_mut, b"new value");

        leave_default_data(&mut fe_mut);

        do_db_write(fe_mut.as_mut(), key_ptr, value_ptr).unwrap();

        let val = fe_mut
            .data()
            .with_storage_from_context::<_, _>(|store| {
                Ok(store
                    .get(b"new storage key")
                    .0
                    .expect("error getting value"))
            })
            .unwrap();
        assert_eq!(val, Some(b"new value".to_vec()));
    }

    #[test]
    fn do_db_write_can_override() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let key_ptr = write_data(&mut fe_mut, KEY1);
        let value_ptr = write_data(&mut fe_mut, VALUE2);

        leave_default_data(&mut fe_mut);

        do_db_write(fe_mut.as_mut(), key_ptr, value_ptr).unwrap();

        let val = fe_mut
            .data()
            .with_storage_from_context::<_, _>(|store| {
                Ok(store.get(KEY1).0.expect("error getting value"))
            })
            .unwrap();
        assert_eq!(val, Some(VALUE2.to_vec()));
    }

    #[test]
    fn do_db_write_works_for_empty_value() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let key_ptr = write_data(&mut fe_mut, b"new storage key");
        let value_ptr = write_data(&mut fe_mut, b"");

        leave_default_data(&mut fe_mut);

        do_db_write(fe_mut.as_mut(), key_ptr, value_ptr).unwrap();

        let val = fe_mut
            .data()
            .with_storage_from_context::<_, _>(|store| {
                Ok(store
                    .get(b"new storage key")
                    .0
                    .expect("error getting value"))
            })
            .unwrap();
        assert_eq!(val, Some(b"".to_vec()));
    }

    #[test]
    fn do_db_write_fails_for_large_key() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        const KEY_SIZE: usize = 300 * 1024;
        let key_ptr = write_data(&mut fe_mut, &vec![4u8; KEY_SIZE]);
        let value_ptr = write_data(&mut fe_mut, b"new value");

        leave_default_data(&mut fe_mut);

        let result = do_db_write(fe_mut, key_ptr, value_ptr);
        assert_eq!(result.unwrap_err().to_string(), format!("Generic error: Key too big. Tried to write {KEY_SIZE} bytes to storage, limit is {MAX_LENGTH_DB_KEY}."));
    }

    #[test]
    fn do_db_write_fails_for_large_value() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        const VAL_SIZE: usize = 300 * 1024;
        let key_ptr = write_data(&mut fe_mut, b"new storage key");
        let value_ptr = write_data(&mut fe_mut, &vec![5u8; VAL_SIZE]);

        leave_default_data(&mut fe_mut);

        let result = do_db_write(fe_mut, key_ptr, value_ptr);
        assert_eq!(result.unwrap_err().to_string(), format!("Generic error: Value too big. Tried to write {VAL_SIZE} bytes to storage, limit is {MAX_LENGTH_DB_VALUE}."));
    }

    #[test]
    fn do_db_write_is_prohibited_in_readonly_contexts() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let key_ptr = write_data(&mut fe_mut, b"new storage key");
        let value_ptr = write_data(&mut fe_mut, b"new value");

        leave_default_data(&mut fe_mut);
        fe_mut.data().set_storage_readonly(true);

        let result = do_db_write(fe_mut, key_ptr, value_ptr);
        match result.unwrap_err() {
            VmError::WriteAccessDenied { .. } => {}
            e => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn do_db_remove_works() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let existing_key = KEY1;
        let key_ptr = write_data(&mut fe_mut, existing_key);

        leave_default_data(&mut fe_mut);

        fe_mut
            .data()
            .with_storage_from_context::<_, _>(|store| {
                println!("{store:?}");
                Ok(())
            })
            .unwrap();

        do_db_remove(fe_mut.as_mut(), key_ptr).unwrap();

        fe_mut
            .data()
            .with_storage_from_context::<_, _>(|store| {
                println!("{store:?}");
                Ok(())
            })
            .unwrap();

        let value = fe_mut
            .data()
            .with_storage_from_context::<_, _>(|store| {
                Ok(store.get(existing_key).0.expect("error getting value"))
            })
            .unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn do_db_remove_works_for_non_existent_key() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let non_existent_key = b"I do not exist";
        let key_ptr = write_data(&mut fe_mut, non_existent_key);

        leave_default_data(&mut fe_mut);

        // Note: right now we cannot differentiate between an existent and a non-existent key
        do_db_remove(fe_mut.as_mut(), key_ptr).unwrap();

        let value = fe_mut
            .data()
            .with_storage_from_context::<_, _>(|store| {
                Ok(store.get(non_existent_key).0.expect("error getting value"))
            })
            .unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn do_db_remove_fails_for_large_key() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let key_ptr = write_data(&mut fe_mut, &vec![26u8; 300 * 1024]);

        leave_default_data(&mut fe_mut);

        let result = do_db_remove(fe_mut, key_ptr);
        match result.unwrap_err() {
            VmError::CommunicationErr {
                source:
                    CommunicationError::RegionLengthTooBig {
                        length, max_length, ..
                    },
                ..
            } => {
                assert_eq!(length, 300 * 1024);
                assert_eq!(max_length, MAX_LENGTH_DB_KEY);
            }
            err => panic!("unexpected error: {err:?}"),
        };
    }

    #[test]
    fn do_db_remove_is_prohibited_in_readonly_contexts() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let key_ptr = write_data(&mut fe_mut, b"a storage key");

        leave_default_data(&mut fe_mut);
        fe_mut.data().set_storage_readonly(true);

        let result = do_db_remove(fe_mut, key_ptr);
        match result.unwrap_err() {
            VmError::WriteAccessDenied { .. } => {}
            e => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn do_addr_validate_works() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let source_ptr1 = write_data(&mut fe_mut, b"foo");
        let source_ptr2 = write_data(&mut fe_mut, b"eth1n48g2mjh9ezz7zjtya37wtgg5r5emr0drkwlgw");

        let res = do_addr_validate(fe_mut.as_mut(), source_ptr1).unwrap();
        assert_eq!(res, 0);
        let res = do_addr_validate(fe_mut.as_mut(), source_ptr2).unwrap();
        assert_eq!(res, 0);
    }

    #[test]
    fn do_addr_validate_reports_invalid_input_back_to_contract() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let source_ptr1 = write_data(&mut fe_mut, b"fo\x80o"); // invalid UTF-8 (fo�o)
        let source_ptr2 = write_data(&mut fe_mut, b""); // empty
        let source_ptr3 = write_data(&mut fe_mut, b"addressexceedingaddressspacesuperlongreallylongiamensuringthatitislongerthaneverything"); // too long
        let source_ptr4 = write_data(&mut fe_mut, b"fooBar"); // Not normalized. The definition of normalized is chain-dependent but the MockApi requires lower case.

        let res = do_addr_validate(fe_mut.as_mut(), source_ptr1).unwrap();
        assert_ne!(res, 0);
        let err = String::from_utf8(force_read(&mut fe_mut, res)).unwrap();
        assert_eq!(err, "Input is not valid UTF-8");

        let res = do_addr_validate(fe_mut.as_mut(), source_ptr2).unwrap();
        assert_ne!(res, 0);
        let err = String::from_utf8(force_read(&mut fe_mut, res)).unwrap();
        assert_eq!(err, "Input is empty");

        let res = do_addr_validate(fe_mut.as_mut(), source_ptr3).unwrap();
        assert_ne!(res, 0);
        let err = String::from_utf8(force_read(&mut fe_mut, res)).unwrap();
        assert_eq!(
            err,
            "Invalid input: human address too long for this mock implementation (must be <= 64)."
        );

        let res = do_addr_validate(fe_mut.as_mut(), source_ptr4).unwrap();
        assert_ne!(res, 0);
        let err = String::from_utf8(force_read(&mut fe_mut, res)).unwrap();
        assert_eq!(err, "Address is not normalized");
    }

    #[test]
    fn do_addr_validate_fails_for_broken_backend() {
        let api = MockApi::new_failing("Temporarily unavailable");
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let source_ptr = write_data(&mut fe_mut, b"foo");

        leave_default_data(&mut fe_mut);

        let result = do_addr_validate(fe_mut, source_ptr);
        match result.unwrap_err() {
            VmError::BackendErr {
                source: BackendError::Unknown { msg, .. },
                ..
            } => assert_eq!(msg, "Temporarily unavailable"),
            err => panic!("Incorrect error returned: {err:?}"),
        }
    }

    #[test]
    fn do_addr_validate_fails_for_large_inputs() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let source_ptr = write_data(&mut fe_mut, &[61; 333]);

        leave_default_data(&mut fe_mut);

        let result = do_addr_validate(fe_mut, source_ptr);
        match result.unwrap_err() {
            VmError::CommunicationErr {
                source:
                    CommunicationError::RegionLengthTooBig {
                        length, max_length, ..
                    },
                ..
            } => {
                assert_eq!(length, 333);
                assert_eq!(max_length, 256);
            }
            err => panic!("Incorrect error returned: {err:?}"),
        }
    }

    #[test]
    fn do_addr_canonicalize_works() {
        let api = MockApi::default();
        let (fe, mut store, instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);
        let api = MockApi::default();

        let source_ptr = write_data(&mut fe_mut, b"foo");
        let dest_ptr = create_empty(&instance, &mut fe_mut, api.canonical_length() as u32);

        leave_default_data(&mut fe_mut);

        let api = MockApi::default();
        let res = do_addr_canonicalize(fe_mut.as_mut(), source_ptr, dest_ptr).unwrap();
        assert_eq!(res, 0);
        let data = force_read(&mut fe_mut, dest_ptr);
        assert_eq!(data.len(), api.canonical_length());
    }

    #[test]
    fn do_addr_canonicalize_reports_invalid_input_back_to_contract() {
        let api = MockApi::default();
        let (fe, mut store, instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let source_ptr1 = write_data(&mut fe_mut, b"fo\x80o"); // invalid UTF-8 (fo�o)
        let source_ptr2 = write_data(&mut fe_mut, b""); // empty
        let source_ptr3 = write_data(&mut fe_mut, b"addressexceedingaddressspacesuperlongreallylongiamensuringthatitislongerthaneverything"); // too long
        let dest_ptr = create_empty(&instance, &mut fe_mut, 70);

        leave_default_data(&mut fe_mut);

        let res = do_addr_canonicalize(fe_mut.as_mut(), source_ptr1, dest_ptr).unwrap();
        assert_ne!(res, 0);
        let err = String::from_utf8(force_read(&mut fe_mut, res)).unwrap();
        assert_eq!(err, "Input is not valid UTF-8");

        let res = do_addr_canonicalize(fe_mut.as_mut(), source_ptr2, dest_ptr).unwrap();
        assert_ne!(res, 0);
        let err = String::from_utf8(force_read(&mut fe_mut, res)).unwrap();
        assert_eq!(err, "Input is empty");

        let res = do_addr_canonicalize(fe_mut.as_mut(), source_ptr3, dest_ptr).unwrap();
        assert_ne!(res, 0);
        let err = String::from_utf8(force_read(&mut fe_mut, res)).unwrap();
        assert_eq!(
            err,
            "Invalid input: human address too long for this mock implementation (must be <= 64)."
        );
    }

    #[test]
    fn do_addr_canonicalize_fails_for_broken_backend() {
        let api = MockApi::new_failing("Temporarily unavailable");
        let (fe, mut store, instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let source_ptr = write_data(&mut fe_mut, b"foo");
        let dest_ptr = create_empty(&instance, &mut fe_mut, 7);

        leave_default_data(&mut fe_mut);

        let result = do_addr_canonicalize(fe_mut.as_mut(), source_ptr, dest_ptr);
        match result.unwrap_err() {
            VmError::BackendErr {
                source: BackendError::Unknown { msg, .. },
                ..
            } => assert_eq!(msg, "Temporarily unavailable"),
            err => panic!("Incorrect error returned: {err:?}"),
        }
    }

    #[test]
    fn do_addr_canonicalize_fails_for_large_inputs() {
        let api = MockApi::default();
        let (fe, mut store, instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let source_ptr = write_data(&mut fe_mut, &[61; 333]);
        let dest_ptr = create_empty(&instance, &mut fe_mut, 8);

        leave_default_data(&mut fe_mut);

        let result = do_addr_canonicalize(fe_mut.as_mut(), source_ptr, dest_ptr);
        match result.unwrap_err() {
            VmError::CommunicationErr {
                source:
                    CommunicationError::RegionLengthTooBig {
                        length, max_length, ..
                    },
                ..
            } => {
                assert_eq!(length, 333);
                assert_eq!(max_length, 256);
            }
            err => panic!("Incorrect error returned: {err:?}"),
        }
    }

    #[test]
    fn do_addr_canonicalize_fails_for_small_destination_region() {
        let api = MockApi::default();
        let (fe, mut store, instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let source_ptr = write_data(&mut fe_mut, b"foo");
        let dest_ptr = create_empty(&instance, &mut fe_mut, 7);

        leave_default_data(&mut fe_mut);

        let result = do_addr_canonicalize(fe_mut, source_ptr, dest_ptr);
        match result.unwrap_err() {
            VmError::CommunicationErr {
                source: CommunicationError::RegionTooSmall { size, required, .. },
                ..
            } => {
                assert_eq!(size, 7);
                assert_eq!(required, api.canonical_length());
            }
            err => panic!("Incorrect error returned: {err:?}"),
        }
    }

    #[test]
    fn do_addr_humanize_works() {
        let api = MockApi::default();
        let (fe, mut store, instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);
        let api = MockApi::default();

        let source_data = vec![0x22; api.canonical_length()];
        let source_ptr = write_data(&mut fe_mut, &source_data);
        let dest_ptr = create_empty(&instance, &mut fe_mut, 70);

        leave_default_data(&mut fe_mut);

        let error_ptr = do_addr_humanize(fe_mut.as_mut(), source_ptr, dest_ptr).unwrap();
        assert_eq!(error_ptr, 0);
        assert_eq!(force_read(&mut fe_mut, dest_ptr), source_data);
    }

    #[test]
    fn do_addr_humanize_reports_invalid_input_back_to_contract() {
        let api = MockApi::default();
        let (fe, mut store, instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let source_ptr = write_data(&mut fe_mut, b"foo"); // too short
        let dest_ptr = create_empty(&instance, &mut fe_mut, 70);

        leave_default_data(&mut fe_mut);

        let res = do_addr_humanize(fe_mut.as_mut(), source_ptr, dest_ptr).unwrap();
        assert_ne!(res, 0);
        let err = String::from_utf8(force_read(&mut fe_mut, res)).unwrap();
        assert_eq!(err, "Invalid input: canonical address length not correct");
    }

    #[test]
    fn do_addr_humanize_fails_for_broken_backend() {
        let api = MockApi::new_failing("Temporarily unavailable");
        let (fe, mut store, instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let source_ptr = write_data(&mut fe_mut, b"foo\0\0\0\0\0");
        let dest_ptr = create_empty(&instance, &mut fe_mut, 70);

        leave_default_data(&mut fe_mut);

        let result = do_addr_humanize(fe_mut, source_ptr, dest_ptr);
        match result.unwrap_err() {
            VmError::BackendErr {
                source: BackendError::Unknown { msg, .. },
                ..
            } => assert_eq!(msg, "Temporarily unavailable"),
            err => panic!("Incorrect error returned: {err:?}"),
        };
    }

    #[test]
    fn do_addr_humanize_fails_for_input_too_long() {
        let api = MockApi::default();
        let (fe, mut store, instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let source_ptr = write_data(&mut fe_mut, &[61; 65]);
        let dest_ptr = create_empty(&instance, &mut fe_mut, 70);

        leave_default_data(&mut fe_mut);

        let result = do_addr_humanize(fe_mut, source_ptr, dest_ptr);
        match result.unwrap_err() {
            VmError::CommunicationErr {
                source:
                    CommunicationError::RegionLengthTooBig {
                        length, max_length, ..
                    },
                ..
            } => {
                assert_eq!(length, 65);
                assert_eq!(max_length, 64);
            }
            err => panic!("Incorrect error returned: {err:?}"),
        }
    }

    #[test]
    fn do_addr_humanize_fails_for_destination_region_too_small() {
        let api = MockApi::default();
        let (fe, mut store, instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);
        let api = MockApi::default();

        let source_data = vec![0x22; api.canonical_length()];
        let source_ptr = write_data(&mut fe_mut, &source_data);
        let dest_ptr = create_empty(&instance, &mut fe_mut, 2);

        leave_default_data(&mut fe_mut);

        let result = do_addr_humanize(fe_mut, source_ptr, dest_ptr);
        match result.unwrap_err() {
            VmError::CommunicationErr {
                source: CommunicationError::RegionTooSmall { size, required, .. },
                ..
            } => {
                assert_eq!(size, 2);
                assert_eq!(required, api.canonical_length());
            }
            err => panic!("Incorrect error returned: {err:?}"),
        }
    }

    #[test]
    fn do_secp256k1_verify_works() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let hash = hex::decode(ECDSA_HASH_HEX).unwrap();
        let hash_ptr = write_data(&mut fe_mut, &hash);
        let sig = hex::decode(ECDSA_SIG_HEX).unwrap();
        let sig_ptr = write_data(&mut fe_mut, &sig);
        let pubkey = hex::decode(ECDSA_PUBKEY_HEX).unwrap();
        let pubkey_ptr = write_data(&mut fe_mut, &pubkey);

        assert_eq!(
            do_secp256k1_verify(fe_mut, hash_ptr, sig_ptr, pubkey_ptr).unwrap(),
            0
        );
    }

    #[test]
    fn do_secp256k1_verify_wrong_hash_verify_fails() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let mut hash = hex::decode(ECDSA_HASH_HEX).unwrap();
        // alter hash
        hash[0] ^= 0x01;
        let hash_ptr = write_data(&mut fe_mut, &hash);
        let sig = hex::decode(ECDSA_SIG_HEX).unwrap();
        let sig_ptr = write_data(&mut fe_mut, &sig);
        let pubkey = hex::decode(ECDSA_PUBKEY_HEX).unwrap();
        let pubkey_ptr = write_data(&mut fe_mut, &pubkey);

        assert_eq!(
            do_secp256k1_verify(fe_mut, hash_ptr, sig_ptr, pubkey_ptr).unwrap(),
            1
        );
    }

    #[test]
    fn do_secp256k1_verify_larger_hash_fails() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let mut hash = hex::decode(ECDSA_HASH_HEX).unwrap();
        // extend / break hash
        hash.push(0x00);
        let hash_ptr = write_data(&mut fe_mut, &hash);
        let sig = hex::decode(ECDSA_SIG_HEX).unwrap();
        let sig_ptr = write_data(&mut fe_mut, &sig);
        let pubkey = hex::decode(ECDSA_PUBKEY_HEX).unwrap();
        let pubkey_ptr = write_data(&mut fe_mut, &pubkey);

        let result = do_secp256k1_verify(fe_mut, hash_ptr, sig_ptr, pubkey_ptr);
        match result.unwrap_err() {
            VmError::CommunicationErr {
                source: CommunicationError::RegionLengthTooBig { length, .. },
                ..
            } => assert_eq!(length, MESSAGE_HASH_MAX_LEN + 1),
            e => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn do_secp256k1_verify_shorter_hash_fails() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let mut hash = hex::decode(ECDSA_HASH_HEX).unwrap();
        // reduce / break hash
        hash.pop();
        let hash_ptr = write_data(&mut fe_mut, &hash);
        let sig = hex::decode(ECDSA_SIG_HEX).unwrap();
        let sig_ptr = write_data(&mut fe_mut, &sig);
        let pubkey = hex::decode(ECDSA_PUBKEY_HEX).unwrap();
        let pubkey_ptr = write_data(&mut fe_mut, &pubkey);

        assert_eq!(
            do_secp256k1_verify(fe_mut, hash_ptr, sig_ptr, pubkey_ptr).unwrap(),
            3 // mapped InvalidHashFormat
        );
    }

    #[test]
    fn do_secp256k1_verify_wrong_sig_verify_fails() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let hash = hex::decode(ECDSA_HASH_HEX).unwrap();
        let hash_ptr = write_data(&mut fe_mut, &hash);
        let mut sig = hex::decode(ECDSA_SIG_HEX).unwrap();
        // alter sig
        sig[0] ^= 0x01;
        let sig_ptr = write_data(&mut fe_mut, &sig);
        let pubkey = hex::decode(ECDSA_PUBKEY_HEX).unwrap();
        let pubkey_ptr = write_data(&mut fe_mut, &pubkey);

        assert_eq!(
            do_secp256k1_verify(fe_mut, hash_ptr, sig_ptr, pubkey_ptr).unwrap(),
            1
        );
    }

    #[test]
    fn do_secp256k1_verify_larger_sig_fails() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let hash = hex::decode(ECDSA_HASH_HEX).unwrap();
        let hash_ptr = write_data(&mut fe_mut, &hash);
        let mut sig = hex::decode(ECDSA_SIG_HEX).unwrap();
        // extend / break sig
        sig.push(0x00);
        let sig_ptr = write_data(&mut fe_mut, &sig);
        let pubkey = hex::decode(ECDSA_PUBKEY_HEX).unwrap();
        let pubkey_ptr = write_data(&mut fe_mut, &pubkey);

        let result = do_secp256k1_verify(fe_mut, hash_ptr, sig_ptr, pubkey_ptr);
        match result.unwrap_err() {
            VmError::CommunicationErr {
                source: CommunicationError::RegionLengthTooBig { length, .. },
                ..
            } => assert_eq!(length, ECDSA_SIGNATURE_LEN + 1),
            e => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn do_secp256k1_verify_shorter_sig_fails() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let hash = hex::decode(ECDSA_HASH_HEX).unwrap();
        let hash_ptr = write_data(&mut fe_mut, &hash);
        let mut sig = hex::decode(ECDSA_SIG_HEX).unwrap();
        // reduce / break sig
        sig.pop();
        let sig_ptr = write_data(&mut fe_mut, &sig);
        let pubkey = hex::decode(ECDSA_PUBKEY_HEX).unwrap();
        let pubkey_ptr = write_data(&mut fe_mut, &pubkey);

        assert_eq!(
            do_secp256k1_verify(fe_mut, hash_ptr, sig_ptr, pubkey_ptr).unwrap(),
            4 // mapped InvalidSignatureFormat
        )
    }

    #[test]
    fn do_secp256k1_verify_wrong_pubkey_format_fails() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let hash = hex::decode(ECDSA_HASH_HEX).unwrap();
        let hash_ptr = write_data(&mut fe_mut, &hash);
        let sig = hex::decode(ECDSA_SIG_HEX).unwrap();
        let sig_ptr = write_data(&mut fe_mut, &sig);
        let mut pubkey = hex::decode(ECDSA_PUBKEY_HEX).unwrap();
        // alter pubkey format
        pubkey[0] ^= 0x01;
        let pubkey_ptr = write_data(&mut fe_mut, &pubkey);

        assert_eq!(
            do_secp256k1_verify(fe_mut, hash_ptr, sig_ptr, pubkey_ptr).unwrap(),
            5 // mapped InvalidPubkeyFormat
        )
    }

    #[test]
    fn do_secp256k1_verify_wrong_pubkey_fails() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let hash = hex::decode(ECDSA_HASH_HEX).unwrap();
        let hash_ptr = write_data(&mut fe_mut, &hash);
        let sig = hex::decode(ECDSA_SIG_HEX).unwrap();
        let sig_ptr = write_data(&mut fe_mut, &sig);
        let mut pubkey = hex::decode(ECDSA_PUBKEY_HEX).unwrap();
        // alter pubkey
        pubkey[1] ^= 0x01;
        let pubkey_ptr = write_data(&mut fe_mut, &pubkey);

        assert_eq!(
            do_secp256k1_verify(fe_mut, hash_ptr, sig_ptr, pubkey_ptr).unwrap(),
            10 // mapped GenericErr
        )
    }

    #[test]
    fn do_secp256k1_verify_larger_pubkey_fails() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let hash = hex::decode(ECDSA_HASH_HEX).unwrap();
        let hash_ptr = write_data(&mut fe_mut, &hash);
        let sig = hex::decode(ECDSA_SIG_HEX).unwrap();
        let sig_ptr = write_data(&mut fe_mut, &sig);
        let mut pubkey = hex::decode(ECDSA_PUBKEY_HEX).unwrap();
        // extend / break pubkey
        pubkey.push(0x00);
        let pubkey_ptr = write_data(&mut fe_mut, &pubkey);

        let result = do_secp256k1_verify(fe_mut, hash_ptr, sig_ptr, pubkey_ptr);
        match result.unwrap_err() {
            VmError::CommunicationErr {
                source: CommunicationError::RegionLengthTooBig { length, .. },
                ..
            } => assert_eq!(length, ECDSA_PUBKEY_MAX_LEN + 1),
            e => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn do_secp256k1_verify_shorter_pubkey_fails() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let hash = hex::decode(ECDSA_HASH_HEX).unwrap();
        let hash_ptr = write_data(&mut fe_mut, &hash);
        let sig = hex::decode(ECDSA_SIG_HEX).unwrap();
        let sig_ptr = write_data(&mut fe_mut, &sig);
        let mut pubkey = hex::decode(ECDSA_PUBKEY_HEX).unwrap();
        // reduce / break pubkey
        pubkey.pop();
        let pubkey_ptr = write_data(&mut fe_mut, &pubkey);

        assert_eq!(
            do_secp256k1_verify(fe_mut, hash_ptr, sig_ptr, pubkey_ptr).unwrap(),
            5 // mapped InvalidPubkeyFormat
        )
    }

    #[test]
    fn do_secp256k1_verify_empty_pubkey_fails() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let hash = hex::decode(ECDSA_HASH_HEX).unwrap();
        let hash_ptr = write_data(&mut fe_mut, &hash);
        let sig = hex::decode(ECDSA_SIG_HEX).unwrap();
        let sig_ptr = write_data(&mut fe_mut, &sig);
        let pubkey = vec![];
        let pubkey_ptr = write_data(&mut fe_mut, &pubkey);

        assert_eq!(
            do_secp256k1_verify(fe_mut, hash_ptr, sig_ptr, pubkey_ptr).unwrap(),
            5 // mapped InvalidPubkeyFormat
        )
    }

    #[test]
    fn do_secp256k1_verify_wrong_data_fails() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let hash = vec![0x22; MESSAGE_HASH_MAX_LEN];
        let hash_ptr = write_data(&mut fe_mut, &hash);
        let sig = vec![0x22; ECDSA_SIGNATURE_LEN];
        let sig_ptr = write_data(&mut fe_mut, &sig);
        let pubkey = vec![0x04; ECDSA_PUBKEY_MAX_LEN];
        let pubkey_ptr = write_data(&mut fe_mut, &pubkey);

        assert_eq!(
            do_secp256k1_verify(fe_mut, hash_ptr, sig_ptr, pubkey_ptr).unwrap(),
            10 // mapped GenericErr
        )
    }

    #[test]
    fn do_secp256k1_recover_pubkey_works() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        // https://gist.github.com/webmaster128/130b628d83621a33579751846699ed15
        let hash = hex!("5ae8317d34d1e595e3fa7247db80c0af4320cce1116de187f8f7e2e099c0d8d0");
        let sig = hex!("45c0b7f8c09a9e1f1cea0c25785594427b6bf8f9f878a8af0b1abbb48e16d0920d8becd0c220f67c51217eecfd7184ef0732481c843857e6bc7fc095c4f6b788");
        let recovery_param = 1;
        let expected = hex!("044a071e8a6e10aada2b8cf39fa3b5fb3400b04e99ea8ae64ceea1a977dbeaf5d5f8c8fbd10b71ab14cd561f7df8eb6da50f8a8d81ba564342244d26d1d4211595");

        let hash_ptr = write_data(&mut fe_mut, &hash);
        let sig_ptr = write_data(&mut fe_mut, &sig);
        let result =
            do_secp256k1_recover_pubkey(fe_mut.as_mut(), hash_ptr, sig_ptr, recovery_param)
                .unwrap();
        let error = result >> 32;
        let pubkey_ptr: u32 = (result & 0xFFFFFFFF).try_into().unwrap();
        assert_eq!(error, 0);
        assert_eq!(force_read(&mut fe_mut, pubkey_ptr), expected);
    }

    #[test]
    fn do_ed25519_verify_works() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let msg = hex::decode(EDDSA_MSG_HEX).unwrap();
        let msg_ptr = write_data(&mut fe_mut, &msg);
        let sig = hex::decode(EDDSA_SIG_HEX).unwrap();
        let sig_ptr = write_data(&mut fe_mut, &sig);
        let pubkey = hex::decode(EDDSA_PUBKEY_HEX).unwrap();
        let pubkey_ptr = write_data(&mut fe_mut, &pubkey);

        assert_eq!(
            do_ed25519_verify(fe_mut, msg_ptr, sig_ptr, pubkey_ptr).unwrap(),
            0
        );
    }

    #[test]
    fn do_ed25519_verify_wrong_msg_verify_fails() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let mut msg = hex::decode(EDDSA_MSG_HEX).unwrap();
        // alter msg
        msg.push(0x01);
        let msg_ptr = write_data(&mut fe_mut, &msg);
        let sig = hex::decode(EDDSA_SIG_HEX).unwrap();
        let sig_ptr = write_data(&mut fe_mut, &sig);
        let pubkey = hex::decode(EDDSA_PUBKEY_HEX).unwrap();
        let pubkey_ptr = write_data(&mut fe_mut, &pubkey);

        assert_eq!(
            do_ed25519_verify(fe_mut, msg_ptr, sig_ptr, pubkey_ptr).unwrap(),
            1
        );
    }

    #[test]
    fn do_ed25519_verify_larger_msg_fails() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let mut msg = hex::decode(EDDSA_MSG_HEX).unwrap();
        // extend / break msg
        msg.extend_from_slice(&[0x00; MAX_LENGTH_ED25519_MESSAGE + 1]);
        let msg_ptr = write_data(&mut fe_mut, &msg);
        let sig = hex::decode(EDDSA_SIG_HEX).unwrap();
        let sig_ptr = write_data(&mut fe_mut, &sig);
        let pubkey = hex::decode(EDDSA_PUBKEY_HEX).unwrap();
        let pubkey_ptr = write_data(&mut fe_mut, &pubkey);

        let result = do_ed25519_verify(fe_mut, msg_ptr, sig_ptr, pubkey_ptr);
        match result.unwrap_err() {
            VmError::CommunicationErr {
                source: CommunicationError::RegionLengthTooBig { length, .. },
                ..
            } => assert_eq!(length, msg.len()),
            e => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn do_ed25519_verify_wrong_sig_verify_fails() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let msg = hex::decode(EDDSA_MSG_HEX).unwrap();
        let msg_ptr = write_data(&mut fe_mut, &msg);
        let mut sig = hex::decode(EDDSA_SIG_HEX).unwrap();
        // alter sig
        sig[0] ^= 0x01;
        let sig_ptr = write_data(&mut fe_mut, &sig);
        let pubkey = hex::decode(EDDSA_PUBKEY_HEX).unwrap();
        let pubkey_ptr = write_data(&mut fe_mut, &pubkey);

        assert_eq!(
            do_ed25519_verify(fe_mut, msg_ptr, sig_ptr, pubkey_ptr).unwrap(),
            1
        );
    }

    #[test]
    fn do_ed25519_verify_larger_sig_fails() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let msg = hex::decode(EDDSA_MSG_HEX).unwrap();
        let msg_ptr = write_data(&mut fe_mut, &msg);
        let mut sig = hex::decode(EDDSA_SIG_HEX).unwrap();
        // extend / break sig
        sig.push(0x00);
        let sig_ptr = write_data(&mut fe_mut, &sig);
        let pubkey = hex::decode(EDDSA_PUBKEY_HEX).unwrap();
        let pubkey_ptr = write_data(&mut fe_mut, &pubkey);

        let result = do_ed25519_verify(fe_mut, msg_ptr, sig_ptr, pubkey_ptr);
        match result.unwrap_err() {
            VmError::CommunicationErr {
                source: CommunicationError::RegionLengthTooBig { length, .. },
                ..
            } => assert_eq!(length, MAX_LENGTH_ED25519_SIGNATURE + 1),
            e => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn do_ed25519_verify_shorter_sig_fails() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let msg = hex::decode(EDDSA_MSG_HEX).unwrap();
        let msg_ptr = write_data(&mut fe_mut, &msg);
        let mut sig = hex::decode(EDDSA_SIG_HEX).unwrap();
        // reduce / break sig
        sig.pop();
        let sig_ptr = write_data(&mut fe_mut, &sig);
        let pubkey = hex::decode(EDDSA_PUBKEY_HEX).unwrap();
        let pubkey_ptr = write_data(&mut fe_mut, &pubkey);

        assert_eq!(
            do_ed25519_verify(fe_mut, msg_ptr, sig_ptr, pubkey_ptr).unwrap(),
            4 // mapped InvalidSignatureFormat
        )
    }

    #[test]
    fn do_ed25519_verify_wrong_pubkey_verify_fails() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let msg = hex::decode(EDDSA_MSG_HEX).unwrap();
        let msg_ptr = write_data(&mut fe_mut, &msg);
        let sig = hex::decode(EDDSA_SIG_HEX).unwrap();
        let sig_ptr = write_data(&mut fe_mut, &sig);
        let mut pubkey = hex::decode(EDDSA_PUBKEY_HEX).unwrap();
        // alter pubkey
        pubkey[1] ^= 0x01;
        let pubkey_ptr = write_data(&mut fe_mut, &pubkey);

        assert_eq!(
            do_ed25519_verify(fe_mut, msg_ptr, sig_ptr, pubkey_ptr).unwrap(),
            1
        );
    }

    #[test]
    fn do_ed25519_verify_larger_pubkey_fails() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let msg = hex::decode(EDDSA_MSG_HEX).unwrap();
        let msg_ptr = write_data(&mut fe_mut, &msg);
        let sig = hex::decode(EDDSA_SIG_HEX).unwrap();
        let sig_ptr = write_data(&mut fe_mut, &sig);
        let mut pubkey = hex::decode(EDDSA_PUBKEY_HEX).unwrap();
        // extend / break pubkey
        pubkey.push(0x00);
        let pubkey_ptr = write_data(&mut fe_mut, &pubkey);

        let result = do_ed25519_verify(fe_mut, msg_ptr, sig_ptr, pubkey_ptr);
        match result.unwrap_err() {
            VmError::CommunicationErr {
                source: CommunicationError::RegionLengthTooBig { length, .. },
                ..
            } => assert_eq!(length, EDDSA_PUBKEY_LEN + 1),
            e => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn do_ed25519_verify_shorter_pubkey_fails() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let msg = hex::decode(EDDSA_MSG_HEX).unwrap();
        let msg_ptr = write_data(&mut fe_mut, &msg);
        let sig = hex::decode(EDDSA_SIG_HEX).unwrap();
        let sig_ptr = write_data(&mut fe_mut, &sig);
        let mut pubkey = hex::decode(EDDSA_PUBKEY_HEX).unwrap();
        // reduce / break pubkey
        pubkey.pop();
        let pubkey_ptr = write_data(&mut fe_mut, &pubkey);

        assert_eq!(
            do_ed25519_verify(fe_mut, msg_ptr, sig_ptr, pubkey_ptr).unwrap(),
            5 // mapped InvalidPubkeyFormat
        )
    }

    #[test]
    fn do_ed25519_verify_empty_pubkey_fails() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let msg = hex::decode(EDDSA_MSG_HEX).unwrap();
        let msg_ptr = write_data(&mut fe_mut, &msg);
        let sig = hex::decode(EDDSA_SIG_HEX).unwrap();
        let sig_ptr = write_data(&mut fe_mut, &sig);
        let pubkey = vec![];
        let pubkey_ptr = write_data(&mut fe_mut, &pubkey);

        assert_eq!(
            do_ed25519_verify(fe_mut, msg_ptr, sig_ptr, pubkey_ptr).unwrap(),
            5 // mapped InvalidPubkeyFormat
        )
    }

    #[test]
    fn do_ed25519_verify_wrong_data_fails() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let msg = vec![0x22; MESSAGE_HASH_MAX_LEN];
        let msg_ptr = write_data(&mut fe_mut, &msg);
        let sig = vec![0x22; MAX_LENGTH_ED25519_SIGNATURE];
        let sig_ptr = write_data(&mut fe_mut, &sig);
        let pubkey = vec![0x04; EDDSA_PUBKEY_LEN];
        let pubkey_ptr = write_data(&mut fe_mut, &pubkey);

        assert_eq!(
            do_ed25519_verify(fe_mut, msg_ptr, sig_ptr, pubkey_ptr).unwrap(),
            1 // verification failure
        )
    }

    #[test]
    fn do_query_chain_works() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let request: QueryRequest<Empty> = QueryRequest::Bank(BankQuery::AllBalances {
            address: INIT_ADDR.to_string(),
        });
        let request_data = cosmwasm_std::to_json_vec(&request).unwrap();
        let request_ptr = write_data(&mut fe_mut, &request_data);

        leave_default_data(&mut fe_mut);

        let response_ptr = do_query_chain(fe_mut.as_mut(), request_ptr).unwrap();
        let response = force_read(&mut fe_mut, response_ptr);

        let query_result: cosmwasm_std::QuerierResult = cosmwasm_std::from_json(response).unwrap();
        let query_result_inner = query_result.unwrap();
        let query_result_inner_inner = query_result_inner.unwrap();
        let parsed_again: AllBalanceResponse = from_json(query_result_inner_inner).unwrap();
        assert_eq!(parsed_again.amount, coins(INIT_AMOUNT, INIT_DENOM));
    }

    #[test]
    fn do_query_chain_fails_for_broken_request() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let request = b"Not valid JSON for sure";
        let request_ptr = write_data(&mut fe_mut, request);

        leave_default_data(&mut fe_mut);

        let response_ptr = do_query_chain(fe_mut.as_mut(), request_ptr).unwrap();
        let response = force_read(&mut fe_mut, response_ptr);

        let query_result: cosmwasm_std::QuerierResult = cosmwasm_std::from_json(response).unwrap();
        match query_result {
            SystemResult::Ok(_) => panic!("This must not succeed"),
            SystemResult::Err(SystemError::InvalidRequest { request: err, .. }) => {
                assert_eq!(err.as_slice(), request)
            }
            SystemResult::Err(err) => panic!("Unexpected error: {err:?}"),
        }
    }

    #[test]
    fn do_query_chain_fails_for_missing_contract() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let request: QueryRequest<Empty> = QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: String::from("non-existent"),
            msg: Binary::from(b"{}" as &[u8]),
        });
        let request_data = cosmwasm_std::to_json_vec(&request).unwrap();
        let request_ptr = write_data(&mut fe_mut, &request_data);

        leave_default_data(&mut fe_mut);

        let response_ptr = do_query_chain(fe_mut.as_mut(), request_ptr).unwrap();
        let response = force_read(&mut fe_mut, response_ptr);

        let query_result: cosmwasm_std::QuerierResult = cosmwasm_std::from_json(response).unwrap();
        match query_result {
            SystemResult::Ok(_) => panic!("This must not succeed"),
            SystemResult::Err(SystemError::NoSuchContract { addr }) => {
                assert_eq!(addr, "non-existent")
            }
            SystemResult::Err(err) => panic!("Unexpected error: {err:?}"),
        }
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_db_scan_unbound_works() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);
        leave_default_data(&mut fe_mut);

        // set up iterator over all space
        let id = do_db_scan(fe_mut.as_mut(), 0, 0, Order::Ascending.into()).unwrap();
        assert_eq!(1, id);

        let item = fe_mut
            .data()
            .with_storage_from_context::<_, _>(|store| Ok(store.next(id)))
            .unwrap();
        assert_eq!(item.0.unwrap().unwrap(), (KEY1.to_vec(), VALUE1.to_vec()));

        let item = fe_mut
            .data()
            .with_storage_from_context::<_, _>(|store| Ok(store.next(id)))
            .unwrap();
        assert_eq!(item.0.unwrap().unwrap(), (KEY2.to_vec(), VALUE2.to_vec()));

        let item = fe_mut
            .data()
            .with_storage_from_context::<_, _>(|store| Ok(store.next(id)))
            .unwrap();
        assert!(item.0.unwrap().is_none());
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_db_scan_unbound_descending_works() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);
        leave_default_data(&mut fe_mut);

        // set up iterator over all space
        let id = do_db_scan(fe_mut.as_mut(), 0, 0, Order::Descending.into()).unwrap();
        assert_eq!(1, id);

        let item = fe_mut
            .data()
            .with_storage_from_context::<_, _>(|store| Ok(store.next(id)))
            .unwrap();
        assert_eq!(item.0.unwrap().unwrap(), (KEY2.to_vec(), VALUE2.to_vec()));

        let item = fe_mut
            .data()
            .with_storage_from_context::<_, _>(|store| Ok(store.next(id)))
            .unwrap();
        assert_eq!(item.0.unwrap().unwrap(), (KEY1.to_vec(), VALUE1.to_vec()));

        let item = fe_mut
            .data()
            .with_storage_from_context::<_, _>(|store| Ok(store.next(id)))
            .unwrap();
        assert!(item.0.unwrap().is_none());
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_db_scan_bound_works() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        let start = write_data(&mut fe_mut, b"anna");
        let end = write_data(&mut fe_mut, b"bert");

        leave_default_data(&mut fe_mut);

        let id = do_db_scan(fe_mut.as_mut(), start, end, Order::Ascending.into()).unwrap();

        let item = fe_mut
            .data()
            .with_storage_from_context::<_, _>(|store| Ok(store.next(id)))
            .unwrap();
        assert_eq!(item.0.unwrap().unwrap(), (KEY1.to_vec(), VALUE1.to_vec()));

        let item = fe_mut
            .data()
            .with_storage_from_context::<_, _>(|store| Ok(store.next(id)))
            .unwrap();
        assert!(item.0.unwrap().is_none());
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_db_scan_multiple_iterators() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);
        leave_default_data(&mut fe_mut);

        // unbounded, ascending and descending
        let id1 = do_db_scan(fe_mut.as_mut(), 0, 0, Order::Ascending.into()).unwrap();
        let id2 = do_db_scan(fe_mut.as_mut(), 0, 0, Order::Descending.into()).unwrap();
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);

        // first item, first iterator
        let item = fe_mut
            .data()
            .with_storage_from_context::<_, _>(|store| Ok(store.next(id1)))
            .unwrap();
        assert_eq!(item.0.unwrap().unwrap(), (KEY1.to_vec(), VALUE1.to_vec()));

        // second item, first iterator
        let item = fe_mut
            .data()
            .with_storage_from_context::<_, _>(|store| Ok(store.next(id1)))
            .unwrap();
        assert_eq!(item.0.unwrap().unwrap(), (KEY2.to_vec(), VALUE2.to_vec()));

        // first item, second iterator
        let item = fe_mut
            .data()
            .with_storage_from_context::<_, _>(|store| Ok(store.next(id2)))
            .unwrap();
        assert_eq!(item.0.unwrap().unwrap(), (KEY2.to_vec(), VALUE2.to_vec()));

        // end, first iterator
        let item = fe_mut
            .data()
            .with_storage_from_context::<_, _>(|store| Ok(store.next(id1)))
            .unwrap();
        assert!(item.0.unwrap().is_none());

        // second item, second iterator
        let item = fe_mut
            .data()
            .with_storage_from_context::<_, _>(|store| Ok(store.next(id2)))
            .unwrap();
        assert_eq!(item.0.unwrap().unwrap(), (KEY1.to_vec(), VALUE1.to_vec()));
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_db_scan_errors_for_invalid_order_value() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);
        leave_default_data(&mut fe_mut);

        // set up iterator over all space
        let result = do_db_scan(fe_mut, 0, 0, 42);
        match result.unwrap_err() {
            VmError::CommunicationErr {
                source: CommunicationError::InvalidOrder { .. },
                ..
            } => {}
            e => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_db_next_works() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        leave_default_data(&mut fe_mut);

        let id = do_db_scan(fe_mut.as_mut(), 0, 0, Order::Ascending.into()).unwrap();

        // Entry 1
        let kv_region_ptr = do_db_next(fe_mut.as_mut(), id).unwrap();
        assert_eq!(
            force_read(&mut fe_mut, kv_region_ptr),
            [KEY1, b"\0\0\0\x03", VALUE1, b"\0\0\0\x06"].concat()
        );

        // Entry 2
        let kv_region_ptr = do_db_next(fe_mut.as_mut(), id).unwrap();
        assert_eq!(
            force_read(&mut fe_mut, kv_region_ptr),
            [KEY2, b"\0\0\0\x04", VALUE2, b"\0\0\0\x05"].concat()
        );

        // End
        let kv_region_ptr = do_db_next(fe_mut.as_mut(), id).unwrap();
        assert_eq!(force_read(&mut fe_mut, kv_region_ptr), b"\0\0\0\0\0\0\0\0");
        // API makes no guarantees for value_ptr in this case
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_db_next_fails_for_non_existent_id() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        leave_default_data(&mut fe_mut);

        let non_existent_id = 42u32;
        let result = do_db_next(fe_mut.as_mut(), non_existent_id);
        match result.unwrap_err() {
            VmError::BackendErr {
                source: BackendError::IteratorDoesNotExist { id, .. },
                ..
            } => assert_eq!(id, non_existent_id),
            e => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_db_next_key_works() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        leave_default_data(&mut fe_mut);

        let id = do_db_scan(fe_mut.as_mut(), 0, 0, Order::Ascending.into()).unwrap();

        // Entry 1
        let key_region_ptr = do_db_next_key(fe_mut.as_mut(), id).unwrap();
        assert_eq!(force_read(&mut fe_mut, key_region_ptr), KEY1);

        // Entry 2
        let key_region_ptr = do_db_next_key(fe_mut.as_mut(), id).unwrap();
        assert_eq!(force_read(&mut fe_mut, key_region_ptr), KEY2);

        // End
        let key_region_ptr: u32 = do_db_next_key(fe_mut.as_mut(), id).unwrap();
        assert_eq!(key_region_ptr, 0);
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_db_next_value_works() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        leave_default_data(&mut fe_mut);

        let id = do_db_scan(fe_mut.as_mut(), 0, 0, Order::Ascending.into()).unwrap();

        // Entry 1
        let value_region_ptr = do_db_next_value(fe_mut.as_mut(), id).unwrap();
        assert_eq!(force_read(&mut fe_mut, value_region_ptr), VALUE1);

        // Entry 2
        let value_region_ptr = do_db_next_value(fe_mut.as_mut(), id).unwrap();
        assert_eq!(force_read(&mut fe_mut, value_region_ptr), VALUE2);

        // End
        let value_region_ptr = do_db_next_value(fe_mut.as_mut(), id).unwrap();
        assert_eq!(value_region_ptr, 0);
    }

    #[test]
    #[cfg(feature = "iterator")]
    fn do_db_next_works_mixed() {
        let api = MockApi::default();
        let (fe, mut store, _instance) = make_instance(api);
        let mut fe_mut = fe.into_mut(&mut store);

        leave_default_data(&mut fe_mut);

        let id = do_db_scan(fe_mut.as_mut(), 0, 0, Order::Ascending.into()).unwrap();

        // Key 1
        let key_region_ptr = do_db_next_key(fe_mut.as_mut(), id).unwrap();
        assert_eq!(force_read(&mut fe_mut, key_region_ptr), KEY1);

        // Value 2
        let value_region_ptr = do_db_next_value(fe_mut.as_mut(), id).unwrap();
        assert_eq!(force_read(&mut fe_mut, value_region_ptr), VALUE2);

        // End
        let kv_region_ptr = do_db_next(fe_mut.as_mut(), id).unwrap();
        assert_eq!(force_read(&mut fe_mut, kv_region_ptr), b"\0\0\0\0\0\0\0\0");
    }
}
