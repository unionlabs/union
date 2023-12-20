use wasmer::wasmparser::Operator;
use wasmer::{
    FunctionMiddleware, LocalFunctionIndex, MiddlewareError, MiddlewareReaderState,
    ModuleMiddleware,
};

#[derive(Debug, Clone, Copy)]
struct GatekeeperConfig {
    /// True iff float operations are allowed.
    ///
    /// Note: there are float operations in the SIMD block as well and we do not yet handle
    /// any combination of `allow_floats` and `allow_feature_simd` properly.
    allow_floats: bool,
    //
    // Standardized features
    //
    /// True iff operations of the "Bulk memory operations" feature are allowed.
    /// See <https://webassembly.org/roadmap/> and <https://github.com/WebAssembly/bulk-memory-operations/blob/master/proposals/bulk-memory-operations/Overview.md>.
    allow_feature_bulk_memory_operations: bool,
    /// True iff operations of the "Reference types" feature are allowed.
    /// See <https://webassembly.org/roadmap/> and <https://github.com/WebAssembly/reference-types/blob/master/proposals/reference-types/Overview.md>.
    allow_feature_reference_types: bool,
    /// True iff operations of the "Fixed-width SIMD" feature are allowed.
    /// See <https://webassembly.org/roadmap/> and <https://github.com/WebAssembly/simd/blob/master/proposals/simd/SIMD.md>.
    allow_feature_simd: bool,
    //
    // In-progress proposals
    //
    /// True iff operations of the "Exception handling" feature are allowed.
    /// Note, this feature is not yet standardized!
    /// See <https://webassembly.org/roadmap/> and <https://github.com/WebAssembly/exception-handling/blob/master/proposals/exception-handling/Exceptions.md>.
    allow_feature_exception_handling: bool,
    /// True iff operations of the "Threads and atomics" feature are allowed.
    /// Note, this feature is not yet standardized!
    /// See <https://webassembly.org/roadmap/> and <https://github.com/WebAssembly/threads/blob/master/proposals/threads/Overview.md>.
    allow_feature_threads: bool,
}

/// A middleware that ensures only deterministic operations are used (i.e. no floats).
/// It also disallows the use of Wasm features that are not explicitly enabled.
#[derive(Debug)]
#[non_exhaustive]
pub struct Gatekeeper {
    config: GatekeeperConfig,
}

impl Gatekeeper {
    /// Creates a new Gatekeeper with a custom config.
    ///
    /// A costum configuration is potentially dangerous (non-final Wasm proposals, floats in SIMD operation).
    /// For this reason, only [`Gatekeeper::default()`] is public.
    fn new(config: GatekeeperConfig) -> Self {
        Self { config }
    }
}

impl Default for Gatekeeper {
    fn default() -> Self {
        Self::new(GatekeeperConfig {
            allow_floats: true,
            allow_feature_bulk_memory_operations: false,
            allow_feature_reference_types: false,
            allow_feature_simd: false,
            allow_feature_exception_handling: false,
            allow_feature_threads: false,
        })
    }
}

impl ModuleMiddleware for Gatekeeper {
    /// Generates a `FunctionMiddleware` for a given function.
    fn generate_function_middleware(&self, _: LocalFunctionIndex) -> Box<dyn FunctionMiddleware> {
        Box::new(FunctionGatekeeper::new(self.config))
    }
}

#[derive(Debug)]
#[non_exhaustive]
struct FunctionGatekeeper {
    config: GatekeeperConfig,
}

impl FunctionGatekeeper {
    fn new(config: GatekeeperConfig) -> Self {
        Self { config }
    }
}

/// The name used in errors
const MIDDLEWARE_NAME: &str = "Gatekeeper";

impl FunctionMiddleware for FunctionGatekeeper {
    fn feed<'a>(
        &mut self,
        operator: Operator<'a>,
        state: &mut MiddlewareReaderState<'a>,
    ) -> Result<(), MiddlewareError> {
        match operator {
            Operator::Unreachable
            | Operator::Nop
            | Operator::Block { .. }
            | Operator::Loop { .. }
            | Operator::If { .. }
            | Operator::Else
            | Operator::End
            | Operator::Br { .. }
            | Operator::BrIf { .. }
            | Operator::BrTable { .. }
            | Operator::Return
            | Operator::Call { .. }
            | Operator::CallIndirect { .. }
            | Operator::Drop
            | Operator::Select
            | Operator::LocalGet { .. }
            | Operator::LocalSet { .. }
            | Operator::LocalTee { .. }
            | Operator::GlobalGet { .. }
            | Operator::GlobalSet { .. }
            | Operator::I32Load { .. }
            | Operator::I64Load { .. }
            | Operator::I32Load8S { .. }
            | Operator::I32Load8U { .. }
            | Operator::I32Load16S { .. }
            | Operator::I32Load16U { .. }
            | Operator::I64Load8S { .. }
            | Operator::I64Load8U { .. }
            | Operator::I64Load16S { .. }
            | Operator::I64Load16U { .. }
            | Operator::I64Load32S { .. }
            | Operator::I64Load32U { .. }
            | Operator::I32Store { .. }
            | Operator::I64Store { .. }
            | Operator::I32Store8 { .. }
            | Operator::I32Store16 { .. }
            | Operator::I64Store8 { .. }
            | Operator::I64Store16 { .. }
            | Operator::I64Store32 { .. }
            | Operator::MemorySize { .. }
            | Operator::MemoryGrow { .. }
            | Operator::I32Const { .. }
            | Operator::I64Const { .. }
            | Operator::I32Eqz
            | Operator::I32Eq
            | Operator::I32Ne
            | Operator::I32LtS
            | Operator::I32LtU
            | Operator::I32GtS
            | Operator::I32GtU
            | Operator::I32LeS
            | Operator::I32LeU
            | Operator::I32GeS
            | Operator::I32GeU
            | Operator::I64Eqz
            | Operator::I64Eq
            | Operator::I64Ne
            | Operator::I64LtS
            | Operator::I64LtU
            | Operator::I64GtS
            | Operator::I64GtU
            | Operator::I64LeS
            | Operator::I64LeU
            | Operator::I64GeS
            | Operator::I64GeU
            | Operator::I32Clz
            | Operator::I32Ctz
            | Operator::I32Popcnt
            | Operator::I32Add
            | Operator::I32Sub
            | Operator::I32Mul
            | Operator::I32DivS
            | Operator::I32DivU
            | Operator::I32RemS
            | Operator::I32RemU
            | Operator::I32And
            | Operator::I32Or
            | Operator::I32Xor
            | Operator::I32Shl
            | Operator::I32ShrS
            | Operator::I32ShrU
            | Operator::I32Rotl
            | Operator::I32Rotr
            | Operator::I64Clz
            | Operator::I64Ctz
            | Operator::I64Popcnt
            | Operator::I64Add
            | Operator::I64Sub
            | Operator::I64Mul
            | Operator::I64DivS
            | Operator::I64DivU
            | Operator::I64RemS
            | Operator::I64RemU
            | Operator::I64And
            | Operator::I64Or
            | Operator::I64Xor
            | Operator::I64Shl
            | Operator::I64ShrS
            | Operator::I64ShrU
            | Operator::I64Rotl
            | Operator::I64Rotr
            | Operator::I32WrapI64
            // Those are part of the MVP
            // https://github.com/bytecodealliance/wasm-tools/blob/wasmparser-0.107.0/crates/wasmparser/src/lib.rs#L287-L288
            | Operator::I64ExtendI32S
            | Operator::I64ExtendI32U
            // Sign-extension
            // https://github.com/bytecodealliance/wasm-tools/blob/wasmparser-0.107.0/crates/wasmparser/src/lib.rs#L307-L311
            | Operator::I32Extend8S
            | Operator::I32Extend16S
            | Operator::I64Extend8S
            | Operator::I64Extend16S
            | Operator::I64Extend32S => {
                state.push_operator(operator);
                Ok(())
            }
            Operator::RefNull { .. }
            | Operator::RefIsNull
            | Operator::RefFunc { .. }
            | Operator::ReturnCall { .. }
            | Operator::ReturnCallIndirect { .. }
            | Operator::TypedSelect { .. }
            | Operator::TableGet { .. }
            | Operator::TableSet { .. }
            | Operator::TableGrow { .. }
            | Operator::TableSize { .. } => {
                if self.config.allow_feature_reference_types {
                    state.push_operator(operator);
                    Ok(())
                } else {
                    let msg = format!("Reference type operation detected: {operator:?}. Reference types are not supported.");
                    Err(MiddlewareError::new(MIDDLEWARE_NAME, msg))
                }
            }
            Operator::MemoryAtomicNotify { .. }
            | Operator::MemoryAtomicWait32 { .. }
            | Operator::MemoryAtomicWait64 { .. }
            | Operator::AtomicFence { .. }
            | Operator::I32AtomicLoad { .. }
            | Operator::I64AtomicLoad { .. }
            | Operator::I32AtomicLoad8U { .. }
            | Operator::I32AtomicLoad16U { .. }
            | Operator::I64AtomicLoad8U { .. }
            | Operator::I64AtomicLoad16U { .. }
            | Operator::I64AtomicLoad32U { .. }
            | Operator::I32AtomicStore { .. }
            | Operator::I64AtomicStore { .. }
            | Operator::I32AtomicStore8 { .. }
            | Operator::I32AtomicStore16 { .. }
            | Operator::I64AtomicStore8 { .. }
            | Operator::I64AtomicStore16 { .. }
            | Operator::I64AtomicStore32 { .. }
            | Operator::I32AtomicRmwAdd { .. }
            | Operator::I64AtomicRmwAdd { .. }
            | Operator::I32AtomicRmw8AddU { .. }
            | Operator::I32AtomicRmw16AddU { .. }
            | Operator::I64AtomicRmw8AddU { .. }
            | Operator::I64AtomicRmw16AddU { .. }
            | Operator::I64AtomicRmw32AddU { .. }
            | Operator::I32AtomicRmwSub { .. }
            | Operator::I64AtomicRmwSub { .. }
            | Operator::I32AtomicRmw8SubU { .. }
            | Operator::I32AtomicRmw16SubU { .. }
            | Operator::I64AtomicRmw8SubU { .. }
            | Operator::I64AtomicRmw16SubU { .. }
            | Operator::I64AtomicRmw32SubU { .. }
            | Operator::I32AtomicRmwAnd { .. }
            | Operator::I64AtomicRmwAnd { .. }
            | Operator::I32AtomicRmw8AndU { .. }
            | Operator::I32AtomicRmw16AndU { .. }
            | Operator::I64AtomicRmw8AndU { .. }
            | Operator::I64AtomicRmw16AndU { .. }
            | Operator::I64AtomicRmw32AndU { .. }
            | Operator::I32AtomicRmwOr { .. }
            | Operator::I64AtomicRmwOr { .. }
            | Operator::I32AtomicRmw8OrU { .. }
            | Operator::I32AtomicRmw16OrU { .. }
            | Operator::I64AtomicRmw8OrU { .. }
            | Operator::I64AtomicRmw16OrU { .. }
            | Operator::I64AtomicRmw32OrU { .. }
            | Operator::I32AtomicRmwXor { .. }
            | Operator::I64AtomicRmwXor { .. }
            | Operator::I32AtomicRmw8XorU { .. }
            | Operator::I32AtomicRmw16XorU { .. }
            | Operator::I64AtomicRmw8XorU { .. }
            | Operator::I64AtomicRmw16XorU { .. }
            | Operator::I64AtomicRmw32XorU { .. }
            | Operator::I32AtomicRmwXchg { .. }
            | Operator::I64AtomicRmwXchg { .. }
            | Operator::I32AtomicRmw8XchgU { .. }
            | Operator::I32AtomicRmw16XchgU { .. }
            | Operator::I64AtomicRmw8XchgU { .. }
            | Operator::I64AtomicRmw16XchgU { .. }
            | Operator::I64AtomicRmw32XchgU { .. }
            | Operator::I32AtomicRmwCmpxchg { .. }
            | Operator::I64AtomicRmwCmpxchg { .. }
            | Operator::I32AtomicRmw8CmpxchgU { .. }
            | Operator::I32AtomicRmw16CmpxchgU { .. }
            | Operator::I64AtomicRmw8CmpxchgU { .. }
            | Operator::I64AtomicRmw16CmpxchgU { .. }
            | Operator::I64AtomicRmw32CmpxchgU { .. } => {
                if self.config.allow_feature_threads {
                    state.push_operator(operator);
                    Ok(())
                } else {
                    let msg = format!("Threads operator detected: {operator:?}. The Wasm Threads extension is not supported.");
                    Err(MiddlewareError::new(MIDDLEWARE_NAME, msg))
                }
            }
            Operator::V128Load { .. }
            | Operator::V128Store { .. }
            | Operator::V128Const { .. }
            | Operator::I8x16Splat
            | Operator::I8x16ExtractLaneS { .. }
            | Operator::I8x16ExtractLaneU { .. }
            | Operator::I8x16ReplaceLane { .. }
            | Operator::I16x8Splat
            | Operator::I16x8ExtractLaneS { .. }
            | Operator::I16x8ExtractLaneU { .. }
            | Operator::I16x8ReplaceLane { .. }
            | Operator::I32x4Splat
            | Operator::I32x4ExtractLane { .. }
            | Operator::I32x4ReplaceLane { .. }
            | Operator::I64x2Splat
            | Operator::I64x2ExtractLane { .. }
            | Operator::I64x2ReplaceLane { .. }
            | Operator::I8x16Eq
            | Operator::I8x16Ne
            | Operator::I8x16LtS
            | Operator::I8x16LtU
            | Operator::I8x16GtS
            | Operator::I8x16GtU
            | Operator::I8x16LeS
            | Operator::I8x16LeU
            | Operator::I8x16GeS
            | Operator::I8x16GeU
            | Operator::I16x8Eq
            | Operator::I16x8Ne
            | Operator::I16x8LtS
            | Operator::I16x8LtU
            | Operator::I16x8GtS
            | Operator::I16x8GtU
            | Operator::I16x8LeS
            | Operator::I16x8LeU
            | Operator::I16x8GeS
            | Operator::I16x8GeU
            | Operator::I32x4Eq
            | Operator::I32x4Ne
            | Operator::I32x4LtS
            | Operator::I32x4LtU
            | Operator::I32x4GtS
            | Operator::I32x4GtU
            | Operator::I32x4LeS
            | Operator::I32x4LeU
            | Operator::I32x4GeS
            | Operator::I32x4GeU
            | Operator::V128Not
            | Operator::V128And
            | Operator::V128AndNot
            | Operator::V128Or
            | Operator::V128Xor
            | Operator::V128Bitselect
            | Operator::I8x16Abs
            | Operator::I8x16Neg
            | Operator::V128AnyTrue
            | Operator::I8x16AllTrue
            | Operator::I8x16Bitmask
            | Operator::I8x16Shl
            | Operator::I8x16ShrS
            | Operator::I8x16ShrU
            | Operator::I8x16Add
            | Operator::I8x16AddSatS
            | Operator::I8x16AddSatU
            | Operator::I8x16Sub
            | Operator::I8x16SubSatS
            | Operator::I8x16SubSatU
            | Operator::I8x16MinS
            | Operator::I8x16MinU
            | Operator::I8x16MaxS
            | Operator::I8x16MaxU
            | Operator::I16x8Abs
            | Operator::I16x8Neg
            | Operator::I16x8AllTrue
            | Operator::I16x8Bitmask
            | Operator::I16x8Shl
            | Operator::I16x8ShrS
            | Operator::I16x8ShrU
            | Operator::I16x8Add
            | Operator::I16x8AddSatS
            | Operator::I16x8AddSatU
            | Operator::I16x8Sub
            | Operator::I16x8SubSatS
            | Operator::I16x8SubSatU
            | Operator::I16x8Mul
            | Operator::I16x8MinS
            | Operator::I16x8MinU
            | Operator::I16x8MaxS
            | Operator::I16x8MaxU
            | Operator::I32x4Abs
            | Operator::I32x4Neg
            | Operator::I32x4AllTrue
            | Operator::I32x4Bitmask
            | Operator::I32x4Shl
            | Operator::I32x4ShrS
            | Operator::I32x4ShrU
            | Operator::I32x4Add
            | Operator::I32x4Sub
            | Operator::I32x4Mul
            | Operator::I32x4MinS
            | Operator::I32x4MinU
            | Operator::I32x4MaxS
            | Operator::I32x4MaxU
            | Operator::I32x4DotI16x8S
            | Operator::I64x2Neg
            | Operator::I64x2Shl
            | Operator::I64x2ShrS
            | Operator::I64x2ShrU
            | Operator::I64x2Add
            | Operator::I64x2Sub
            | Operator::I64x2Mul
            | Operator::I8x16Swizzle
            | Operator::I8x16Shuffle { .. }
            | Operator::V128Load8Splat { .. }
            | Operator::V128Load16Splat { .. }
            | Operator::V128Load32Splat { .. }
            | Operator::V128Load32Zero { .. }
            | Operator::V128Load64Splat { .. }
            | Operator::V128Load64Zero { .. }
            | Operator::I8x16NarrowI16x8S
            | Operator::I8x16NarrowI16x8U
            | Operator::I16x8NarrowI32x4S
            | Operator::I16x8NarrowI32x4U
            | Operator::I16x8ExtendLowI8x16S
            | Operator::I16x8ExtendHighI8x16S
            | Operator::I16x8ExtendLowI8x16U
            | Operator::I16x8ExtendHighI8x16U
            | Operator::I32x4ExtendLowI16x8S
            | Operator::I32x4ExtendHighI16x8S
            | Operator::I32x4ExtendLowI16x8U
            | Operator::I32x4ExtendHighI16x8U
            | Operator::V128Load8x8S { .. }
            | Operator::V128Load8x8U { .. }
            | Operator::V128Load16x4S { .. }
            | Operator::V128Load16x4U { .. }
            | Operator::V128Load32x2S { .. }
            | Operator::V128Load32x2U { .. }
            | Operator::V128Load8Lane { .. }
            | Operator::V128Load16Lane { .. }
            | Operator::V128Load32Lane { .. }
            | Operator::V128Load64Lane { .. }
            | Operator::V128Store8Lane { .. }
            | Operator::V128Store16Lane { .. }
            | Operator::V128Store32Lane { .. }
            | Operator::V128Store64Lane { .. }
            | Operator::I64x2Eq
            | Operator::I64x2Ne
            | Operator::I64x2LtS
            | Operator::I64x2GtS
            | Operator::I64x2LeS
            | Operator::I64x2GeS
            | Operator::I8x16Popcnt
            | Operator::I16x8AvgrU
            | Operator::I16x8ExtAddPairwiseI8x16S
            | Operator::I16x8ExtAddPairwiseI8x16U
            | Operator::I16x8Q15MulrSatS
            | Operator::I16x8ExtMulLowI8x16S
            | Operator::I16x8ExtMulHighI8x16S
            | Operator::I16x8ExtMulLowI8x16U
            | Operator::I16x8ExtMulHighI8x16U
            | Operator::I32x4ExtAddPairwiseI16x8S
            | Operator::I32x4ExtAddPairwiseI16x8U
            | Operator::I32x4ExtMulLowI16x8S
            | Operator::I32x4ExtMulHighI16x8S
            | Operator::I32x4ExtMulLowI16x8U
            | Operator::I32x4ExtMulHighI16x8U
            | Operator::I64x2Abs
            | Operator::I64x2AllTrue
            | Operator::I64x2Bitmask
            | Operator::I64x2ExtendLowI32x4S
            | Operator::I64x2ExtendHighI32x4S
            | Operator::I64x2ExtendLowI32x4U
            | Operator::I64x2ExtendHighI32x4U
            | Operator::I64x2ExtMulLowI32x4S
            | Operator::I64x2ExtMulHighI32x4S
            | Operator::I64x2ExtMulLowI32x4U
            | Operator::I64x2ExtMulHighI32x4U
            | Operator::I32x4TruncSatF64x2SZero
            | Operator::I32x4TruncSatF64x2UZero
            | Operator::F64x2ConvertLowI32x4S
            | Operator::F64x2ConvertLowI32x4U
            | Operator::F32x4DemoteF64x2Zero
            | Operator::F64x2PromoteLowF32x4
            | Operator::I8x16AvgrU => {
                if self.config.allow_feature_simd {
                    state.push_operator(operator);
                    Ok(())
                } else {
                    let msg = format!(
                        "SIMD operator detected: {operator:?}. The Wasm SIMD extension is not supported."
                    );
                    Err(MiddlewareError::new(MIDDLEWARE_NAME, msg))
                }
            }
            // Relaxed SIMD operators
            Operator::I8x16RelaxedSwizzle
            | Operator::I32x4RelaxedTruncSatF32x4S
            | Operator::I32x4RelaxedTruncSatF32x4U
            | Operator::I32x4RelaxedTruncSatF64x2SZero
            | Operator::I32x4RelaxedTruncSatF64x2UZero
            | Operator::F32x4RelaxedFma
            | Operator::F32x4RelaxedFnma
            | Operator::F64x2RelaxedFma
            | Operator::F64x2RelaxedFnma
            | Operator::I8x16RelaxedLaneselect
            | Operator::I16x8RelaxedLaneselect
            | Operator::I32x4RelaxedLaneselect
            | Operator::I64x2RelaxedLaneselect
            | Operator::F32x4RelaxedMin
            | Operator::F32x4RelaxedMax
            | Operator::F64x2RelaxedMin
            | Operator::F64x2RelaxedMax
            | Operator::I16x8RelaxedQ15mulrS
            | Operator::I16x8DotI8x16I7x16S
            | Operator::I32x4DotI8x16I7x16AddS
            | Operator::F32x4RelaxedDotBf16x8AddF32x4 => {
                let msg = format!(
                    "Relaxed SIMD operator detected: {operator:?}. The Wasm Relaxed SIMD extension is not supported."
                );
                Err(MiddlewareError::new(MIDDLEWARE_NAME, msg))
            }
            Operator::F32Load { .. }
            | Operator::F64Load { .. }
            | Operator::F32Store { .. }
            | Operator::F64Store { .. }
            | Operator::F32Const { .. }
            | Operator::F64Const { .. }
            | Operator::F32Eq
            | Operator::F32Ne
            | Operator::F32Lt
            | Operator::F32Gt
            | Operator::F32Le
            | Operator::F32Ge
            | Operator::F64Eq
            | Operator::F64Ne
            | Operator::F64Lt
            | Operator::F64Gt
            | Operator::F64Le
            | Operator::F64Ge
            | Operator::F32Abs
            | Operator::F32Neg
            | Operator::F32Ceil
            | Operator::F32Floor
            | Operator::F32Trunc
            | Operator::F32Nearest
            | Operator::F32Sqrt
            | Operator::F32Add
            | Operator::F32Sub
            | Operator::F32Mul
            | Operator::F32Div
            | Operator::F32Min
            | Operator::F32Max
            | Operator::F32Copysign
            | Operator::F64Abs
            | Operator::F64Neg
            | Operator::F64Ceil
            | Operator::F64Floor
            | Operator::F64Trunc
            | Operator::F64Nearest
            | Operator::F64Sqrt
            | Operator::F64Add
            | Operator::F64Sub
            | Operator::F64Mul
            | Operator::F64Div
            | Operator::F64Min
            | Operator::F64Max
            | Operator::F64Copysign
            | Operator::I32TruncF32S
            | Operator::I32TruncF32U
            | Operator::I32TruncF64S
            | Operator::I32TruncF64U
            | Operator::I64TruncF32S
            | Operator::I64TruncF32U
            | Operator::I64TruncF64S
            | Operator::I64TruncF64U
            | Operator::F32ConvertI32S
            | Operator::F32ConvertI32U
            | Operator::F32ConvertI64S
            | Operator::F32ConvertI64U
            | Operator::F32DemoteF64
            | Operator::F64ConvertI32S
            | Operator::F64ConvertI32U
            | Operator::F64ConvertI64S
            | Operator::F64ConvertI64U
            | Operator::F64PromoteF32
            | Operator::I32ReinterpretF32
            | Operator::I64ReinterpretF64
            | Operator::F32ReinterpretI32
            | Operator::F64ReinterpretI64
            | Operator::I32TruncSatF32S
            | Operator::I32TruncSatF32U
            | Operator::I32TruncSatF64S
            | Operator::I32TruncSatF64U
            | Operator::I64TruncSatF32S
            | Operator::I64TruncSatF32U
            | Operator::I64TruncSatF64S
            | Operator::I64TruncSatF64U
            | Operator::F32x4Splat
            | Operator::F32x4ExtractLane { .. }
            | Operator::F32x4ReplaceLane { .. }
            | Operator::F64x2Splat
            | Operator::F64x2ExtractLane { .. }
            | Operator::F64x2ReplaceLane { .. }
            | Operator::F32x4Eq
            | Operator::F32x4Ne
            | Operator::F32x4Lt
            | Operator::F32x4Gt
            | Operator::F32x4Le
            | Operator::F32x4Ge
            | Operator::F64x2Eq
            | Operator::F64x2Ne
            | Operator::F64x2Lt
            | Operator::F64x2Gt
            | Operator::F64x2Le
            | Operator::F64x2Ge
            | Operator::F32x4Ceil
            | Operator::F32x4Floor
            | Operator::F32x4Trunc
            | Operator::F32x4Nearest
            | Operator::F64x2Ceil
            | Operator::F64x2Floor
            | Operator::F64x2Trunc
            | Operator::F64x2Nearest
            | Operator::F32x4Abs
            | Operator::F32x4Neg
            | Operator::F32x4Sqrt
            | Operator::F32x4Add
            | Operator::F32x4Sub
            | Operator::F32x4Mul
            | Operator::F32x4Div
            | Operator::F32x4Min
            | Operator::F32x4Max
            | Operator::F32x4PMin
            | Operator::F32x4PMax
            | Operator::F64x2Abs
            | Operator::F64x2Neg
            | Operator::F64x2Sqrt
            | Operator::F64x2Add
            | Operator::F64x2Sub
            | Operator::F64x2Mul
            | Operator::F64x2Div
            | Operator::F64x2Min
            | Operator::F64x2Max
            | Operator::F64x2PMin
            | Operator::F64x2PMax
            | Operator::I32x4TruncSatF32x4S
            | Operator::I32x4TruncSatF32x4U
            | Operator::F32x4ConvertI32x4S
            | Operator::F32x4ConvertI32x4U => {
                if self.config.allow_floats {
                    state.push_operator(operator);
                    Ok(())
                } else {
                    let msg = format!(
                        "Float operator detected: {operator:?}. The use of floats is not supported."
                    );
                    Err(MiddlewareError::new(MIDDLEWARE_NAME, msg))
                }
            }
            Operator::MemoryInit { .. }
            | Operator::DataDrop { .. }
            | Operator::MemoryCopy { .. }
            | Operator::MemoryFill { .. }
            | Operator::TableInit { .. }
            | Operator::ElemDrop { .. }
            | Operator::TableCopy { .. }
            | Operator::TableFill { .. } => {
                if self.config.allow_feature_bulk_memory_operations {
                    state.push_operator(operator);
                    Ok(())
                } else {
                    let msg = format!("Bulk memory operation detected: {operator:?}. Bulk memory operations are not supported.");
                    Err(MiddlewareError::new(MIDDLEWARE_NAME, msg))
                }
            }
            Operator::Try { .. }
            | Operator::Catch { .. }
            | Operator::Throw { .. }
            | Operator::Rethrow { .. }
            | Operator::Delegate { .. }
            | Operator::CatchAll => {
                if self.config.allow_feature_exception_handling {
                    state.push_operator(operator);
                    Ok(())
                } else {
                    let msg = format!("Exception handling operation detected: {operator:?}. Exception handling is not supported.");
                    Err(MiddlewareError::new(MIDDLEWARE_NAME, msg))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use wasmer::{CompilerConfig, Cranelift, Module, Store};

    #[test]
    fn valid_wasm_instance_sanity() {
        let wasm = wat::parse_str(
            r#"
            (module
                (func (export "sum") (param i32 i32) (result i32)
                    get_local 0
                    get_local 1
                    i32.add
                ))
            "#,
        )
        .unwrap();

        let deterministic = Arc::new(Gatekeeper::default());
        let mut compiler = Cranelift::default();
        compiler.push_middleware(deterministic);
        let store = Store::new(compiler);
        let result = Module::new(&store, wasm);
        assert!(result.is_ok());
    }

    #[test]
    fn parser_floats_are_supported() {
        let wasm = wat::parse_str(
            r#"
            (module
                (func $to_float (param i32) (result f32)
                    get_local 0
                    f32.convert_u/i32
                ))
            "#,
        )
        .unwrap();

        let deterministic = Arc::new(Gatekeeper::default());
        let mut compiler = Cranelift::default();
        compiler.push_middleware(deterministic);
        let store = Store::new(compiler);
        let result = Module::new(&store, wasm);
        assert!(result.is_ok());
    }

    #[test]
    fn bulk_operations_not_supported() {
        let wasm = wat::parse_str(
            r#"
            (module
              (memory (export "memory") 1)
              (func (param $dst i32) (param $src i32) (param $size i32) (result i32)
                local.get $dst
                local.get $src
                local.get $size
                memory.copy
                local.get $dst))
            "#,
        )
        .unwrap();

        let deterministic = Arc::new(Gatekeeper::default());
        let mut compiler = Cranelift::default();
        compiler.push_middleware(deterministic);
        let store = Store::new(compiler);
        let result = Module::new(&store, wasm);
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Bulk memory operation"));
    }
}
