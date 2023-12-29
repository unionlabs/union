use crate::address_map::get_function_address_map;
use crate::codegen_error;
#[cfg(feature = "unwind")]
use crate::dwarf::WriterRelocate;
use crate::location::{Location, Reg};
use crate::machine::{Label, Machine, MachineStackOffset, NATIVE_PAGE_SIZE};
use crate::unwind::UnwindFrame;
use crate::{common_decl::*, config::Singlepass};
#[cfg(feature = "unwind")]
use gimli::write::Address;
use smallvec::{smallvec, SmallVec};
use std::cmp;
use std::iter;
use wasmer_compiler::wasmparser::{BlockType as WpTypeOrFuncType, Operator, ValType as WpType};
use wasmer_compiler::FunctionBodyData;
#[cfg(feature = "unwind")]
use wasmer_types::CompiledFunctionUnwindInfo;
use wasmer_types::{
    entity::{EntityRef, PrimaryMap},
    CallingConvention, CompileError, FunctionIndex, FunctionType, GlobalIndex, LocalFunctionIndex,
    LocalMemoryIndex, MemoryIndex, MemoryStyle, ModuleInfo, Relocation, RelocationTarget,
    SectionIndex, SignatureIndex, TableIndex, TableStyle, TrapCode, Type, VMBuiltinFunctionIndex,
    VMOffsets,
};
use wasmer_types::{CompiledFunction, CompiledFunctionFrameInfo, FunctionBody};

/// The singlepass per-function code generator.
pub struct FuncGen<'a, M: Machine> {
    // Immutable properties assigned at creation time.
    /// Static module information.
    module: &'a ModuleInfo,

    /// ModuleInfo compilation config.
    config: &'a Singlepass,

    /// Offsets of vmctx fields.
    vmoffsets: &'a VMOffsets,

    // // Memory plans.
    memory_styles: &'a PrimaryMap<MemoryIndex, MemoryStyle>,

    // // Table plans.
    // table_styles: &'a PrimaryMap<TableIndex, TableStyle>,
    /// Function signature.
    signature: FunctionType,

    // Working storage.
    /// Memory locations of local variables.
    locals: Vec<Location<M::GPR, M::SIMD>>,

    /// Types of local variables, including arguments.
    local_types: Vec<WpType>,

    /// Value stack.
    value_stack: Vec<Location<M::GPR, M::SIMD>>,

    /// Metadata about floating point values on the stack.
    fp_stack: Vec<FloatValue>,

    /// A list of frames describing the current control stack.
    control_stack: Vec<ControlFrame>,

    stack_offset: MachineStackOffset,

    save_area_offset: Option<MachineStackOffset>,

    state: MachineState,

    track_state: bool,

    /// Low-level machine state.
    machine: M,

    /// Nesting level of unreachable code.
    unreachable_depth: usize,

    /// Function state map. Not yet used in the reborn version but let's keep it.
    fsm: FunctionStateMap,

    /// Relocation information.
    relocations: Vec<Relocation>,

    /// A set of special labels for trapping.
    special_labels: SpecialLabelSet,

    /// Calling convention to use.
    calling_convention: CallingConvention,
}

struct SpecialLabelSet {
    integer_division_by_zero: Label,
    integer_overflow: Label,
    heap_access_oob: Label,
    table_access_oob: Label,
    indirect_call_null: Label,
    bad_signature: Label,
    unaligned_atomic: Label,
}

/// Metadata about a floating-point value.
#[derive(Copy, Clone, Debug)]
struct FloatValue {
    /// Do we need to canonicalize the value before its bit pattern is next observed? If so, how?
    canonicalization: Option<CanonicalizeType>,

    /// Corresponding depth in the main value stack.
    depth: usize,
}

impl FloatValue {
    fn new(depth: usize) -> Self {
        FloatValue {
            canonicalization: None,
            depth,
        }
    }

    fn cncl_f32(depth: usize) -> Self {
        FloatValue {
            canonicalization: Some(CanonicalizeType::F32),
            depth,
        }
    }

    fn cncl_f64(depth: usize) -> Self {
        FloatValue {
            canonicalization: Some(CanonicalizeType::F64),
            depth,
        }
    }

    fn promote(self, depth: usize) -> Result<FloatValue, CompileError> {
        let ret = FloatValue {
            canonicalization: match self.canonicalization {
                Some(CanonicalizeType::F32) => Some(CanonicalizeType::F64),
                Some(CanonicalizeType::F64) => codegen_error!("cannot promote F64"),
                None => None,
            },
            depth,
        };
        Ok(ret)
    }

    fn demote(self, depth: usize) -> Result<FloatValue, CompileError> {
        let ret = FloatValue {
            canonicalization: match self.canonicalization {
                Some(CanonicalizeType::F64) => Some(CanonicalizeType::F32),
                Some(CanonicalizeType::F32) => codegen_error!("cannot demote F32"),
                None => None,
            },
            depth,
        };
        Ok(ret)
    }
}

/// Type of a pending canonicalization floating point value.
/// Sometimes we don't have the type information elsewhere and therefore we need to track it here.
#[derive(Copy, Clone, Debug)]
enum CanonicalizeType {
    F32,
    F64,
}

impl CanonicalizeType {
    fn to_size(self) -> Size {
        match self {
            CanonicalizeType::F32 => Size::S32,
            CanonicalizeType::F64 => Size::S64,
        }
    }
}

trait PopMany<T> {
    fn peek1(&self) -> Result<&T, CompileError>;
    fn pop1(&mut self) -> Result<T, CompileError>;
    fn pop2(&mut self) -> Result<(T, T), CompileError>;
}

impl<T> PopMany<T> for Vec<T> {
    fn peek1(&self) -> Result<&T, CompileError> {
        self.last()
            .ok_or_else(|| CompileError::Codegen("peek1() expects at least 1 element".to_owned()))
    }
    fn pop1(&mut self) -> Result<T, CompileError> {
        self.pop()
            .ok_or_else(|| CompileError::Codegen("pop1() expects at least 1 element".to_owned()))
    }
    fn pop2(&mut self) -> Result<(T, T), CompileError> {
        if self.len() < 2 {
            return Err(CompileError::Codegen(
                "pop2() expects at least 2 elements".to_owned(),
            ));
        }

        let right = self.pop().unwrap();
        let left = self.pop().unwrap();
        Ok((left, right))
    }
}

trait WpTypeExt {
    fn is_float(&self) -> bool;
}

impl WpTypeExt for WpType {
    fn is_float(&self) -> bool {
        matches!(self, WpType::F32 | WpType::F64)
    }
}

#[derive(Debug, Clone)]
pub struct ControlFrame {
    pub label: Label,
    pub loop_like: bool,
    pub if_else: IfElseState,
    pub returns: SmallVec<[WpType; 1]>,
    pub value_stack_depth: usize,
    pub fp_stack_depth: usize,
    pub state: MachineState,
    pub state_diff_id: usize,
}

#[derive(Debug, Copy, Clone)]
pub enum IfElseState {
    None,
    If(Label),
    Else,
}

fn type_to_wp_type(ty: Type) -> WpType {
    match ty {
        Type::I32 => WpType::I32,
        Type::I64 => WpType::I64,
        Type::F32 => WpType::F32,
        Type::F64 => WpType::F64,
        Type::V128 => WpType::V128,
        Type::ExternRef => WpType::ExternRef,
        Type::FuncRef => WpType::FuncRef, // TODO: FuncRef or Func?
    }
}

/// Abstraction for a 2-input, 1-output operator. Can be an integer/floating-point
/// binop/cmpop.
struct I2O1<R: Reg, S: Reg> {
    loc_a: Location<R, S>,
    loc_b: Location<R, S>,
    ret: Location<R, S>,
}

impl<'a, M: Machine> FuncGen<'a, M> {
    fn get_stack_offset(&self) -> usize {
        self.stack_offset.0
    }

    /// Acquires locations from the machine state.
    ///
    /// If the returned locations are used for stack value, `release_location` needs to be called on them;
    /// Otherwise, if the returned locations are used for locals, `release_location` does not need to be called on them.
    #[allow(clippy::type_complexity)]
    fn acquire_locations(
        &mut self,
        tys: &[(WpType, MachineValue)],
        zeroed: bool,
    ) -> Result<SmallVec<[Location<M::GPR, M::SIMD>; 1]>, CompileError> {
        let mut ret = smallvec![];
        let mut delta_stack_offset: usize = 0;

        for (ty, mv) in tys {
            let loc = match *ty {
                WpType::F32 | WpType::F64 => self.machine.pick_simd().map(Location::SIMD),
                WpType::I32 | WpType::I64 => self.machine.pick_gpr().map(Location::GPR),
                WpType::FuncRef | WpType::ExternRef => self.machine.pick_gpr().map(Location::GPR),
                _ => codegen_error!("can't acquire location for type {:?}", ty),
            };

            let loc = if let Some(x) = loc {
                x
            } else {
                self.stack_offset.0 += 8;
                delta_stack_offset += 8;
                self.machine.local_on_stack(self.stack_offset.0 as i32)
            };
            if let Location::GPR(x) = loc {
                self.machine.reserve_gpr(x);
                self.state.register_values[self.machine.index_from_gpr(x).0] = mv.clone();
            } else if let Location::SIMD(x) = loc {
                self.machine.reserve_simd(x);
                self.state.register_values[self.machine.index_from_simd(x).0] = mv.clone();
            } else {
                self.state.stack_values.push(mv.clone());
            }
            self.state.wasm_stack.push(WasmAbstractValue::Runtime);
            ret.push(loc);
        }

        let delta_stack_offset = self.machine.round_stack_adjust(delta_stack_offset);
        if delta_stack_offset != 0 {
            self.machine.adjust_stack(delta_stack_offset as u32)?;
        }
        if zeroed {
            for i in 0..tys.len() {
                self.machine.zero_location(Size::S64, ret[i])?;
            }
        }
        Ok(ret)
    }

    /// Releases locations used for stack value.
    fn release_locations(
        &mut self,
        locs: &[Location<M::GPR, M::SIMD>],
    ) -> Result<(), CompileError> {
        let mut delta_stack_offset: usize = 0;

        for loc in locs.iter().rev() {
            match *loc {
                Location::GPR(ref x) => {
                    self.machine.release_gpr(*x);
                    self.state.register_values[self.machine.index_from_gpr(*x).0] =
                        MachineValue::Undefined;
                }
                Location::SIMD(ref x) => {
                    self.machine.release_simd(*x);
                    self.state.register_values[self.machine.index_from_simd(*x).0] =
                        MachineValue::Undefined;
                }
                Location::Memory(y, x) => {
                    if y == self.machine.local_pointer() {
                        if x >= 0 {
                            codegen_error!("Invalid memory offset {}", x);
                        }
                        let offset = (-x) as usize;
                        if offset != self.stack_offset.0 {
                            codegen_error!(
                                "Invalid memory offset {}!={}",
                                offset,
                                self.stack_offset.0
                            );
                        }
                        self.stack_offset.0 -= 8;
                        delta_stack_offset += 8;
                        self.state
                            .stack_values
                            .pop()
                            .ok_or_else(|| CompileError::Codegen("Empty stack_value".to_owned()))?;
                    }
                }
                _ => {}
            }
            self.state
                .wasm_stack
                .pop()
                .ok_or_else(|| CompileError::Codegen("Pop with wasm stack empty".to_owned()))?;
        }
        let delta_stack_offset = self.machine.round_stack_adjust(delta_stack_offset);
        if delta_stack_offset != 0 {
            self.machine.restore_stack(delta_stack_offset as u32)?;
        }
        Ok(())
    }
    /// Releases locations used for stack value.
    fn release_locations_value(&mut self, stack_depth: usize) -> Result<(), CompileError> {
        let mut delta_stack_offset: usize = 0;
        let locs: &[Location<M::GPR, M::SIMD>] = &self.value_stack[stack_depth..];

        for loc in locs.iter().rev() {
            match *loc {
                Location::GPR(ref x) => {
                    self.machine.release_gpr(*x);
                    self.state.register_values[self.machine.index_from_gpr(*x).0] =
                        MachineValue::Undefined;
                }
                Location::SIMD(ref x) => {
                    self.machine.release_simd(*x);
                    self.state.register_values[self.machine.index_from_simd(*x).0] =
                        MachineValue::Undefined;
                }
                Location::Memory(y, x) => {
                    if y == self.machine.local_pointer() {
                        if x >= 0 {
                            codegen_error!("Invalid memory offset {}", x);
                        }
                        let offset = (-x) as usize;
                        if offset != self.stack_offset.0 {
                            codegen_error!(
                                "Invalid memory offset {}!={}",
                                offset,
                                self.stack_offset.0
                            );
                        }
                        self.stack_offset.0 -= 8;
                        delta_stack_offset += 8;
                        self.state.stack_values.pop().ok_or_else(|| {
                            CompileError::Codegen("Pop with values stack empty".to_owned())
                        })?;
                    }
                }
                _ => {}
            }
            self.state
                .wasm_stack
                .pop()
                .ok_or_else(|| CompileError::Codegen("Pop with wasm stack empty".to_owned()))?;
        }

        let delta_stack_offset = self.machine.round_stack_adjust(delta_stack_offset);
        if delta_stack_offset != 0 {
            self.machine.adjust_stack(delta_stack_offset as u32)?;
        }
        Ok(())
    }

    fn release_locations_only_regs(
        &mut self,
        locs: &[Location<M::GPR, M::SIMD>],
    ) -> Result<(), CompileError> {
        for loc in locs.iter().rev() {
            match *loc {
                Location::GPR(ref x) => {
                    self.machine.release_gpr(*x);
                    self.state.register_values[self.machine.index_from_gpr(*x).0] =
                        MachineValue::Undefined;
                }
                Location::SIMD(ref x) => {
                    self.machine.release_simd(*x);
                    self.state.register_values[self.machine.index_from_simd(*x).0] =
                        MachineValue::Undefined;
                }
                _ => {}
            }
            // Wasm state popping is deferred to `release_locations_only_osr_state`.
        }
        Ok(())
    }

    fn release_locations_only_stack(
        &mut self,
        locs: &[Location<M::GPR, M::SIMD>],
    ) -> Result<(), CompileError> {
        let mut delta_stack_offset: usize = 0;

        for loc in locs.iter().rev() {
            if let Location::Memory(y, x) = *loc {
                if y == self.machine.local_pointer() {
                    if x >= 0 {
                        codegen_error!("Invalid memory offset {}", x);
                    }
                    let offset = (-x) as usize;
                    if offset != self.stack_offset.0 {
                        codegen_error!("Invalid memory offset {}!={}", offset, self.stack_offset.0);
                    }
                    self.stack_offset.0 -= 8;
                    delta_stack_offset += 8;
                    self.state.stack_values.pop().ok_or_else(|| {
                        CompileError::Codegen("Pop on empty value stack".to_owned())
                    })?;
                }
            }
            // Wasm state popping is deferred to `release_locations_only_osr_state`.
        }

        let delta_stack_offset = self.machine.round_stack_adjust(delta_stack_offset);
        if delta_stack_offset != 0 {
            self.machine.pop_stack_locals(delta_stack_offset as u32)?;
        }
        Ok(())
    }

    fn release_locations_only_osr_state(&mut self, n: usize) -> Result<(), CompileError> {
        let new_length = self
            .state
            .wasm_stack
            .len()
            .checked_sub(n)
            .expect("release_locations_only_osr_state: length underflow");
        self.state.wasm_stack.truncate(new_length);
        Ok(())
    }

    fn release_locations_keep_state(&mut self, stack_depth: usize) -> Result<(), CompileError> {
        let mut delta_stack_offset: usize = 0;
        let mut stack_offset = self.stack_offset.0;
        let locs = &self.value_stack[stack_depth..];

        for loc in locs.iter().rev() {
            if let Location::Memory(y, x) = *loc {
                if y == self.machine.local_pointer() {
                    if x >= 0 {
                        codegen_error!("Invalid memory offset {}", x);
                    }
                    let offset = (-x) as usize;
                    if offset != stack_offset {
                        codegen_error!("Invalid memory offset {}!={}", offset, self.stack_offset.0);
                    }
                    stack_offset -= 8;
                    delta_stack_offset += 8;
                }
            }
        }

        let delta_stack_offset = self.machine.round_stack_adjust(delta_stack_offset);
        if delta_stack_offset != 0 {
            self.machine.pop_stack_locals(delta_stack_offset as u32)?;
        }
        Ok(())
    }

    #[allow(clippy::type_complexity)]
    fn init_locals(
        &mut self,
        n: usize,
        sig: FunctionType,
        calling_convention: CallingConvention,
    ) -> Result<Vec<Location<M::GPR, M::SIMD>>, CompileError> {
        // How many machine stack slots will all the locals use?
        let num_mem_slots = (0..n)
            .filter(|&x| self.machine.is_local_on_stack(x))
            .count();

        // Total size (in bytes) of the pre-allocated "static area" for this function's
        // locals and callee-saved registers.
        let mut static_area_size: usize = 0;

        // Callee-saved registers used for locals.
        // Keep this consistent with the "Save callee-saved registers" code below.
        for i in 0..n {
            // If a local is not stored on stack, then it is allocated to a callee-saved register.
            if !self.machine.is_local_on_stack(i) {
                static_area_size += 8;
            }
        }

        // Callee-saved vmctx.
        static_area_size += 8;

        // Some ABI (like Windows) needs extrat reg save
        static_area_size += 8 * self.machine.list_to_save(calling_convention).len();

        // Total size of callee saved registers.
        let callee_saved_regs_size = static_area_size;

        // Now we can determine concrete locations for locals.
        let locations: Vec<Location<M::GPR, M::SIMD>> = (0..n)
            .map(|i| self.machine.get_local_location(i, callee_saved_regs_size))
            .collect();

        // Add size of locals on stack.
        static_area_size += num_mem_slots * 8;

        // Allocate save area, without actually writing to it.
        static_area_size = self.machine.round_stack_adjust(static_area_size);

        // Stack probe.
        //
        // `rep stosq` writes data from low address to high address and may skip the stack guard page.
        // so here we probe it explicitly when needed.
        for i in (sig.params().len()..n)
            .step_by(NATIVE_PAGE_SIZE / 8)
            .skip(1)
        {
            self.machine.zero_location(Size::S64, locations[i])?;
        }

        self.machine.adjust_stack(static_area_size as _)?;

        // Save callee-saved registers.
        for loc in locations.iter() {
            if let Location::GPR(x) = *loc {
                self.stack_offset.0 += 8;
                self.machine.move_local(self.stack_offset.0 as i32, *loc)?;
                self.state.stack_values.push(MachineValue::PreserveRegister(
                    self.machine.index_from_gpr(x),
                ));
            }
        }

        // Save the Reg use for vmctx.
        self.stack_offset.0 += 8;
        self.machine.move_local(
            self.stack_offset.0 as i32,
            Location::GPR(self.machine.get_vmctx_reg()),
        )?;
        self.state.stack_values.push(MachineValue::PreserveRegister(
            self.machine.index_from_gpr(self.machine.get_vmctx_reg()),
        ));

        // Check if need to same some CallingConvention specific regs
        let regs_to_save = self.machine.list_to_save(calling_convention);
        for loc in regs_to_save.iter() {
            self.stack_offset.0 += 8;
            self.machine.move_local(self.stack_offset.0 as i32, *loc)?;
        }

        // Save the offset of register save area.
        self.save_area_offset = Some(MachineStackOffset(self.stack_offset.0));

        // Save location information for locals.
        for (i, loc) in locations.iter().enumerate() {
            match *loc {
                Location::GPR(x) => {
                    self.state.register_values[self.machine.index_from_gpr(x).0] =
                        MachineValue::WasmLocal(i);
                }
                Location::Memory(_, _) => {
                    self.state.stack_values.push(MachineValue::WasmLocal(i));
                }
                _ => codegen_error!("singlpass init_local unreachable"),
            }
        }

        // Load in-register parameters into the allocated locations.
        // Locals are allocated on the stack from higher address to lower address,
        // so we won't skip the stack guard page here.
        let mut stack_offset: usize = 0;
        for (i, param) in sig.params().iter().enumerate() {
            let sz = match *param {
                Type::I32 | Type::F32 => Size::S32,
                Type::I64 | Type::F64 => Size::S64,
                Type::ExternRef | Type::FuncRef => Size::S64,
                _ => codegen_error!("singlepass init_local unimplemented"),
            };
            let loc = self.machine.get_call_param_location(
                i + 1,
                sz,
                &mut stack_offset,
                calling_convention,
            );
            self.machine
                .move_location_extend(sz, false, loc, Size::S64, locations[i])?;
        }

        // Load vmctx into it's GPR.
        self.machine.move_location(
            Size::S64,
            self.machine
                .get_simple_param_location(0, calling_convention),
            Location::GPR(self.machine.get_vmctx_reg()),
        )?;

        // Initialize all normal locals to zero.
        let mut init_stack_loc_cnt = 0;
        let mut last_stack_loc = Location::Memory(self.machine.local_pointer(), i32::MAX);
        for location in locations.iter().take(n).skip(sig.params().len()) {
            match location {
                Location::Memory(_, _) => {
                    init_stack_loc_cnt += 1;
                    last_stack_loc = cmp::min(last_stack_loc, *location);
                }
                Location::GPR(_) => {
                    self.machine.zero_location(Size::S64, *location)?;
                }
                _ => codegen_error!("singlepass init_local unreachable"),
            }
        }
        if init_stack_loc_cnt > 0 {
            self.machine
                .init_stack_loc(init_stack_loc_cnt, last_stack_loc)?;
        }

        // Add the size of all locals allocated to stack.
        self.stack_offset.0 += static_area_size - callee_saved_regs_size;

        Ok(locations)
    }

    fn finalize_locals(
        &mut self,
        calling_convention: CallingConvention,
    ) -> Result<(), CompileError> {
        // Unwind stack to the "save area".
        self.machine
            .restore_saved_area(self.save_area_offset.as_ref().unwrap().0 as i32)?;

        let regs_to_save = self.machine.list_to_save(calling_convention);
        for loc in regs_to_save.iter().rev() {
            self.machine.pop_location(*loc)?;
        }

        // Restore register used by vmctx.
        self.machine
            .pop_location(Location::GPR(self.machine.get_vmctx_reg()))?;

        // Restore callee-saved registers.
        for loc in self.locals.iter().rev() {
            if let Location::GPR(_) = *loc {
                self.machine.pop_location(*loc)?;
            }
        }
        Ok(())
    }

    /// Set the source location of the Wasm to the given offset.
    pub fn set_srcloc(&mut self, offset: u32) {
        self.machine.set_srcloc(offset);
    }

    fn get_location_released(
        &mut self,
        loc: Location<M::GPR, M::SIMD>,
    ) -> Result<Location<M::GPR, M::SIMD>, CompileError> {
        self.release_locations(&[loc])?;
        Ok(loc)
    }

    fn pop_value_released(&mut self) -> Result<Location<M::GPR, M::SIMD>, CompileError> {
        let loc = self.value_stack.pop().ok_or_else(|| {
            CompileError::Codegen("pop_value_released: value stack is empty".to_owned())
        })?;
        self.get_location_released(loc)
    }

    /// Prepare data for binary operator with 2 inputs and 1 output.
    fn i2o1_prepare(&mut self, ty: WpType) -> Result<I2O1<M::GPR, M::SIMD>, CompileError> {
        let loc_b = self.pop_value_released()?;
        let loc_a = self.pop_value_released()?;
        let ret = self.acquire_locations(
            &[(ty, MachineValue::WasmStack(self.value_stack.len()))],
            false,
        )?[0];
        self.value_stack.push(ret);
        Ok(I2O1 { loc_a, loc_b, ret })
    }

    fn mark_trappable(&mut self) {
        let state_diff_id = self.get_state_diff();
        let offset = self.machine.assembler_get_offset().0;
        self.fsm.trappable_offsets.insert(
            offset,
            OffsetInfo {
                end_offset: offset + 1,
                activate_offset: offset,
                diff_id: state_diff_id,
            },
        );
        self.fsm.wasm_offset_to_target_offset.insert(
            self.state.wasm_inst_offset,
            SuspendOffset::Trappable(offset),
        );
    }
    fn mark_offset_trappable(&mut self, offset: usize) {
        let state_diff_id = self.get_state_diff();
        self.fsm.trappable_offsets.insert(
            offset,
            OffsetInfo {
                end_offset: offset + 1,
                activate_offset: offset,
                diff_id: state_diff_id,
            },
        );
        self.fsm.wasm_offset_to_target_offset.insert(
            self.state.wasm_inst_offset,
            SuspendOffset::Trappable(offset),
        );
    }

    /// Emits a Native ABI call sequence.
    ///
    /// The caller MUST NOT hold any temporary registers allocated by `acquire_temp_gpr` when calling
    /// this function.
    fn emit_call_native<
        I: Iterator<Item = Location<M::GPR, M::SIMD>>,
        J: Iterator<Item = WpType>,
        F: FnOnce(&mut Self) -> Result<(), CompileError>,
    >(
        &mut self,
        cb: F,
        params: I,
        params_type: J,
    ) -> Result<(), CompileError> {
        // Values pushed in this function are above the shadow region.
        self.state.stack_values.push(MachineValue::ExplicitShadow);

        let params: Vec<_> = params.collect();
        let params_size: Vec<_> = params_type
            .map(|x| match x {
                WpType::F32 | WpType::I32 => Size::S32,
                WpType::V128 => unimplemented!(),
                _ => Size::S64,
            })
            .collect();

        // Save used GPRs. Preserve correct stack alignment
        let used_gprs = self.machine.get_used_gprs();
        let mut used_stack = self.machine.push_used_gpr(&used_gprs)?;
        for r in used_gprs.iter() {
            let content = self.state.register_values[self.machine.index_from_gpr(*r).0].clone();
            if content == MachineValue::Undefined {
                return Err(CompileError::Codegen(
                    "emit_call_native: Undefined used_gprs content".to_owned(),
                ));
            }
            self.state.stack_values.push(content);
        }

        // Save used SIMD registers.
        let used_simds = self.machine.get_used_simd();
        if !used_simds.is_empty() {
            used_stack += self.machine.push_used_simd(&used_simds)?;

            for r in used_simds.iter().rev() {
                let content =
                    self.state.register_values[self.machine.index_from_simd(*r).0].clone();
                if content == MachineValue::Undefined {
                    return Err(CompileError::Codegen(
                        "emit_call_native: Undefined used_simds content".to_owned(),
                    ));
                }
                self.state.stack_values.push(content);
            }
        }
        // mark the GPR used for Call as used
        self.machine
            .reserve_unused_temp_gpr(self.machine.get_grp_for_call());

        let calling_convention = self.calling_convention;

        let stack_padding: usize = match calling_convention {
            CallingConvention::WindowsFastcall => 32,
            _ => 0,
        };

        let mut stack_offset: usize = 0;
        let mut args: Vec<Location<M::GPR, M::SIMD>> = vec![];
        let mut pushed_args: usize = 0;
        // Calculate stack offset.
        for (i, _param) in params.iter().enumerate() {
            args.push(self.machine.get_param_location(
                1 + i,
                params_size[i],
                &mut stack_offset,
                calling_convention,
            ));
        }

        // Align stack to 16 bytes.
        let stack_unaligned =
            (self.machine.round_stack_adjust(self.get_stack_offset()) + used_stack + stack_offset)
                % 16;
        if stack_unaligned != 0 {
            stack_offset += 16 - stack_unaligned;
        }
        self.machine.adjust_stack(stack_offset as u32)?;

        #[allow(clippy::type_complexity)]
        let mut call_movs: Vec<(Location<M::GPR, M::SIMD>, M::GPR)> = vec![];
        // Prepare register & stack parameters.
        for (i, param) in params.iter().enumerate().rev() {
            let loc = args[i];
            match loc {
                Location::GPR(x) => {
                    call_movs.push((*param, x));
                }
                Location::Memory(_, _) => {
                    pushed_args += 1;
                    match *param {
                        Location::GPR(x) => {
                            let content = self.state.register_values
                                [self.machine.index_from_gpr(x).0]
                                .clone();
                            // FIXME: There might be some corner cases (release -> emit_call_native -> acquire?) that cause this assertion to fail.
                            // Hopefully nothing would be incorrect at runtime.

                            //assert!(content != MachineValue::Undefined);
                            self.state.stack_values.push(content);
                        }
                        Location::SIMD(x) => {
                            let content = self.state.register_values
                                [self.machine.index_from_simd(x).0]
                                .clone();
                            //assert!(content != MachineValue::Undefined);
                            self.state.stack_values.push(content);
                        }
                        Location::Memory(reg, offset) => {
                            if reg != self.machine.local_pointer() {
                                return Err(CompileError::Codegen(
                                    "emit_call_native loc param: unreachable code".to_owned(),
                                ));
                            }
                            self.state
                                .stack_values
                                .push(MachineValue::CopyStackBPRelative(offset));
                            // TODO: Read value at this offset
                        }
                        _ => {
                            self.state.stack_values.push(MachineValue::Undefined);
                        }
                    }
                    self.machine
                        .move_location_for_native(params_size[i], *param, loc)?;
                }
                _ => {
                    return Err(CompileError::Codegen(
                        "emit_call_native loc: unreachable code".to_owned(),
                    ))
                }
            }
        }

        // Sort register moves so that register are not overwritten before read.
        Self::sort_call_movs(&mut call_movs);

        // Emit register moves.
        for (loc, gpr) in call_movs {
            if loc != Location::GPR(gpr) {
                self.machine
                    .move_location(Size::S64, loc, Location::GPR(gpr))?;
            }
        }

        // Put vmctx as the first parameter.
        self.machine.move_location(
            Size::S64,
            Location::GPR(self.machine.get_vmctx_reg()),
            self.machine
                .get_simple_param_location(0, calling_convention),
        )?; // vmctx

        if stack_padding > 0 {
            self.machine.adjust_stack(stack_padding as u32)?;
        }
        // release the GPR used for call
        self.machine.release_gpr(self.machine.get_grp_for_call());
        cb(self)?;

        // Offset needs to be after the 'call' instruction.
        // TODO: Now the state information is also inserted for internal calls (e.g. MemoryGrow). Is this expected?
        {
            let state_diff_id = self.get_state_diff();
            let offset = self.machine.assembler_get_offset().0;
            self.fsm.call_offsets.insert(
                offset,
                OffsetInfo {
                    end_offset: offset + 1,
                    activate_offset: offset,
                    diff_id: state_diff_id,
                },
            );
            self.fsm
                .wasm_offset_to_target_offset
                .insert(self.state.wasm_inst_offset, SuspendOffset::Call(offset));
        }

        // Restore stack.
        if stack_offset + stack_padding > 0 {
            self.machine.restore_stack(
                self.machine
                    .round_stack_adjust(stack_offset + stack_padding) as u32,
            )?;
            if (stack_offset % 8) != 0 {
                return Err(CompileError::Codegen(
                    "emit_call_native: Bad restoring stack alignement".to_owned(),
                ));
            }
            for _ in 0..pushed_args {
                self.state
                    .stack_values
                    .pop()
                    .ok_or_else(|| CompileError::Codegen("Pop an empty value stack".to_owned()))?;
            }
        }

        // Restore SIMDs.
        if !used_simds.is_empty() {
            self.machine.pop_used_simd(&used_simds)?;
            for _ in 0..used_simds.len() {
                self.state
                    .stack_values
                    .pop()
                    .ok_or_else(|| CompileError::Codegen("Pop an empty value stack".to_owned()))?;
            }
        }

        // Restore GPRs.
        self.machine.pop_used_gpr(&used_gprs)?;
        for _ in used_gprs.iter().rev() {
            self.state
                .stack_values
                .pop()
                .ok_or_else(|| CompileError::Codegen("Pop an empty value stack".to_owned()))?;
        }

        if self
            .state
            .stack_values
            .pop()
            .ok_or_else(|| CompileError::Codegen("Pop an empty value stack".to_owned()))?
            != MachineValue::ExplicitShadow
        {
            return Err(CompileError::Codegen(
                "emit_call_native: Popped value is not ExplicitShadow".to_owned(),
            ));
        }
        Ok(())
    }

    /// Emits a Native ABI call sequence, specialized for labels as the call target.
    fn _emit_call_native_label<
        I: Iterator<Item = Location<M::GPR, M::SIMD>>,
        J: Iterator<Item = WpType>,
    >(
        &mut self,
        label: Label,
        params: I,
        params_type: J,
    ) -> Result<(), CompileError> {
        self.emit_call_native(
            |this| this.machine.emit_call_label(label),
            params,
            params_type,
        )?;
        Ok(())
    }

    /// Emits a memory operation.
    fn op_memory<
        F: FnOnce(&mut Self, bool, bool, i32, Label, Label) -> Result<(), CompileError>,
    >(
        &mut self,
        cb: F,
    ) -> Result<(), CompileError> {
        let need_check = match self.memory_styles[MemoryIndex::new(0)] {
            MemoryStyle::Static { .. } => false,
            MemoryStyle::Dynamic { .. } => true,
        };

        let offset = if self.module.num_imported_memories != 0 {
            self.vmoffsets
                .vmctx_vmmemory_import_definition(MemoryIndex::new(0))
        } else {
            self.vmoffsets
                .vmctx_vmmemory_definition(LocalMemoryIndex::new(0))
        };
        cb(
            self,
            need_check,
            self.module.num_imported_memories != 0,
            offset as i32,
            self.special_labels.heap_access_oob,
            self.special_labels.unaligned_atomic,
        )
    }

    pub fn get_state_diff(&mut self) -> usize {
        if !self.track_state {
            return std::usize::MAX;
        }
        let last_frame = self.control_stack.last_mut().unwrap();
        let mut diff = self.state.diff(&last_frame.state);
        diff.last = Some(last_frame.state_diff_id);
        let id = self.fsm.diffs.len();
        last_frame.state = self.state.clone();
        last_frame.state_diff_id = id;
        self.fsm.diffs.push(diff);
        id
    }

    fn emit_head(&mut self) -> Result<(), CompileError> {
        self.machine.emit_function_prolog()?;

        // Initialize locals.
        self.locals = self.init_locals(
            self.local_types.len(),
            self.signature.clone(),
            self.calling_convention,
        )?;

        // Mark vmctx register. The actual loading of the vmctx value is handled by init_local.
        self.state.register_values[self.machine.index_from_gpr(self.machine.get_vmctx_reg()).0] =
            MachineValue::Vmctx;

        // TODO: Explicit stack check is not supported for now.
        let diff = self.state.diff(&self.machine.new_machine_state());
        let state_diff_id = self.fsm.diffs.len();
        self.fsm.diffs.push(diff);

        // simulate "red zone" if not supported by the platform
        self.machine.adjust_stack(32)?;

        self.control_stack.push(ControlFrame {
            label: self.machine.get_label(),
            loop_like: false,
            if_else: IfElseState::None,
            returns: self
                .signature
                .results()
                .iter()
                .map(|&x| type_to_wp_type(x))
                .collect(),
            value_stack_depth: 0,
            fp_stack_depth: 0,
            state: self.state.clone(),
            state_diff_id,
        });

        // TODO: Full preemption by explicit signal checking

        // We insert set StackOverflow as the default trap that can happen
        // anywhere in the function prologue.
        self.machine.insert_stackoverflow();

        if self.state.wasm_inst_offset != std::usize::MAX {
            return Err(CompileError::Codegen(
                "emit_head: wasm_inst_offset not std::usize::MAX".to_owned(),
            ));
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        module: &'a ModuleInfo,
        config: &'a Singlepass,
        vmoffsets: &'a VMOffsets,
        memory_styles: &'a PrimaryMap<MemoryIndex, MemoryStyle>,
        _table_styles: &'a PrimaryMap<TableIndex, TableStyle>,
        local_func_index: LocalFunctionIndex,
        local_types_excluding_arguments: &[WpType],
        machine: M,
        calling_convention: CallingConvention,
    ) -> Result<FuncGen<'a, M>, CompileError> {
        let func_index = module.func_index(local_func_index);
        let sig_index = module.functions[func_index];
        let signature = module.signatures[sig_index].clone();

        let mut local_types: Vec<_> = signature
            .params()
            .iter()
            .map(|&x| type_to_wp_type(x))
            .collect();
        local_types.extend_from_slice(local_types_excluding_arguments);

        let mut machine = machine;
        let special_labels = SpecialLabelSet {
            integer_division_by_zero: machine.get_label(),
            integer_overflow: machine.get_label(),
            heap_access_oob: machine.get_label(),
            table_access_oob: machine.get_label(),
            indirect_call_null: machine.get_label(),
            bad_signature: machine.get_label(),
            unaligned_atomic: machine.get_label(),
        };

        let fsm = FunctionStateMap::new(
            machine.new_machine_state(),
            local_func_index.index() as usize,
            32,
            (0..local_types.len())
                .map(|_| WasmAbstractValue::Runtime)
                .collect(),
        );

        let mut fg = FuncGen {
            module,
            config,
            vmoffsets,
            memory_styles,
            // table_styles,
            signature,
            locals: vec![], // initialization deferred to emit_head
            local_types,
            value_stack: vec![],
            fp_stack: vec![],
            control_stack: vec![],
            stack_offset: MachineStackOffset(0),
            save_area_offset: None,
            state: machine.new_machine_state(),
            track_state: true,
            machine,
            unreachable_depth: 0,
            fsm,
            relocations: vec![],
            special_labels,
            calling_convention,
        };
        fg.emit_head()?;
        Ok(fg)
    }

    pub fn has_control_frames(&self) -> bool {
        !self.control_stack.is_empty()
    }

    pub fn feed_operator(&mut self, op: Operator) -> Result<(), CompileError> {
        assert!(self.fp_stack.len() <= self.value_stack.len());

        self.state.wasm_inst_offset = self.state.wasm_inst_offset.wrapping_add(1);

        //println!("{:?} {}", op, self.value_stack.len());
        let was_unreachable;

        if self.unreachable_depth > 0 {
            was_unreachable = true;

            match op {
                Operator::Block { .. } | Operator::Loop { .. } | Operator::If { .. } => {
                    self.unreachable_depth += 1;
                }
                Operator::End => {
                    self.unreachable_depth -= 1;
                }
                Operator::Else => {
                    // We are in a reachable true branch
                    if self.unreachable_depth == 1 {
                        if let Some(IfElseState::If(_)) =
                            self.control_stack.last().map(|x| x.if_else)
                        {
                            self.unreachable_depth -= 1;
                        }
                    }
                }
                _ => {}
            }
            if self.unreachable_depth > 0 {
                return Ok(());
            }
        } else {
            was_unreachable = false;
        }

        match op {
            Operator::GlobalGet { global_index } => {
                let global_index = GlobalIndex::from_u32(global_index);

                let ty = type_to_wp_type(self.module.globals[global_index].ty);
                if ty.is_float() {
                    self.fp_stack.push(FloatValue::new(self.value_stack.len()));
                }
                let loc = self.acquire_locations(
                    &[(ty, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(loc);

                let tmp = self.machine.acquire_temp_gpr().unwrap();

                let src = if let Some(local_global_index) =
                    self.module.local_global_index(global_index)
                {
                    let offset = self.vmoffsets.vmctx_vmglobal_definition(local_global_index);
                    self.machine.emit_relaxed_mov(
                        Size::S64,
                        Location::Memory(self.machine.get_vmctx_reg(), offset as i32),
                        Location::GPR(tmp),
                    )?;
                    Location::Memory(tmp, 0)
                } else {
                    // Imported globals require one level of indirection.
                    let offset = self
                        .vmoffsets
                        .vmctx_vmglobal_import_definition(global_index);
                    self.machine.emit_relaxed_mov(
                        Size::S64,
                        Location::Memory(self.machine.get_vmctx_reg(), offset as i32),
                        Location::GPR(tmp),
                    )?;
                    Location::Memory(tmp, 0)
                };

                self.machine.emit_relaxed_mov(Size::S64, src, loc)?;

                self.machine.release_gpr(tmp);
            }
            Operator::GlobalSet { global_index } => {
                let global_index = GlobalIndex::from_u32(global_index);
                let tmp = self.machine.acquire_temp_gpr().unwrap();
                let dst = if let Some(local_global_index) =
                    self.module.local_global_index(global_index)
                {
                    let offset = self.vmoffsets.vmctx_vmglobal_definition(local_global_index);
                    self.machine.emit_relaxed_mov(
                        Size::S64,
                        Location::Memory(self.machine.get_vmctx_reg(), offset as i32),
                        Location::GPR(tmp),
                    )?;
                    Location::Memory(tmp, 0)
                } else {
                    // Imported globals require one level of indirection.
                    let offset = self
                        .vmoffsets
                        .vmctx_vmglobal_import_definition(global_index);
                    self.machine.emit_relaxed_mov(
                        Size::S64,
                        Location::Memory(self.machine.get_vmctx_reg(), offset as i32),
                        Location::GPR(tmp),
                    )?;
                    Location::Memory(tmp, 0)
                };
                let ty = type_to_wp_type(self.module.globals[global_index].ty);
                let loc = self.pop_value_released()?;
                if ty.is_float() {
                    let fp = self.fp_stack.pop1()?;
                    if self.machine.arch_supports_canonicalize_nan()
                        && self.config.enable_nan_canonicalization
                        && fp.canonicalization.is_some()
                    {
                        self.machine.canonicalize_nan(
                            match ty {
                                WpType::F32 => Size::S32,
                                WpType::F64 => Size::S64,
                                _ => codegen_error!("singlepass Operator::GlobalSet unreachable"),
                            },
                            loc,
                            dst,
                        )?;
                    } else {
                        self.machine.emit_relaxed_mov(Size::S64, loc, dst)?;
                    }
                } else {
                    self.machine.emit_relaxed_mov(Size::S64, loc, dst)?;
                }
                self.machine.release_gpr(tmp);
            }
            Operator::LocalGet { local_index } => {
                let local_index = local_index as usize;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.machine
                    .emit_relaxed_mov(Size::S64, self.locals[local_index], ret)?;
                self.value_stack.push(ret);
                if self.local_types[local_index].is_float() {
                    self.fp_stack
                        .push(FloatValue::new(self.value_stack.len() - 1));
                }
            }
            Operator::LocalSet { local_index } => {
                let local_index = local_index as usize;
                let loc = self.pop_value_released()?;

                if self.local_types[local_index].is_float() {
                    let fp = self.fp_stack.pop1()?;
                    if self.machine.arch_supports_canonicalize_nan()
                        && self.config.enable_nan_canonicalization
                        && fp.canonicalization.is_some()
                    {
                        self.machine.canonicalize_nan(
                            match self.local_types[local_index] {
                                WpType::F32 => Size::S32,
                                WpType::F64 => Size::S64,
                                _ => codegen_error!("singlepass Operator::LocalSet unreachable"),
                            },
                            loc,
                            self.locals[local_index],
                        )
                    } else {
                        self.machine
                            .emit_relaxed_mov(Size::S64, loc, self.locals[local_index])
                    }
                } else {
                    self.machine
                        .emit_relaxed_mov(Size::S64, loc, self.locals[local_index])
                }?;
            }
            Operator::LocalTee { local_index } => {
                let local_index = local_index as usize;
                let loc = *self.value_stack.last().unwrap();

                if self.local_types[local_index].is_float() {
                    let fp = self.fp_stack.peek1()?;
                    if self.machine.arch_supports_canonicalize_nan()
                        && self.config.enable_nan_canonicalization
                        && fp.canonicalization.is_some()
                    {
                        self.machine.canonicalize_nan(
                            match self.local_types[local_index] {
                                WpType::F32 => Size::S32,
                                WpType::F64 => Size::S64,
                                _ => codegen_error!("singlepass Operator::LocalTee unreachable"),
                            },
                            loc,
                            self.locals[local_index],
                        )
                    } else {
                        self.machine
                            .emit_relaxed_mov(Size::S64, loc, self.locals[local_index])
                    }
                } else {
                    self.machine
                        .emit_relaxed_mov(Size::S64, loc, self.locals[local_index])
                }?;
            }
            Operator::I32Const { value } => {
                self.value_stack.push(Location::Imm32(value as u32));
                self.state
                    .wasm_stack
                    .push(WasmAbstractValue::Const(value as u32 as u64));
            }
            Operator::I32Add => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.emit_binop_add32(loc_a, loc_b, ret)?;
            }
            Operator::I32Sub => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.emit_binop_sub32(loc_a, loc_b, ret)?;
            }
            Operator::I32Mul => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.emit_binop_mul32(loc_a, loc_b, ret)?;
            }
            Operator::I32DivU => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                let offset = self.machine.emit_binop_udiv32(
                    loc_a,
                    loc_b,
                    ret,
                    self.special_labels.integer_division_by_zero,
                    self.special_labels.integer_overflow,
                )?;
                self.mark_offset_trappable(offset);
            }
            Operator::I32DivS => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                let offset = self.machine.emit_binop_sdiv32(
                    loc_a,
                    loc_b,
                    ret,
                    self.special_labels.integer_division_by_zero,
                    self.special_labels.integer_overflow,
                )?;
                self.mark_offset_trappable(offset);
            }
            Operator::I32RemU => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                let offset = self.machine.emit_binop_urem32(
                    loc_a,
                    loc_b,
                    ret,
                    self.special_labels.integer_division_by_zero,
                    self.special_labels.integer_overflow,
                )?;
                self.mark_offset_trappable(offset);
            }
            Operator::I32RemS => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                let offset = self.machine.emit_binop_srem32(
                    loc_a,
                    loc_b,
                    ret,
                    self.special_labels.integer_division_by_zero,
                    self.special_labels.integer_overflow,
                )?;
                self.mark_offset_trappable(offset);
            }
            Operator::I32And => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.emit_binop_and32(loc_a, loc_b, ret)?;
            }
            Operator::I32Or => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.emit_binop_or32(loc_a, loc_b, ret)?;
            }
            Operator::I32Xor => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.emit_binop_xor32(loc_a, loc_b, ret)?;
            }
            Operator::I32Eq => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.i32_cmp_eq(loc_a, loc_b, ret)?;
            }
            Operator::I32Ne => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.i32_cmp_ne(loc_a, loc_b, ret)?;
            }
            Operator::I32Eqz => {
                let loc_a = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.machine.i32_cmp_eq(loc_a, Location::Imm32(0), ret)?;
                self.value_stack.push(ret);
            }
            Operator::I32Clz => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.i32_clz(loc, ret)?;
            }
            Operator::I32Ctz => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.i32_ctz(loc, ret)?;
            }
            Operator::I32Popcnt => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.i32_popcnt(loc, ret)?;
            }
            Operator::I32Shl => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.i32_shl(loc_a, loc_b, ret)?;
            }
            Operator::I32ShrU => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.i32_shr(loc_a, loc_b, ret)?;
            }
            Operator::I32ShrS => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.i32_sar(loc_a, loc_b, ret)?;
            }
            Operator::I32Rotl => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.i32_rol(loc_a, loc_b, ret)?;
            }
            Operator::I32Rotr => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.i32_ror(loc_a, loc_b, ret)?;
            }
            Operator::I32LtU => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.i32_cmp_lt_u(loc_a, loc_b, ret)?;
            }
            Operator::I32LeU => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.i32_cmp_le_u(loc_a, loc_b, ret)?;
            }
            Operator::I32GtU => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.i32_cmp_gt_u(loc_a, loc_b, ret)?;
            }
            Operator::I32GeU => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.i32_cmp_ge_u(loc_a, loc_b, ret)?;
            }
            Operator::I32LtS => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.i32_cmp_lt_s(loc_a, loc_b, ret)?;
            }
            Operator::I32LeS => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.i32_cmp_le_s(loc_a, loc_b, ret)?;
            }
            Operator::I32GtS => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.i32_cmp_gt_s(loc_a, loc_b, ret)?;
            }
            Operator::I32GeS => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.i32_cmp_ge_s(loc_a, loc_b, ret)?;
            }
            Operator::I64Const { value } => {
                let value = value as u64;
                self.value_stack.push(Location::Imm64(value));
                self.state.wasm_stack.push(WasmAbstractValue::Const(value));
            }
            Operator::I64Add => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I64)?;
                self.machine.emit_binop_add64(loc_a, loc_b, ret)?;
            }
            Operator::I64Sub => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I64)?;
                self.machine.emit_binop_sub64(loc_a, loc_b, ret)?;
            }
            Operator::I64Mul => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I64)?;
                self.machine.emit_binop_mul64(loc_a, loc_b, ret)?;
            }
            Operator::I64DivU => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I64)?;
                let offset = self.machine.emit_binop_udiv64(
                    loc_a,
                    loc_b,
                    ret,
                    self.special_labels.integer_division_by_zero,
                    self.special_labels.integer_overflow,
                )?;
                self.mark_offset_trappable(offset);
            }
            Operator::I64DivS => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I64)?;
                let offset = self.machine.emit_binop_sdiv64(
                    loc_a,
                    loc_b,
                    ret,
                    self.special_labels.integer_division_by_zero,
                    self.special_labels.integer_overflow,
                )?;
                self.mark_offset_trappable(offset);
            }
            Operator::I64RemU => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I64)?;
                let offset = self.machine.emit_binop_urem64(
                    loc_a,
                    loc_b,
                    ret,
                    self.special_labels.integer_division_by_zero,
                    self.special_labels.integer_overflow,
                )?;
                self.mark_offset_trappable(offset);
            }
            Operator::I64RemS => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I64)?;
                let offset = self.machine.emit_binop_srem64(
                    loc_a,
                    loc_b,
                    ret,
                    self.special_labels.integer_division_by_zero,
                    self.special_labels.integer_overflow,
                )?;
                self.mark_offset_trappable(offset);
            }
            Operator::I64And => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I64)?;
                self.machine.emit_binop_and64(loc_a, loc_b, ret)?;
            }
            Operator::I64Or => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I64)?;
                self.machine.emit_binop_or64(loc_a, loc_b, ret)?;
            }
            Operator::I64Xor => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I64)?;
                self.machine.emit_binop_xor64(loc_a, loc_b, ret)?;
            }
            Operator::I64Eq => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I64)?;
                self.machine.i64_cmp_eq(loc_a, loc_b, ret)?;
            }
            Operator::I64Ne => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I64)?;
                self.machine.i64_cmp_ne(loc_a, loc_b, ret)?;
            }
            Operator::I64Eqz => {
                let loc_a = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.machine.i64_cmp_eq(loc_a, Location::Imm64(0), ret)?;
                self.value_stack.push(ret);
            }
            Operator::I64Clz => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.i64_clz(loc, ret)?;
            }
            Operator::I64Ctz => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.i64_ctz(loc, ret)?;
            }
            Operator::I64Popcnt => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.i64_popcnt(loc, ret)?;
            }
            Operator::I64Shl => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I64)?;
                self.machine.i64_shl(loc_a, loc_b, ret)?;
            }
            Operator::I64ShrU => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I64)?;
                self.machine.i64_shr(loc_a, loc_b, ret)?;
            }
            Operator::I64ShrS => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I64)?;
                self.machine.i64_sar(loc_a, loc_b, ret)?;
            }
            Operator::I64Rotl => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I64)?;
                self.machine.i64_rol(loc_a, loc_b, ret)?;
            }
            Operator::I64Rotr => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I64)?;
                self.machine.i64_ror(loc_a, loc_b, ret)?;
            }
            Operator::I64LtU => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I64)?;
                self.machine.i64_cmp_lt_u(loc_a, loc_b, ret)?;
            }
            Operator::I64LeU => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I64)?;
                self.machine.i64_cmp_le_u(loc_a, loc_b, ret)?;
            }
            Operator::I64GtU => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I64)?;
                self.machine.i64_cmp_gt_u(loc_a, loc_b, ret)?;
            }
            Operator::I64GeU => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I64)?;
                self.machine.i64_cmp_ge_u(loc_a, loc_b, ret)?;
            }
            Operator::I64LtS => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I64)?;
                self.machine.i64_cmp_lt_s(loc_a, loc_b, ret)?;
            }
            Operator::I64LeS => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I64)?;
                self.machine.i64_cmp_le_s(loc_a, loc_b, ret)?;
            }
            Operator::I64GtS => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I64)?;
                self.machine.i64_cmp_gt_s(loc_a, loc_b, ret)?;
            }
            Operator::I64GeS => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I64)?;
                self.machine.i64_cmp_ge_s(loc_a, loc_b, ret)?;
            }
            Operator::I64ExtendI32U => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.emit_relaxed_mov(Size::S32, loc, ret)?;

                // A 32-bit memory write does not automatically clear the upper 32 bits of a 64-bit word.
                // So, we need to explicitly write zero to the upper half here.
                if let Location::Memory(base, off) = ret {
                    self.machine.emit_relaxed_mov(
                        Size::S32,
                        Location::Imm32(0),
                        Location::Memory(base, off + 4),
                    )?;
                }
            }
            Operator::I64ExtendI32S => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine
                    .emit_relaxed_sign_extension(Size::S32, loc, Size::S64, ret)?;
            }
            Operator::I32Extend8S => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);

                self.machine
                    .emit_relaxed_sign_extension(Size::S8, loc, Size::S32, ret)?;
            }
            Operator::I32Extend16S => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);

                self.machine
                    .emit_relaxed_sign_extension(Size::S16, loc, Size::S32, ret)?;
            }
            Operator::I64Extend8S => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);

                self.machine
                    .emit_relaxed_sign_extension(Size::S8, loc, Size::S64, ret)?;
            }
            Operator::I64Extend16S => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);

                self.machine
                    .emit_relaxed_sign_extension(Size::S16, loc, Size::S64, ret)?;
            }
            Operator::I64Extend32S => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);

                self.machine
                    .emit_relaxed_sign_extension(Size::S32, loc, Size::S64, ret)?;
            }
            Operator::I32WrapI64 => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.emit_relaxed_mov(Size::S32, loc, ret)?;
            }

            Operator::F32Const { value } => {
                self.value_stack.push(Location::Imm32(value.bits()));
                self.fp_stack
                    .push(FloatValue::new(self.value_stack.len() - 1));
                self.state
                    .wasm_stack
                    .push(WasmAbstractValue::Const(value.bits() as u64));
            }
            Operator::F32Add => {
                self.fp_stack.pop2()?;
                self.fp_stack
                    .push(FloatValue::cncl_f32(self.value_stack.len() - 2));
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::F64)?;

                self.machine.f32_add(loc_a, loc_b, ret)?;
            }
            Operator::F32Sub => {
                self.fp_stack.pop2()?;
                self.fp_stack
                    .push(FloatValue::cncl_f32(self.value_stack.len() - 2));
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::F64)?;

                self.machine.f32_sub(loc_a, loc_b, ret)?;
            }
            Operator::F32Mul => {
                self.fp_stack.pop2()?;
                self.fp_stack
                    .push(FloatValue::cncl_f32(self.value_stack.len() - 2));
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::F64)?;

                self.machine.f32_mul(loc_a, loc_b, ret)?;
            }
            Operator::F32Div => {
                self.fp_stack.pop2()?;
                self.fp_stack
                    .push(FloatValue::cncl_f32(self.value_stack.len() - 2));
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::F64)?;

                self.machine.f32_div(loc_a, loc_b, ret)?;
            }
            Operator::F32Max => {
                self.fp_stack.pop2()?;
                self.fp_stack
                    .push(FloatValue::new(self.value_stack.len() - 2));
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::F64)?;
                self.machine.f32_max(loc_a, loc_b, ret)?;
            }
            Operator::F32Min => {
                self.fp_stack.pop2()?;
                self.fp_stack
                    .push(FloatValue::new(self.value_stack.len() - 2));
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::F64)?;
                self.machine.f32_min(loc_a, loc_b, ret)?;
            }
            Operator::F32Eq => {
                self.fp_stack.pop2()?;
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.f32_cmp_eq(loc_a, loc_b, ret)?;
            }
            Operator::F32Ne => {
                self.fp_stack.pop2()?;
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.f32_cmp_ne(loc_a, loc_b, ret)?;
            }
            Operator::F32Lt => {
                self.fp_stack.pop2()?;
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.f32_cmp_lt(loc_a, loc_b, ret)?;
            }
            Operator::F32Le => {
                self.fp_stack.pop2()?;
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.f32_cmp_le(loc_a, loc_b, ret)?;
            }
            Operator::F32Gt => {
                self.fp_stack.pop2()?;
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.f32_cmp_gt(loc_a, loc_b, ret)?;
            }
            Operator::F32Ge => {
                self.fp_stack.pop2()?;
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.f32_cmp_ge(loc_a, loc_b, ret)?;
            }
            Operator::F32Nearest => {
                self.fp_stack.pop1()?;
                self.fp_stack
                    .push(FloatValue::cncl_f32(self.value_stack.len() - 1));
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.f32_nearest(loc, ret)?;
            }
            Operator::F32Floor => {
                self.fp_stack.pop1()?;
                self.fp_stack
                    .push(FloatValue::cncl_f32(self.value_stack.len() - 1));
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.f32_floor(loc, ret)?;
            }
            Operator::F32Ceil => {
                self.fp_stack.pop1()?;
                self.fp_stack
                    .push(FloatValue::cncl_f32(self.value_stack.len() - 1));
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.f32_ceil(loc, ret)?;
            }
            Operator::F32Trunc => {
                self.fp_stack.pop1()?;
                self.fp_stack
                    .push(FloatValue::cncl_f32(self.value_stack.len() - 1));
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.f32_trunc(loc, ret)?;
            }
            Operator::F32Sqrt => {
                self.fp_stack.pop1()?;
                self.fp_stack
                    .push(FloatValue::cncl_f32(self.value_stack.len() - 1));
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.f32_sqrt(loc, ret)?;
            }

            Operator::F32Copysign => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::F32)?;

                let (fp_src1, fp_src2) = self.fp_stack.pop2()?;
                self.fp_stack
                    .push(FloatValue::new(self.value_stack.len() - 1));

                let tmp1 = self.machine.acquire_temp_gpr().unwrap();
                let tmp2 = self.machine.acquire_temp_gpr().unwrap();

                if self.machine.arch_supports_canonicalize_nan()
                    && self.config.enable_nan_canonicalization
                {
                    for (fp, loc, tmp) in [(fp_src1, loc_a, tmp1), (fp_src2, loc_b, tmp2)].iter() {
                        match fp.canonicalization {
                            Some(_) => {
                                self.machine
                                    .canonicalize_nan(Size::S32, *loc, Location::GPR(*tmp))
                            }
                            None => {
                                self.machine
                                    .move_location(Size::S32, *loc, Location::GPR(*tmp))
                            }
                        }?;
                    }
                } else {
                    self.machine
                        .move_location(Size::S32, loc_a, Location::GPR(tmp1))?;
                    self.machine
                        .move_location(Size::S32, loc_b, Location::GPR(tmp2))?;
                }
                self.machine.emit_i32_copysign(tmp1, tmp2)?;
                self.machine
                    .move_location(Size::S32, Location::GPR(tmp1), ret)?;
                self.machine.release_gpr(tmp2);
                self.machine.release_gpr(tmp1);
            }

            Operator::F32Abs => {
                // Preserve canonicalization state.

                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);

                self.machine.f32_abs(loc, ret)?;
            }

            Operator::F32Neg => {
                // Preserve canonicalization state.

                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);

                self.machine.f32_neg(loc, ret)?;
            }

            Operator::F64Const { value } => {
                self.value_stack.push(Location::Imm64(value.bits()));
                self.fp_stack
                    .push(FloatValue::new(self.value_stack.len() - 1));
                self.state
                    .wasm_stack
                    .push(WasmAbstractValue::Const(value.bits()));
            }
            Operator::F64Add => {
                self.fp_stack.pop2()?;
                self.fp_stack
                    .push(FloatValue::cncl_f64(self.value_stack.len() - 2));
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::F64)?;

                self.machine.f64_add(loc_a, loc_b, ret)?;
            }
            Operator::F64Sub => {
                self.fp_stack.pop2()?;
                self.fp_stack
                    .push(FloatValue::cncl_f64(self.value_stack.len() - 2));
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::F64)?;

                self.machine.f64_sub(loc_a, loc_b, ret)?;
            }
            Operator::F64Mul => {
                self.fp_stack.pop2()?;
                self.fp_stack
                    .push(FloatValue::cncl_f64(self.value_stack.len() - 2));
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::F64)?;

                self.machine.f64_mul(loc_a, loc_b, ret)?;
            }
            Operator::F64Div => {
                self.fp_stack.pop2()?;
                self.fp_stack
                    .push(FloatValue::cncl_f64(self.value_stack.len() - 2));
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::F64)?;

                self.machine.f64_div(loc_a, loc_b, ret)?;
            }
            Operator::F64Max => {
                self.fp_stack.pop2()?;
                self.fp_stack
                    .push(FloatValue::new(self.value_stack.len() - 2));
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::F64)?;
                self.machine.f64_max(loc_a, loc_b, ret)?;
            }
            Operator::F64Min => {
                self.fp_stack.pop2()?;
                self.fp_stack
                    .push(FloatValue::new(self.value_stack.len() - 2));
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::F64)?;
                self.machine.f64_min(loc_a, loc_b, ret)?;
            }
            Operator::F64Eq => {
                self.fp_stack.pop2()?;
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.f64_cmp_eq(loc_a, loc_b, ret)?;
            }
            Operator::F64Ne => {
                self.fp_stack.pop2()?;
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.f64_cmp_ne(loc_a, loc_b, ret)?;
            }
            Operator::F64Lt => {
                self.fp_stack.pop2()?;
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.f64_cmp_lt(loc_a, loc_b, ret)?;
            }
            Operator::F64Le => {
                self.fp_stack.pop2()?;
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.f64_cmp_le(loc_a, loc_b, ret)?;
            }
            Operator::F64Gt => {
                self.fp_stack.pop2()?;
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.f64_cmp_gt(loc_a, loc_b, ret)?;
            }
            Operator::F64Ge => {
                self.fp_stack.pop2()?;
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::I32)?;
                self.machine.f64_cmp_ge(loc_a, loc_b, ret)?;
            }
            Operator::F64Nearest => {
                self.fp_stack.pop1()?;
                self.fp_stack
                    .push(FloatValue::cncl_f64(self.value_stack.len() - 1));
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.f64_nearest(loc, ret)?;
            }
            Operator::F64Floor => {
                self.fp_stack.pop1()?;
                self.fp_stack
                    .push(FloatValue::cncl_f64(self.value_stack.len() - 1));
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.f64_floor(loc, ret)?;
            }
            Operator::F64Ceil => {
                self.fp_stack.pop1()?;
                self.fp_stack
                    .push(FloatValue::cncl_f64(self.value_stack.len() - 1));
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.f64_ceil(loc, ret)?;
            }
            Operator::F64Trunc => {
                self.fp_stack.pop1()?;
                self.fp_stack
                    .push(FloatValue::cncl_f64(self.value_stack.len() - 1));
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.f64_trunc(loc, ret)?;
            }
            Operator::F64Sqrt => {
                self.fp_stack.pop1()?;
                self.fp_stack
                    .push(FloatValue::cncl_f64(self.value_stack.len() - 1));
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.f64_sqrt(loc, ret)?;
            }

            Operator::F64Copysign => {
                let I2O1 { loc_a, loc_b, ret } = self.i2o1_prepare(WpType::F64)?;

                let (fp_src1, fp_src2) = self.fp_stack.pop2()?;
                self.fp_stack
                    .push(FloatValue::new(self.value_stack.len() - 1));

                let tmp1 = self.machine.acquire_temp_gpr().unwrap();
                let tmp2 = self.machine.acquire_temp_gpr().unwrap();

                if self.machine.arch_supports_canonicalize_nan()
                    && self.config.enable_nan_canonicalization
                {
                    for (fp, loc, tmp) in [(fp_src1, loc_a, tmp1), (fp_src2, loc_b, tmp2)].iter() {
                        match fp.canonicalization {
                            Some(_) => {
                                self.machine
                                    .canonicalize_nan(Size::S64, *loc, Location::GPR(*tmp))
                            }
                            None => {
                                self.machine
                                    .move_location(Size::S64, *loc, Location::GPR(*tmp))
                            }
                        }?;
                    }
                } else {
                    self.machine
                        .move_location(Size::S64, loc_a, Location::GPR(tmp1))?;
                    self.machine
                        .move_location(Size::S64, loc_b, Location::GPR(tmp2))?;
                }
                self.machine.emit_i64_copysign(tmp1, tmp2)?;
                self.machine
                    .move_location(Size::S64, Location::GPR(tmp1), ret)?;

                self.machine.release_gpr(tmp2);
                self.machine.release_gpr(tmp1);
            }

            Operator::F64Abs => {
                // Preserve canonicalization state.

                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);

                self.machine.f64_abs(loc, ret)?;
            }

            Operator::F64Neg => {
                // Preserve canonicalization state.

                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);

                self.machine.f64_neg(loc, ret)?;
            }

            Operator::F64PromoteF32 => {
                let fp = self.fp_stack.pop1()?;
                self.fp_stack.push(fp.promote(self.value_stack.len() - 1)?);
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.convert_f64_f32(loc, ret)?;
            }
            Operator::F32DemoteF64 => {
                let fp = self.fp_stack.pop1()?;
                self.fp_stack.push(fp.demote(self.value_stack.len() - 1)?);
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.convert_f32_f64(loc, ret)?;
            }

            Operator::I32ReinterpretF32 => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                let fp = self.fp_stack.pop1()?;

                if !self.machine.arch_supports_canonicalize_nan()
                    || !self.config.enable_nan_canonicalization
                    || fp.canonicalization.is_none()
                {
                    if loc != ret {
                        self.machine.emit_relaxed_mov(Size::S32, loc, ret)?;
                    }
                } else {
                    self.machine.canonicalize_nan(Size::S32, loc, ret)?;
                }
            }
            Operator::F32ReinterpretI32 => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack
                    .push(FloatValue::new(self.value_stack.len() - 1));

                if loc != ret {
                    self.machine.emit_relaxed_mov(Size::S32, loc, ret)?;
                }
            }

            Operator::I64ReinterpretF64 => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                let fp = self.fp_stack.pop1()?;

                if !self.machine.arch_supports_canonicalize_nan()
                    || !self.config.enable_nan_canonicalization
                    || fp.canonicalization.is_none()
                {
                    if loc != ret {
                        self.machine.emit_relaxed_mov(Size::S64, loc, ret)?;
                    }
                } else {
                    self.machine.canonicalize_nan(Size::S64, loc, ret)?;
                }
            }
            Operator::F64ReinterpretI64 => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack
                    .push(FloatValue::new(self.value_stack.len() - 1));

                if loc != ret {
                    self.machine.emit_relaxed_mov(Size::S64, loc, ret)?;
                }
            }

            Operator::I32TruncF32U => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack.pop1()?;

                self.machine.convert_i32_f32(loc, ret, false, false)?;
            }

            Operator::I32TruncSatF32U => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack.pop1()?;

                self.machine.convert_i32_f32(loc, ret, false, true)?;
            }

            Operator::I32TruncF32S => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack.pop1()?;

                self.machine.convert_i32_f32(loc, ret, true, false)?;
            }
            Operator::I32TruncSatF32S => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack.pop1()?;

                self.machine.convert_i32_f32(loc, ret, true, true)?;
            }

            Operator::I64TruncF32S => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack.pop1()?;

                self.machine.convert_i64_f32(loc, ret, true, false)?;
            }

            Operator::I64TruncSatF32S => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack.pop1()?;

                self.machine.convert_i64_f32(loc, ret, true, true)?;
            }

            Operator::I64TruncF32U => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack.pop1()?;

                self.machine.convert_i64_f32(loc, ret, false, false)?;
            }
            Operator::I64TruncSatF32U => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack.pop1()?;

                self.machine.convert_i64_f32(loc, ret, false, true)?;
            }

            Operator::I32TruncF64U => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack.pop1()?;

                self.machine.convert_i32_f64(loc, ret, false, false)?;
            }

            Operator::I32TruncSatF64U => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack.pop1()?;

                self.machine.convert_i32_f64(loc, ret, false, true)?;
            }

            Operator::I32TruncF64S => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack.pop1()?;

                self.machine.convert_i32_f64(loc, ret, true, false)?;
            }

            Operator::I32TruncSatF64S => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack.pop1()?;

                self.machine.convert_i32_f64(loc, ret, true, true)?;
            }

            Operator::I64TruncF64S => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack.pop1()?;

                self.machine.convert_i64_f64(loc, ret, true, false)?;
            }

            Operator::I64TruncSatF64S => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack.pop1()?;

                self.machine.convert_i64_f64(loc, ret, true, true)?;
            }

            Operator::I64TruncF64U => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack.pop1()?;

                self.machine.convert_i64_f64(loc, ret, false, false)?;
            }

            Operator::I64TruncSatF64U => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack.pop1()?;

                self.machine.convert_i64_f64(loc, ret, false, true)?;
            }

            Operator::F32ConvertI32S => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack
                    .push(FloatValue::new(self.value_stack.len() - 1)); // Converting i32 to f32 never results in NaN.

                self.machine.convert_f32_i32(loc, true, ret)?;
            }
            Operator::F32ConvertI32U => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack
                    .push(FloatValue::new(self.value_stack.len() - 1)); // Converting i32 to f32 never results in NaN.

                self.machine.convert_f32_i32(loc, false, ret)?;
            }
            Operator::F32ConvertI64S => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack
                    .push(FloatValue::new(self.value_stack.len() - 1)); // Converting i64 to f32 never results in NaN.

                self.machine.convert_f32_i64(loc, true, ret)?;
            }
            Operator::F32ConvertI64U => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack
                    .push(FloatValue::new(self.value_stack.len() - 1)); // Converting i64 to f32 never results in NaN.

                self.machine.convert_f32_i64(loc, false, ret)?;
            }

            Operator::F64ConvertI32S => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack
                    .push(FloatValue::new(self.value_stack.len() - 1)); // Converting i32 to f64 never results in NaN.

                self.machine.convert_f64_i32(loc, true, ret)?;
            }
            Operator::F64ConvertI32U => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack
                    .push(FloatValue::new(self.value_stack.len() - 1)); // Converting i32 to f64 never results in NaN.

                self.machine.convert_f64_i32(loc, false, ret)?;
            }
            Operator::F64ConvertI64S => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack
                    .push(FloatValue::new(self.value_stack.len() - 1)); // Converting i64 to f64 never results in NaN.

                self.machine.convert_f64_i64(loc, true, ret)?;
            }
            Operator::F64ConvertI64U => {
                let loc = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack
                    .push(FloatValue::new(self.value_stack.len() - 1)); // Converting i64 to f64 never results in NaN.

                self.machine.convert_f64_i64(loc, false, ret)?;
            }

            Operator::Call { function_index } => {
                let function_index = function_index as usize;

                let sig_index = *self
                    .module
                    .functions
                    .get(FunctionIndex::new(function_index))
                    .unwrap();
                let sig = self.module.signatures.get(sig_index).unwrap();
                let param_types: SmallVec<[WpType; 8]> =
                    sig.params().iter().cloned().map(type_to_wp_type).collect();
                let return_types: SmallVec<[WpType; 1]> =
                    sig.results().iter().cloned().map(type_to_wp_type).collect();

                let params: SmallVec<[_; 8]> = self
                    .value_stack
                    .drain(self.value_stack.len() - param_types.len()..)
                    .collect();
                self.release_locations_only_regs(&params)?;

                self.release_locations_only_osr_state(params.len())?;

                // Pop arguments off the FP stack and canonicalize them if needed.
                //
                // Canonicalization state will be lost across function calls, so early canonicalization
                // is necessary here.
                while let Some(fp) = self.fp_stack.last() {
                    if fp.depth >= self.value_stack.len() {
                        let index = fp.depth - self.value_stack.len();
                        if self.machine.arch_supports_canonicalize_nan()
                            && self.config.enable_nan_canonicalization
                            && fp.canonicalization.is_some()
                        {
                            let size = fp.canonicalization.unwrap().to_size();
                            self.machine
                                .canonicalize_nan(size, params[index], params[index])?;
                        }
                        self.fp_stack.pop().unwrap();
                    } else {
                        break;
                    }
                }

                // Imported functions are called through trampolines placed as custom sections.
                let reloc_target = if function_index < self.module.num_imported_functions {
                    RelocationTarget::CustomSection(SectionIndex::new(function_index))
                } else {
                    RelocationTarget::LocalFunc(LocalFunctionIndex::new(
                        function_index - self.module.num_imported_functions,
                    ))
                };
                let calling_convention = self.calling_convention;

                self.emit_call_native(
                    |this| {
                        let offset = this
                            .machine
                            .mark_instruction_with_trap_code(TrapCode::StackOverflow);
                        let mut relocations = this
                            .machine
                            .emit_call_with_reloc(calling_convention, reloc_target)?;
                        this.machine.mark_instruction_address_end(offset);
                        this.relocations.append(&mut relocations);
                        Ok(())
                    },
                    params.iter().copied(),
                    param_types.iter().copied(),
                )?;

                self.release_locations_only_stack(&params)?;

                if !return_types.is_empty() {
                    let ret = self.acquire_locations(
                        &[(
                            return_types[0],
                            MachineValue::WasmStack(self.value_stack.len()),
                        )],
                        false,
                    )?[0];
                    self.value_stack.push(ret);
                    if return_types[0].is_float() {
                        self.machine.move_location(
                            Size::S64,
                            Location::SIMD(self.machine.get_simd_for_ret()),
                            ret,
                        )?;
                        self.fp_stack
                            .push(FloatValue::new(self.value_stack.len() - 1));
                    } else {
                        self.machine.move_location(
                            Size::S64,
                            Location::GPR(self.machine.get_gpr_for_ret()),
                            ret,
                        )?;
                    }
                }
            }
            Operator::CallIndirect {
                type_index,
                table_index,
                table_byte: _,
            } => {
                // TODO: removed restriction on always being table idx 0;
                // does any code depend on this?
                let table_index = TableIndex::new(table_index as _);
                let index = SignatureIndex::new(type_index as usize);
                let sig = self.module.signatures.get(index).unwrap();
                let param_types: SmallVec<[WpType; 8]> =
                    sig.params().iter().cloned().map(type_to_wp_type).collect();
                let return_types: SmallVec<[WpType; 1]> =
                    sig.results().iter().cloned().map(type_to_wp_type).collect();

                let func_index = self.pop_value_released()?;

                let params: SmallVec<[_; 8]> = self
                    .value_stack
                    .drain(self.value_stack.len() - param_types.len()..)
                    .collect();
                self.release_locations_only_regs(&params)?;

                // Pop arguments off the FP stack and canonicalize them if needed.
                //
                // Canonicalization state will be lost across function calls, so early canonicalization
                // is necessary here.
                while let Some(fp) = self.fp_stack.last() {
                    if fp.depth >= self.value_stack.len() {
                        let index = fp.depth - self.value_stack.len();
                        if self.machine.arch_supports_canonicalize_nan()
                            && self.config.enable_nan_canonicalization
                            && fp.canonicalization.is_some()
                        {
                            let size = fp.canonicalization.unwrap().to_size();
                            self.machine
                                .canonicalize_nan(size, params[index], params[index])?;
                        }
                        self.fp_stack.pop().unwrap();
                    } else {
                        break;
                    }
                }

                let table_base = self.machine.acquire_temp_gpr().unwrap();
                let table_count = self.machine.acquire_temp_gpr().unwrap();
                let sigidx = self.machine.acquire_temp_gpr().unwrap();

                if let Some(local_table_index) = self.module.local_table_index(table_index) {
                    let (vmctx_offset_base, vmctx_offset_len) = (
                        self.vmoffsets.vmctx_vmtable_definition(local_table_index),
                        self.vmoffsets
                            .vmctx_vmtable_definition_current_elements(local_table_index),
                    );
                    self.machine.move_location(
                        Size::S64,
                        Location::Memory(self.machine.get_vmctx_reg(), vmctx_offset_base as i32),
                        Location::GPR(table_base),
                    )?;
                    self.machine.move_location(
                        Size::S32,
                        Location::Memory(self.machine.get_vmctx_reg(), vmctx_offset_len as i32),
                        Location::GPR(table_count),
                    )?;
                } else {
                    // Do an indirection.
                    let import_offset = self.vmoffsets.vmctx_vmtable_import(table_index);
                    self.machine.move_location(
                        Size::S64,
                        Location::Memory(self.machine.get_vmctx_reg(), import_offset as i32),
                        Location::GPR(table_base),
                    )?;

                    // Load len.
                    self.machine.move_location(
                        Size::S32,
                        Location::Memory(
                            table_base,
                            self.vmoffsets.vmtable_definition_current_elements() as _,
                        ),
                        Location::GPR(table_count),
                    )?;

                    // Load base.
                    self.machine.move_location(
                        Size::S64,
                        Location::Memory(table_base, self.vmoffsets.vmtable_definition_base() as _),
                        Location::GPR(table_base),
                    )?;
                }

                self.machine
                    .location_cmp(Size::S32, func_index, Location::GPR(table_count))?;
                self.machine
                    .jmp_on_belowequal(self.special_labels.table_access_oob)?;
                self.machine
                    .move_location(Size::S32, func_index, Location::GPR(table_count))?;
                self.machine.emit_imul_imm32(
                    Size::S64,
                    self.vmoffsets.size_of_vm_funcref() as u32,
                    table_count,
                )?;
                self.machine.location_add(
                    Size::S64,
                    Location::GPR(table_base),
                    Location::GPR(table_count),
                    false,
                )?;

                // deref the table to get a VMFuncRef
                self.machine.move_location(
                    Size::S64,
                    Location::Memory(table_count, self.vmoffsets.vm_funcref_anyfunc_ptr() as i32),
                    Location::GPR(table_count),
                )?;
                // Trap if the FuncRef is null
                self.machine.location_cmp(
                    Size::S64,
                    Location::Imm32(0),
                    Location::GPR(table_count),
                )?;
                self.machine
                    .jmp_on_equal(self.special_labels.indirect_call_null)?;
                self.machine.move_location(
                    Size::S64,
                    Location::Memory(
                        self.machine.get_vmctx_reg(),
                        self.vmoffsets.vmctx_vmshared_signature_id(index) as i32,
                    ),
                    Location::GPR(sigidx),
                )?;

                // Trap if signature mismatches.
                self.machine.location_cmp(
                    Size::S32,
                    Location::GPR(sigidx),
                    Location::Memory(
                        table_count,
                        (self.vmoffsets.vmcaller_checked_anyfunc_type_index() as usize) as i32,
                    ),
                )?;
                self.machine
                    .jmp_on_different(self.special_labels.bad_signature)?;

                self.machine.release_gpr(sigidx);
                self.machine.release_gpr(table_count);
                self.machine.release_gpr(table_base);

                let gpr_for_call = self.machine.get_grp_for_call();
                if table_count != gpr_for_call {
                    self.machine.move_location(
                        Size::S64,
                        Location::GPR(table_count),
                        Location::GPR(gpr_for_call),
                    )?;
                }

                self.release_locations_only_osr_state(params.len())?;

                let vmcaller_checked_anyfunc_func_ptr =
                    self.vmoffsets.vmcaller_checked_anyfunc_func_ptr() as usize;
                let vmcaller_checked_anyfunc_vmctx =
                    self.vmoffsets.vmcaller_checked_anyfunc_vmctx() as usize;
                let calling_convention = self.calling_convention;

                self.emit_call_native(
                    |this| {
                        if this.machine.arch_requires_indirect_call_trampoline() {
                            this.machine
                                .arch_emit_indirect_call_with_trampoline(Location::Memory(
                                    gpr_for_call,
                                    vmcaller_checked_anyfunc_func_ptr as i32,
                                ))
                        } else {
                            let offset = this
                                .machine
                                .mark_instruction_with_trap_code(TrapCode::StackOverflow);

                            // We set the context pointer
                            this.machine.move_location(
                                Size::S64,
                                Location::Memory(
                                    gpr_for_call,
                                    vmcaller_checked_anyfunc_vmctx as i32,
                                ),
                                this.machine
                                    .get_simple_param_location(0, calling_convention),
                            )?;

                            this.machine.emit_call_location(Location::Memory(
                                gpr_for_call,
                                vmcaller_checked_anyfunc_func_ptr as i32,
                            ))?;
                            this.machine.mark_instruction_address_end(offset);
                            Ok(())
                        }
                    },
                    params.iter().copied(),
                    param_types.iter().copied(),
                )?;

                self.release_locations_only_stack(&params)?;

                if !return_types.is_empty() {
                    let ret = self.acquire_locations(
                        &[(
                            return_types[0],
                            MachineValue::WasmStack(self.value_stack.len()),
                        )],
                        false,
                    )?[0];
                    self.value_stack.push(ret);
                    if return_types[0].is_float() {
                        self.machine.move_location(
                            Size::S64,
                            Location::SIMD(self.machine.get_simd_for_ret()),
                            ret,
                        )?;
                        self.fp_stack
                            .push(FloatValue::new(self.value_stack.len() - 1));
                    } else {
                        self.machine.move_location(
                            Size::S64,
                            Location::GPR(self.machine.get_gpr_for_ret()),
                            ret,
                        )?;
                    }
                }
            }
            Operator::If { blockty } => {
                let label_end = self.machine.get_label();
                let label_else = self.machine.get_label();

                let cond = self.pop_value_released()?;

                let frame = ControlFrame {
                    label: label_end,
                    loop_like: false,
                    if_else: IfElseState::If(label_else),
                    returns: match blockty {
                        WpTypeOrFuncType::Empty => smallvec![],
                        WpTypeOrFuncType::Type(inner_ty) => smallvec![inner_ty],
                        _ => {
                            return Err(CompileError::Codegen(
                                "If: multi-value returns not yet implemented".to_owned(),
                            ))
                        }
                    },
                    value_stack_depth: self.value_stack.len(),
                    fp_stack_depth: self.fp_stack.len(),
                    state: self.state.clone(),
                    state_diff_id: self.get_state_diff(),
                };
                self.control_stack.push(frame);
                self.machine
                    .emit_relaxed_cmp(Size::S32, Location::Imm32(0), cond)?;
                self.machine.jmp_on_equal(label_else)?;
            }
            Operator::Else => {
                let frame = self.control_stack.last_mut().unwrap();

                if !was_unreachable && !frame.returns.is_empty() {
                    let first_return = frame.returns[0];
                    let loc = *self.value_stack.last().unwrap();
                    let canonicalize = if first_return.is_float() {
                        let fp = self.fp_stack.peek1()?;
                        self.machine.arch_supports_canonicalize_nan()
                            && self.config.enable_nan_canonicalization
                            && fp.canonicalization.is_some()
                    } else {
                        false
                    };
                    self.machine
                        .emit_function_return_value(first_return, canonicalize, loc)?;
                }

                let frame = &self.control_stack.last_mut().unwrap();
                let stack_depth = frame.value_stack_depth;
                let fp_depth = frame.fp_stack_depth;
                self.release_locations_value(stack_depth)?;
                self.value_stack.truncate(stack_depth);
                self.fp_stack.truncate(fp_depth);
                let mut frame = &mut self.control_stack.last_mut().unwrap();

                match frame.if_else {
                    IfElseState::If(label) => {
                        self.machine.jmp_unconditionnal(frame.label)?;
                        self.machine.emit_label(label)?;
                        frame.if_else = IfElseState::Else;
                    }
                    _ => {
                        return Err(CompileError::Codegen(
                            "Else: frame.if_else unreachable code".to_owned(),
                        ))
                    }
                }
            }
            // `TypedSelect` must be used for extern refs so ref counting should
            // be done with TypedSelect. But otherwise they're the same.
            Operator::TypedSelect { .. } | Operator::Select => {
                let cond = self.pop_value_released()?;
                let v_b = self.pop_value_released()?;
                let v_a = self.pop_value_released()?;
                let cncl: Option<(Option<CanonicalizeType>, Option<CanonicalizeType>)> =
                    if self.fp_stack.len() >= 2
                        && self.fp_stack[self.fp_stack.len() - 2].depth == self.value_stack.len()
                        && self.fp_stack[self.fp_stack.len() - 1].depth
                            == self.value_stack.len() + 1
                    {
                        let (left, right) = self.fp_stack.pop2()?;
                        self.fp_stack.push(FloatValue::new(self.value_stack.len()));
                        Some((left.canonicalization, right.canonicalization))
                    } else {
                        None
                    };
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);

                let end_label = self.machine.get_label();
                let zero_label = self.machine.get_label();

                self.machine
                    .emit_relaxed_cmp(Size::S32, Location::Imm32(0), cond)?;
                self.machine.jmp_on_equal(zero_label)?;
                match cncl {
                    Some((Some(fp), _))
                        if self.machine.arch_supports_canonicalize_nan()
                            && self.config.enable_nan_canonicalization =>
                    {
                        self.machine.canonicalize_nan(fp.to_size(), v_a, ret)?;
                    }
                    _ => {
                        if v_a != ret {
                            self.machine.emit_relaxed_mov(Size::S64, v_a, ret)?;
                        }
                    }
                }
                self.machine.jmp_unconditionnal(end_label)?;
                self.machine.emit_label(zero_label)?;
                match cncl {
                    Some((_, Some(fp)))
                        if self.machine.arch_supports_canonicalize_nan()
                            && self.config.enable_nan_canonicalization =>
                    {
                        self.machine.canonicalize_nan(fp.to_size(), v_b, ret)?;
                    }
                    _ => {
                        if v_b != ret {
                            self.machine.emit_relaxed_mov(Size::S64, v_b, ret)?;
                        }
                    }
                }
                self.machine.emit_label(end_label)?;
            }
            Operator::Block { blockty } => {
                let frame = ControlFrame {
                    label: self.machine.get_label(),
                    loop_like: false,
                    if_else: IfElseState::None,
                    returns: match blockty {
                        WpTypeOrFuncType::Empty => smallvec![],
                        WpTypeOrFuncType::Type(inner_ty) => smallvec![inner_ty],
                        _ => {
                            return Err(CompileError::Codegen(
                                "Block: multi-value returns not yet implemented".to_owned(),
                            ))
                        }
                    },
                    value_stack_depth: self.value_stack.len(),
                    fp_stack_depth: self.fp_stack.len(),
                    state: self.state.clone(),
                    state_diff_id: self.get_state_diff(),
                };
                self.control_stack.push(frame);
            }
            Operator::Loop { blockty } => {
                self.machine.align_for_loop()?;
                let label = self.machine.get_label();
                let state_diff_id = self.get_state_diff();
                let _activate_offset = self.machine.assembler_get_offset().0;

                self.control_stack.push(ControlFrame {
                    label,
                    loop_like: true,
                    if_else: IfElseState::None,
                    returns: match blockty {
                        WpTypeOrFuncType::Empty => smallvec![],
                        WpTypeOrFuncType::Type(inner_ty) => smallvec![inner_ty],
                        _ => {
                            return Err(CompileError::Codegen(
                                "Loop: multi-value returns not yet implemented".to_owned(),
                            ))
                        }
                    },
                    value_stack_depth: self.value_stack.len(),
                    fp_stack_depth: self.fp_stack.len(),
                    state: self.state.clone(),
                    state_diff_id,
                });
                self.machine.emit_label(label)?;

                // TODO: Re-enable interrupt signal check without branching
            }
            Operator::Nop => {}
            Operator::MemorySize { mem, mem_byte: _ } => {
                let memory_index = MemoryIndex::new(mem as usize);
                self.machine.move_location(
                    Size::S64,
                    Location::Memory(
                        self.machine.get_vmctx_reg(),
                        self.vmoffsets.vmctx_builtin_function(
                            if self.module.local_memory_index(memory_index).is_some() {
                                VMBuiltinFunctionIndex::get_memory32_size_index()
                            } else {
                                VMBuiltinFunctionIndex::get_imported_memory32_size_index()
                            },
                        ) as i32,
                    ),
                    Location::GPR(self.machine.get_grp_for_call()),
                )?;
                self.emit_call_native(
                    |this| {
                        this.machine
                            .emit_call_register(this.machine.get_grp_for_call())
                    },
                    // [vmctx, memory_index]
                    iter::once(Location::Imm32(memory_index.index() as u32)),
                    iter::once(WpType::I64),
                )?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.move_location(
                    Size::S64,
                    Location::GPR(self.machine.get_gpr_for_ret()),
                    ret,
                )?;
            }
            Operator::MemoryInit { data_index, mem } => {
                let len = self.value_stack.pop().unwrap();
                let src = self.value_stack.pop().unwrap();
                let dst = self.value_stack.pop().unwrap();
                self.release_locations_only_regs(&[len, src, dst])?;

                self.machine.move_location(
                    Size::S64,
                    Location::Memory(
                        self.machine.get_vmctx_reg(),
                        self.vmoffsets
                            .vmctx_builtin_function(VMBuiltinFunctionIndex::get_memory_init_index())
                            as i32,
                    ),
                    Location::GPR(self.machine.get_grp_for_call()),
                )?;

                // TODO: should this be 3?
                self.release_locations_only_osr_state(1)?;

                self.emit_call_native(
                    |this| {
                        this.machine
                            .emit_call_register(this.machine.get_grp_for_call())
                    },
                    // [vmctx, memory_index, data_index, dst, src, len]
                    [
                        Location::Imm32(mem),
                        Location::Imm32(data_index),
                        dst,
                        src,
                        len,
                    ]
                    .iter()
                    .cloned(),
                    [
                        WpType::I64,
                        WpType::I64,
                        WpType::I64,
                        WpType::I64,
                        WpType::I64,
                    ]
                    .iter()
                    .cloned(),
                )?;
                self.release_locations_only_stack(&[dst, src, len])?;
            }
            Operator::DataDrop { data_index } => {
                self.machine.move_location(
                    Size::S64,
                    Location::Memory(
                        self.machine.get_vmctx_reg(),
                        self.vmoffsets
                            .vmctx_builtin_function(VMBuiltinFunctionIndex::get_data_drop_index())
                            as i32,
                    ),
                    Location::GPR(self.machine.get_grp_for_call()),
                )?;

                self.emit_call_native(
                    |this| {
                        this.machine
                            .emit_call_register(this.machine.get_grp_for_call())
                    },
                    // [vmctx, data_index]
                    iter::once(Location::Imm32(data_index)),
                    iter::once(WpType::I64),
                )?;
            }
            Operator::MemoryCopy { dst_mem, src_mem } => {
                // ignore until we support multiple memories
                let _dst = dst_mem;
                let len = self.value_stack.pop().unwrap();
                let src_pos = self.value_stack.pop().unwrap();
                let dst_pos = self.value_stack.pop().unwrap();
                self.release_locations_only_regs(&[len, src_pos, dst_pos])?;

                let memory_index = MemoryIndex::new(src_mem as usize);
                let (memory_copy_index, memory_index) =
                    if self.module.local_memory_index(memory_index).is_some() {
                        (
                            VMBuiltinFunctionIndex::get_memory_copy_index(),
                            memory_index,
                        )
                    } else {
                        (
                            VMBuiltinFunctionIndex::get_imported_memory_copy_index(),
                            memory_index,
                        )
                    };

                self.machine.move_location(
                    Size::S64,
                    Location::Memory(
                        self.machine.get_vmctx_reg(),
                        self.vmoffsets.vmctx_builtin_function(memory_copy_index) as i32,
                    ),
                    Location::GPR(self.machine.get_grp_for_call()),
                )?;

                // TODO: should this be 3?
                self.release_locations_only_osr_state(1)?;

                self.emit_call_native(
                    |this| {
                        this.machine
                            .emit_call_register(this.machine.get_grp_for_call())
                    },
                    // [vmctx, memory_index, dst, src, len]
                    [
                        Location::Imm32(memory_index.index() as u32),
                        dst_pos,
                        src_pos,
                        len,
                    ]
                    .iter()
                    .cloned(),
                    [WpType::I32, WpType::I64, WpType::I64, WpType::I64]
                        .iter()
                        .cloned(),
                )?;
                self.release_locations_only_stack(&[dst_pos, src_pos, len])?;
            }
            Operator::MemoryFill { mem } => {
                let len = self.value_stack.pop().unwrap();
                let val = self.value_stack.pop().unwrap();
                let dst = self.value_stack.pop().unwrap();
                self.release_locations_only_regs(&[len, val, dst])?;

                let memory_index = MemoryIndex::new(mem as usize);
                let (memory_fill_index, memory_index) =
                    if self.module.local_memory_index(memory_index).is_some() {
                        (
                            VMBuiltinFunctionIndex::get_memory_fill_index(),
                            memory_index,
                        )
                    } else {
                        (
                            VMBuiltinFunctionIndex::get_imported_memory_fill_index(),
                            memory_index,
                        )
                    };

                self.machine.move_location(
                    Size::S64,
                    Location::Memory(
                        self.machine.get_vmctx_reg(),
                        self.vmoffsets.vmctx_builtin_function(memory_fill_index) as i32,
                    ),
                    Location::GPR(self.machine.get_grp_for_call()),
                )?;

                // TODO: should this be 3?
                self.release_locations_only_osr_state(1)?;

                self.emit_call_native(
                    |this| {
                        this.machine
                            .emit_call_register(this.machine.get_grp_for_call())
                    },
                    // [vmctx, memory_index, dst, src, len]
                    [Location::Imm32(memory_index.index() as u32), dst, val, len]
                        .iter()
                        .cloned(),
                    [WpType::I32, WpType::I64, WpType::I64, WpType::I64]
                        .iter()
                        .cloned(),
                )?;
                self.release_locations_only_stack(&[dst, val, len])?;
            }
            Operator::MemoryGrow { mem, mem_byte: _ } => {
                let memory_index = MemoryIndex::new(mem as usize);
                let param_pages = self.value_stack.pop().unwrap();

                self.release_locations_only_regs(&[param_pages])?;

                self.machine.move_location(
                    Size::S64,
                    Location::Memory(
                        self.machine.get_vmctx_reg(),
                        self.vmoffsets.vmctx_builtin_function(
                            if self.module.local_memory_index(memory_index).is_some() {
                                VMBuiltinFunctionIndex::get_memory32_grow_index()
                            } else {
                                VMBuiltinFunctionIndex::get_imported_memory32_grow_index()
                            },
                        ) as i32,
                    ),
                    Location::GPR(self.machine.get_grp_for_call()),
                )?;

                self.release_locations_only_osr_state(1)?;

                self.emit_call_native(
                    |this| {
                        this.machine
                            .emit_call_register(this.machine.get_grp_for_call())
                    },
                    // [vmctx, val, memory_index]
                    iter::once(param_pages)
                        .chain(iter::once(Location::Imm32(memory_index.index() as u32))),
                    [WpType::I64, WpType::I64].iter().cloned(),
                )?;

                self.release_locations_only_stack(&[param_pages])?;

                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.move_location(
                    Size::S64,
                    Location::GPR(self.machine.get_gpr_for_ret()),
                    ret,
                )?;
            }
            Operator::I32Load { ref memarg } => {
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_load(
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::F32Load { ref memarg } => {
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack
                    .push(FloatValue::new(self.value_stack.len() - 1));
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.f32_load(
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32Load8U { ref memarg } => {
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_load_8u(
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32Load8S { ref memarg } => {
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_load_8s(
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32Load16U { ref memarg } => {
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_load_16u(
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32Load16S { ref memarg } => {
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_load_16s(
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32Store { ref memarg } => {
                let target_value = self.pop_value_released()?;
                let target_addr = self.pop_value_released()?;
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_save(
                            target_value,
                            memarg,
                            target_addr,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::F32Store { ref memarg } => {
                let target_value = self.pop_value_released()?;
                let target_addr = self.pop_value_released()?;
                let fp = self.fp_stack.pop1()?;
                let config_nan_canonicalization = self.config.enable_nan_canonicalization;
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.f32_save(
                            target_value,
                            memarg,
                            target_addr,
                            config_nan_canonicalization && fp.canonicalization.is_some(),
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32Store8 { ref memarg } => {
                let target_value = self.pop_value_released()?;
                let target_addr = self.pop_value_released()?;
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_save_8(
                            target_value,
                            memarg,
                            target_addr,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32Store16 { ref memarg } => {
                let target_value = self.pop_value_released()?;
                let target_addr = self.pop_value_released()?;
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_save_16(
                            target_value,
                            memarg,
                            target_addr,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64Load { ref memarg } => {
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_load(
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::F64Load { ref memarg } => {
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::F64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.fp_stack
                    .push(FloatValue::new(self.value_stack.len() - 1));
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.f64_load(
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64Load8U { ref memarg } => {
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_load_8u(
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64Load8S { ref memarg } => {
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_load_8s(
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64Load16U { ref memarg } => {
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_load_16u(
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64Load16S { ref memarg } => {
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_load_16s(
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64Load32U { ref memarg } => {
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_load_32u(
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64Load32S { ref memarg } => {
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_load_32s(
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64Store { ref memarg } => {
                let target_value = self.pop_value_released()?;
                let target_addr = self.pop_value_released()?;

                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_save(
                            target_value,
                            memarg,
                            target_addr,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::F64Store { ref memarg } => {
                let target_value = self.pop_value_released()?;
                let target_addr = self.pop_value_released()?;
                let fp = self.fp_stack.pop1()?;
                let config_nan_canonicalization = self.config.enable_nan_canonicalization;
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.f64_save(
                            target_value,
                            memarg,
                            target_addr,
                            config_nan_canonicalization && fp.canonicalization.is_some(),
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64Store8 { ref memarg } => {
                let target_value = self.pop_value_released()?;
                let target_addr = self.pop_value_released()?;
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_save_8(
                            target_value,
                            memarg,
                            target_addr,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64Store16 { ref memarg } => {
                let target_value = self.pop_value_released()?;
                let target_addr = self.pop_value_released()?;
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_save_16(
                            target_value,
                            memarg,
                            target_addr,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64Store32 { ref memarg } => {
                let target_value = self.pop_value_released()?;
                let target_addr = self.pop_value_released()?;
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_save_32(
                            target_value,
                            memarg,
                            target_addr,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::Unreachable => {
                self.mark_trappable();
                self.machine
                    .emit_illegal_op(TrapCode::UnreachableCodeReached)?;
                self.unreachable_depth = 1;
            }
            Operator::Return => {
                let frame = &self.control_stack[0];
                if !frame.returns.is_empty() {
                    if frame.returns.len() != 1 {
                        return Err(CompileError::Codegen(
                            "Return: incorrect frame.returns".to_owned(),
                        ));
                    }
                    let first_return = frame.returns[0];
                    let loc = *self.value_stack.last().unwrap();
                    let canonicalize = if first_return.is_float() {
                        let fp = self.fp_stack.peek1()?;
                        self.machine.arch_supports_canonicalize_nan()
                            && self.config.enable_nan_canonicalization
                            && fp.canonicalization.is_some()
                    } else {
                        false
                    };
                    self.machine
                        .emit_function_return_value(first_return, canonicalize, loc)?;
                }
                let frame = &self.control_stack[0];
                let frame_depth = frame.value_stack_depth;
                let label = frame.label;
                self.release_locations_keep_state(frame_depth)?;
                self.machine.jmp_unconditionnal(label)?;
                self.unreachable_depth = 1;
            }
            Operator::Br { relative_depth } => {
                let frame =
                    &self.control_stack[self.control_stack.len() - 1 - (relative_depth as usize)];
                if !frame.loop_like && !frame.returns.is_empty() {
                    if frame.returns.len() != 1 {
                        return Err(CompileError::Codegen(
                            "Br: incorrect frame.returns".to_owned(),
                        ));
                    }
                    let first_return = frame.returns[0];
                    let loc = *self.value_stack.last().unwrap();
                    let canonicalize = if first_return.is_float() {
                        let fp = self.fp_stack.peek1()?;
                        self.machine.arch_supports_canonicalize_nan()
                            && self.config.enable_nan_canonicalization
                            && fp.canonicalization.is_some()
                    } else {
                        false
                    };
                    self.machine
                        .emit_function_return_value(first_return, canonicalize, loc)?;
                }
                let stack_len = self.control_stack.len();
                let frame = &mut self.control_stack[stack_len - 1 - (relative_depth as usize)];
                let frame_depth = frame.value_stack_depth;
                let label = frame.label;

                self.release_locations_keep_state(frame_depth)?;
                self.machine.jmp_unconditionnal(label)?;
                self.unreachable_depth = 1;
            }
            Operator::BrIf { relative_depth } => {
                let after = self.machine.get_label();
                let cond = self.pop_value_released()?;
                self.machine
                    .emit_relaxed_cmp(Size::S32, Location::Imm32(0), cond)?;
                self.machine.jmp_on_equal(after)?;

                let frame =
                    &self.control_stack[self.control_stack.len() - 1 - (relative_depth as usize)];
                if !frame.loop_like && !frame.returns.is_empty() {
                    if frame.returns.len() != 1 {
                        return Err(CompileError::Codegen(
                            "BrIf: incorrect frame.returns".to_owned(),
                        ));
                    }

                    let first_return = frame.returns[0];
                    let loc = *self.value_stack.last().unwrap();
                    let canonicalize = if first_return.is_float() {
                        let fp = self.fp_stack.peek1()?;
                        self.machine.arch_supports_canonicalize_nan()
                            && self.config.enable_nan_canonicalization
                            && fp.canonicalization.is_some()
                    } else {
                        false
                    };
                    self.machine
                        .emit_function_return_value(first_return, canonicalize, loc)?;
                }
                let stack_len = self.control_stack.len();
                let frame = &mut self.control_stack[stack_len - 1 - (relative_depth as usize)];
                let stack_depth = frame.value_stack_depth;
                let label = frame.label;
                self.release_locations_keep_state(stack_depth)?;
                self.machine.jmp_unconditionnal(label)?;

                self.machine.emit_label(after)?;
            }
            Operator::BrTable { ref targets } => {
                let default_target = targets.default();
                let targets = targets
                    .targets()
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| CompileError::Codegen(format!("BrTable read_table: {:?}", e)))?;
                let cond = self.pop_value_released()?;
                let table_label = self.machine.get_label();
                let mut table: Vec<Label> = vec![];
                let default_br = self.machine.get_label();
                self.machine.emit_relaxed_cmp(
                    Size::S32,
                    Location::Imm32(targets.len() as u32),
                    cond,
                )?;
                self.machine.jmp_on_aboveequal(default_br)?;

                self.machine.emit_jmp_to_jumptable(table_label, cond)?;

                for target in targets.iter() {
                    let label = self.machine.get_label();
                    self.machine.emit_label(label)?;
                    table.push(label);
                    let frame =
                        &self.control_stack[self.control_stack.len() - 1 - (*target as usize)];
                    if !frame.loop_like && !frame.returns.is_empty() {
                        if frame.returns.len() != 1 {
                            return Err(CompileError::Codegen(format!(
                                "BrTable: incorrect frame.returns for {:?}",
                                target
                            )));
                        }

                        let first_return = frame.returns[0];
                        let loc = *self.value_stack.last().unwrap();
                        let canonicalize = if first_return.is_float() {
                            let fp = self.fp_stack.peek1()?;
                            self.machine.arch_supports_canonicalize_nan()
                                && self.config.enable_nan_canonicalization
                                && fp.canonicalization.is_some()
                        } else {
                            false
                        };
                        self.machine
                            .emit_function_return_value(first_return, canonicalize, loc)?;
                    }
                    let frame =
                        &self.control_stack[self.control_stack.len() - 1 - (*target as usize)];
                    let stack_depth = frame.value_stack_depth;
                    let label = frame.label;
                    self.release_locations_keep_state(stack_depth)?;
                    self.machine.jmp_unconditionnal(label)?;
                }
                self.machine.emit_label(default_br)?;

                {
                    let frame = &self.control_stack
                        [self.control_stack.len() - 1 - (default_target as usize)];
                    if !frame.loop_like && !frame.returns.is_empty() {
                        if frame.returns.len() != 1 {
                            return Err(CompileError::Codegen(
                                "BrTable: incorrect frame.returns".to_owned(),
                            ));
                        }

                        let first_return = frame.returns[0];
                        let loc = *self.value_stack.last().unwrap();
                        let canonicalize = if first_return.is_float() {
                            let fp = self.fp_stack.peek1()?;
                            self.machine.arch_supports_canonicalize_nan()
                                && self.config.enable_nan_canonicalization
                                && fp.canonicalization.is_some()
                        } else {
                            false
                        };
                        self.machine
                            .emit_function_return_value(first_return, canonicalize, loc)?;
                    }
                    let frame = &self.control_stack
                        [self.control_stack.len() - 1 - (default_target as usize)];
                    let stack_depth = frame.value_stack_depth;
                    let label = frame.label;
                    self.release_locations_keep_state(stack_depth)?;
                    self.machine.jmp_unconditionnal(label)?;
                }

                self.machine.emit_label(table_label)?;
                for x in table {
                    self.machine.jmp_unconditionnal(x)?;
                }
                self.unreachable_depth = 1;
            }
            Operator::Drop => {
                self.pop_value_released()?;
                if let Some(x) = self.fp_stack.last() {
                    if x.depth == self.value_stack.len() {
                        self.fp_stack.pop1()?;
                    }
                }
            }
            Operator::End => {
                let frame = self.control_stack.pop().unwrap();

                if !was_unreachable && !frame.returns.is_empty() {
                    let loc = *self.value_stack.last().unwrap();
                    let canonicalize = if frame.returns[0].is_float() {
                        let fp = self.fp_stack.peek1()?;
                        self.machine.arch_supports_canonicalize_nan()
                            && self.config.enable_nan_canonicalization
                            && fp.canonicalization.is_some()
                    } else {
                        false
                    };
                    self.machine
                        .emit_function_return_value(frame.returns[0], canonicalize, loc)?;
                }

                if self.control_stack.is_empty() {
                    self.machine.emit_label(frame.label)?;
                    self.finalize_locals(self.calling_convention)?;
                    self.machine.emit_function_epilog()?;

                    // Make a copy of the return value in XMM0, as required by the SysV CC.
                    match self.signature.results() {
                        [x] if *x == Type::F32 || *x == Type::F64 => {
                            self.machine.emit_function_return_float()?;
                        }
                        _ => {}
                    }
                    self.machine.emit_ret()?;
                } else {
                    let released = &self.value_stack.clone()[frame.value_stack_depth..];
                    self.release_locations(released)?;
                    self.value_stack.truncate(frame.value_stack_depth);
                    self.fp_stack.truncate(frame.fp_stack_depth);

                    if !frame.loop_like {
                        self.machine.emit_label(frame.label)?;
                    }

                    if let IfElseState::If(label) = frame.if_else {
                        self.machine.emit_label(label)?;
                    }

                    if !frame.returns.is_empty() {
                        if frame.returns.len() != 1 {
                            return Err(CompileError::Codegen(
                                "End: incorrect frame.returns".to_owned(),
                            ));
                        }
                        let loc = self.acquire_locations(
                            &[(
                                frame.returns[0],
                                MachineValue::WasmStack(self.value_stack.len()),
                            )],
                            false,
                        )?[0];
                        self.machine.move_location(
                            Size::S64,
                            Location::GPR(self.machine.get_gpr_for_ret()),
                            loc,
                        )?;
                        self.value_stack.push(loc);
                        if frame.returns[0].is_float() {
                            self.fp_stack
                                .push(FloatValue::new(self.value_stack.len() - 1));
                            // we already canonicalized at the `Br*` instruction or here previously.
                        }
                    }
                }
            }
            Operator::AtomicFence => {
                // Fence is a nop.
                //
                // Fence was added to preserve information about fences from
                // source languages. If in the future Wasm extends the memory
                // model, and if we hadn't recorded what fences used to be there,
                // it would lead to data races that weren't present in the
                // original source language.
                self.machine.emit_memory_fence()?;
            }
            Operator::I32AtomicLoad { ref memarg } => {
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_load(
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32AtomicLoad8U { ref memarg } => {
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_load_8u(
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32AtomicLoad16U { ref memarg } => {
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_load_16u(
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32AtomicStore { ref memarg } => {
                let target_value = self.pop_value_released()?;
                let target_addr = self.pop_value_released()?;
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_save(
                            target_value,
                            memarg,
                            target_addr,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32AtomicStore8 { ref memarg } => {
                let target_value = self.pop_value_released()?;
                let target_addr = self.pop_value_released()?;
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_save_8(
                            target_value,
                            memarg,
                            target_addr,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32AtomicStore16 { ref memarg } => {
                let target_value = self.pop_value_released()?;
                let target_addr = self.pop_value_released()?;
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_save_16(
                            target_value,
                            memarg,
                            target_addr,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicLoad { ref memarg } => {
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_load(
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicLoad8U { ref memarg } => {
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_load_8u(
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicLoad16U { ref memarg } => {
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_load_16u(
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicLoad32U { ref memarg } => {
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_load_32u(
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicStore { ref memarg } => {
                let target_value = self.pop_value_released()?;
                let target_addr = self.pop_value_released()?;
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_save(
                            target_value,
                            memarg,
                            target_addr,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicStore8 { ref memarg } => {
                let target_value = self.pop_value_released()?;
                let target_addr = self.pop_value_released()?;
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_save_8(
                            target_value,
                            memarg,
                            target_addr,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicStore16 { ref memarg } => {
                let target_value = self.pop_value_released()?;
                let target_addr = self.pop_value_released()?;
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_save_16(
                            target_value,
                            memarg,
                            target_addr,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicStore32 { ref memarg } => {
                let target_value = self.pop_value_released()?;
                let target_addr = self.pop_value_released()?;
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_save_32(
                            target_value,
                            memarg,
                            target_addr,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32AtomicRmwAdd { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_add(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmwAdd { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_add(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32AtomicRmw8AddU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_add_8u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32AtomicRmw16AddU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_add_16u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmw8AddU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_add_8u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmw16AddU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_add_16u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmw32AddU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_add_32u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32AtomicRmwSub { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_sub(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmwSub { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_sub(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32AtomicRmw8SubU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_sub_8u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32AtomicRmw16SubU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_sub_16u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmw8SubU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_sub_8u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmw16SubU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_sub_16u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmw32SubU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_sub_32u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32AtomicRmwAnd { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_and(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmwAnd { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_and(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32AtomicRmw8AndU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_and_8u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32AtomicRmw16AndU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_and_16u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmw8AndU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_and_8u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmw16AndU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_and_16u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmw32AndU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_and_32u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32AtomicRmwOr { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_or(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmwOr { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_or(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32AtomicRmw8OrU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_or_8u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32AtomicRmw16OrU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_or_16u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmw8OrU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_or_8u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmw16OrU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_or_16u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmw32OrU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_or_32u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32AtomicRmwXor { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_xor(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmwXor { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_xor(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32AtomicRmw8XorU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_xor_8u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32AtomicRmw16XorU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_xor_16u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmw8XorU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_xor_8u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmw16XorU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_xor_16u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmw32XorU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_xor_32u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32AtomicRmwXchg { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_xchg(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmwXchg { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_xchg(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32AtomicRmw8XchgU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_xchg_8u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32AtomicRmw16XchgU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_xchg_16u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmw8XchgU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_xchg_8u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmw16XchgU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_xchg_16u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmw32XchgU { ref memarg } => {
                let loc = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_xchg_32u(
                            loc,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32AtomicRmwCmpxchg { ref memarg } => {
                let new = self.pop_value_released()?;
                let cmp = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_cmpxchg(
                            new,
                            cmp,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmwCmpxchg { ref memarg } => {
                let new = self.pop_value_released()?;
                let cmp = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_cmpxchg(
                            new,
                            cmp,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32AtomicRmw8CmpxchgU { ref memarg } => {
                let new = self.pop_value_released()?;
                let cmp = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_cmpxchg_8u(
                            new,
                            cmp,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I32AtomicRmw16CmpxchgU { ref memarg } => {
                let new = self.pop_value_released()?;
                let cmp = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i32_atomic_cmpxchg_16u(
                            new,
                            cmp,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmw8CmpxchgU { ref memarg } => {
                let new = self.pop_value_released()?;
                let cmp = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_cmpxchg_8u(
                            new,
                            cmp,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmw16CmpxchgU { ref memarg } => {
                let new = self.pop_value_released()?;
                let cmp = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_cmpxchg_16u(
                            new,
                            cmp,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }
            Operator::I64AtomicRmw32CmpxchgU { ref memarg } => {
                let new = self.pop_value_released()?;
                let cmp = self.pop_value_released()?;
                let target = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I64, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.op_memory(
                    |this,
                     need_check,
                     imported_memories,
                     offset,
                     heap_access_oob,
                     unaligned_atomic| {
                        this.machine.i64_atomic_cmpxchg_32u(
                            new,
                            cmp,
                            target,
                            memarg,
                            ret,
                            need_check,
                            imported_memories,
                            offset,
                            heap_access_oob,
                            unaligned_atomic,
                        )
                    },
                )?;
            }

            Operator::RefNull { .. } => {
                self.value_stack.push(Location::Imm64(0));
                self.state.wasm_stack.push(WasmAbstractValue::Const(0));
            }
            Operator::RefFunc { function_index } => {
                self.machine.move_location(
                    Size::S64,
                    Location::Memory(
                        self.machine.get_vmctx_reg(),
                        self.vmoffsets
                            .vmctx_builtin_function(VMBuiltinFunctionIndex::get_func_ref_index())
                            as i32,
                    ),
                    Location::GPR(self.machine.get_grp_for_call()),
                )?;

                // TODO: unclear if we need this? check other new insts with no stack ops
                //.machine.release_locations_only_osr_state(1);
                self.emit_call_native(
                    |this| {
                        this.machine
                            .emit_call_register(this.machine.get_grp_for_call())
                    },
                    // [vmctx, func_index] -> funcref
                    iter::once(Location::Imm32(function_index as u32)),
                    iter::once(WpType::I64),
                )?;

                let ret = self.acquire_locations(
                    &[(
                        WpType::FuncRef,
                        MachineValue::WasmStack(self.value_stack.len()),
                    )],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.move_location(
                    Size::S64,
                    Location::GPR(self.machine.get_gpr_for_ret()),
                    ret,
                )?;
            }
            Operator::RefIsNull => {
                let loc_a = self.pop_value_released()?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.machine.i64_cmp_eq(loc_a, Location::Imm64(0), ret)?;
                self.value_stack.push(ret);
            }
            Operator::TableSet { table: index } => {
                let table_index = TableIndex::new(index as _);
                let value = self.value_stack.pop().unwrap();
                let index = self.value_stack.pop().unwrap();
                // double check this does what I think it does
                self.release_locations_only_regs(&[value, index])?;

                self.machine.move_location(
                    Size::S64,
                    Location::Memory(
                        self.machine.get_vmctx_reg(),
                        self.vmoffsets.vmctx_builtin_function(
                            if self.module.local_table_index(table_index).is_some() {
                                VMBuiltinFunctionIndex::get_table_set_index()
                            } else {
                                VMBuiltinFunctionIndex::get_imported_table_set_index()
                            },
                        ) as i32,
                    ),
                    Location::GPR(self.machine.get_grp_for_call()),
                )?;

                // TODO: should this be 2?
                self.release_locations_only_osr_state(1)?;
                self.emit_call_native(
                    |this| {
                        this.machine
                            .emit_call_register(this.machine.get_grp_for_call())
                    },
                    // [vmctx, table_index, elem_index, reftype]
                    [Location::Imm32(table_index.index() as u32), index, value]
                        .iter()
                        .cloned(),
                    [WpType::I32, WpType::I64, WpType::I64].iter().cloned(),
                )?;

                self.release_locations_only_stack(&[index, value])?;
            }
            Operator::TableGet { table: index } => {
                let table_index = TableIndex::new(index as _);
                let index = self.value_stack.pop().unwrap();
                self.release_locations_only_regs(&[index])?;

                self.machine.move_location(
                    Size::S64,
                    Location::Memory(
                        self.machine.get_vmctx_reg(),
                        self.vmoffsets.vmctx_builtin_function(
                            if self.module.local_table_index(table_index).is_some() {
                                VMBuiltinFunctionIndex::get_table_get_index()
                            } else {
                                VMBuiltinFunctionIndex::get_imported_table_get_index()
                            },
                        ) as i32,
                    ),
                    Location::GPR(self.machine.get_grp_for_call()),
                )?;

                self.release_locations_only_osr_state(1)?;
                self.emit_call_native(
                    |this| {
                        this.machine
                            .emit_call_register(this.machine.get_grp_for_call())
                    },
                    // [vmctx, table_index, elem_index] -> reftype
                    [Location::Imm32(table_index.index() as u32), index]
                        .iter()
                        .cloned(),
                    [WpType::I32, WpType::I64].iter().cloned(),
                )?;

                self.release_locations_only_stack(&[index])?;

                let ret = self.acquire_locations(
                    &[(
                        WpType::FuncRef,
                        MachineValue::WasmStack(self.value_stack.len()),
                    )],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.move_location(
                    Size::S64,
                    Location::GPR(self.machine.get_gpr_for_ret()),
                    ret,
                )?;
            }
            Operator::TableSize { table: index } => {
                let table_index = TableIndex::new(index as _);

                self.machine.move_location(
                    Size::S64,
                    Location::Memory(
                        self.machine.get_vmctx_reg(),
                        self.vmoffsets.vmctx_builtin_function(
                            if self.module.local_table_index(table_index).is_some() {
                                VMBuiltinFunctionIndex::get_table_size_index()
                            } else {
                                VMBuiltinFunctionIndex::get_imported_table_size_index()
                            },
                        ) as i32,
                    ),
                    Location::GPR(self.machine.get_grp_for_call()),
                )?;

                self.emit_call_native(
                    |this| {
                        this.machine
                            .emit_call_register(this.machine.get_grp_for_call())
                    },
                    // [vmctx, table_index] -> i32
                    iter::once(Location::Imm32(table_index.index() as u32)),
                    iter::once(WpType::I32),
                )?;

                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.move_location(
                    Size::S32,
                    Location::GPR(self.machine.get_gpr_for_ret()),
                    ret,
                )?;
            }
            Operator::TableGrow { table: index } => {
                let table_index = TableIndex::new(index as _);
                let delta = self.value_stack.pop().unwrap();
                let init_value = self.value_stack.pop().unwrap();
                self.release_locations_only_regs(&[delta, init_value])?;

                self.machine.move_location(
                    Size::S64,
                    Location::Memory(
                        self.machine.get_vmctx_reg(),
                        self.vmoffsets.vmctx_builtin_function(
                            if self.module.local_table_index(table_index).is_some() {
                                VMBuiltinFunctionIndex::get_table_grow_index()
                            } else {
                                VMBuiltinFunctionIndex::get_imported_table_get_index()
                            },
                        ) as i32,
                    ),
                    Location::GPR(self.machine.get_grp_for_call()),
                )?;

                // TODO: should this be 2?
                self.release_locations_only_osr_state(1)?;
                self.emit_call_native(
                    |this| {
                        this.machine
                            .emit_call_register(this.machine.get_grp_for_call())
                    },
                    // [vmctx, init_value, delta, table_index] -> u32
                    [
                        init_value,
                        delta,
                        Location::Imm32(table_index.index() as u32),
                    ]
                    .iter()
                    .cloned(),
                    [WpType::I64, WpType::I64, WpType::I64].iter().cloned(),
                )?;

                self.release_locations_only_stack(&[init_value, delta])?;

                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.move_location(
                    Size::S32,
                    Location::GPR(self.machine.get_gpr_for_ret()),
                    ret,
                )?;
            }
            Operator::TableCopy {
                dst_table,
                src_table,
            } => {
                let len = self.value_stack.pop().unwrap();
                let src = self.value_stack.pop().unwrap();
                let dest = self.value_stack.pop().unwrap();
                self.release_locations_only_regs(&[len, src, dest])?;

                self.machine.move_location(
                    Size::S64,
                    Location::Memory(
                        self.machine.get_vmctx_reg(),
                        self.vmoffsets
                            .vmctx_builtin_function(VMBuiltinFunctionIndex::get_table_copy_index())
                            as i32,
                    ),
                    Location::GPR(self.machine.get_grp_for_call()),
                )?;

                // TODO: should this be 3?
                self.release_locations_only_osr_state(1)?;
                self.emit_call_native(
                    |this| {
                        this.machine
                            .emit_call_register(this.machine.get_grp_for_call())
                    },
                    // [vmctx, dst_table_index, src_table_index, dst, src, len]
                    [
                        Location::Imm32(dst_table),
                        Location::Imm32(src_table),
                        dest,
                        src,
                        len,
                    ]
                    .iter()
                    .cloned(),
                    [
                        WpType::I32,
                        WpType::I32,
                        WpType::I64,
                        WpType::I64,
                        WpType::I64,
                    ]
                    .iter()
                    .cloned(),
                )?;

                self.release_locations_only_stack(&[dest, src, len])?;
            }

            Operator::TableFill { table } => {
                let len = self.value_stack.pop().unwrap();
                let val = self.value_stack.pop().unwrap();
                let dest = self.value_stack.pop().unwrap();
                self.release_locations_only_regs(&[len, val, dest])?;

                self.machine.move_location(
                    Size::S64,
                    Location::Memory(
                        self.machine.get_vmctx_reg(),
                        self.vmoffsets
                            .vmctx_builtin_function(VMBuiltinFunctionIndex::get_table_fill_index())
                            as i32,
                    ),
                    Location::GPR(self.machine.get_grp_for_call()),
                )?;

                // TODO: should this be 3?
                self.release_locations_only_osr_state(1)?;
                self.emit_call_native(
                    |this| {
                        this.machine
                            .emit_call_register(this.machine.get_grp_for_call())
                    },
                    // [vmctx, table_index, start_idx, item, len]
                    [Location::Imm32(table), dest, val, len].iter().cloned(),
                    [WpType::I32, WpType::I64, WpType::I64, WpType::I64]
                        .iter()
                        .cloned(),
                )?;

                self.release_locations_only_stack(&[dest, val, len])?;
            }
            Operator::TableInit { elem_index, table } => {
                let len = self.value_stack.pop().unwrap();
                let src = self.value_stack.pop().unwrap();
                let dest = self.value_stack.pop().unwrap();
                self.release_locations_only_regs(&[len, src, dest])?;

                self.machine.move_location(
                    Size::S64,
                    Location::Memory(
                        self.machine.get_vmctx_reg(),
                        self.vmoffsets
                            .vmctx_builtin_function(VMBuiltinFunctionIndex::get_table_init_index())
                            as i32,
                    ),
                    Location::GPR(self.machine.get_grp_for_call()),
                )?;

                // TODO: should this be 3?
                self.release_locations_only_osr_state(1)?;
                self.emit_call_native(
                    |this| {
                        this.machine
                            .emit_call_register(this.machine.get_grp_for_call())
                    },
                    // [vmctx, table_index, elem_index, dst, src, len]
                    [
                        Location::Imm32(table),
                        Location::Imm32(elem_index),
                        dest,
                        src,
                        len,
                    ]
                    .iter()
                    .cloned(),
                    [
                        WpType::I32,
                        WpType::I32,
                        WpType::I64,
                        WpType::I64,
                        WpType::I64,
                    ]
                    .iter()
                    .cloned(),
                )?;

                self.release_locations_only_stack(&[dest, src, len])?;
            }
            Operator::ElemDrop { elem_index } => {
                self.machine.move_location(
                    Size::S64,
                    Location::Memory(
                        self.machine.get_vmctx_reg(),
                        self.vmoffsets
                            .vmctx_builtin_function(VMBuiltinFunctionIndex::get_elem_drop_index())
                            as i32,
                    ),
                    Location::GPR(self.machine.get_grp_for_call()),
                )?;

                // TODO: do we need this?
                //.machine.release_locations_only_osr_state(1);
                self.emit_call_native(
                    |this| {
                        this.machine
                            .emit_call_register(this.machine.get_grp_for_call())
                    },
                    // [vmctx, elem_index]
                    [Location::Imm32(elem_index)].iter().cloned(),
                    [WpType::I32].iter().cloned(),
                )?;
            }
            Operator::MemoryAtomicWait32 { ref memarg } => {
                let timeout = self.value_stack.pop().unwrap();
                let val = self.value_stack.pop().unwrap();
                let dst = self.value_stack.pop().unwrap();
                self.release_locations_only_regs(&[timeout, val, dst])?;

                let memory_index = MemoryIndex::new(memarg.memory as usize);
                let (memory_atomic_wait32, memory_index) =
                    if self.module.local_memory_index(memory_index).is_some() {
                        (
                            VMBuiltinFunctionIndex::get_memory_atomic_wait32_index(),
                            memory_index,
                        )
                    } else {
                        (
                            VMBuiltinFunctionIndex::get_imported_memory_atomic_wait32_index(),
                            memory_index,
                        )
                    };

                self.machine.move_location(
                    Size::S64,
                    Location::Memory(
                        self.machine.get_vmctx_reg(),
                        self.vmoffsets.vmctx_builtin_function(memory_atomic_wait32) as i32,
                    ),
                    Location::GPR(self.machine.get_grp_for_call()),
                )?;

                // TODO: should this be 3?
                self.release_locations_only_osr_state(1)?;

                self.emit_call_native(
                    |this| {
                        this.machine
                            .emit_call_register(this.machine.get_grp_for_call())
                    },
                    // [vmctx, memory_index, dst, src, timeout]
                    [
                        Location::Imm32(memory_index.index() as u32),
                        dst,
                        val,
                        timeout,
                    ]
                    .iter()
                    .cloned(),
                    [WpType::I32, WpType::I32, WpType::I32, WpType::I64]
                        .iter()
                        .cloned(),
                )?;
                self.release_locations_only_stack(&[dst, val, timeout])?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.move_location(
                    Size::S32,
                    Location::GPR(self.machine.get_gpr_for_ret()),
                    ret,
                )?;
            }
            Operator::MemoryAtomicWait64 { ref memarg } => {
                let timeout = self.value_stack.pop().unwrap();
                let val = self.value_stack.pop().unwrap();
                let dst = self.value_stack.pop().unwrap();
                self.release_locations_only_regs(&[timeout, val, dst])?;

                let memory_index = MemoryIndex::new(memarg.memory as usize);
                let (memory_atomic_wait64, memory_index) =
                    if self.module.local_memory_index(memory_index).is_some() {
                        (
                            VMBuiltinFunctionIndex::get_memory_atomic_wait64_index(),
                            memory_index,
                        )
                    } else {
                        (
                            VMBuiltinFunctionIndex::get_imported_memory_atomic_wait64_index(),
                            memory_index,
                        )
                    };

                self.machine.move_location(
                    Size::S64,
                    Location::Memory(
                        self.machine.get_vmctx_reg(),
                        self.vmoffsets.vmctx_builtin_function(memory_atomic_wait64) as i32,
                    ),
                    Location::GPR(self.machine.get_grp_for_call()),
                )?;

                // TODO: should this be 3?
                self.release_locations_only_osr_state(1)?;

                self.emit_call_native(
                    |this| {
                        this.machine
                            .emit_call_register(this.machine.get_grp_for_call())
                    },
                    // [vmctx, memory_index, dst, src, timeout]
                    [
                        Location::Imm32(memory_index.index() as u32),
                        dst,
                        val,
                        timeout,
                    ]
                    .iter()
                    .cloned(),
                    [WpType::I32, WpType::I32, WpType::I64, WpType::I64]
                        .iter()
                        .cloned(),
                )?;
                self.release_locations_only_stack(&[dst, val, timeout])?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.move_location(
                    Size::S32,
                    Location::GPR(self.machine.get_gpr_for_ret()),
                    ret,
                )?;
            }
            Operator::MemoryAtomicNotify { ref memarg } => {
                let cnt = self.value_stack.pop().unwrap();
                let dst = self.value_stack.pop().unwrap();
                self.release_locations_only_regs(&[cnt, dst])?;

                let memory_index = MemoryIndex::new(memarg.memory as usize);
                let (memory_atomic_notify, memory_index) =
                    if self.module.local_memory_index(memory_index).is_some() {
                        (
                            VMBuiltinFunctionIndex::get_memory_atomic_notify_index(),
                            memory_index,
                        )
                    } else {
                        (
                            VMBuiltinFunctionIndex::get_imported_memory_atomic_notify_index(),
                            memory_index,
                        )
                    };

                self.machine.move_location(
                    Size::S64,
                    Location::Memory(
                        self.machine.get_vmctx_reg(),
                        self.vmoffsets.vmctx_builtin_function(memory_atomic_notify) as i32,
                    ),
                    Location::GPR(self.machine.get_grp_for_call()),
                )?;

                // TODO: should this be 3?
                self.release_locations_only_osr_state(1)?;

                self.emit_call_native(
                    |this| {
                        this.machine
                            .emit_call_register(this.machine.get_grp_for_call())
                    },
                    // [vmctx, memory_index, dst, src, timeout]
                    [Location::Imm32(memory_index.index() as u32), dst]
                        .iter()
                        .cloned(),
                    [WpType::I32, WpType::I32].iter().cloned(),
                )?;
                self.release_locations_only_stack(&[dst, cnt])?;
                let ret = self.acquire_locations(
                    &[(WpType::I32, MachineValue::WasmStack(self.value_stack.len()))],
                    false,
                )?[0];
                self.value_stack.push(ret);
                self.machine.move_location(
                    Size::S32,
                    Location::GPR(self.machine.get_gpr_for_ret()),
                    ret,
                )?;
            }
            _ => {
                return Err(CompileError::Codegen(format!(
                    "not yet implemented: {:?}",
                    op
                )));
            }
        }

        Ok(())
    }

    pub fn finalize(
        mut self,
        data: &FunctionBodyData,
    ) -> Result<(CompiledFunction, Option<UnwindFrame>), CompileError> {
        // Generate actual code for special labels.
        self.machine
            .emit_label(self.special_labels.integer_division_by_zero)?;
        self.machine
            .emit_illegal_op(TrapCode::IntegerDivisionByZero)?;

        self.machine
            .emit_label(self.special_labels.integer_overflow)?;
        self.machine.emit_illegal_op(TrapCode::IntegerOverflow)?;

        self.machine
            .emit_label(self.special_labels.heap_access_oob)?;
        self.machine
            .emit_illegal_op(TrapCode::HeapAccessOutOfBounds)?;

        self.machine
            .emit_label(self.special_labels.table_access_oob)?;
        self.machine
            .emit_illegal_op(TrapCode::TableAccessOutOfBounds)?;

        self.machine
            .emit_label(self.special_labels.indirect_call_null)?;
        self.machine.emit_illegal_op(TrapCode::IndirectCallToNull)?;

        self.machine.emit_label(self.special_labels.bad_signature)?;
        self.machine.emit_illegal_op(TrapCode::BadSignature)?;

        self.machine
            .emit_label(self.special_labels.unaligned_atomic)?;
        self.machine.emit_illegal_op(TrapCode::UnalignedAtomic)?;

        // Notify the assembler backend to generate necessary code at end of function.
        self.machine.finalize_function()?;

        let body_len = self.machine.assembler_get_offset().0;

        let mut unwind_info = None;
        let mut fde = None;
        #[cfg(feature = "unwind")]
        match self.calling_convention {
            CallingConvention::SystemV | CallingConvention::AppleAarch64 => {
                let unwind = self.machine.gen_dwarf_unwind_info(body_len);
                if let Some(unwind) = unwind {
                    fde = Some(unwind.to_fde(Address::Symbol {
                        symbol: WriterRelocate::FUNCTION_SYMBOL,
                        addend: self.fsm.local_function_id as _,
                    }));
                    unwind_info = Some(CompiledFunctionUnwindInfo::Dwarf);
                }
            }
            CallingConvention::WindowsFastcall => {
                let unwind = self.machine.gen_windows_unwind_info(body_len);
                if let Some(unwind) = unwind {
                    unwind_info = Some(CompiledFunctionUnwindInfo::WindowsX64(unwind));
                }
            }
            _ => (),
        };

        let address_map =
            get_function_address_map(self.machine.instructions_address_map(), data, body_len);
        let traps = self.machine.collect_trap_information();
        let mut body = self.machine.assembler_finalize();
        body.shrink_to_fit();

        Ok((
            CompiledFunction {
                body: FunctionBody { body, unwind_info },
                relocations: self.relocations.clone(),
                frame_info: CompiledFunctionFrameInfo { traps, address_map },
            },
            fde,
        ))
    }
    // FIXME: This implementation seems to be not enough to resolve all kinds of register dependencies
    // at call place.
    #[allow(clippy::type_complexity)]
    fn sort_call_movs(movs: &mut [(Location<M::GPR, M::SIMD>, M::GPR)]) {
        for i in 0..movs.len() {
            for j in (i + 1)..movs.len() {
                if let Location::GPR(src_gpr) = movs[j].0 {
                    if src_gpr == movs[i].1 {
                        movs.swap(i, j);
                    }
                }
            }
        }
    }

    // Cycle detector. Uncomment this to debug possibly incorrect call-mov sequences.
    /*
    {
        use std::collections::{HashMap, HashSet, VecDeque};
        let mut mov_map: HashMap<GPR, HashSet<GPR>> = HashMap::new();
        for mov in movs.iter() {
            if let Location::GPR(src_gpr) = mov.0 {
                if src_gpr != mov.1 {
                    mov_map.entry(src_gpr).or_insert_with(|| HashSet::new()).insert(mov.1);
                }
            }
        }

        for (start, _) in mov_map.iter() {
            let mut q: VecDeque<GPR> = VecDeque::new();
            let mut black: HashSet<GPR> = HashSet::new();

            q.push_back(*start);
            black.insert(*start);

            while q.len() > 0 {
                let reg = q.pop_front().unwrap();
                let empty_set = HashSet::new();
                for x in mov_map.get(&reg).unwrap_or(&empty_set).iter() {
                    if black.contains(x) {
                        panic!("cycle detected");
                    }
                    q.push_back(*x);
                    black.insert(*x);
                }
            }
        }
    }
    */
}
