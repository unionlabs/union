// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

//! All the runtime support necessary for the wasm to cranelift translation is formalized by the
//! traits `FunctionEnvironment`.

use super::func_state::FuncTranslationState;
use super::translation_utils::reference_type;
use core::convert::From;
use cranelift_codegen::cursor::FuncCursor;
use cranelift_codegen::ir::immediates::Offset32;
use cranelift_codegen::ir::{self, InstBuilder};
use cranelift_codegen::isa::TargetFrontendConfig;
use cranelift_frontend::FunctionBuilder;
use wasmer_compiler::wasmparser::{Operator, Type};
use wasmer_compiler::WasmResult;
use wasmer_types::{
    FunctionIndex, FunctionType, GlobalIndex, LocalFunctionIndex, MemoryIndex, SignatureIndex,
    TableIndex, Type as WasmerType,
};

/// The value of a WebAssembly global variable.
#[derive(Clone, Copy)]
pub enum GlobalVariable {
    #[allow(dead_code)]
    /// This is a constant global with a value known at compile time.
    Const(ir::Value),

    /// This is a variable in memory that should be referenced through a `GlobalValue`.
    Memory {
        /// The address of the global variable storage.
        gv: ir::GlobalValue,
        /// An offset to add to the address.
        offset: Offset32,
        /// The global variable's type.
        ty: ir::Type,
    },

    #[allow(dead_code)]
    /// This is a global variable that needs to be handled by the environment.
    Custom,
}

#[allow(dead_code)]
/// How to return from functions.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ReturnMode {
    /// Use normal return instructions as needed.
    NormalReturns,
    /// Use a single fallthrough return at the end of the function.
    FallthroughReturn,
}

/// Environment affecting the translation of a WebAssembly.
pub trait TargetEnvironment {
    /// Get the information needed to produce Cranelift IR for the given target.
    fn target_config(&self) -> TargetFrontendConfig;

    /// Get the Cranelift integer type to use for native pointers.
    ///
    /// This returns `I64` for 64-bit architectures and `I32` for 32-bit architectures.
    fn pointer_type(&self) -> ir::Type {
        ir::Type::int(u16::from(self.target_config().pointer_bits())).unwrap()
    }

    /// Get the size of a native pointer, in bytes.
    fn pointer_bytes(&self) -> u8 {
        self.target_config().pointer_bytes()
    }

    /// Get the Cranelift reference type to use for native references.
    ///
    /// This returns `R64` for 64-bit architectures and `R32` for 32-bit architectures.
    fn reference_type(&self) -> ir::Type {
        reference_type(self.target_config()).expect("expected reference type")
    }
}

/// Environment affecting the translation of a single WebAssembly function.
///
/// A `FuncEnvironment` trait object is required to translate a WebAssembly function to Cranelift
/// IR. The function environment provides information about the WebAssembly module as well as the
/// runtime environment.
pub trait FuncEnvironment: TargetEnvironment {
    /// Is the given parameter of the given function a wasm-level parameter, as opposed to a hidden
    /// parameter added for use by the implementation?
    fn is_wasm_parameter(&self, signature: &ir::Signature, index: usize) -> bool {
        signature.params[index].purpose == ir::ArgumentPurpose::Normal
    }

    /// Is the given return of the given function a wasm-level parameter, as
    /// opposed to a hidden parameter added for use by the implementation?
    fn is_wasm_return(&self, signature: &ir::Signature, index: usize) -> bool {
        signature.returns[index].purpose == ir::ArgumentPurpose::Normal
    }

    /// Should the code be structured to use a single `fallthrough_return` instruction at the end
    /// of the function body, rather than `return` instructions as needed? This is used by VMs
    /// to append custom epilogues.
    fn return_mode(&self) -> ReturnMode {
        ReturnMode::NormalReturns
    }

    /// Set up the necessary preamble definitions in `func` to access the global variable
    /// identified by `index`.
    ///
    /// The index space covers both imported globals and globals defined by the module.
    ///
    /// Return the global variable reference that should be used to access the global and the
    /// WebAssembly type of the global.
    fn make_global(
        &mut self,
        func: &mut ir::Function,
        index: GlobalIndex,
    ) -> WasmResult<GlobalVariable>;

    /// Set up the necessary preamble definitions in `func` to access the linear memory identified
    /// by `index`.
    ///
    /// The index space covers both imported and locally declared memories.
    fn make_heap(&mut self, func: &mut ir::Function, index: MemoryIndex) -> WasmResult<ir::Heap>;

    /// Set up the necessary preamble definitions in `func` to access the table identified
    /// by `index`.
    ///
    /// The index space covers both imported and locally declared tables.
    fn make_table(&mut self, func: &mut ir::Function, index: TableIndex) -> WasmResult<ir::Table>;

    /// Set up a signature definition in the preamble of `func` that can be used for an indirect
    /// call with signature `index`.
    ///
    /// The signature may contain additional arguments needed for an indirect call, but the
    /// arguments marked as `ArgumentPurpose::Normal` must correspond to the WebAssembly signature
    /// arguments.
    ///
    /// The signature will only be used for indirect calls, even if the module has direct function
    /// calls with the same WebAssembly type.
    fn make_indirect_sig(
        &mut self,
        func: &mut ir::Function,
        index: SignatureIndex,
    ) -> WasmResult<ir::SigRef>;

    /// Set up an external function definition in the preamble of `func` that can be used to
    /// directly call the function `index`.
    ///
    /// The index space covers both imported functions and functions defined in the current module.
    ///
    /// The function's signature may contain additional arguments needed for a direct call, but the
    /// arguments marked as `ArgumentPurpose::Normal` must correspond to the WebAssembly signature
    /// arguments.
    ///
    /// The function's signature will only be used for direct calls, even if the module has
    /// indirect calls with the same WebAssembly type.
    fn make_direct_func(
        &mut self,
        func: &mut ir::Function,
        index: FunctionIndex,
    ) -> WasmResult<ir::FuncRef>;

    /// Translate a `call_indirect` WebAssembly instruction at `pos`.
    ///
    /// Insert instructions at `pos` for an indirect call to the function `callee` in the table
    /// `table_index` with WebAssembly signature `sig_index`. The `callee` value will have type
    /// `i32`.
    ///
    /// The signature `sig_ref` was previously created by `make_indirect_sig()`.
    ///
    /// Return the call instruction whose results are the WebAssembly return values.
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::too_many_arguments))]
    fn translate_call_indirect(
        &mut self,
        pos: FuncCursor,
        table_index: TableIndex,
        table: ir::Table,
        sig_index: SignatureIndex,
        sig_ref: ir::SigRef,
        callee: ir::Value,
        call_args: &[ir::Value],
    ) -> WasmResult<ir::Inst>;

    /// Translate a `call` WebAssembly instruction at `pos`.
    ///
    /// Insert instructions at `pos` for a direct call to the function `callee_index`.
    ///
    /// The function reference `callee` was previously created by `make_direct_func()`.
    ///
    /// Return the call instruction whose results are the WebAssembly return values.
    fn translate_call(
        &mut self,
        mut pos: FuncCursor,
        _callee_index: FunctionIndex,
        callee: ir::FuncRef,
        call_args: &[ir::Value],
    ) -> WasmResult<ir::Inst> {
        Ok(pos.ins().call(callee, call_args))
    }

    /// Translate a `memory.grow` WebAssembly instruction.
    ///
    /// The `index` provided identifies the linear memory to grow, and `heap` is the heap reference
    /// returned by `make_heap` for the same index.
    ///
    /// The `val` value is the requested memory size in pages.
    ///
    /// Returns the old size (in pages) of the memory.
    fn translate_memory_grow(
        &mut self,
        pos: FuncCursor,
        index: MemoryIndex,
        heap: ir::Heap,
        val: ir::Value,
    ) -> WasmResult<ir::Value>;

    /// Translates a `memory.size` WebAssembly instruction.
    ///
    /// The `index` provided identifies the linear memory to query, and `heap` is the heap reference
    /// returned by `make_heap` for the same index.
    ///
    /// Returns the size in pages of the memory.
    fn translate_memory_size(
        &mut self,
        pos: FuncCursor,
        index: MemoryIndex,
        heap: ir::Heap,
    ) -> WasmResult<ir::Value>;

    /// Translate a `memory.copy` WebAssembly instruction.
    ///
    /// The `index` provided identifies the linear memory to query, and `heap` is the heap reference
    /// returned by `make_heap` for the same index.
    fn translate_memory_copy(
        &mut self,
        pos: FuncCursor,
        src_index: MemoryIndex,
        src_heap: ir::Heap,
        dst_index: MemoryIndex,
        dst_heap: ir::Heap,
        dst: ir::Value,
        src: ir::Value,
        len: ir::Value,
    ) -> WasmResult<()>;

    /// Translate a `memory.fill` WebAssembly instruction.
    ///
    /// The `index` provided identifies the linear memory to query, and `heap` is the heap reference
    /// returned by `make_heap` for the same index.
    fn translate_memory_fill(
        &mut self,
        pos: FuncCursor,
        index: MemoryIndex,
        heap: ir::Heap,
        dst: ir::Value,
        val: ir::Value,
        len: ir::Value,
    ) -> WasmResult<()>;

    /// Translate a `memory.init` WebAssembly instruction.
    ///
    /// The `index` provided identifies the linear memory to query, and `heap` is the heap reference
    /// returned by `make_heap` for the same index. `seg_index` is the index of the segment to copy
    /// from.
    #[allow(clippy::too_many_arguments)]
    fn translate_memory_init(
        &mut self,
        pos: FuncCursor,
        index: MemoryIndex,
        heap: ir::Heap,
        seg_index: u32,
        dst: ir::Value,
        src: ir::Value,
        len: ir::Value,
    ) -> WasmResult<()>;

    /// Translate a `data.drop` WebAssembly instruction.
    fn translate_data_drop(&mut self, pos: FuncCursor, seg_index: u32) -> WasmResult<()>;

    /// Translate a `table.size` WebAssembly instruction.
    fn translate_table_size(
        &mut self,
        pos: FuncCursor,
        index: TableIndex,
        table: ir::Table,
    ) -> WasmResult<ir::Value>;

    /// Translate a `table.grow` WebAssembly instruction.
    fn translate_table_grow(
        &mut self,
        pos: FuncCursor,
        table_index: TableIndex,
        table: ir::Table,
        delta: ir::Value,
        init_value: ir::Value,
    ) -> WasmResult<ir::Value>;

    /// Translate a `table.get` WebAssembly instruction.
    fn translate_table_get(
        &mut self,
        builder: &mut FunctionBuilder,
        table_index: TableIndex,
        table: ir::Table,
        index: ir::Value,
    ) -> WasmResult<ir::Value>;

    /// Translate a `table.set` WebAssembly instruction.
    fn translate_table_set(
        &mut self,
        builder: &mut FunctionBuilder,
        table_index: TableIndex,
        table: ir::Table,
        value: ir::Value,
        index: ir::Value,
    ) -> WasmResult<()>;

    /// Translate a `table.copy` WebAssembly instruction.
    #[allow(clippy::too_many_arguments)]
    fn translate_table_copy(
        &mut self,
        pos: FuncCursor,
        dst_table_index: TableIndex,
        dst_table: ir::Table,
        src_table_index: TableIndex,
        src_table: ir::Table,
        dst: ir::Value,
        src: ir::Value,
        len: ir::Value,
    ) -> WasmResult<()>;

    /// Translate a `table.fill` WebAssembly instruction.
    fn translate_table_fill(
        &mut self,
        pos: FuncCursor,
        table_index: TableIndex,
        dst: ir::Value,
        val: ir::Value,
        len: ir::Value,
    ) -> WasmResult<()>;

    /// Translates an externref ref count increment.
    fn translate_externref_inc(
        &mut self,
        pos: cranelift_codegen::cursor::FuncCursor<'_>,
        externref: ir::Value,
    ) -> WasmResult<()>;

    /// Translates an externref ref count decrement.
    fn translate_externref_dec(
        &mut self,
        pos: cranelift_codegen::cursor::FuncCursor<'_>,
        externref: ir::Value,
    ) -> WasmResult<()>;

    /// Translate a `table.init` WebAssembly instruction.
    #[allow(clippy::too_many_arguments)]
    fn translate_table_init(
        &mut self,
        pos: FuncCursor,
        seg_index: u32,
        table_index: TableIndex,
        table: ir::Table,
        dst: ir::Value,
        src: ir::Value,
        len: ir::Value,
    ) -> WasmResult<()>;

    /// Translate a `elem.drop` WebAssembly instruction.
    fn translate_elem_drop(&mut self, pos: FuncCursor, seg_index: u32) -> WasmResult<()>;

    /// Translate a `ref.null T` WebAssembly instruction.
    ///
    /// By default, translates into a null reference type.
    ///
    /// Override this if you don't use Cranelift reference types for all Wasm
    /// reference types (e.g. you use a raw pointer for `funcref`s) or if the
    /// null sentinel is not a null reference type pointer for your type. If you
    /// override this method, then you should also override
    /// `translate_ref_is_null` as well.
    fn translate_ref_null(&mut self, pos: FuncCursor, ty: Type) -> WasmResult<ir::Value>;
    // {
    //     let _ = ty;
    //     Ok(pos.ins().null(self.reference_type(ty)))
    // }

    /// Translate a `ref.is_null` WebAssembly instruction.
    ///
    /// By default, assumes that `value` is a Cranelift reference type, and that
    /// a null Cranelift reference type is the null value for all Wasm reference
    /// types.
    ///
    /// If you override this method, you probably also want to override
    /// `translate_ref_null` as well.
    fn translate_ref_is_null(
        &mut self,
        mut pos: FuncCursor,
        value: ir::Value,
    ) -> WasmResult<ir::Value> {
        let is_null = pos.ins().is_null(value);
        Ok(pos.ins().bint(ir::types::I64, is_null))
    }

    /// Translate a `ref.func` WebAssembly instruction.
    fn translate_ref_func(
        &mut self,
        pos: FuncCursor,
        func_index: FunctionIndex,
    ) -> WasmResult<ir::Value>;

    /// Translate a `global.get` WebAssembly instruction at `pos` for a global
    /// that is custom.
    fn translate_custom_global_get(
        &mut self,
        pos: FuncCursor,
        global_index: GlobalIndex,
    ) -> WasmResult<ir::Value>;

    /// Translate a `global.set` WebAssembly instruction at `pos` for a global
    /// that is custom.
    fn translate_custom_global_set(
        &mut self,
        pos: FuncCursor,
        global_index: GlobalIndex,
        val: ir::Value,
    ) -> WasmResult<()>;

    /// Translate an `i32.atomic.wait` or `i64.atomic.wait` WebAssembly instruction.
    /// The `index` provided identifies the linear memory containing the value
    /// to wait on, and `heap` is the heap reference returned by `make_heap`
    /// for the same index.  Whether the waited-on value is 32- or 64-bit can be
    /// determined by examining the type of `expected`, which must be only I32 or I64.
    ///
    /// Returns an i32, which is negative if the helper call failed.
    fn translate_atomic_wait(
        &mut self,
        pos: FuncCursor,
        index: MemoryIndex,
        heap: ir::Heap,
        addr: ir::Value,
        expected: ir::Value,
        timeout: ir::Value,
    ) -> WasmResult<ir::Value>;

    /// Translate an `atomic.notify` WebAssembly instruction.
    /// The `index` provided identifies the linear memory containing the value
    /// to wait on, and `heap` is the heap reference returned by `make_heap`
    /// for the same index.
    ///
    /// Returns an i64, which is negative if the helper call failed.
    fn translate_atomic_notify(
        &mut self,
        pos: FuncCursor,
        index: MemoryIndex,
        heap: ir::Heap,
        addr: ir::Value,
        count: ir::Value,
    ) -> WasmResult<ir::Value>;

    /// Emit code at the beginning of every wasm loop.
    ///
    /// This can be used to insert explicit interrupt or safepoint checking at
    /// the beginnings of loops.
    fn translate_loop_header(&mut self, _pos: FuncCursor) -> WasmResult<()> {
        // By default, don't emit anything.
        Ok(())
    }

    /// Optional callback for the `FunctionEnvironment` performing this translation to maintain
    /// internal state or prepare custom state for the operator to translate
    fn before_translate_operator(
        &mut self,
        _op: &Operator,
        _builder: &mut FunctionBuilder,
        _state: &FuncTranslationState,
    ) -> WasmResult<()> {
        Ok(())
    }

    /// Optional callback for the `FunctionEnvironment` performing this translation to maintain
    /// internal state or finalize custom state for the operator that was translated
    fn after_translate_operator(
        &mut self,
        _op: &Operator,
        _builder: &mut FunctionBuilder,
        _state: &FuncTranslationState,
    ) -> WasmResult<()> {
        Ok(())
    }

    /// Get the type of the global at the given index.
    fn get_global_type(&self, global_index: GlobalIndex) -> Option<WasmerType>;

    /// Push a local declaration on to the stack to track the type of locals.
    fn push_local_decl_on_stack(&mut self, ty: WasmerType);

    /// Push locals for a the params of a function on to the stack.
    fn push_params_on_stack(&mut self, function_index: LocalFunctionIndex);

    /// Get the type of the local at the given index.
    fn get_local_type(&self, local_index: u32) -> Option<WasmerType>;

    /// Get the types of all the current locals.
    fn get_local_types(&self) -> &[WasmerType];

    /// Get the type of the local at the given index.
    fn get_function_type(&self, function_index: FunctionIndex) -> Option<&FunctionType>;

    /// Get the type of a function with the given signature index.
    fn get_function_sig(&self, sig_index: SignatureIndex) -> Option<&FunctionType>;

    /// Drops all locals that need to be dropped. Useful for returning from functions.
    fn translate_drop_locals(&mut self, builder: &mut FunctionBuilder) -> WasmResult<()>;
}
