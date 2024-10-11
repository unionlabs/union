use cosmwasm_vm::{BackendApi, BackendError, BackendResult, GasInfo};

use crate::error::GoError;
use crate::memory::{U8SliceView, UnmanagedVector};
use crate::Vtable;

// this represents something passed in from the caller side of FFI
// in this case a struct with go function pointers
#[repr(C)]
pub struct api_t {
    _private: [u8; 0],
}

// These functions should return GoError but because we don't trust them here, we treat the return value as i32
// and then check it when converting to GoError manually
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct GoApiVtable {
    pub humanize_address: Option<
        extern "C" fn(
            api: *const api_t,
            input: U8SliceView,
            humanized_address_out: *mut UnmanagedVector,
            err_msg_out: *mut UnmanagedVector,
            gas_used: *mut u64,
        ) -> i32,
    >,
    pub canonicalize_address: Option<
        extern "C" fn(
            api: *const api_t,
            input: U8SliceView,
            canonicalized_address_out: *mut UnmanagedVector,
            err_msg_out: *mut UnmanagedVector,
            gas_used: *mut u64,
        ) -> i32,
    >,
    pub validate_address: Option<
        extern "C" fn(
            api: *const api_t,
            input: U8SliceView,
            err_msg_out: *mut UnmanagedVector,
            gas_used: *mut u64,
        ) -> i32,
    >,
}

impl Vtable for GoApiVtable {}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct GoApi {
    pub state: *const api_t,
    pub vtable: GoApiVtable,
}

// We must declare that these are safe to Send, to use in wasm.
// The known go caller passes in immutable function pointers, but this is indeed
// unsafe for possible other callers.
//
// see: https://stackoverflow.com/questions/50258359/can-a-struct-containing-a-raw-pointer-implement-send-and-be-ffi-safe
unsafe impl Send for GoApi {}

impl BackendApi for GoApi {
    fn addr_canonicalize(&self, human: &str) -> BackendResult<Vec<u8>> {
        let mut output = UnmanagedVector::default();
        let mut error_msg = UnmanagedVector::default();
        let mut used_gas = 0_u64;
        let canonicalize_address = self
            .vtable
            .canonicalize_address
            .expect("vtable function 'canonicalize_address' not set");
        let go_error: GoError = canonicalize_address(
            self.state,
            U8SliceView::new(Some(human.as_bytes())),
            &mut output as *mut UnmanagedVector,
            &mut error_msg as *mut UnmanagedVector,
            &mut used_gas as *mut u64,
        )
        .into();
        // We destruct the UnmanagedVector here, no matter if we need the data.
        let output = output.consume();

        let gas_info = GasInfo::with_cost(used_gas);

        // return complete error message (reading from buffer for GoError::Other)
        let default = || format!("Failed to canonicalize the address: {human}");
        unsafe {
            if let Err(err) = go_error.into_result(error_msg, default) {
                return (Err(err), gas_info);
            }
        }

        let result = output.ok_or_else(|| BackendError::unknown("Unset output"));
        (result, gas_info)
    }

    fn addr_humanize(&self, canonical: &[u8]) -> BackendResult<String> {
        let mut output = UnmanagedVector::default();
        let mut error_msg = UnmanagedVector::default();
        let mut used_gas = 0_u64;
        let humanize_address = self
            .vtable
            .humanize_address
            .expect("vtable function 'humanize_address' not set");
        let go_error: GoError = humanize_address(
            self.state,
            U8SliceView::new(Some(canonical)),
            &mut output as *mut UnmanagedVector,
            &mut error_msg as *mut UnmanagedVector,
            &mut used_gas as *mut u64,
        )
        .into();
        // We destruct the UnmanagedVector here, no matter if we need the data.
        let output = output.consume();

        let gas_info = GasInfo::with_cost(used_gas);

        // return complete error message (reading from buffer for GoError::Other)
        let default = || {
            format!(
                "Failed to humanize the address: {}",
                hex::encode_upper(canonical)
            )
        };
        unsafe {
            if let Err(err) = go_error.into_result(error_msg, default) {
                return (Err(err), gas_info);
            }
        }

        let result = output
            .ok_or_else(|| BackendError::unknown("Unset output"))
            .and_then(|human_data| String::from_utf8(human_data).map_err(BackendError::from));
        (result, gas_info)
    }

    fn addr_validate(&self, input: &str) -> BackendResult<()> {
        let mut error_msg = UnmanagedVector::default();
        let mut used_gas = 0_u64;
        let validate_address = self
            .vtable
            .validate_address
            .expect("vtable function 'validate_address' not set");
        let go_error: GoError = validate_address(
            self.state,
            U8SliceView::new(Some(input.as_bytes())),
            &mut error_msg as *mut UnmanagedVector,
            &mut used_gas as *mut u64,
        )
        .into();

        let gas_info = GasInfo::with_cost(used_gas);

        // return complete error message (reading from buffer for GoError::Other)
        let default = || format!("Failed to validate the address: {input}");
        let result = unsafe { go_error.into_result(error_msg, default) };
        (result, gas_info)
    }
}
