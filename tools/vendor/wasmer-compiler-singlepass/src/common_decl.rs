use std::collections::BTreeMap;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct RegisterIndex(pub usize);

/// Whether a value is determined at compile-time or run-time.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum WasmAbstractValue {
    /// This value is only known at runtime.
    Runtime,
    /// A constant value.
    Const(u64),
}

/// A container for the state of a running wasm instance.
#[derive(Clone, Debug)]
pub struct MachineState {
    /// Stack values.
    pub stack_values: Vec<MachineValue>,
    /// Register values.
    pub register_values: Vec<MachineValue>,
    /// Previous frame.
    pub prev_frame: BTreeMap<usize, MachineValue>,
    /// Wasm stack.
    pub wasm_stack: Vec<WasmAbstractValue>,
    /// Wasm instruction offset.
    pub wasm_inst_offset: usize,
}

/// A diff of two `MachineState`s.
///
/// A `MachineStateDiff` can only be applied after the `MachineStateDiff` its `last` field
/// points to is already applied.
#[derive(Clone, Debug, Default)]
pub struct MachineStateDiff {
    /// Link to the previous diff this diff is based on, or `None` if this is the first diff.
    pub last: Option<usize>,

    /// What values are pushed onto the stack?
    pub stack_push: Vec<MachineValue>,

    /// How many values are popped from the stack?
    pub stack_pop: usize,

    /// Register diff.
    pub reg_diff: Vec<(RegisterIndex, MachineValue)>,

    /// Changes in the previous frame's data.
    pub prev_frame_diff: BTreeMap<usize, Option<MachineValue>>, // None for removal

    /// Values pushed to the Wasm stack.
    pub wasm_stack_push: Vec<WasmAbstractValue>,

    /// # of values popped from the Wasm stack.
    pub wasm_stack_pop: usize,

    /// Wasm instruction offset.
    pub wasm_inst_offset: usize, // absolute value; not a diff.
}

/// A kind of machine value.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum MachineValue {
    /// Undefined.
    Undefined,
    /// Vmctx.
    Vmctx,
    /// Vmctx Deref.
    _VmctxDeref(Vec<usize>),
    /// Preserve Register.
    PreserveRegister(RegisterIndex),
    /// Copy Stack BP Relative.
    CopyStackBPRelative(i32), // relative to Base Pointer, in byte offset
    /// Explicit Shadow.
    ExplicitShadow, // indicates that all values above this are above the shadow region
    /// Wasm Stack.
    WasmStack(usize),
    /// Wasm Local.
    WasmLocal(usize),
    /// Two Halves.
    _TwoHalves(Box<(MachineValue, MachineValue)>), // 32-bit values. TODO: optimize: add another type for inner "half" value to avoid boxing?
}

/// A map of function states.
#[derive(Clone, Debug)]
pub struct FunctionStateMap {
    /// Initial.
    pub initial: MachineState,
    /// Local Function Id.
    pub local_function_id: usize,
    /// Locals.
    pub locals: Vec<WasmAbstractValue>,
    /// Shadow size.
    pub shadow_size: usize, // for single-pass backend, 32 bytes on x86-64
    /// Diffs.
    pub diffs: Vec<MachineStateDiff>,
    /// Wasm Function Header target offset.
    pub wasm_function_header_target_offset: Option<SuspendOffset>,
    /// Wasm offset to target offset
    pub wasm_offset_to_target_offset: BTreeMap<usize, SuspendOffset>,
    /// Loop offsets.
    pub loop_offsets: BTreeMap<usize, OffsetInfo>, /* suspend_offset -> info */
    /// Call offsets.
    pub call_offsets: BTreeMap<usize, OffsetInfo>, /* suspend_offset -> info */
    /// Trappable offsets.
    pub trappable_offsets: BTreeMap<usize, OffsetInfo>, /* suspend_offset -> info */
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Size {
    S8,
    S16,
    S32,
    S64,
}

/// A kind of suspend offset.
#[derive(Clone, Copy, Debug)]
pub enum SuspendOffset {
    /// A loop.
    _Loop(usize),
    /// A call.
    Call(usize),
    /// A trappable.
    Trappable(usize),
}

/// Description of a machine code range following an offset.
#[derive(Clone, Debug)]
pub struct OffsetInfo {
    /// Exclusive range-end offset.
    pub end_offset: usize,
    /// Index pointing to the `MachineStateDiff` entry.
    pub diff_id: usize,
    /// Offset at which execution can be continued.
    pub activate_offset: usize,
}

impl FunctionStateMap {
    /// Creates a new `FunctionStateMap` with the given parameters.
    pub fn new(
        initial: MachineState,
        local_function_id: usize,
        shadow_size: usize,
        locals: Vec<WasmAbstractValue>,
    ) -> FunctionStateMap {
        FunctionStateMap {
            initial,
            local_function_id,
            shadow_size,
            locals,
            diffs: vec![],
            wasm_function_header_target_offset: None,
            wasm_offset_to_target_offset: BTreeMap::new(),
            loop_offsets: BTreeMap::new(),
            call_offsets: BTreeMap::new(),
            trappable_offsets: BTreeMap::new(),
        }
    }
}

impl MachineState {
    /// Creates a `MachineStateDiff` from self and the given `&MachineState`.
    pub fn diff(&self, old: &MachineState) -> MachineStateDiff {
        let first_diff_stack_depth: usize = self
            .stack_values
            .iter()
            .zip(old.stack_values.iter())
            .enumerate()
            .find(|&(_, (a, b))| a != b)
            .map(|x| x.0)
            .unwrap_or_else(|| old.stack_values.len().min(self.stack_values.len()));
        assert_eq!(self.register_values.len(), old.register_values.len());
        let reg_diff: Vec<_> = self
            .register_values
            .iter()
            .zip(old.register_values.iter())
            .enumerate()
            .filter(|&(_, (a, b))| a != b)
            .map(|(i, (a, _))| (RegisterIndex(i), a.clone()))
            .collect();
        let prev_frame_diff: BTreeMap<usize, Option<MachineValue>> = self
            .prev_frame
            .iter()
            .filter(|(k, v)| {
                if let Some(ref old_v) = old.prev_frame.get(k) {
                    v != old_v
                } else {
                    true
                }
            })
            .map(|(&k, v)| (k, Some(v.clone())))
            .chain(
                old.prev_frame
                    .iter()
                    .filter(|(k, _)| self.prev_frame.get(k).is_none())
                    .map(|(&k, _)| (k, None)),
            )
            .collect();
        let first_diff_wasm_stack_depth: usize = self
            .wasm_stack
            .iter()
            .zip(old.wasm_stack.iter())
            .enumerate()
            .find(|&(_, (a, b))| a != b)
            .map(|x| x.0)
            .unwrap_or_else(|| old.wasm_stack.len().min(self.wasm_stack.len()));
        MachineStateDiff {
            last: None,
            stack_push: self.stack_values[first_diff_stack_depth..].to_vec(),
            stack_pop: old.stack_values.len() - first_diff_stack_depth,
            reg_diff,

            prev_frame_diff,

            wasm_stack_push: self.wasm_stack[first_diff_wasm_stack_depth..].to_vec(),
            wasm_stack_pop: old.wasm_stack.len() - first_diff_wasm_stack_depth,

            wasm_inst_offset: self.wasm_inst_offset,
        }
    }
}

impl MachineStateDiff {
    /// Creates a `MachineState` from the given `&FunctionStateMap`.
    pub fn _build_state(&self, m: &FunctionStateMap) -> MachineState {
        let mut chain: Vec<&MachineStateDiff> = vec![self];
        let mut current = self.last;
        while let Some(x) = current {
            let that = &m.diffs[x];
            current = that.last;
            chain.push(that);
        }
        chain.reverse();
        let mut state = m.initial.clone();
        for x in chain {
            for _ in 0..x.stack_pop {
                state.stack_values.pop().unwrap();
            }
            for v in &x.stack_push {
                state.stack_values.push(v.clone());
            }
            for &(index, ref v) in &x.reg_diff {
                state.register_values[index.0] = v.clone();
            }
            for (index, ref v) in &x.prev_frame_diff {
                if let Some(ref x) = v {
                    state.prev_frame.insert(*index, x.clone());
                } else {
                    state.prev_frame.remove(index).unwrap();
                }
            }
            for _ in 0..x.wasm_stack_pop {
                state.wasm_stack.pop().unwrap();
            }
            for v in &x.wasm_stack_push {
                state.wasm_stack.push(*v);
            }
        }
        state.wasm_inst_offset = self.wasm_inst_offset;
        state
    }
}
