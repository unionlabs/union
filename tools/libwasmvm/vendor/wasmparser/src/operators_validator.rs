/* Copyright 2019 Mozilla Foundation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

// The basic validation algorithm here is copied from the "Validation
// Algorithm" section of the WebAssembly specification -
// https://webassembly.github.io/spec/core/appendix/algorithm.html.
//
// That algorithm is followed pretty closely here, namely `push_operand`,
// `pop_operand`, `push_ctrl`, and `pop_ctrl`. If anything here is a bit
// confusing it's recomended to read over that section to see how it maps to
// the various methods here.

use crate::limits::MAX_WASM_FUNCTION_LOCALS;
use crate::primitives::{MemoryImmediate, Operator, SIMDLaneIndex, Type, TypeOrFuncType};
use crate::{BinaryReaderError, Result, WasmFeatures, WasmFuncType, WasmModuleResources};

/// A wrapper around a `BinaryReaderError` where the inner error's offset is a
/// temporary placeholder value. This can be converted into a proper
/// `BinaryReaderError` via the `set_offset` method, which replaces the
/// placeholder offset with an actual offset.
pub(crate) struct OperatorValidatorError(pub(crate) BinaryReaderError);

/// Create an `OperatorValidatorError` with a format string.
macro_rules! format_op_err {
    ( $( $arg:expr ),* $(,)* ) => {
        OperatorValidatorError::new(format!( $( $arg ),* ))
    }
}

/// Early return an `Err(OperatorValidatorError)` with a format string.
macro_rules! bail_op_err {
    ( $( $arg:expr ),* $(,)* ) => {
        return Err(format_op_err!( $( $arg ),* ))
    }
}

impl OperatorValidatorError {
    /// Create a new `OperatorValidatorError` with a placeholder offset.
    pub(crate) fn new(message: impl Into<String>) -> Self {
        let offset = std::usize::MAX;
        let e = BinaryReaderError::new(message, offset);
        OperatorValidatorError(e)
    }

    /// Convert this `OperatorValidatorError` into a `BinaryReaderError` by
    /// supplying an actual offset to replace the internal placeholder offset.
    pub(crate) fn set_offset(mut self, offset: usize) -> BinaryReaderError {
        debug_assert_eq!(self.0.inner.offset, std::usize::MAX);
        self.0.inner.offset = offset;
        self.0
    }
}

type OperatorValidatorResult<T> = std::result::Result<T, OperatorValidatorError>;

pub(crate) struct OperatorValidator {
    // The total number of locals that this function contains
    num_locals: u32,
    // This is a "compressed" list of locals for this function. The list of
    // locals are represented as a list of tuples. The second element is the
    // type of the local, and the first element is monotonically increasing as
    // you visit elements of this list. The first element is the maximum index
    // of the local, after the previous index, of the type specified.
    //
    // This allows us to do a binary search on the list for a local's index for
    // `local.{get,set,tee}`. We do a binary search for the index desired, and
    // it either lies in a "hole" where the maximum index is specified later,
    // or it's at the end of the list meaning it's out of bounds.
    locals: Vec<(u32, Type)>,

    // The `operands` is the current type stack, and the `control` list is the
    // list of blocks that we're currently in.
    pub(crate) operands: Vec<Option<Type>>,
    control: Vec<Frame>,

    // This is a list of flags for wasm features which are used to gate various
    // instructions.
    pub(crate) features: WasmFeatures,

    // Temporary storage used during the validation of `br_table`.
    br_table_tmp: Vec<Option<Type>>,
}

// This structure corresponds to `ctrl_frame` as specified at in the validation
// appendix of the wasm spec
struct Frame {
    // Indicator for what kind of instruction pushed this frame.
    kind: FrameKind,
    // The type signature of this frame, represented as a singular return type
    // or a type index pointing into the module's types.
    block_type: TypeOrFuncType,
    // The index, below which, this frame cannot modify the operand stack.
    height: usize,
    // Whether this frame is unreachable so far.
    unreachable: bool,
}

#[derive(PartialEq, Copy, Clone)]
enum FrameKind {
    Block,
    If,
    Else,
    Loop,
    Try,
    Catch,
    CatchAll,
}

impl OperatorValidator {
    /// Creates a new operator validator which will be used to validate a
    /// function whose type is the `ty` index specified.
    ///
    /// The `resources` are used to learn about the function type underlying
    /// `ty`.
    pub fn new_func(
        ty: u32,
        offset: usize,
        features: &WasmFeatures,
        resources: &impl WasmModuleResources,
    ) -> Result<OperatorValidator> {
        let locals = func_type_at(resources, ty)
            .map_err(|e| e.set_offset(offset))?
            .inputs()
            .enumerate()
            .map(|(i, ty)| (i as u32, ty))
            .collect::<Vec<_>>();
        Ok(OperatorValidator {
            num_locals: locals.len() as u32,
            locals,
            operands: Vec::new(),
            control: vec![Frame {
                kind: FrameKind::Block,
                block_type: TypeOrFuncType::FuncType(ty),
                height: 0,
                unreachable: false,
            }],
            features: *features,
            br_table_tmp: Vec::new(),
        })
    }

    /// Creates a new operator validator which will be used to validate an
    /// `init_expr` constant expression which should result in the `ty`
    /// specified.
    pub fn new_init_expr(features: &WasmFeatures, ty: Type) -> OperatorValidator {
        OperatorValidator {
            num_locals: 0,
            locals: Vec::new(),
            operands: Vec::new(),
            control: vec![Frame {
                kind: FrameKind::Block,
                block_type: TypeOrFuncType::Type(ty),
                height: 0,
                unreachable: false,
            }],
            features: *features,
            br_table_tmp: Vec::new(),
        }
    }

    pub fn define_locals(&mut self, offset: usize, count: u32, ty: Type) -> Result<()> {
        self.features
            .check_value_type(ty)
            .map_err(|e| BinaryReaderError::new(e, offset))?;
        if count == 0 {
            return Ok(());
        }
        match self.num_locals.checked_add(count) {
            Some(n) => self.num_locals = n,
            None => return Err(BinaryReaderError::new("locals overflow", offset)),
        }
        if self.num_locals > (MAX_WASM_FUNCTION_LOCALS as u32) {
            return Err(BinaryReaderError::new("locals exceed maximum", offset));
        }
        self.locals.push((self.num_locals - 1, ty));
        Ok(())
    }

    /// Fetches the type for the local at `idx`, returning an error if it's out
    /// of bounds.
    fn local(&self, idx: u32) -> OperatorValidatorResult<Type> {
        match self.locals.binary_search_by_key(&idx, |(idx, _)| *idx) {
            // If this index would be inserted at the end of the list, then the
            // index is out of bounds and we return an error.
            Err(i) if i == self.locals.len() => {
                bail_op_err!("unknown local {}: local index out of bounds", idx)
            }
            // If `Ok` is returned we found the index exactly, or if `Err` is
            // returned the position is the one which is the least index
            // greater thatn `idx`, which is still the type of `idx` according
            // to our "compressed" representation. In both cases we access the
            // list at index `i`.
            Ok(i) | Err(i) => Ok(self.locals[i].1),
        }
    }

    /// Pushes a type onto the operand stack.
    ///
    /// This is used by instructions to represent a value that is pushed to the
    /// operand stack. This can fail, but only if `Type` is feature gated.
    /// Otherwise the push operation always succeeds.
    fn push_operand(&mut self, ty: Type) -> OperatorValidatorResult<()> {
        self.features
            .check_value_type(ty)
            .map_err(OperatorValidatorError::new)?;
        self.operands.push(Some(ty));
        Ok(())
    }

    /// Attempts to pop a type from the operand stack.
    ///
    /// This function is used to remove types from the operand stack. The
    /// `expected` argument can be used to indicate that a type is required, or
    /// simply that something is needed to be popped.
    ///
    /// If `expected` is `Some(T)` then this will be guaranteed to return
    /// `Some(T)`, and it will only return success if the current block is
    /// unreachable or if `T` was found at the top of the operand stack.
    ///
    /// If `expected` is `None` then it indicates that something must be on the
    /// operand stack, but it doesn't matter what's on the operand stack. This
    /// is useful for polymorphic instructions like `select`.
    ///
    /// If `Some(T)` is returned then `T` was popped from the operand stack and
    /// matches `expected`. If `None` is returned then it means that `None` was
    /// expected and a type was successfully popped, but its exact type is
    /// indeterminate because the current block is unreachable.
    fn pop_operand(&mut self, expected: Option<Type>) -> OperatorValidatorResult<Option<Type>> {
        let control = self.control.last().unwrap();
        let actual = if self.operands.len() == control.height {
            if control.unreachable {
                None
            } else {
                let desc = match expected {
                    Some(ty) => ty_to_str(ty),
                    None => "a type",
                };
                bail_op_err!("type mismatch: expected {} but nothing on stack", desc)
            }
        } else {
            self.operands.pop().unwrap()
        };
        if let (Some(actual_ty), Some(expected_ty)) = (actual, expected) {
            if actual_ty != expected_ty {
                bail_op_err!(
                    "type mismatch: expected {}, found {}",
                    ty_to_str(expected_ty),
                    ty_to_str(actual_ty)
                )
            }
        }
        Ok(actual)
    }

    /// Flags the current control frame as unreachable, additionally truncating
    /// the currently active operand stack.
    fn unreachable(&mut self) {
        let control = self.control.last_mut().unwrap();
        self.operands.truncate(control.height);
        control.unreachable = true;
    }

    /// Pushes a new frame onto the control stack.
    ///
    /// This operation is used when entering a new block such as an if, loop,
    /// or block itself. The `kind` of block is specified which indicates how
    /// breaks interact with this block's type. Additionally the type signature
    /// of the block is specified by `ty`.
    fn push_ctrl(
        &mut self,
        kind: FrameKind,
        ty: TypeOrFuncType,
        resources: &impl WasmModuleResources,
    ) -> OperatorValidatorResult<()> {
        // Push a new frame which has a snapshot of the height of the current
        // operand stack.
        self.control.push(Frame {
            kind,
            block_type: ty,
            height: self.operands.len(),
            unreachable: false,
        });
        // All of the parameters are now also available in this control frame,
        // so we push them here in order.
        for ty in params(ty, resources)? {
            self.push_operand(ty)?;
        }
        Ok(())
    }

    /// Pops a frame from the control stack.
    ///
    /// This function is used when exiting a block and leaves a block scope.
    /// Internally this will validate that blocks have the correct result type.
    fn pop_ctrl(&mut self, resources: &impl WasmModuleResources) -> OperatorValidatorResult<Frame> {
        // Read the expected type and expected height of the operand stack the
        // end of the frame.
        let frame = self.control.last().unwrap();
        let ty = frame.block_type;
        let height = frame.height;

        // Pop all the result types, in reverse order, from the operand stack.
        // These types will, possibly, be transferred to the next frame.
        for ty in results(ty, resources)?.rev() {
            self.pop_operand(Some(ty))?;
        }

        // Make sure that the operand stack has returned to is original
        // height...
        if self.operands.len() != height {
            bail_op_err!("type mismatch: values remaining on stack at end of block");
        }

        // And then we can remove it!
        Ok(self.control.pop().unwrap())
    }

    /// Validates a relative jump to the `depth` specified.
    ///
    /// Returns the type signature of the block that we're jumping to as well
    /// as the kind of block if the jump is valid. Otherwise returns an error.
    fn jump(&self, depth: u32) -> OperatorValidatorResult<(TypeOrFuncType, FrameKind)> {
        match (self.control.len() - 1).checked_sub(depth as usize) {
            Some(i) => {
                let frame = &self.control[i];
                Ok((frame.block_type, frame.kind))
            }
            None => bail_op_err!("unknown label: branch depth too large"),
        }
    }

    /// Validates that `memory_index` is valid in this module, and returns the
    /// type of address used to index the memory specified.
    fn check_memory_index(
        &self,
        memory_index: u32,
        resources: impl WasmModuleResources,
    ) -> OperatorValidatorResult<Type> {
        if memory_index > 0 && !self.features.multi_memory {
            return Err(OperatorValidatorError::new(
                "multi-memory support is not enabled",
            ));
        }
        match resources.memory_at(memory_index) {
            Some(mem) => Ok(mem.index_type()),
            None => bail_op_err!("unknown memory {}", memory_index),
        }
    }

    /// Validates a `memarg for alignment and such (also the memory it
    /// references), and returns the type of index used to address the memory.
    fn check_memarg(
        &self,
        memarg: MemoryImmediate,
        max_align: u8,
        resources: impl WasmModuleResources,
    ) -> OperatorValidatorResult<Type> {
        let index_ty = self.check_memory_index(memarg.memory, resources)?;
        let align = memarg.align;
        if align > max_align {
            return Err(OperatorValidatorError::new(
                "alignment must not be larger than natural",
            ));
        }
        if index_ty == Type::I32 && memarg.offset > u64::from(u32::MAX) {
            return Err(OperatorValidatorError::new(
                "offset out of range: must be <= 2**32",
            ));
        }
        Ok(index_ty)
    }

    #[cfg(feature = "deterministic")]
    fn check_non_deterministic_enabled(&self) -> OperatorValidatorResult<()> {
        if !self.features.deterministic_only {
            return Err(OperatorValidatorError::new(
                "deterministic_only support is not enabled",
            ));
        }
        Ok(())
    }

    #[inline(always)]
    #[cfg(not(feature = "deterministic"))]
    fn check_non_deterministic_enabled(&self) -> OperatorValidatorResult<()> {
        Ok(())
    }

    fn check_threads_enabled(&self) -> OperatorValidatorResult<()> {
        if !self.features.threads {
            return Err(OperatorValidatorError::new(
                "threads support is not enabled",
            ));
        }
        Ok(())
    }

    fn check_reference_types_enabled(&self) -> OperatorValidatorResult<()> {
        if !self.features.reference_types {
            return Err(OperatorValidatorError::new(
                "reference types support is not enabled",
            ));
        }
        Ok(())
    }

    fn check_simd_enabled(&self) -> OperatorValidatorResult<()> {
        if !self.features.simd {
            return Err(OperatorValidatorError::new("SIMD support is not enabled"));
        }
        Ok(())
    }

    fn check_relaxed_simd_enabled(&self) -> OperatorValidatorResult<()> {
        // Relaxed SIMD operators make sense only with SIMD and be non-deterministic.
        self.check_non_deterministic_enabled()?;
        self.check_simd_enabled()?;
        if !self.features.relaxed_simd {
            return Err(OperatorValidatorError::new(
                "Relaxed SIMD support is not enabled",
            ));
        }
        Ok(())
    }

    fn check_exceptions_enabled(&self) -> OperatorValidatorResult<()> {
        if !self.features.exceptions {
            return Err(OperatorValidatorError::new(
                "Exceptions support is not enabled",
            ));
        }
        Ok(())
    }

    fn check_bulk_memory_enabled(&self) -> OperatorValidatorResult<()> {
        if !self.features.bulk_memory {
            return Err(OperatorValidatorError::new(
                "bulk memory support is not enabled",
            ));
        }
        Ok(())
    }

    fn check_shared_memarg_wo_align(
        &self,
        memarg: MemoryImmediate,
        resources: impl WasmModuleResources,
    ) -> OperatorValidatorResult<Type> {
        self.check_memory_index(memarg.memory, resources)
    }

    fn check_simd_lane_index(&self, index: SIMDLaneIndex, max: u8) -> OperatorValidatorResult<()> {
        if index >= max {
            return Err(OperatorValidatorError::new("SIMD index out of bounds"));
        }
        Ok(())
    }

    /// Validates a block type, primarily with various in-flight proposals.
    fn check_block_type(
        &self,
        ty: TypeOrFuncType,
        resources: impl WasmModuleResources,
    ) -> OperatorValidatorResult<()> {
        match ty {
            TypeOrFuncType::Type(Type::EmptyBlockType)
            | TypeOrFuncType::Type(Type::I32)
            | TypeOrFuncType::Type(Type::I64)
            | TypeOrFuncType::Type(Type::F32)
            | TypeOrFuncType::Type(Type::F64) => Ok(()),
            TypeOrFuncType::Type(Type::ExternRef) | TypeOrFuncType::Type(Type::FuncRef) => {
                self.check_reference_types_enabled()
            }
            TypeOrFuncType::Type(Type::V128) => self.check_simd_enabled(),
            TypeOrFuncType::FuncType(_) if !self.features.multi_value => {
                return Err(OperatorValidatorError::new(
                    "blocks, loops, and ifs may only produce a resulttype when multi-value is not \
                    enabled",
                ));
            }

            TypeOrFuncType::FuncType(idx) => func_type_at(&resources, idx).map(|_| ()),
            _ => Err(OperatorValidatorError::new("invalid block return type")),
        }
    }

    /// Validates a `call` instruction, ensuring that the function index is
    /// in-bounds and the right types are on the stack to call the function.
    fn check_call(
        &mut self,
        function_index: u32,
        resources: &impl WasmModuleResources,
    ) -> OperatorValidatorResult<()> {
        let ty = match resources.type_of_function(function_index) {
            Some(i) => i,
            None => {
                bail_op_err!(
                    "unknown function {}: function index out of bounds",
                    function_index
                );
            }
        };
        for ty in ty.inputs().rev() {
            self.pop_operand(Some(ty))?;
        }
        for ty in ty.outputs() {
            self.push_operand(ty)?;
        }
        Ok(())
    }

    /// Validates a call to an indirect function, very similar to `check_call`.
    fn check_call_indirect(
        &mut self,
        index: u32,
        table_index: u32,
        resources: &impl WasmModuleResources,
    ) -> OperatorValidatorResult<()> {
        match resources.table_at(table_index) {
            None => {
                return Err(OperatorValidatorError::new(
                    "unknown table: table index out of bounds",
                ));
            }
            Some(tab) => {
                if tab.element_type != Type::FuncRef {
                    return Err(OperatorValidatorError::new(
                        "indirect calls must go through a table of funcref",
                    ));
                }
            }
        }
        let ty = func_type_at(&resources, index)?;
        self.pop_operand(Some(Type::I32))?;
        for ty in ty.inputs().rev() {
            self.pop_operand(Some(ty))?;
        }
        for ty in ty.outputs() {
            self.push_operand(ty)?;
        }
        Ok(())
    }

    /// Validates a `return` instruction, popping types from the operand
    /// stack that the function needs.
    fn check_return(
        &mut self,
        resources: &impl WasmModuleResources,
    ) -> OperatorValidatorResult<()> {
        for ty in results(self.control[0].block_type, resources)?.rev() {
            self.pop_operand(Some(ty))?;
        }
        self.unreachable();
        Ok(())
    }

    pub fn process_operator(
        &mut self,
        operator: &Operator,
        resources: &impl WasmModuleResources,
    ) -> OperatorValidatorResult<()> {
        if self.control.is_empty() {
            bail_op_err!("operators remaining after end of function");
        }
        match *operator {
            Operator::Nop => {}

            // Note that the handling of these control flow operators are the
            // same as specified in the "Validation Algorithm" appendix of the
            // online wasm specification (referenced at the top of this module).
            Operator::Unreachable => self.unreachable(),
            Operator::Block { ty } => {
                self.check_block_type(ty, resources)?;
                for ty in params(ty, resources)?.rev() {
                    self.pop_operand(Some(ty))?;
                }
                self.push_ctrl(FrameKind::Block, ty, resources)?;
            }
            Operator::Loop { ty } => {
                self.check_block_type(ty, resources)?;
                for ty in params(ty, resources)?.rev() {
                    self.pop_operand(Some(ty))?;
                }
                self.push_ctrl(FrameKind::Loop, ty, resources)?;
            }
            Operator::If { ty } => {
                self.check_block_type(ty, resources)?;
                self.pop_operand(Some(Type::I32))?;
                for ty in params(ty, resources)?.rev() {
                    self.pop_operand(Some(ty))?;
                }
                self.push_ctrl(FrameKind::If, ty, resources)?;
            }
            Operator::Else => {
                let frame = self.pop_ctrl(resources)?;
                if frame.kind != FrameKind::If {
                    bail_op_err!("else found outside of an `if` block");
                }
                self.push_ctrl(FrameKind::Else, frame.block_type, resources)?
            }
            Operator::Try { ty } => {
                self.check_exceptions_enabled()?;
                self.check_block_type(ty, resources)?;
                for ty in params(ty, resources)?.rev() {
                    self.pop_operand(Some(ty))?;
                }
                self.push_ctrl(FrameKind::Try, ty, resources)?;
            }
            Operator::Catch { index } => {
                self.check_exceptions_enabled()?;
                let frame = self.pop_ctrl(resources)?;
                if frame.kind != FrameKind::Try && frame.kind != FrameKind::Catch {
                    bail_op_err!("catch found outside of an `try` block");
                }
                // Start a new frame and push `exnref` value.
                self.control.push(Frame {
                    kind: FrameKind::Catch,
                    block_type: frame.block_type,
                    height: self.operands.len(),
                    unreachable: false,
                });
                // Push exception argument types.
                let ty = tag_at(&resources, index)?;
                for ty in ty.inputs() {
                    self.push_operand(ty)?;
                }
            }
            Operator::Throw { index } => {
                self.check_exceptions_enabled()?;
                // Check values associated with the exception.
                let ty = tag_at(&resources, index)?;
                for ty in ty.inputs().rev() {
                    self.pop_operand(Some(ty))?;
                }
                if ty.outputs().len() > 0 {
                    bail_op_err!("result type expected to be empty for exception");
                }
                self.unreachable();
            }
            Operator::Rethrow { relative_depth } => {
                self.check_exceptions_enabled()?;
                // This is not a jump, but we need to check that the `rethrow`
                // targets an actual `catch` to get the exception.
                let (_, kind) = self.jump(relative_depth)?;
                if kind != FrameKind::Catch && kind != FrameKind::CatchAll {
                    bail_op_err!("invalid rethrow label: target was not a `catch` block");
                }
                self.unreachable();
            }
            Operator::Delegate { relative_depth } => {
                self.check_exceptions_enabled()?;
                let frame = self.pop_ctrl(resources)?;
                if frame.kind != FrameKind::Try {
                    bail_op_err!("delegate found outside of an `try` block");
                }
                // This operation is not a jump, but we need to check the
                // depth for validity
                let _ = self.jump(relative_depth)?;
                for ty in results(frame.block_type, resources)? {
                    self.push_operand(ty)?;
                }
            }
            Operator::CatchAll => {
                self.check_exceptions_enabled()?;
                let frame = self.pop_ctrl(resources)?;
                if frame.kind == FrameKind::CatchAll {
                    bail_op_err!("only one catch_all allowed per `try` block");
                } else if frame.kind != FrameKind::Try && frame.kind != FrameKind::Catch {
                    bail_op_err!("catch_all found outside of a `try` block");
                }
                self.control.push(Frame {
                    kind: FrameKind::CatchAll,
                    block_type: frame.block_type,
                    height: self.operands.len(),
                    unreachable: false,
                });
            }
            Operator::End => {
                let mut frame = self.pop_ctrl(resources)?;

                // Note that this `if` isn't included in the appendix right
                // now, but it's used to allow for `if` statements that are
                // missing an `else` block which have the same parameter/return
                // types on the block (since that's valid).
                if frame.kind == FrameKind::If {
                    self.push_ctrl(FrameKind::Else, frame.block_type, resources)?;
                    frame = self.pop_ctrl(resources)?;
                }
                for ty in results(frame.block_type, resources)? {
                    self.push_operand(ty)?;
                }
            }
            Operator::Br { relative_depth } => {
                let (ty, kind) = self.jump(relative_depth)?;
                for ty in label_types(ty, resources, kind)?.rev() {
                    self.pop_operand(Some(ty))?;
                }
                self.unreachable();
            }
            Operator::BrIf { relative_depth } => {
                self.pop_operand(Some(Type::I32))?;
                let (ty, kind) = self.jump(relative_depth)?;
                for ty in label_types(ty, resources, kind)?.rev() {
                    self.pop_operand(Some(ty))?;
                }
                for ty in label_types(ty, resources, kind)? {
                    self.push_operand(ty)?;
                }
            }
            Operator::BrTable { ref table } => {
                self.pop_operand(Some(Type::I32))?;
                let default = self.jump(table.default())?;
                let default_types = label_types(default.0, resources, default.1)?;
                for element in table.targets() {
                    let relative_depth = element.map_err(|mut e| {
                        e.inner.offset = usize::max_value();
                        OperatorValidatorError(e)
                    })?;
                    let block = self.jump(relative_depth)?;
                    let tys = label_types(block.0, resources, block.1)?;
                    if tys.len() != default_types.len() {
                        bail_op_err!(
                            "type mismatch: br_table target labels have different number of types"
                        );
                    }
                    debug_assert!(self.br_table_tmp.is_empty());
                    for ty in tys.rev() {
                        let ty = self.pop_operand(Some(ty))?;
                        self.br_table_tmp.push(ty);
                    }
                    self.operands.extend(self.br_table_tmp.drain(..).rev());
                }
                for ty in default_types.rev() {
                    self.pop_operand(Some(ty))?;
                }
                self.unreachable();
            }
            Operator::Return => self.check_return(resources)?,
            Operator::Call { function_index } => self.check_call(function_index, resources)?,
            Operator::ReturnCall { function_index } => {
                if !self.features.tail_call {
                    return Err(OperatorValidatorError::new(
                        "tail calls support is not enabled",
                    ));
                }
                self.check_call(function_index, resources)?;
                self.check_return(resources)?;
            }
            Operator::CallIndirect { index, table_index } => {
                self.check_call_indirect(index, table_index, resources)?
            }
            Operator::ReturnCallIndirect { index, table_index } => {
                if !self.features.tail_call {
                    return Err(OperatorValidatorError::new(
                        "tail calls support is not enabled",
                    ));
                }
                self.check_call_indirect(index, table_index, resources)?;
                self.check_return(resources)?;
            }
            Operator::Drop => {
                self.pop_operand(None)?;
            }
            Operator::Select => {
                self.pop_operand(Some(Type::I32))?;
                let ty1 = self.pop_operand(None)?;
                let ty2 = self.pop_operand(None)?;
                fn is_num(ty: Option<Type>) -> bool {
                    matches!(
                        ty,
                        Some(Type::I32)
                            | Some(Type::I64)
                            | Some(Type::F32)
                            | Some(Type::F64)
                            | Some(Type::V128)
                            | None
                    )
                }
                if !is_num(ty1) || !is_num(ty2) {
                    bail_op_err!("type mismatch: select only takes integral types")
                }
                if ty1 != ty2 && ty1 != None && ty2 != None {
                    bail_op_err!("type mismatch: select operands have different types")
                }
                self.operands.push(ty1.or(ty2));
            }
            Operator::TypedSelect { ty } => {
                self.pop_operand(Some(Type::I32))?;
                self.pop_operand(Some(ty))?;
                self.pop_operand(Some(ty))?;
                self.push_operand(ty)?;
            }
            Operator::LocalGet { local_index } => {
                let ty = self.local(local_index)?;
                self.push_operand(ty)?;
            }
            Operator::LocalSet { local_index } => {
                let ty = self.local(local_index)?;
                self.pop_operand(Some(ty))?;
            }
            Operator::LocalTee { local_index } => {
                let ty = self.local(local_index)?;
                self.pop_operand(Some(ty))?;
                self.push_operand(ty)?;
            }
            Operator::GlobalGet { global_index } => {
                if let Some(ty) = resources.global_at(global_index) {
                    self.push_operand(ty.content_type)?;
                } else {
                    return Err(OperatorValidatorError::new(
                        "unknown global: global index out of bounds",
                    ));
                };
            }
            Operator::GlobalSet { global_index } => {
                if let Some(ty) = resources.global_at(global_index) {
                    if !ty.mutable {
                        return Err(OperatorValidatorError::new(
                            "global is immutable: cannot modify it with `global.set`",
                        ));
                    }
                    self.pop_operand(Some(ty.content_type))?;
                } else {
                    return Err(OperatorValidatorError::new(
                        "unknown global: global index out of bounds",
                    ));
                };
            }
            Operator::I32Load { memarg } => {
                let ty = self.check_memarg(memarg, 2, resources)?;
                self.pop_operand(Some(ty))?;
                self.push_operand(Type::I32)?;
            }
            Operator::I64Load { memarg } => {
                let ty = self.check_memarg(memarg, 3, resources)?;
                self.pop_operand(Some(ty))?;
                self.push_operand(Type::I64)?;
            }
            Operator::F32Load { memarg } => {
                self.check_non_deterministic_enabled()?;
                let ty = self.check_memarg(memarg, 2, resources)?;
                self.pop_operand(Some(ty))?;
                self.push_operand(Type::F32)?;
            }
            Operator::F64Load { memarg } => {
                self.check_non_deterministic_enabled()?;
                let ty = self.check_memarg(memarg, 3, resources)?;
                self.pop_operand(Some(ty))?;
                self.push_operand(Type::F64)?;
            }
            Operator::I32Load8S { memarg } | Operator::I32Load8U { memarg } => {
                let ty = self.check_memarg(memarg, 0, resources)?;
                self.pop_operand(Some(ty))?;
                self.push_operand(Type::I32)?;
            }
            Operator::I32Load16S { memarg } | Operator::I32Load16U { memarg } => {
                let ty = self.check_memarg(memarg, 1, resources)?;
                self.pop_operand(Some(ty))?;
                self.push_operand(Type::I32)?;
            }
            Operator::I64Load8S { memarg } | Operator::I64Load8U { memarg } => {
                let ty = self.check_memarg(memarg, 0, resources)?;
                self.pop_operand(Some(ty))?;
                self.push_operand(Type::I64)?;
            }
            Operator::I64Load16S { memarg } | Operator::I64Load16U { memarg } => {
                let ty = self.check_memarg(memarg, 1, resources)?;
                self.pop_operand(Some(ty))?;
                self.push_operand(Type::I64)?;
            }
            Operator::I64Load32S { memarg } | Operator::I64Load32U { memarg } => {
                let ty = self.check_memarg(memarg, 2, resources)?;
                self.pop_operand(Some(ty))?;
                self.push_operand(Type::I64)?;
            }
            Operator::I32Store { memarg } => {
                let ty = self.check_memarg(memarg, 2, resources)?;
                self.pop_operand(Some(Type::I32))?;
                self.pop_operand(Some(ty))?;
            }
            Operator::I64Store { memarg } => {
                let ty = self.check_memarg(memarg, 3, resources)?;
                self.pop_operand(Some(Type::I64))?;
                self.pop_operand(Some(ty))?;
            }
            Operator::F32Store { memarg } => {
                self.check_non_deterministic_enabled()?;
                let ty = self.check_memarg(memarg, 2, resources)?;
                self.pop_operand(Some(Type::F32))?;
                self.pop_operand(Some(ty))?;
            }
            Operator::F64Store { memarg } => {
                self.check_non_deterministic_enabled()?;
                let ty = self.check_memarg(memarg, 3, resources)?;
                self.pop_operand(Some(Type::F64))?;
                self.pop_operand(Some(ty))?;
            }
            Operator::I32Store8 { memarg } => {
                let ty = self.check_memarg(memarg, 0, resources)?;
                self.pop_operand(Some(Type::I32))?;
                self.pop_operand(Some(ty))?;
            }
            Operator::I32Store16 { memarg } => {
                let ty = self.check_memarg(memarg, 1, resources)?;
                self.pop_operand(Some(Type::I32))?;
                self.pop_operand(Some(ty))?;
            }
            Operator::I64Store8 { memarg } => {
                let ty = self.check_memarg(memarg, 0, resources)?;
                self.pop_operand(Some(Type::I64))?;
                self.pop_operand(Some(ty))?;
            }
            Operator::I64Store16 { memarg } => {
                let ty = self.check_memarg(memarg, 1, resources)?;
                self.pop_operand(Some(Type::I64))?;
                self.pop_operand(Some(ty))?;
            }
            Operator::I64Store32 { memarg } => {
                let ty = self.check_memarg(memarg, 2, resources)?;
                self.pop_operand(Some(Type::I64))?;
                self.pop_operand(Some(ty))?;
            }
            Operator::MemorySize { mem, mem_byte } => {
                if mem_byte != 0 && !self.features.multi_memory {
                    return Err(OperatorValidatorError::new("multi-memory not enabled"));
                }
                let index_ty = self.check_memory_index(mem, resources)?;
                self.push_operand(index_ty)?;
            }
            Operator::MemoryGrow { mem, mem_byte } => {
                if mem_byte != 0 && !self.features.multi_memory {
                    return Err(OperatorValidatorError::new("multi-memory not enabled"));
                }
                let index_ty = self.check_memory_index(mem, resources)?;
                self.pop_operand(Some(index_ty))?;
                self.push_operand(index_ty)?;
            }
            Operator::I32Const { .. } => self.push_operand(Type::I32)?,
            Operator::I64Const { .. } => self.push_operand(Type::I64)?,
            Operator::F32Const { .. } => {
                self.check_non_deterministic_enabled()?;
                self.push_operand(Type::F32)?;
            }
            Operator::F64Const { .. } => {
                self.check_non_deterministic_enabled()?;
                self.push_operand(Type::F64)?;
            }
            Operator::I32Eqz => {
                self.pop_operand(Some(Type::I32))?;
                self.push_operand(Type::I32)?;
            }
            Operator::I32Eq
            | Operator::I32Ne
            | Operator::I32LtS
            | Operator::I32LtU
            | Operator::I32GtS
            | Operator::I32GtU
            | Operator::I32LeS
            | Operator::I32LeU
            | Operator::I32GeS
            | Operator::I32GeU => {
                self.pop_operand(Some(Type::I32))?;
                self.pop_operand(Some(Type::I32))?;
                self.push_operand(Type::I32)?;
            }
            Operator::I64Eqz => {
                self.pop_operand(Some(Type::I64))?;
                self.push_operand(Type::I32)?;
            }
            Operator::I64Eq
            | Operator::I64Ne
            | Operator::I64LtS
            | Operator::I64LtU
            | Operator::I64GtS
            | Operator::I64GtU
            | Operator::I64LeS
            | Operator::I64LeU
            | Operator::I64GeS
            | Operator::I64GeU => {
                self.pop_operand(Some(Type::I64))?;
                self.pop_operand(Some(Type::I64))?;
                self.push_operand(Type::I32)?;
            }
            Operator::F32Eq
            | Operator::F32Ne
            | Operator::F32Lt
            | Operator::F32Gt
            | Operator::F32Le
            | Operator::F32Ge => {
                self.check_non_deterministic_enabled()?;
                self.pop_operand(Some(Type::F32))?;
                self.pop_operand(Some(Type::F32))?;
                self.push_operand(Type::I32)?;
            }
            Operator::F64Eq
            | Operator::F64Ne
            | Operator::F64Lt
            | Operator::F64Gt
            | Operator::F64Le
            | Operator::F64Ge => {
                self.check_non_deterministic_enabled()?;
                self.pop_operand(Some(Type::F64))?;
                self.pop_operand(Some(Type::F64))?;
                self.push_operand(Type::I32)?;
            }
            Operator::I32Clz | Operator::I32Ctz | Operator::I32Popcnt => {
                self.pop_operand(Some(Type::I32))?;
                self.push_operand(Type::I32)?;
            }
            Operator::I32Add
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
            | Operator::I32Rotr => {
                self.pop_operand(Some(Type::I32))?;
                self.pop_operand(Some(Type::I32))?;
                self.push_operand(Type::I32)?;
            }
            Operator::I64Clz | Operator::I64Ctz | Operator::I64Popcnt => {
                self.pop_operand(Some(Type::I64))?;
                self.push_operand(Type::I64)?;
            }
            Operator::I64Add
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
            | Operator::I64Rotr => {
                self.pop_operand(Some(Type::I64))?;
                self.pop_operand(Some(Type::I64))?;
                self.push_operand(Type::I64)?;
            }
            Operator::F32Abs
            | Operator::F32Neg
            | Operator::F32Ceil
            | Operator::F32Floor
            | Operator::F32Trunc
            | Operator::F32Nearest
            | Operator::F32Sqrt => {
                self.check_non_deterministic_enabled()?;
                self.pop_operand(Some(Type::F32))?;
                self.push_operand(Type::F32)?;
            }
            Operator::F32Add
            | Operator::F32Sub
            | Operator::F32Mul
            | Operator::F32Div
            | Operator::F32Min
            | Operator::F32Max
            | Operator::F32Copysign => {
                self.check_non_deterministic_enabled()?;
                self.pop_operand(Some(Type::F32))?;
                self.pop_operand(Some(Type::F32))?;
                self.push_operand(Type::F32)?;
            }
            Operator::F64Abs
            | Operator::F64Neg
            | Operator::F64Ceil
            | Operator::F64Floor
            | Operator::F64Trunc
            | Operator::F64Nearest
            | Operator::F64Sqrt => {
                self.check_non_deterministic_enabled()?;
                self.pop_operand(Some(Type::F64))?;
                self.push_operand(Type::F64)?;
            }
            Operator::F64Add
            | Operator::F64Sub
            | Operator::F64Mul
            | Operator::F64Div
            | Operator::F64Min
            | Operator::F64Max
            | Operator::F64Copysign => {
                self.check_non_deterministic_enabled()?;
                self.pop_operand(Some(Type::F64))?;
                self.pop_operand(Some(Type::F64))?;
                self.push_operand(Type::F64)?;
            }
            Operator::I32WrapI64 => {
                self.pop_operand(Some(Type::I64))?;
                self.push_operand(Type::I32)?;
            }
            Operator::I32TruncF32S | Operator::I32TruncF32U => {
                self.pop_operand(Some(Type::F32))?;
                self.push_operand(Type::I32)?;
            }
            Operator::I32TruncF64S | Operator::I32TruncF64U => {
                self.pop_operand(Some(Type::F64))?;
                self.push_operand(Type::I32)?;
            }
            Operator::I64ExtendI32S | Operator::I64ExtendI32U => {
                self.pop_operand(Some(Type::I32))?;
                self.push_operand(Type::I64)?;
            }
            Operator::I64TruncF32S | Operator::I64TruncF32U => {
                self.pop_operand(Some(Type::F32))?;
                self.push_operand(Type::I64)?;
            }
            Operator::I64TruncF64S | Operator::I64TruncF64U => {
                self.pop_operand(Some(Type::F64))?;
                self.push_operand(Type::I64)?;
            }
            Operator::F32ConvertI32S | Operator::F32ConvertI32U => {
                self.check_non_deterministic_enabled()?;
                self.pop_operand(Some(Type::I32))?;
                self.push_operand(Type::F32)?;
            }
            Operator::F32ConvertI64S | Operator::F32ConvertI64U => {
                self.check_non_deterministic_enabled()?;
                self.pop_operand(Some(Type::I64))?;
                self.push_operand(Type::F32)?;
            }
            Operator::F32DemoteF64 => {
                self.check_non_deterministic_enabled()?;
                self.pop_operand(Some(Type::F64))?;
                self.push_operand(Type::F32)?;
            }
            Operator::F64ConvertI32S | Operator::F64ConvertI32U => {
                self.check_non_deterministic_enabled()?;
                self.pop_operand(Some(Type::I32))?;
                self.push_operand(Type::F64)?;
            }
            Operator::F64ConvertI64S | Operator::F64ConvertI64U => {
                self.check_non_deterministic_enabled()?;
                self.pop_operand(Some(Type::I64))?;
                self.push_operand(Type::F64)?;
            }
            Operator::F64PromoteF32 => {
                self.check_non_deterministic_enabled()?;
                self.pop_operand(Some(Type::F32))?;
                self.push_operand(Type::F64)?;
            }
            Operator::I32ReinterpretF32 => {
                self.pop_operand(Some(Type::F32))?;
                self.push_operand(Type::I32)?;
            }
            Operator::I64ReinterpretF64 => {
                self.pop_operand(Some(Type::F64))?;
                self.push_operand(Type::I64)?;
            }
            Operator::F32ReinterpretI32 => {
                self.check_non_deterministic_enabled()?;
                self.pop_operand(Some(Type::I32))?;
                self.push_operand(Type::F32)?;
            }
            Operator::F64ReinterpretI64 => {
                self.check_non_deterministic_enabled()?;
                self.pop_operand(Some(Type::I64))?;
                self.push_operand(Type::F64)?;
            }
            Operator::I32TruncSatF32S | Operator::I32TruncSatF32U => {
                if !self.features.saturating_float_to_int {
                    return Err(OperatorValidatorError::new(
                        "saturating float to int conversions support is not enabled",
                    ));
                }
                self.pop_operand(Some(Type::F32))?;
                self.push_operand(Type::I32)?;
            }
            Operator::I32TruncSatF64S | Operator::I32TruncSatF64U => {
                if !self.features.saturating_float_to_int {
                    return Err(OperatorValidatorError::new(
                        "saturating float to int conversions support is not enabled",
                    ));
                }
                self.pop_operand(Some(Type::F64))?;
                self.push_operand(Type::I32)?;
            }
            Operator::I64TruncSatF32S | Operator::I64TruncSatF32U => {
                if !self.features.saturating_float_to_int {
                    return Err(OperatorValidatorError::new(
                        "saturating float to int conversions support is not enabled",
                    ));
                }
                self.pop_operand(Some(Type::F32))?;
                self.push_operand(Type::I64)?;
            }
            Operator::I64TruncSatF64S | Operator::I64TruncSatF64U => {
                if !self.features.saturating_float_to_int {
                    return Err(OperatorValidatorError::new(
                        "saturating float to int conversions support is not enabled",
                    ));
                }
                self.pop_operand(Some(Type::F64))?;
                self.push_operand(Type::I64)?;
            }
            Operator::I32Extend16S | Operator::I32Extend8S => {
                if !self.features.sign_extension {
                    return Err(OperatorValidatorError::new(
                        "sign extension operations support is not enabled",
                    ));
                }
                self.pop_operand(Some(Type::I32))?;
                self.push_operand(Type::I32)?;
            }

            Operator::I64Extend32S | Operator::I64Extend16S | Operator::I64Extend8S => {
                if !self.features.sign_extension {
                    return Err(OperatorValidatorError::new(
                        "sign extension operations support is not enabled",
                    ));
                }
                self.pop_operand(Some(Type::I64))?;
                self.push_operand(Type::I64)?;
            }

            Operator::I32AtomicLoad { memarg }
            | Operator::I32AtomicLoad16U { memarg }
            | Operator::I32AtomicLoad8U { memarg } => {
                self.check_threads_enabled()?;
                let ty = self.check_shared_memarg_wo_align(memarg, resources)?;
                self.pop_operand(Some(ty))?;
                self.push_operand(Type::I32)?;
            }
            Operator::I64AtomicLoad { memarg }
            | Operator::I64AtomicLoad32U { memarg }
            | Operator::I64AtomicLoad16U { memarg }
            | Operator::I64AtomicLoad8U { memarg } => {
                self.check_threads_enabled()?;
                let ty = self.check_shared_memarg_wo_align(memarg, resources)?;
                self.pop_operand(Some(ty))?;
                self.push_operand(Type::I64)?;
            }
            Operator::I32AtomicStore { memarg }
            | Operator::I32AtomicStore16 { memarg }
            | Operator::I32AtomicStore8 { memarg } => {
                self.check_threads_enabled()?;
                let ty = self.check_shared_memarg_wo_align(memarg, resources)?;
                self.pop_operand(Some(Type::I32))?;
                self.pop_operand(Some(ty))?;
            }
            Operator::I64AtomicStore { memarg }
            | Operator::I64AtomicStore32 { memarg }
            | Operator::I64AtomicStore16 { memarg }
            | Operator::I64AtomicStore8 { memarg } => {
                self.check_threads_enabled()?;
                let ty = self.check_shared_memarg_wo_align(memarg, resources)?;
                self.pop_operand(Some(Type::I64))?;
                self.pop_operand(Some(ty))?;
            }
            Operator::I32AtomicRmwAdd { memarg }
            | Operator::I32AtomicRmwSub { memarg }
            | Operator::I32AtomicRmwAnd { memarg }
            | Operator::I32AtomicRmwOr { memarg }
            | Operator::I32AtomicRmwXor { memarg }
            | Operator::I32AtomicRmw16AddU { memarg }
            | Operator::I32AtomicRmw16SubU { memarg }
            | Operator::I32AtomicRmw16AndU { memarg }
            | Operator::I32AtomicRmw16OrU { memarg }
            | Operator::I32AtomicRmw16XorU { memarg }
            | Operator::I32AtomicRmw8AddU { memarg }
            | Operator::I32AtomicRmw8SubU { memarg }
            | Operator::I32AtomicRmw8AndU { memarg }
            | Operator::I32AtomicRmw8OrU { memarg }
            | Operator::I32AtomicRmw8XorU { memarg } => {
                self.check_threads_enabled()?;
                let ty = self.check_shared_memarg_wo_align(memarg, resources)?;
                self.pop_operand(Some(Type::I32))?;
                self.pop_operand(Some(ty))?;
                self.push_operand(Type::I32)?;
            }
            Operator::I64AtomicRmwAdd { memarg }
            | Operator::I64AtomicRmwSub { memarg }
            | Operator::I64AtomicRmwAnd { memarg }
            | Operator::I64AtomicRmwOr { memarg }
            | Operator::I64AtomicRmwXor { memarg }
            | Operator::I64AtomicRmw32AddU { memarg }
            | Operator::I64AtomicRmw32SubU { memarg }
            | Operator::I64AtomicRmw32AndU { memarg }
            | Operator::I64AtomicRmw32OrU { memarg }
            | Operator::I64AtomicRmw32XorU { memarg }
            | Operator::I64AtomicRmw16AddU { memarg }
            | Operator::I64AtomicRmw16SubU { memarg }
            | Operator::I64AtomicRmw16AndU { memarg }
            | Operator::I64AtomicRmw16OrU { memarg }
            | Operator::I64AtomicRmw16XorU { memarg }
            | Operator::I64AtomicRmw8AddU { memarg }
            | Operator::I64AtomicRmw8SubU { memarg }
            | Operator::I64AtomicRmw8AndU { memarg }
            | Operator::I64AtomicRmw8OrU { memarg }
            | Operator::I64AtomicRmw8XorU { memarg } => {
                self.check_threads_enabled()?;
                let ty = self.check_shared_memarg_wo_align(memarg, resources)?;
                self.pop_operand(Some(Type::I64))?;
                self.pop_operand(Some(ty))?;
                self.push_operand(Type::I64)?;
            }
            Operator::I32AtomicRmwXchg { memarg }
            | Operator::I32AtomicRmw16XchgU { memarg }
            | Operator::I32AtomicRmw8XchgU { memarg } => {
                self.check_threads_enabled()?;
                let ty = self.check_shared_memarg_wo_align(memarg, resources)?;
                self.pop_operand(Some(Type::I32))?;
                self.pop_operand(Some(ty))?;
                self.push_operand(Type::I32)?;
            }
            Operator::I32AtomicRmwCmpxchg { memarg }
            | Operator::I32AtomicRmw16CmpxchgU { memarg }
            | Operator::I32AtomicRmw8CmpxchgU { memarg } => {
                self.check_threads_enabled()?;
                let ty = self.check_shared_memarg_wo_align(memarg, resources)?;
                self.pop_operand(Some(Type::I32))?;
                self.pop_operand(Some(Type::I32))?;
                self.pop_operand(Some(ty))?;
                self.push_operand(Type::I32)?;
            }
            Operator::I64AtomicRmwXchg { memarg }
            | Operator::I64AtomicRmw32XchgU { memarg }
            | Operator::I64AtomicRmw16XchgU { memarg }
            | Operator::I64AtomicRmw8XchgU { memarg } => {
                self.check_threads_enabled()?;
                let ty = self.check_shared_memarg_wo_align(memarg, resources)?;
                self.pop_operand(Some(Type::I64))?;
                self.pop_operand(Some(ty))?;
                self.push_operand(Type::I64)?;
            }
            Operator::I64AtomicRmwCmpxchg { memarg }
            | Operator::I64AtomicRmw32CmpxchgU { memarg }
            | Operator::I64AtomicRmw16CmpxchgU { memarg }
            | Operator::I64AtomicRmw8CmpxchgU { memarg } => {
                self.check_threads_enabled()?;
                let ty = self.check_shared_memarg_wo_align(memarg, resources)?;
                self.pop_operand(Some(Type::I64))?;
                self.pop_operand(Some(Type::I64))?;
                self.pop_operand(Some(ty))?;
                self.push_operand(Type::I64)?;
            }
            Operator::MemoryAtomicNotify { memarg } => {
                self.check_threads_enabled()?;
                let ty = self.check_shared_memarg_wo_align(memarg, resources)?;
                self.pop_operand(Some(Type::I32))?;
                self.pop_operand(Some(ty))?;
                self.push_operand(Type::I32)?;
            }
            Operator::MemoryAtomicWait32 { memarg } => {
                self.check_threads_enabled()?;
                let ty = self.check_shared_memarg_wo_align(memarg, resources)?;
                self.pop_operand(Some(Type::I64))?;
                self.pop_operand(Some(Type::I32))?;
                self.pop_operand(Some(ty))?;
                self.push_operand(Type::I32)?;
            }
            Operator::MemoryAtomicWait64 { memarg } => {
                self.check_threads_enabled()?;
                let ty = self.check_shared_memarg_wo_align(memarg, resources)?;
                self.pop_operand(Some(Type::I64))?;
                self.pop_operand(Some(Type::I64))?;
                self.pop_operand(Some(ty))?;
                self.push_operand(Type::I32)?;
            }
            Operator::AtomicFence { ref flags } => {
                self.check_threads_enabled()?;
                if *flags != 0 {
                    return Err(OperatorValidatorError::new(
                        "non-zero flags for fence not supported yet",
                    ));
                }
            }
            Operator::RefNull { ty } => {
                self.check_reference_types_enabled()?;
                match ty {
                    Type::FuncRef | Type::ExternRef => {}
                    _ => {
                        return Err(OperatorValidatorError::new(
                            "invalid reference type in ref.null",
                        ))
                    }
                }
                self.push_operand(ty)?;
            }
            Operator::RefIsNull => {
                self.check_reference_types_enabled()?;
                match self.pop_operand(None)? {
                    None | Some(Type::FuncRef) | Some(Type::ExternRef) => {}
                    _ => {
                        return Err(OperatorValidatorError::new(
                            "type mismatch: invalid reference type in ref.is_null",
                        ))
                    }
                }
                self.push_operand(Type::I32)?;
            }
            Operator::RefFunc { function_index } => {
                self.check_reference_types_enabled()?;
                if resources.type_of_function(function_index).is_none() {
                    return Err(OperatorValidatorError::new(format!(
                        "unknown function {}: function index out of bounds",
                        function_index,
                    )));
                }
                if !resources.is_function_referenced(function_index) {
                    return Err(OperatorValidatorError::new("undeclared function reference"));
                }
                self.push_operand(Type::FuncRef)?;
            }
            Operator::V128Load { memarg } => {
                self.check_simd_enabled()?;
                let ty = self.check_memarg(memarg, 4, resources)?;
                self.pop_operand(Some(ty))?;
                self.push_operand(Type::V128)?;
            }
            Operator::V128Store { memarg } => {
                self.check_simd_enabled()?;
                let ty = self.check_memarg(memarg, 4, resources)?;
                self.pop_operand(Some(Type::V128))?;
                self.pop_operand(Some(ty))?;
            }
            Operator::V128Const { .. } => {
                self.check_simd_enabled()?;
                self.push_operand(Type::V128)?;
            }
            Operator::I8x16Splat | Operator::I16x8Splat | Operator::I32x4Splat => {
                self.check_simd_enabled()?;
                self.pop_operand(Some(Type::I32))?;
                self.push_operand(Type::V128)?;
            }
            Operator::I64x2Splat => {
                self.check_simd_enabled()?;
                self.pop_operand(Some(Type::I64))?;
                self.push_operand(Type::V128)?;
            }
            Operator::F32x4Splat => {
                self.check_non_deterministic_enabled()?;
                self.check_simd_enabled()?;
                self.pop_operand(Some(Type::F32))?;
                self.push_operand(Type::V128)?;
            }
            Operator::F64x2Splat => {
                self.check_non_deterministic_enabled()?;
                self.check_simd_enabled()?;
                self.pop_operand(Some(Type::F64))?;
                self.push_operand(Type::V128)?;
            }
            Operator::I8x16ExtractLaneS { lane } | Operator::I8x16ExtractLaneU { lane } => {
                self.check_simd_enabled()?;
                self.check_simd_lane_index(lane, 16)?;
                self.pop_operand(Some(Type::V128))?;
                self.push_operand(Type::I32)?;
            }
            Operator::I16x8ExtractLaneS { lane } | Operator::I16x8ExtractLaneU { lane } => {
                self.check_simd_enabled()?;
                self.check_simd_lane_index(lane, 8)?;
                self.pop_operand(Some(Type::V128))?;
                self.push_operand(Type::I32)?;
            }
            Operator::I32x4ExtractLane { lane } => {
                self.check_simd_enabled()?;
                self.check_simd_lane_index(lane, 4)?;
                self.pop_operand(Some(Type::V128))?;
                self.push_operand(Type::I32)?;
            }
            Operator::I8x16ReplaceLane { lane } => {
                self.check_simd_enabled()?;
                self.check_simd_lane_index(lane, 16)?;
                self.pop_operand(Some(Type::I32))?;
                self.pop_operand(Some(Type::V128))?;
                self.push_operand(Type::V128)?;
            }
            Operator::I16x8ReplaceLane { lane } => {
                self.check_simd_enabled()?;
                self.check_simd_lane_index(lane, 8)?;
                self.pop_operand(Some(Type::I32))?;
                self.pop_operand(Some(Type::V128))?;
                self.push_operand(Type::V128)?;
            }
            Operator::I32x4ReplaceLane { lane } => {
                self.check_simd_enabled()?;
                self.check_simd_lane_index(lane, 4)?;
                self.pop_operand(Some(Type::I32))?;
                self.pop_operand(Some(Type::V128))?;
                self.push_operand(Type::V128)?;
            }
            Operator::I64x2ExtractLane { lane } => {
                self.check_simd_enabled()?;
                self.check_simd_lane_index(lane, 2)?;
                self.pop_operand(Some(Type::V128))?;
                self.push_operand(Type::I64)?;
            }
            Operator::I64x2ReplaceLane { lane } => {
                self.check_simd_enabled()?;
                self.check_simd_lane_index(lane, 2)?;
                self.pop_operand(Some(Type::I64))?;
                self.pop_operand(Some(Type::V128))?;
                self.push_operand(Type::V128)?;
            }
            Operator::F32x4ExtractLane { lane } => {
                self.check_non_deterministic_enabled()?;
                self.check_simd_enabled()?;
                self.check_simd_lane_index(lane, 4)?;
                self.pop_operand(Some(Type::V128))?;
                self.push_operand(Type::F32)?;
            }
            Operator::F32x4ReplaceLane { lane } => {
                self.check_non_deterministic_enabled()?;
                self.check_simd_enabled()?;
                self.check_simd_lane_index(lane, 4)?;
                self.pop_operand(Some(Type::F32))?;
                self.pop_operand(Some(Type::V128))?;
                self.push_operand(Type::V128)?;
            }
            Operator::F64x2ExtractLane { lane } => {
                self.check_non_deterministic_enabled()?;
                self.check_simd_enabled()?;
                self.check_simd_lane_index(lane, 2)?;
                self.pop_operand(Some(Type::V128))?;
                self.push_operand(Type::F64)?;
            }
            Operator::F64x2ReplaceLane { lane } => {
                self.check_non_deterministic_enabled()?;
                self.check_simd_enabled()?;
                self.check_simd_lane_index(lane, 2)?;
                self.pop_operand(Some(Type::F64))?;
                self.pop_operand(Some(Type::V128))?;
                self.push_operand(Type::V128)?;
            }
            Operator::F32x4Eq
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
            | Operator::F32x4Add
            | Operator::F32x4Sub
            | Operator::F32x4Mul
            | Operator::F32x4Div
            | Operator::F32x4Min
            | Operator::F32x4Max
            | Operator::F32x4PMin
            | Operator::F32x4PMax
            | Operator::F64x2Add
            | Operator::F64x2Sub
            | Operator::F64x2Mul
            | Operator::F64x2Div
            | Operator::F64x2Min
            | Operator::F64x2Max
            | Operator::F64x2PMin
            | Operator::F64x2PMax => {
                self.check_non_deterministic_enabled()?;
                self.check_simd_enabled()?;
                self.pop_operand(Some(Type::V128))?;
                self.pop_operand(Some(Type::V128))?;
                self.push_operand(Type::V128)?;
            }
            Operator::F32x4RelaxedMin
            | Operator::F32x4RelaxedMax
            | Operator::F64x2RelaxedMin
            | Operator::F64x2RelaxedMax => {
                self.check_relaxed_simd_enabled()?;
                self.pop_operand(Some(Type::V128))?;
                self.pop_operand(Some(Type::V128))?;
                self.push_operand(Type::V128)?;
            }
            Operator::I8x16Eq
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
            | Operator::I64x2Eq
            | Operator::I64x2Ne
            | Operator::I64x2LtS
            | Operator::I64x2GtS
            | Operator::I64x2LeS
            | Operator::I64x2GeS
            | Operator::V128And
            | Operator::V128AndNot
            | Operator::V128Or
            | Operator::V128Xor
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
            | Operator::I32x4Add
            | Operator::I32x4Sub
            | Operator::I32x4Mul
            | Operator::I32x4MinS
            | Operator::I32x4MinU
            | Operator::I32x4MaxS
            | Operator::I32x4MaxU
            | Operator::I32x4DotI16x8S
            | Operator::I64x2Add
            | Operator::I64x2Sub
            | Operator::I64x2Mul
            | Operator::I8x16RoundingAverageU
            | Operator::I16x8RoundingAverageU
            | Operator::I8x16NarrowI16x8S
            | Operator::I8x16NarrowI16x8U
            | Operator::I16x8NarrowI32x4S
            | Operator::I16x8NarrowI32x4U
            | Operator::I16x8ExtMulLowI8x16S
            | Operator::I16x8ExtMulHighI8x16S
            | Operator::I16x8ExtMulLowI8x16U
            | Operator::I16x8ExtMulHighI8x16U
            | Operator::I32x4ExtMulLowI16x8S
            | Operator::I32x4ExtMulHighI16x8S
            | Operator::I32x4ExtMulLowI16x8U
            | Operator::I32x4ExtMulHighI16x8U
            | Operator::I64x2ExtMulLowI32x4S
            | Operator::I64x2ExtMulHighI32x4S
            | Operator::I64x2ExtMulLowI32x4U
            | Operator::I64x2ExtMulHighI32x4U
            | Operator::I16x8Q15MulrSatS => {
                self.check_simd_enabled()?;
                self.pop_operand(Some(Type::V128))?;
                self.pop_operand(Some(Type::V128))?;
                self.push_operand(Type::V128)?;
            }
            Operator::F32x4Ceil
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
            | Operator::F64x2Abs
            | Operator::F64x2Neg
            | Operator::F64x2Sqrt
            | Operator::F32x4DemoteF64x2Zero
            | Operator::F64x2PromoteLowF32x4
            | Operator::F64x2ConvertLowI32x4S
            | Operator::F64x2ConvertLowI32x4U
            | Operator::I32x4TruncSatF32x4S
            | Operator::I32x4TruncSatF32x4U
            | Operator::I32x4TruncSatF64x2SZero
            | Operator::I32x4TruncSatF64x2UZero
            | Operator::F32x4ConvertI32x4S
            | Operator::F32x4ConvertI32x4U => {
                self.check_non_deterministic_enabled()?;
                self.check_simd_enabled()?;
                self.pop_operand(Some(Type::V128))?;
                self.push_operand(Type::V128)?;
            }
            Operator::V128Not
            | Operator::I8x16Abs
            | Operator::I8x16Neg
            | Operator::I8x16Popcnt
            | Operator::I16x8Abs
            | Operator::I16x8Neg
            | Operator::I32x4Abs
            | Operator::I32x4Neg
            | Operator::I64x2Abs
            | Operator::I64x2Neg
            | Operator::I16x8ExtendLowI8x16S
            | Operator::I16x8ExtendHighI8x16S
            | Operator::I16x8ExtendLowI8x16U
            | Operator::I16x8ExtendHighI8x16U
            | Operator::I32x4ExtendLowI16x8S
            | Operator::I32x4ExtendHighI16x8S
            | Operator::I32x4ExtendLowI16x8U
            | Operator::I32x4ExtendHighI16x8U
            | Operator::I64x2ExtendLowI32x4S
            | Operator::I64x2ExtendHighI32x4S
            | Operator::I64x2ExtendLowI32x4U
            | Operator::I64x2ExtendHighI32x4U
            | Operator::I16x8ExtAddPairwiseI8x16S
            | Operator::I16x8ExtAddPairwiseI8x16U
            | Operator::I32x4ExtAddPairwiseI16x8S
            | Operator::I32x4ExtAddPairwiseI16x8U => {
                self.check_simd_enabled()?;
                self.pop_operand(Some(Type::V128))?;
                self.push_operand(Type::V128)?;
            }
            Operator::I32x4RelaxedTruncSatF32x4S
            | Operator::I32x4RelaxedTruncSatF32x4U
            | Operator::I32x4RelaxedTruncSatF64x2SZero
            | Operator::I32x4RelaxedTruncSatF64x2UZero => {
                self.check_relaxed_simd_enabled()?;
                self.pop_operand(Some(Type::V128))?;
                self.push_operand(Type::V128)?;
            }
            Operator::V128Bitselect => {
                self.check_simd_enabled()?;
                self.pop_operand(Some(Type::V128))?;
                self.pop_operand(Some(Type::V128))?;
                self.pop_operand(Some(Type::V128))?;
                self.push_operand(Type::V128)?;
            }
            Operator::F32x4Fma
            | Operator::F32x4Fms
            | Operator::F64x2Fma
            | Operator::F64x2Fms
            | Operator::I8x16LaneSelect
            | Operator::I16x8LaneSelect
            | Operator::I32x4LaneSelect
            | Operator::I64x2LaneSelect => {
                self.check_relaxed_simd_enabled()?;
                self.pop_operand(Some(Type::V128))?;
                self.pop_operand(Some(Type::V128))?;
                self.pop_operand(Some(Type::V128))?;
                self.push_operand(Type::V128)?;
            }
            Operator::V128AnyTrue
            | Operator::I8x16AllTrue
            | Operator::I8x16Bitmask
            | Operator::I16x8AllTrue
            | Operator::I16x8Bitmask
            | Operator::I32x4AllTrue
            | Operator::I32x4Bitmask
            | Operator::I64x2AllTrue
            | Operator::I64x2Bitmask => {
                self.check_simd_enabled()?;
                self.pop_operand(Some(Type::V128))?;
                self.push_operand(Type::I32)?;
            }
            Operator::I8x16Shl
            | Operator::I8x16ShrS
            | Operator::I8x16ShrU
            | Operator::I16x8Shl
            | Operator::I16x8ShrS
            | Operator::I16x8ShrU
            | Operator::I32x4Shl
            | Operator::I32x4ShrS
            | Operator::I32x4ShrU
            | Operator::I64x2Shl
            | Operator::I64x2ShrS
            | Operator::I64x2ShrU => {
                self.check_simd_enabled()?;
                self.pop_operand(Some(Type::I32))?;
                self.pop_operand(Some(Type::V128))?;
                self.push_operand(Type::V128)?;
            }
            Operator::I8x16Swizzle => {
                self.check_simd_enabled()?;
                self.pop_operand(Some(Type::V128))?;
                self.pop_operand(Some(Type::V128))?;
                self.push_operand(Type::V128)?;
            }
            Operator::I8x16RelaxedSwizzle => {
                self.check_relaxed_simd_enabled()?;
                self.pop_operand(Some(Type::V128))?;
                self.pop_operand(Some(Type::V128))?;
                self.push_operand(Type::V128)?;
            }
            Operator::I8x16Shuffle { ref lanes } => {
                self.check_simd_enabled()?;
                self.pop_operand(Some(Type::V128))?;
                self.pop_operand(Some(Type::V128))?;
                for i in lanes {
                    self.check_simd_lane_index(*i, 32)?;
                }
                self.push_operand(Type::V128)?;
            }
            Operator::V128Load8Splat { memarg } => {
                self.check_simd_enabled()?;
                let ty = self.check_memarg(memarg, 0, resources)?;
                self.pop_operand(Some(ty))?;
                self.push_operand(Type::V128)?;
            }
            Operator::V128Load16Splat { memarg } => {
                self.check_simd_enabled()?;
                let ty = self.check_memarg(memarg, 1, resources)?;
                self.pop_operand(Some(ty))?;
                self.push_operand(Type::V128)?;
            }
            Operator::V128Load32Splat { memarg } | Operator::V128Load32Zero { memarg } => {
                self.check_simd_enabled()?;
                let ty = self.check_memarg(memarg, 2, resources)?;
                self.pop_operand(Some(ty))?;
                self.push_operand(Type::V128)?;
            }
            Operator::V128Load64Splat { memarg }
            | Operator::V128Load64Zero { memarg }
            | Operator::V128Load8x8S { memarg }
            | Operator::V128Load8x8U { memarg }
            | Operator::V128Load16x4S { memarg }
            | Operator::V128Load16x4U { memarg }
            | Operator::V128Load32x2S { memarg }
            | Operator::V128Load32x2U { memarg } => {
                self.check_simd_enabled()?;
                let idx = self.check_memarg(memarg, 3, resources)?;
                self.pop_operand(Some(idx))?;
                self.push_operand(Type::V128)?;
            }
            Operator::V128Load8Lane { memarg, lane } => {
                self.check_simd_enabled()?;
                let idx = self.check_memarg(memarg, 0, resources)?;
                self.check_simd_lane_index(lane, 16)?;
                self.pop_operand(Some(Type::V128))?;
                self.pop_operand(Some(idx))?;
                self.push_operand(Type::V128)?;
            }
            Operator::V128Load16Lane { memarg, lane } => {
                self.check_simd_enabled()?;
                let idx = self.check_memarg(memarg, 1, resources)?;
                self.check_simd_lane_index(lane, 8)?;
                self.pop_operand(Some(Type::V128))?;
                self.pop_operand(Some(idx))?;
                self.push_operand(Type::V128)?;
            }
            Operator::V128Load32Lane { memarg, lane } => {
                self.check_simd_enabled()?;
                let idx = self.check_memarg(memarg, 2, resources)?;
                self.check_simd_lane_index(lane, 4)?;
                self.pop_operand(Some(Type::V128))?;
                self.pop_operand(Some(idx))?;
                self.push_operand(Type::V128)?;
            }
            Operator::V128Load64Lane { memarg, lane } => {
                self.check_simd_enabled()?;
                let idx = self.check_memarg(memarg, 3, resources)?;
                self.check_simd_lane_index(lane, 2)?;
                self.pop_operand(Some(Type::V128))?;
                self.pop_operand(Some(idx))?;
                self.push_operand(Type::V128)?;
            }
            Operator::V128Store8Lane { memarg, lane } => {
                self.check_simd_enabled()?;
                let idx = self.check_memarg(memarg, 0, resources)?;
                self.check_simd_lane_index(lane, 16)?;
                self.pop_operand(Some(Type::V128))?;
                self.pop_operand(Some(idx))?;
            }
            Operator::V128Store16Lane { memarg, lane } => {
                self.check_simd_enabled()?;
                let idx = self.check_memarg(memarg, 1, resources)?;
                self.check_simd_lane_index(lane, 8)?;
                self.pop_operand(Some(Type::V128))?;
                self.pop_operand(Some(idx))?;
            }
            Operator::V128Store32Lane { memarg, lane } => {
                self.check_simd_enabled()?;
                let idx = self.check_memarg(memarg, 2, resources)?;
                self.check_simd_lane_index(lane, 4)?;
                self.pop_operand(Some(Type::V128))?;
                self.pop_operand(Some(idx))?;
            }
            Operator::V128Store64Lane { memarg, lane } => {
                self.check_simd_enabled()?;
                let idx = self.check_memarg(memarg, 3, resources)?;
                self.check_simd_lane_index(lane, 2)?;
                self.pop_operand(Some(Type::V128))?;
                self.pop_operand(Some(idx))?;
            }
            Operator::MemoryInit { mem, segment } => {
                self.check_bulk_memory_enabled()?;
                let ty = self.check_memory_index(mem, resources)?;
                if segment >= resources.data_count() {
                    bail_op_err!("unknown data segment {}", segment);
                }
                self.pop_operand(Some(Type::I32))?;
                self.pop_operand(Some(Type::I32))?;
                self.pop_operand(Some(ty))?;
            }
            Operator::DataDrop { segment } => {
                self.check_bulk_memory_enabled()?;
                if segment >= resources.data_count() {
                    bail_op_err!("unknown data segment {}", segment);
                }
            }
            Operator::MemoryCopy { src, dst } => {
                self.check_bulk_memory_enabled()?;
                let dst_ty = self.check_memory_index(dst, resources)?;
                let src_ty = self.check_memory_index(src, resources)?;

                // The length operand here is the smaller of src/dst, which is
                // i32 if one is i32
                self.pop_operand(Some(match src_ty {
                    Type::I32 => Type::I32,
                    _ => dst_ty,
                }))?;

                // ... and the offset into each memory is required to be
                // whatever the indexing type is for that memory
                self.pop_operand(Some(src_ty))?;
                self.pop_operand(Some(dst_ty))?;
            }
            Operator::MemoryFill { mem } => {
                self.check_bulk_memory_enabled()?;
                let ty = self.check_memory_index(mem, resources)?;
                self.pop_operand(Some(ty))?;
                self.pop_operand(Some(Type::I32))?;
                self.pop_operand(Some(ty))?;
            }
            Operator::TableInit { segment, table } => {
                self.check_bulk_memory_enabled()?;
                if table > 0 {
                    self.check_reference_types_enabled()?;
                }
                let table = match resources.table_at(table) {
                    Some(table) => table,
                    None => bail_op_err!("unknown table {}: table index out of bounds", table),
                };
                let segment_ty = match resources.element_type_at(segment) {
                    Some(ty) => ty,
                    None => bail_op_err!(
                        "unknown elem segment {}: segment index out of bounds",
                        segment
                    ),
                };
                if segment_ty != table.element_type {
                    return Err(OperatorValidatorError::new("type mismatch"));
                }
                self.pop_operand(Some(Type::I32))?;
                self.pop_operand(Some(Type::I32))?;
                self.pop_operand(Some(Type::I32))?;
            }
            Operator::ElemDrop { segment } => {
                self.check_bulk_memory_enabled()?;
                if segment >= resources.element_count() {
                    bail_op_err!(
                        "unknown elem segment {}: segment index out of bounds",
                        segment
                    );
                }
            }
            Operator::TableCopy {
                src_table,
                dst_table,
            } => {
                self.check_bulk_memory_enabled()?;
                if src_table > 0 || dst_table > 0 {
                    self.check_reference_types_enabled()?;
                }
                let (src, dst) =
                    match (resources.table_at(src_table), resources.table_at(dst_table)) {
                        (Some(a), Some(b)) => (a, b),
                        _ => return Err(OperatorValidatorError::new("table index out of bounds")),
                    };
                if src.element_type != dst.element_type {
                    return Err(OperatorValidatorError::new("type mismatch"));
                }
                self.pop_operand(Some(Type::I32))?;
                self.pop_operand(Some(Type::I32))?;
                self.pop_operand(Some(Type::I32))?;
            }
            Operator::TableGet { table } => {
                self.check_reference_types_enabled()?;
                let ty = match resources.table_at(table) {
                    Some(ty) => ty.element_type,
                    None => return Err(OperatorValidatorError::new("table index out of bounds")),
                };
                self.pop_operand(Some(Type::I32))?;
                self.push_operand(ty)?;
            }
            Operator::TableSet { table } => {
                self.check_reference_types_enabled()?;
                let ty = match resources.table_at(table) {
                    Some(ty) => ty.element_type,
                    None => return Err(OperatorValidatorError::new("table index out of bounds")),
                };
                self.pop_operand(Some(ty))?;
                self.pop_operand(Some(Type::I32))?;
            }
            Operator::TableGrow { table } => {
                self.check_reference_types_enabled()?;
                let ty = match resources.table_at(table) {
                    Some(ty) => ty.element_type,
                    None => return Err(OperatorValidatorError::new("table index out of bounds")),
                };
                self.pop_operand(Some(Type::I32))?;
                self.pop_operand(Some(ty))?;
                self.push_operand(Type::I32)?;
            }
            Operator::TableSize { table } => {
                self.check_reference_types_enabled()?;
                if resources.table_at(table).is_none() {
                    return Err(OperatorValidatorError::new("table index out of bounds"));
                }
                self.push_operand(Type::I32)?;
            }
            Operator::TableFill { table } => {
                self.check_bulk_memory_enabled()?;
                let ty = match resources.table_at(table) {
                    Some(ty) => ty.element_type,
                    None => return Err(OperatorValidatorError::new("table index out of bounds")),
                };
                self.pop_operand(Some(Type::I32))?;
                self.pop_operand(Some(ty))?;
                self.pop_operand(Some(Type::I32))?;
            }
        }
        Ok(())
    }

    pub fn finish(&mut self) -> OperatorValidatorResult<()> {
        if !self.control.is_empty() {
            bail_op_err!("control frames remain at end of function");
        }
        Ok(())
    }
}

fn func_type_at<T: WasmModuleResources>(
    resources: &T,
    at: u32,
) -> OperatorValidatorResult<&T::FuncType> {
    resources
        .func_type_at(at)
        .ok_or_else(|| OperatorValidatorError::new("unknown type: type index out of bounds"))
}

fn tag_at<T: WasmModuleResources>(resources: &T, at: u32) -> OperatorValidatorResult<&T::FuncType> {
    resources.tag_at(at).ok_or_else(|| {
        OperatorValidatorError::new(format!("unknown tag {}: tag index out of bounds", at))
    })
}

enum Either<A, B> {
    A(A),
    B(B),
}

impl<A, B> Iterator for Either<A, B>
where
    A: Iterator,
    B: Iterator<Item = A::Item>,
{
    type Item = A::Item;
    fn next(&mut self) -> Option<A::Item> {
        match self {
            Either::A(a) => a.next(),
            Either::B(b) => b.next(),
        }
    }
}

impl<A, B> DoubleEndedIterator for Either<A, B>
where
    A: DoubleEndedIterator,
    B: DoubleEndedIterator<Item = A::Item>,
{
    fn next_back(&mut self) -> Option<A::Item> {
        match self {
            Either::A(a) => a.next_back(),
            Either::B(b) => b.next_back(),
        }
    }
}

impl<A, B> ExactSizeIterator for Either<A, B>
where
    A: ExactSizeIterator,
    B: ExactSizeIterator<Item = A::Item>,
{
    fn len(&self) -> usize {
        match self {
            Either::A(a) => a.len(),
            Either::B(b) => b.len(),
        }
    }
}

trait PreciseIterator: ExactSizeIterator + DoubleEndedIterator {}
impl<T: ExactSizeIterator + DoubleEndedIterator> PreciseIterator for T {}

fn params(
    ty: TypeOrFuncType,
    resources: &impl WasmModuleResources,
) -> OperatorValidatorResult<impl PreciseIterator<Item = Type> + '_> {
    Ok(match ty {
        TypeOrFuncType::FuncType(t) => Either::A(func_type_at(resources, t)?.inputs()),
        TypeOrFuncType::Type(_) => Either::B(None.into_iter()),
    })
}

fn results(
    ty: TypeOrFuncType,
    resources: &impl WasmModuleResources,
) -> OperatorValidatorResult<impl PreciseIterator<Item = Type> + '_> {
    Ok(match ty {
        TypeOrFuncType::FuncType(t) => Either::A(func_type_at(resources, t)?.outputs()),
        TypeOrFuncType::Type(Type::EmptyBlockType) => Either::B(None.into_iter()),
        TypeOrFuncType::Type(t) => Either::B(Some(t).into_iter()),
    })
}

fn label_types(
    ty: TypeOrFuncType,
    resources: &impl WasmModuleResources,
    kind: FrameKind,
) -> OperatorValidatorResult<impl PreciseIterator<Item = Type> + '_> {
    Ok(match kind {
        FrameKind::Loop => Either::A(params(ty, resources)?),
        _ => Either::B(results(ty, resources)?),
    })
}

fn ty_to_str(ty: Type) -> &'static str {
    match ty {
        Type::I32 => "i32",
        Type::I64 => "i64",
        Type::F32 => "f32",
        Type::F64 => "f64",
        Type::V128 => "v128",
        Type::FuncRef => "funcref",
        Type::ExternRef => "externref",
        Type::ExnRef => "exnref",
        Type::Func => "func",
        Type::EmptyBlockType => "nil",
    }
}
