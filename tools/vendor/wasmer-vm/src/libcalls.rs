// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

//! Runtime library calls.
//!
//! Note that Wasm compilers may sometimes perform these inline rather than
//! calling them, particularly when CPUs have special instructions which compute
//! them directly.
//!
//! These functions are called by compiled Wasm code, and therefore must take
//! certain care about some things:
//!
//! * They must always be `pub extern "C"` and should only contain basic, raw
//!   i32/i64/f32/f64/pointer parameters that are safe to pass across the system
//!   ABI!
//!
//! * If any nested function propagates an `Err(trap)` out to the library
//!   function frame, we need to raise it. This involves some nasty and quite
//!   unsafe code under the covers! Notable, after raising the trap, drops
//!   **will not** be run for local variables! This can lead to things like
//!   leaking `VMInstance`s which leads to never deallocating JIT code,
//!   instances, and modules! Therefore, always use nested blocks to ensure
//!   drops run before raising a trap:
//!
//!   ```ignore
//!   pub extern "C" fn my_lib_function(...) {
//!       let result = {
//!           // Do everything in here so drops run at the end of the block.
//!           ...
//!       };
//!       if let Err(trap) = result {
//!           // Now we can safely raise the trap without leaking!
//!           raise_lib_trap(trap);
//!       }
//!   }
//!   ```

#![allow(missing_docs)] // For some reason lint fails saying that `LibCall` is not documented, when it actually is

use crate::probestack::PROBESTACK;
use crate::table::{RawTableElement, TableElement};
use crate::trap::{raise_lib_trap, Trap, TrapCode};
use crate::vmcontext::VMContext;
use crate::{on_host_stack, VMFuncRef};
pub use wasmer_types::LibCall;
use wasmer_types::{
    DataIndex, ElemIndex, FunctionIndex, LocalMemoryIndex, LocalTableIndex, MemoryIndex,
    TableIndex, Type,
};

/// Implementation of f32.ceil
#[no_mangle]
pub extern "C" fn wasmer_vm_f32_ceil(x: f32) -> f32 {
    x.ceil()
}

/// Implementation of f32.floor
#[no_mangle]
pub extern "C" fn wasmer_vm_f32_floor(x: f32) -> f32 {
    x.floor()
}

/// Implementation of f32.trunc
#[no_mangle]
pub extern "C" fn wasmer_vm_f32_trunc(x: f32) -> f32 {
    x.trunc()
}

/// Implementation of f32.nearest
#[allow(clippy::float_arithmetic, clippy::float_cmp)]
#[no_mangle]
pub extern "C" fn wasmer_vm_f32_nearest(x: f32) -> f32 {
    // Rust doesn't have a nearest function, so do it manually.
    if x == 0.0 {
        // Preserve the sign of zero.
        x
    } else {
        // Nearest is either ceil or floor depending on which is nearest or even.
        let u = x.ceil();
        let d = x.floor();
        let um = (x - u).abs();
        let dm = (x - d).abs();
        if um < dm
            || (um == dm && {
                let h = u / 2.;
                h.floor() == h
            })
        {
            u
        } else {
            d
        }
    }
}

/// Implementation of f64.ceil
#[no_mangle]
pub extern "C" fn wasmer_vm_f64_ceil(x: f64) -> f64 {
    x.ceil()
}

/// Implementation of f64.floor
#[no_mangle]
pub extern "C" fn wasmer_vm_f64_floor(x: f64) -> f64 {
    x.floor()
}

/// Implementation of f64.trunc
#[no_mangle]
pub extern "C" fn wasmer_vm_f64_trunc(x: f64) -> f64 {
    x.trunc()
}

/// Implementation of f64.nearest
#[allow(clippy::float_arithmetic, clippy::float_cmp)]
#[no_mangle]
pub extern "C" fn wasmer_vm_f64_nearest(x: f64) -> f64 {
    // Rust doesn't have a nearest function, so do it manually.
    if x == 0.0 {
        // Preserve the sign of zero.
        x
    } else {
        // Nearest is either ceil or floor depending on which is nearest or even.
        let u = x.ceil();
        let d = x.floor();
        let um = (x - u).abs();
        let dm = (x - d).abs();
        if um < dm
            || (um == dm && {
                let h = u / 2.;
                h.floor() == h
            })
        {
            u
        } else {
            d
        }
    }
}

/// Implementation of memory.grow for locally-defined 32-bit memories.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_memory32_grow(
    vmctx: *mut VMContext,
    delta: u32,
    memory_index: u32,
) -> u32 {
    on_host_stack(|| {
        let instance = (*vmctx).instance_mut();
        let memory_index = LocalMemoryIndex::from_u32(memory_index);

        instance
            .memory_grow(memory_index, delta)
            .map(|pages| pages.0)
            .unwrap_or(u32::max_value())
    })
}

/// Implementation of memory.grow for imported 32-bit memories.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_imported_memory32_grow(
    vmctx: *mut VMContext,
    delta: u32,
    memory_index: u32,
) -> u32 {
    on_host_stack(|| {
        let instance = (*vmctx).instance_mut();
        let memory_index = MemoryIndex::from_u32(memory_index);

        instance
            .imported_memory_grow(memory_index, delta)
            .map(|pages| pages.0)
            .unwrap_or(u32::max_value())
    })
}

/// Implementation of memory.size for locally-defined 32-bit memories.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_memory32_size(vmctx: *mut VMContext, memory_index: u32) -> u32 {
    let instance = (*vmctx).instance();
    let memory_index = LocalMemoryIndex::from_u32(memory_index);

    instance.memory_size(memory_index).0
}

/// Implementation of memory.size for imported 32-bit memories.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_imported_memory32_size(
    vmctx: *mut VMContext,
    memory_index: u32,
) -> u32 {
    let instance = (*vmctx).instance();
    let memory_index = MemoryIndex::from_u32(memory_index);

    instance.imported_memory_size(memory_index).0
}

/// Implementation of `table.copy`.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_table_copy(
    vmctx: *mut VMContext,
    dst_table_index: u32,
    src_table_index: u32,
    dst: u32,
    src: u32,
    len: u32,
) {
    let result = {
        let dst_table_index = TableIndex::from_u32(dst_table_index);
        let src_table_index = TableIndex::from_u32(src_table_index);
        if dst_table_index == src_table_index {
            let table = (*vmctx).instance_mut().get_table(dst_table_index);
            table.copy_within(dst, src, len)
        } else {
            let dst_table = (*vmctx).instance_mut().get_table(dst_table_index);
            let src_table = (*vmctx).instance_mut().get_table(src_table_index);
            dst_table.copy(src_table, dst, src, len)
        }
    };
    if let Err(trap) = result {
        raise_lib_trap(trap);
    }
}

/// Implementation of `table.init`.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_table_init(
    vmctx: *mut VMContext,
    table_index: u32,
    elem_index: u32,
    dst: u32,
    src: u32,
    len: u32,
) {
    let result = {
        let table_index = TableIndex::from_u32(table_index);
        let elem_index = ElemIndex::from_u32(elem_index);
        let instance = (*vmctx).instance_mut();
        instance.table_init(table_index, elem_index, dst, src, len)
    };
    if let Err(trap) = result {
        raise_lib_trap(trap);
    }
}

/// Implementation of `table.fill`.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_table_fill(
    vmctx: *mut VMContext,
    table_index: u32,
    start_idx: u32,
    item: RawTableElement,
    len: u32,
) {
    let result = {
        let table_index = TableIndex::from_u32(table_index);
        let instance = (*vmctx).instance_mut();
        let elem = match instance.get_table(table_index).ty().ty {
            Type::ExternRef => TableElement::ExternRef(item.extern_ref),
            Type::FuncRef => TableElement::FuncRef(item.func_ref),
            _ => panic!("Unrecognized table type: does not contain references"),
        };

        instance.table_fill(table_index, start_idx, elem, len)
    };
    if let Err(trap) = result {
        raise_lib_trap(trap);
    }
}

/// Implementation of `table.size`.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_table_size(vmctx: *mut VMContext, table_index: u32) -> u32 {
    let instance = (*vmctx).instance();
    let table_index = LocalTableIndex::from_u32(table_index);

    instance.table_size(table_index)
}

/// Implementation of `table.size` for imported tables.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_imported_table_size(
    vmctx: *mut VMContext,
    table_index: u32,
) -> u32 {
    let instance = (*vmctx).instance();
    let table_index = TableIndex::from_u32(table_index);

    instance.imported_table_size(table_index)
}

/// Implementation of `table.get`.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_table_get(
    vmctx: *mut VMContext,
    table_index: u32,
    elem_index: u32,
) -> RawTableElement {
    let instance = (*vmctx).instance();
    let table_index = LocalTableIndex::from_u32(table_index);

    // TODO: type checking, maybe have specialized accessors
    match instance.table_get(table_index, elem_index) {
        Some(table_ref) => table_ref.into(),
        None => raise_lib_trap(Trap::lib(TrapCode::TableAccessOutOfBounds)),
    }
}

/// Implementation of `table.get` for imported tables.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_imported_table_get(
    vmctx: *mut VMContext,
    table_index: u32,
    elem_index: u32,
) -> RawTableElement {
    let instance = (*vmctx).instance_mut();
    let table_index = TableIndex::from_u32(table_index);

    // TODO: type checking, maybe have specialized accessors
    match instance.imported_table_get(table_index, elem_index) {
        Some(table_ref) => table_ref.into(),
        None => raise_lib_trap(Trap::lib(TrapCode::TableAccessOutOfBounds)),
    }
}

/// Implementation of `table.set`.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
///
/// It is the caller's responsibility to increment the ref count of any ref counted
/// type before passing it to this function.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_table_set(
    vmctx: *mut VMContext,
    table_index: u32,
    elem_index: u32,
    value: RawTableElement,
) {
    let instance = (*vmctx).instance_mut();
    let table_index = TableIndex::from_u32(table_index);
    let table_index = instance
        .module_ref()
        .local_table_index(table_index)
        .unwrap();

    let elem = match instance.get_local_table(table_index).ty().ty {
        Type::ExternRef => TableElement::ExternRef(value.extern_ref),
        Type::FuncRef => TableElement::FuncRef(value.func_ref),
        _ => panic!("Unrecognized table type: does not contain references"),
    };

    // TODO: type checking, maybe have specialized accessors
    let result = instance.table_set(table_index, elem_index, elem);

    if let Err(trap) = result {
        raise_lib_trap(trap);
    }
}

/// Implementation of `table.set` for imported tables.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_imported_table_set(
    vmctx: *mut VMContext,
    table_index: u32,
    elem_index: u32,
    value: RawTableElement,
) {
    let instance = (*vmctx).instance_mut();
    let table_index = TableIndex::from_u32(table_index);
    let elem = match instance.get_table(table_index).ty().ty {
        Type::ExternRef => TableElement::ExternRef(value.extern_ref),
        Type::FuncRef => TableElement::FuncRef(value.func_ref),
        _ => panic!("Unrecognized table type: does not contain references"),
    };

    let result = instance.imported_table_set(table_index, elem_index, elem);

    if let Err(trap) = result {
        raise_lib_trap(trap);
    }
}

/// Implementation of `table.grow` for locally-defined tables.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_table_grow(
    vmctx: *mut VMContext,
    init_value: RawTableElement,
    delta: u32,
    table_index: u32,
) -> u32 {
    on_host_stack(|| {
        let instance = (*vmctx).instance_mut();
        let table_index = LocalTableIndex::from_u32(table_index);

        let init_value = match instance.get_local_table(table_index).ty().ty {
            Type::ExternRef => TableElement::ExternRef(init_value.extern_ref),
            Type::FuncRef => TableElement::FuncRef(init_value.func_ref),
            _ => panic!("Unrecognized table type: does not contain references"),
        };

        instance
            .table_grow(table_index, delta, init_value)
            .unwrap_or(u32::max_value())
    })
}

/// Implementation of `table.grow` for imported tables.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_imported_table_grow(
    vmctx: *mut VMContext,
    init_value: RawTableElement,
    delta: u32,
    table_index: u32,
) -> u32 {
    on_host_stack(|| {
        let instance = (*vmctx).instance_mut();
        let table_index = TableIndex::from_u32(table_index);
        let init_value = match instance.get_table(table_index).ty().ty {
            Type::ExternRef => TableElement::ExternRef(init_value.extern_ref),
            Type::FuncRef => TableElement::FuncRef(init_value.func_ref),
            _ => panic!("Unrecognized table type: does not contain references"),
        };

        instance
            .imported_table_grow(table_index, delta, init_value)
            .unwrap_or(u32::max_value())
    })
}

/// Implementation of `func.ref`.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_func_ref(
    vmctx: *mut VMContext,
    function_index: u32,
) -> VMFuncRef {
    let instance = (*vmctx).instance();
    let function_index = FunctionIndex::from_u32(function_index);

    instance.func_ref(function_index).unwrap()
}

/// Implementation of `elem.drop`.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_elem_drop(vmctx: *mut VMContext, elem_index: u32) {
    on_host_stack(|| {
        let elem_index = ElemIndex::from_u32(elem_index);
        let instance = (*vmctx).instance();
        instance.elem_drop(elem_index);
    })
}

/// Implementation of `memory.copy` for locally defined memories.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_memory32_copy(
    vmctx: *mut VMContext,
    memory_index: u32,
    dst: u32,
    src: u32,
    len: u32,
) {
    let result = {
        let memory_index = LocalMemoryIndex::from_u32(memory_index);
        let instance = (*vmctx).instance();
        instance.local_memory_copy(memory_index, dst, src, len)
    };
    if let Err(trap) = result {
        raise_lib_trap(trap);
    }
}

/// Implementation of `memory.copy` for imported memories.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_imported_memory32_copy(
    vmctx: *mut VMContext,
    memory_index: u32,
    dst: u32,
    src: u32,
    len: u32,
) {
    let result = {
        let memory_index = MemoryIndex::from_u32(memory_index);
        let instance = (*vmctx).instance();
        instance.imported_memory_copy(memory_index, dst, src, len)
    };
    if let Err(trap) = result {
        raise_lib_trap(trap);
    }
}

/// Implementation of `memory.fill` for locally defined memories.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_memory32_fill(
    vmctx: *mut VMContext,
    memory_index: u32,
    dst: u32,
    val: u32,
    len: u32,
) {
    let result = {
        let memory_index = LocalMemoryIndex::from_u32(memory_index);
        let instance = (*vmctx).instance();
        instance.local_memory_fill(memory_index, dst, val, len)
    };
    if let Err(trap) = result {
        raise_lib_trap(trap);
    }
}

/// Implementation of `memory.fill` for imported memories.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_imported_memory32_fill(
    vmctx: *mut VMContext,
    memory_index: u32,
    dst: u32,
    val: u32,
    len: u32,
) {
    let result = {
        let memory_index = MemoryIndex::from_u32(memory_index);
        let instance = (*vmctx).instance();
        instance.imported_memory_fill(memory_index, dst, val, len)
    };
    if let Err(trap) = result {
        raise_lib_trap(trap);
    }
}

/// Implementation of `memory.init`.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_memory32_init(
    vmctx: *mut VMContext,
    memory_index: u32,
    data_index: u32,
    dst: u32,
    src: u32,
    len: u32,
) {
    let result = {
        let memory_index = MemoryIndex::from_u32(memory_index);
        let data_index = DataIndex::from_u32(data_index);
        let instance = (*vmctx).instance();
        instance.memory_init(memory_index, data_index, dst, src, len)
    };
    if let Err(trap) = result {
        raise_lib_trap(trap);
    }
}

/// Implementation of `data.drop`.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_data_drop(vmctx: *mut VMContext, data_index: u32) {
    on_host_stack(|| {
        let data_index = DataIndex::from_u32(data_index);
        let instance = (*vmctx).instance();
        instance.data_drop(data_index)
    })
}

/// Implementation for raising a trap
///
/// # Safety
///
/// Only safe to call when wasm code is on the stack, aka `wasmer_call` or
/// `wasmer_call_trampoline` must have been previously called.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_raise_trap(trap_code: TrapCode) -> ! {
    let trap = Trap::lib(trap_code);
    raise_lib_trap(trap)
}

/// Probestack check
///
/// # Safety
///
/// This function does not follow the standard function ABI, and is called as
/// part of the function prologue.
#[no_mangle]
pub static wasmer_vm_probestack: unsafe extern "C" fn() = PROBESTACK;

/// Implementation of memory.wait32 for locally-defined 32-bit memories.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_memory32_atomic_wait32(
    vmctx: *mut VMContext,
    memory_index: u32,
    dst: u32,
    val: u32,
    timeout: i64,
) -> u32 {
    let result = {
        let instance = (*vmctx).instance_mut();
        let memory_index = LocalMemoryIndex::from_u32(memory_index);

        instance.local_memory_wait32(memory_index, dst, val, timeout)
    };
    if let Err(trap) = result {
        raise_lib_trap(trap);
    }
    result.unwrap()
}

/// Implementation of memory.wait32 for imported 32-bit memories.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_imported_memory32_atomic_wait32(
    vmctx: *mut VMContext,
    memory_index: u32,
    dst: u32,
    val: u32,
    timeout: i64,
) -> u32 {
    let result = {
        let instance = (*vmctx).instance_mut();
        let memory_index = MemoryIndex::from_u32(memory_index);

        instance.imported_memory_wait32(memory_index, dst, val, timeout)
    };
    if let Err(trap) = result {
        raise_lib_trap(trap);
    }
    result.unwrap()
}

/// Implementation of memory.wait64 for locally-defined 32-bit memories.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_memory32_atomic_wait64(
    vmctx: *mut VMContext,
    memory_index: u32,
    dst: u32,
    val: u64,
    timeout: i64,
) -> u32 {
    let result = {
        let instance = (*vmctx).instance_mut();
        let memory_index = LocalMemoryIndex::from_u32(memory_index);

        instance.local_memory_wait64(memory_index, dst, val, timeout)
    };
    if let Err(trap) = result {
        raise_lib_trap(trap);
    }
    result.unwrap()
}

/// Implementation of memory.wait64 for imported 32-bit memories.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_imported_memory32_atomic_wait64(
    vmctx: *mut VMContext,
    memory_index: u32,
    dst: u32,
    val: u64,
    timeout: i64,
) -> u32 {
    let result = {
        let instance = (*vmctx).instance_mut();
        let memory_index = MemoryIndex::from_u32(memory_index);

        instance.imported_memory_wait64(memory_index, dst, val, timeout)
    };
    if let Err(trap) = result {
        raise_lib_trap(trap);
    }
    result.unwrap()
}

/// Implementation of memory.notfy for locally-defined 32-bit memories.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_memory32_atomic_notify(
    vmctx: *mut VMContext,
    memory_index: u32,
    dst: u32,
    cnt: u32,
) -> u32 {
    let result = {
        let instance = (*vmctx).instance_mut();
        let memory_index = LocalMemoryIndex::from_u32(memory_index);

        instance.local_memory_notify(memory_index, dst, cnt)
    };
    if let Err(trap) = result {
        raise_lib_trap(trap);
    }
    result.unwrap()
}

/// Implementation of memory.notfy for imported 32-bit memories.
///
/// # Safety
///
/// `vmctx` must be dereferenceable.
#[no_mangle]
pub unsafe extern "C" fn wasmer_vm_imported_memory32_atomic_notify(
    vmctx: *mut VMContext,
    memory_index: u32,
    dst: u32,
    cnt: u32,
) -> u32 {
    let result = {
        let instance = (*vmctx).instance_mut();
        let memory_index = MemoryIndex::from_u32(memory_index);

        instance.imported_memory_notify(memory_index, dst, cnt)
    };
    if let Err(trap) = result {
        raise_lib_trap(trap);
    }
    result.unwrap()
}

/// The function pointer to a libcall
pub fn function_pointer(libcall: LibCall) -> usize {
    match libcall {
        LibCall::CeilF32 => wasmer_vm_f32_ceil as usize,
        LibCall::CeilF64 => wasmer_vm_f64_ceil as usize,
        LibCall::FloorF32 => wasmer_vm_f32_floor as usize,
        LibCall::FloorF64 => wasmer_vm_f64_floor as usize,
        LibCall::NearestF32 => wasmer_vm_f32_nearest as usize,
        LibCall::NearestF64 => wasmer_vm_f64_nearest as usize,
        LibCall::TruncF32 => wasmer_vm_f32_trunc as usize,
        LibCall::TruncF64 => wasmer_vm_f64_trunc as usize,
        LibCall::Memory32Size => wasmer_vm_memory32_size as usize,
        LibCall::ImportedMemory32Size => wasmer_vm_imported_memory32_size as usize,
        LibCall::TableCopy => wasmer_vm_table_copy as usize,
        LibCall::TableInit => wasmer_vm_table_init as usize,
        LibCall::TableFill => wasmer_vm_table_fill as usize,
        LibCall::TableSize => wasmer_vm_table_size as usize,
        LibCall::ImportedTableSize => wasmer_vm_imported_table_size as usize,
        LibCall::TableGet => wasmer_vm_table_get as usize,
        LibCall::ImportedTableGet => wasmer_vm_imported_table_get as usize,
        LibCall::TableSet => wasmer_vm_table_set as usize,
        LibCall::ImportedTableSet => wasmer_vm_imported_table_set as usize,
        LibCall::TableGrow => wasmer_vm_table_grow as usize,
        LibCall::ImportedTableGrow => wasmer_vm_imported_table_grow as usize,
        LibCall::FuncRef => wasmer_vm_func_ref as usize,
        LibCall::ElemDrop => wasmer_vm_elem_drop as usize,
        LibCall::Memory32Copy => wasmer_vm_memory32_copy as usize,
        LibCall::ImportedMemory32Copy => wasmer_vm_imported_memory32_copy as usize,
        LibCall::Memory32Fill => wasmer_vm_memory32_fill as usize,
        LibCall::ImportedMemory32Fill => wasmer_vm_imported_memory32_fill as usize,
        LibCall::Memory32Init => wasmer_vm_memory32_init as usize,
        LibCall::DataDrop => wasmer_vm_data_drop as usize,
        LibCall::Probestack => wasmer_vm_probestack as usize,
        LibCall::RaiseTrap => wasmer_vm_raise_trap as usize,
        LibCall::Memory32AtomicWait32 => wasmer_vm_memory32_atomic_wait32 as usize,
        LibCall::ImportedMemory32AtomicWait32 => wasmer_vm_imported_memory32_atomic_wait32 as usize,
        LibCall::Memory32AtomicWait64 => wasmer_vm_memory32_atomic_wait64 as usize,
        LibCall::ImportedMemory32AtomicWait64 => wasmer_vm_imported_memory32_atomic_wait64 as usize,
        LibCall::Memory32AtomicNotify => wasmer_vm_memory32_atomic_notify as usize,
        LibCall::ImportedMemory32AtomicNotify => wasmer_vm_imported_memory32_atomic_notify as usize,
    }
}
