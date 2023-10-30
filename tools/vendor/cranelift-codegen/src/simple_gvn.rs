//! A simple GVN pass.

use crate::cursor::{Cursor, FuncCursor};
use crate::dominator_tree::DominatorTree;
use crate::ir::{Function, Inst, InstructionData, Opcode, Type};
use crate::scoped_hash_map::ScopedHashMap;
use crate::timing;
use alloc::vec::Vec;
use core::cell::{Ref, RefCell};
use core::hash::{Hash, Hasher};

/// Test whether the given opcode is unsafe to even consider for GVN.
fn trivially_unsafe_for_gvn(opcode: Opcode) -> bool {
    opcode.is_call()
        || opcode.is_branch()
        || opcode.is_terminator()
        || opcode.is_return()
        || opcode.can_trap()
        || opcode.other_side_effects()
        || opcode.can_store()
        || opcode.writes_cpu_flags()
}

/// Test that, if the specified instruction is a load, it doesn't have the `readonly` memflag.
fn is_load_and_not_readonly(inst_data: &InstructionData) -> bool {
    match *inst_data {
        InstructionData::Load { flags, .. } | InstructionData::LoadComplex { flags, .. } => {
            !flags.readonly()
        }
        _ => inst_data.opcode().can_load(),
    }
}

/// Wrapper around `InstructionData` which implements `Eq` and `Hash`
#[derive(Clone)]
struct HashKey<'a, 'f: 'a> {
    inst: InstructionData,
    ty: Type,
    pos: &'a RefCell<FuncCursor<'f>>,
}
impl<'a, 'f: 'a> Hash for HashKey<'a, 'f> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let pool = &self.pos.borrow().func.dfg.value_lists;
        self.inst.hash(state, pool);
        self.ty.hash(state);
    }
}
impl<'a, 'f: 'a> PartialEq for HashKey<'a, 'f> {
    fn eq(&self, other: &Self) -> bool {
        let pool = &self.pos.borrow().func.dfg.value_lists;
        self.inst.eq(&other.inst, pool) && self.ty == other.ty
    }
}
impl<'a, 'f: 'a> Eq for HashKey<'a, 'f> {}

/// Perform simple GVN on `func`.
///
pub fn do_simple_gvn(func: &mut Function, domtree: &mut DominatorTree) {
    let _tt = timing::gvn();
    debug_assert!(domtree.is_valid());

    // Visit blocks in a reverse post-order.
    //
    // The RefCell here is a bit ugly since the HashKeys in the ScopedHashMap
    // need a reference to the function.
    let pos = RefCell::new(FuncCursor::new(func));

    let mut visible_values: ScopedHashMap<HashKey, Inst> = ScopedHashMap::new();
    let mut scope_stack: Vec<Inst> = Vec::new();

    for &block in domtree.cfg_postorder().iter().rev() {
        {
            // Pop any scopes that we just exited.
            let layout = &pos.borrow().func.layout;
            loop {
                if let Some(current) = scope_stack.last() {
                    if domtree.dominates(*current, block, layout) {
                        break;
                    }
                } else {
                    break;
                }
                scope_stack.pop();
                visible_values.decrement_depth();
            }

            // Push a scope for the current block.
            scope_stack.push(layout.first_inst(block).unwrap());
            visible_values.increment_depth();
        }

        pos.borrow_mut().goto_top(block);
        while let Some(inst) = {
            let mut pos = pos.borrow_mut();
            pos.next_inst()
        } {
            // Resolve aliases, particularly aliases we created earlier.
            pos.borrow_mut().func.dfg.resolve_aliases_in_arguments(inst);

            let func = Ref::map(pos.borrow(), |pos| &pos.func);

            let opcode = func.dfg[inst].opcode();

            if opcode.is_branch() && !opcode.is_terminator() {
                scope_stack.push(func.layout.next_inst(inst).unwrap());
                visible_values.increment_depth();
            }

            if trivially_unsafe_for_gvn(opcode) {
                continue;
            }

            // These are split up to separate concerns.
            if is_load_and_not_readonly(&func.dfg[inst]) {
                continue;
            }

            let ctrl_typevar = func.dfg.ctrl_typevar(inst);
            let key = HashKey {
                inst: func.dfg[inst].clone(),
                ty: ctrl_typevar,
                pos: &pos,
            };
            use crate::scoped_hash_map::Entry::*;
            match visible_values.entry(key) {
                Occupied(entry) => {
                    #[allow(clippy::debug_assert_with_mut_call)]
                    {
                        // Clippy incorrectly believes `&func.layout` should not be used here:
                        // https://github.com/rust-lang/rust-clippy/issues/4737
                        debug_assert!(domtree.dominates(*entry.get(), inst, &func.layout));
                    }

                    // If the redundant instruction is representing the current
                    // scope, pick a new representative.
                    let old = scope_stack.last_mut().unwrap();
                    if *old == inst {
                        *old = func.layout.next_inst(inst).unwrap();
                    }
                    // Replace the redundant instruction and remove it.
                    drop(func);
                    let mut pos = pos.borrow_mut();
                    pos.func.dfg.replace_with_aliases(inst, *entry.get());
                    pos.remove_inst_and_step_back();
                }
                Vacant(entry) => {
                    entry.insert(inst);
                }
            }
        }
    }
}
