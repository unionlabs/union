use cosmwasm_std::{Binary, ContractResult, SystemError, SystemResult};
use cosmwasm_vm::{BackendResult, GasInfo, Querier};

use crate::error::GoError;
use crate::memory::{U8SliceView, UnmanagedVector};
use crate::Vtable;

// this represents something passed in from the caller side of FFI
#[repr(C)]
#[derive(Clone)]
pub struct querier_t {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Default)]
pub struct QuerierVtable {
    // We return errors through the return buffer, but may return non-zero error codes on panic
    pub query_external: Option<
        extern "C" fn(
            querier: *const querier_t,
            gas_limit: u64,
            gas_used: *mut u64,
            request: U8SliceView,
            result_out: *mut UnmanagedVector, // A JSON encoded SystemResult<ContractResult<Binary>>
            err_msg_out: *mut UnmanagedVector,
        ) -> i32,
    >,
}

impl Vtable for QuerierVtable {}

#[repr(C)]
#[derive(Clone)]
pub struct GoQuerier {
    pub state: *const querier_t,
    pub vtable: QuerierVtable,
}

// TODO: check if we can do this safer...
unsafe impl Send for GoQuerier {}

impl Querier for GoQuerier {
    fn query_raw(
        &self,
        request: &[u8],
        gas_limit: u64,
    ) -> BackendResult<SystemResult<ContractResult<Binary>>> {
        let mut output = UnmanagedVector::default();
        let mut error_msg = UnmanagedVector::default();
        let mut used_gas = 0_u64;
        let query_external = self
            .vtable
            .query_external
            .expect("vtable function 'query_external' not set");
        let go_result: GoError = query_external(
            self.state,
            gas_limit,
            &mut used_gas as *mut u64,
            U8SliceView::new(Some(request)),
            &mut output as *mut UnmanagedVector,
            &mut error_msg as *mut UnmanagedVector,
        )
        .into();
        // We destruct the UnmanagedVector here, no matter if we need the data.
        let output = output.consume();

        let gas_info = GasInfo::with_externally_used(used_gas);

        // return complete error message (reading from buffer for GoError::Other)
        let default = || {
            format!(
                "Failed to query another contract with this request: {}",
                String::from_utf8_lossy(request)
            )
        };
        unsafe {
            if let Err(err) = go_result.into_result(error_msg, default) {
                return (Err(err), gas_info);
            }
        }

        let bin_result: Vec<u8> = output.unwrap_or_default();
        let sys_result: SystemResult<_> = serde_json::from_slice(&bin_result).unwrap_or_else(|e| {
            SystemResult::Err(SystemError::InvalidResponse {
                error: format!("Parsing Go response: {e}"),
                response: bin_result.into(),
            })
        });
        (Ok(sys_result), gas_info)
    }
}
