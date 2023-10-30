//! Converting Cranelift IR to text.
//!
//! The `write` module provides the `write_function` function which converts an IR `Function` to an
//! equivalent textual form. This textual form can be read back by the `cranelift-reader` crate.

use crate::entity::SecondaryMap;
use crate::ir::entities::AnyEntity;
use crate::ir::{Block, DataFlowGraph, Function, Inst, SigRef, Type, Value, ValueDef};
use crate::packed_option::ReservedValue;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{self, Write};

/// A `FuncWriter` used to decorate functions during printing.
pub trait FuncWriter {
    /// Write the basic block header for the current function.
    fn write_block_header(
        &mut self,
        w: &mut dyn Write,
        func: &Function,
        block: Block,
        indent: usize,
    ) -> fmt::Result;

    /// Write the given `inst` to `w`.
    fn write_instruction(
        &mut self,
        w: &mut dyn Write,
        func: &Function,
        aliases: &SecondaryMap<Value, Vec<Value>>,
        inst: Inst,
        indent: usize,
    ) -> fmt::Result;

    /// Write the preamble to `w`. By default, this uses `write_entity_definition`.
    fn write_preamble(&mut self, w: &mut dyn Write, func: &Function) -> Result<bool, fmt::Error> {
        self.super_preamble(w, func)
    }

    /// Default impl of `write_preamble`
    fn super_preamble(&mut self, w: &mut dyn Write, func: &Function) -> Result<bool, fmt::Error> {
        let mut any = false;

        for (ss, slot) in func.stack_slots.iter() {
            any = true;
            self.write_entity_definition(w, func, ss.into(), slot)?;
        }

        for (gv, gv_data) in &func.global_values {
            any = true;
            self.write_entity_definition(w, func, gv.into(), gv_data)?;
        }

        for (heap, heap_data) in &func.heaps {
            if !heap_data.index_type.is_invalid() {
                any = true;
                self.write_entity_definition(w, func, heap.into(), heap_data)?;
            }
        }

        for (table, table_data) in &func.tables {
            if !table_data.index_type.is_invalid() {
                any = true;
                self.write_entity_definition(w, func, table.into(), table_data)?;
            }
        }

        // Write out all signatures before functions since function declarations can refer to
        // signatures.
        for (sig, sig_data) in &func.dfg.signatures {
            any = true;
            self.write_entity_definition(w, func, sig.into(), &sig_data)?;
        }

        for (fnref, ext_func) in &func.dfg.ext_funcs {
            if ext_func.signature != SigRef::reserved_value() {
                any = true;
                self.write_entity_definition(w, func, fnref.into(), ext_func)?;
            }
        }

        for (jt, jt_data) in &func.jump_tables {
            any = true;
            self.write_entity_definition(w, func, jt.into(), jt_data)?;
        }

        for (&cref, cval) in func.dfg.constants.iter() {
            any = true;
            self.write_entity_definition(w, func, cref.into(), cval)?;
        }

        if let Some(limit) = func.stack_limit {
            any = true;
            self.write_entity_definition(w, func, AnyEntity::StackLimit, &limit)?;
        }

        Ok(any)
    }

    /// Write an entity definition defined in the preamble to `w`.
    fn write_entity_definition(
        &mut self,
        w: &mut dyn Write,
        func: &Function,
        entity: AnyEntity,
        value: &dyn fmt::Display,
    ) -> fmt::Result {
        self.super_entity_definition(w, func, entity, value)
    }

    /// Default impl of `write_entity_definition`
    #[allow(unused_variables)]
    fn super_entity_definition(
        &mut self,
        w: &mut dyn Write,
        func: &Function,
        entity: AnyEntity,
        value: &dyn fmt::Display,
    ) -> fmt::Result {
        writeln!(w, "    {} = {}", entity, value)
    }
}

/// A `PlainWriter` that doesn't decorate the function.
pub struct PlainWriter;

impl FuncWriter for PlainWriter {
    fn write_instruction(
        &mut self,
        w: &mut dyn Write,
        func: &Function,
        aliases: &SecondaryMap<Value, Vec<Value>>,
        inst: Inst,
        indent: usize,
    ) -> fmt::Result {
        write_instruction(w, func, aliases, inst, indent)
    }

    fn write_block_header(
        &mut self,
        w: &mut dyn Write,
        func: &Function,
        block: Block,
        indent: usize,
    ) -> fmt::Result {
        write_block_header(w, func, block, indent)
    }
}

/// Write `func` to `w` as equivalent text.
/// Use `isa` to emit ISA-dependent annotations.
pub fn write_function(w: &mut dyn Write, func: &Function) -> fmt::Result {
    decorate_function(&mut PlainWriter, w, func)
}

/// Create a reverse-alias map from a value to all aliases having that value as a direct target
fn alias_map(func: &Function) -> SecondaryMap<Value, Vec<Value>> {
    let mut aliases = SecondaryMap::<_, Vec<_>>::new();
    for v in func.dfg.values() {
        // VADFS returns the immediate target of an alias
        if let Some(k) = func.dfg.value_alias_dest_for_serialization(v) {
            aliases[k].push(v);
        }
    }
    aliases
}

/// Writes `func` to `w` as text.
/// write_function_plain is passed as 'closure' to print instructions as text.
/// pretty_function_error is passed as 'closure' to add error decoration.
pub fn decorate_function<FW: FuncWriter>(
    func_w: &mut FW,
    w: &mut dyn Write,
    func: &Function,
) -> fmt::Result {
    write!(w, "function ")?;
    write_spec(w, func)?;
    writeln!(w, " {{")?;
    let aliases = alias_map(func);
    let mut any = func_w.write_preamble(w, func)?;
    for block in &func.layout {
        if any {
            writeln!(w)?;
        }
        decorate_block(func_w, w, func, &aliases, block)?;
        any = true;
    }
    writeln!(w, "}}")
}

//----------------------------------------------------------------------
//
// Function spec.

fn write_spec(w: &mut dyn Write, func: &Function) -> fmt::Result {
    write!(w, "{}{}", func.name, func.signature)
}

//----------------------------------------------------------------------
//
// Basic blocks

fn write_arg(w: &mut dyn Write, func: &Function, arg: Value) -> fmt::Result {
    write!(w, "{}: {}", arg, func.dfg.value_type(arg))
}

/// Write out the basic block header, outdented:
///
///    block1:
///    block1(v1: i32):
///    block10(v4: f64, v5: b1):
///
pub fn write_block_header(
    w: &mut dyn Write,
    func: &Function,
    block: Block,
    indent: usize,
) -> fmt::Result {
    let cold = if func.layout.is_cold(block) {
        " cold"
    } else {
        ""
    };

    // The `indent` is the instruction indentation. block headers are 4 spaces out from that.
    write!(w, "{1:0$}{2}", indent - 4, "", block)?;

    let mut args = func.dfg.block_params(block).iter().cloned();
    match args.next() {
        None => return writeln!(w, "{}:", cold),
        Some(arg) => {
            write!(w, "(")?;
            write_arg(w, func, arg)?;
        }
    }
    // Remaining arguments.
    for arg in args {
        write!(w, ", ")?;
        write_arg(w, func, arg)?;
    }
    writeln!(w, "){}:", cold)
}

fn decorate_block<FW: FuncWriter>(
    func_w: &mut FW,
    w: &mut dyn Write,
    func: &Function,
    aliases: &SecondaryMap<Value, Vec<Value>>,
    block: Block,
) -> fmt::Result {
    // Indent all instructions if any srclocs are present.
    let indent = if func.srclocs.is_empty() { 4 } else { 36 };

    func_w.write_block_header(w, func, block, indent)?;
    for a in func.dfg.block_params(block).iter().cloned() {
        write_value_aliases(w, aliases, a, indent)?;
    }

    for inst in func.layout.block_insts(block) {
        func_w.write_instruction(w, func, aliases, inst, indent)?;
    }

    Ok(())
}

//----------------------------------------------------------------------
//
// Instructions

// Should `inst` be printed with a type suffix?
//
// Polymorphic instructions may need a suffix indicating the value of the controlling type variable
// if it can't be trivially inferred.
//
fn type_suffix(func: &Function, inst: Inst) -> Option<Type> {
    let inst_data = &func.dfg[inst];
    let constraints = inst_data.opcode().constraints();

    if !constraints.is_polymorphic() {
        return None;
    }

    // If the controlling type variable can be inferred from the type of the designated value input
    // operand, we don't need the type suffix.
    if constraints.use_typevar_operand() {
        let ctrl_var = inst_data.typevar_operand(&func.dfg.value_lists).unwrap();
        let def_block = match func.dfg.value_def(ctrl_var) {
            ValueDef::Result(instr, _) => func.layout.inst_block(instr),
            ValueDef::Param(block, _) => Some(block),
        };
        if def_block.is_some() && def_block == func.layout.inst_block(inst) {
            return None;
        }
    }

    let rtype = func.dfg.ctrl_typevar(inst);
    assert!(
        !rtype.is_invalid(),
        "Polymorphic instruction must produce a result"
    );
    Some(rtype)
}

/// Write out any aliases to the given target, including indirect aliases
fn write_value_aliases(
    w: &mut dyn Write,
    aliases: &SecondaryMap<Value, Vec<Value>>,
    target: Value,
    indent: usize,
) -> fmt::Result {
    let mut todo_stack = vec![target];
    while let Some(target) = todo_stack.pop() {
        for &a in &aliases[target] {
            writeln!(w, "{1:0$}{2} -> {3}", indent, "", a, target)?;
            todo_stack.push(a);
        }
    }

    Ok(())
}

fn write_instruction(
    w: &mut dyn Write,
    func: &Function,
    aliases: &SecondaryMap<Value, Vec<Value>>,
    inst: Inst,
    indent: usize,
) -> fmt::Result {
    // Prefix containing source location, encoding, and value locations.
    let mut s = String::with_capacity(16);

    // Source location goes first.
    let srcloc = func.srclocs[inst];
    if !srcloc.is_default() {
        write!(s, "{} ", srcloc)?;
    }

    // Write out prefix and indent the instruction.
    write!(w, "{1:0$}", indent, s)?;

    // Write out the result values, if any.
    let mut has_results = false;
    for r in func.dfg.inst_results(inst) {
        if !has_results {
            has_results = true;
            write!(w, "{}", r)?;
        } else {
            write!(w, ", {}", r)?;
        }
    }
    if has_results {
        write!(w, " = ")?;
    }

    // Then the opcode, possibly with a '.type' suffix.
    let opcode = func.dfg[inst].opcode();

    match type_suffix(func, inst) {
        Some(suf) => write!(w, "{}.{}", opcode, suf)?,
        None => write!(w, "{}", opcode)?,
    }

    write_operands(w, &func.dfg, inst)?;
    writeln!(w)?;

    // Value aliases come out on lines after the instruction defining the referent.
    for r in func.dfg.inst_results(inst) {
        write_value_aliases(w, aliases, *r, indent)?;
    }
    Ok(())
}

/// Write the operands of `inst` to `w` with a prepended space.
pub fn write_operands(w: &mut dyn Write, dfg: &DataFlowGraph, inst: Inst) -> fmt::Result {
    let pool = &dfg.value_lists;
    use crate::ir::instructions::InstructionData::*;
    match dfg[inst] {
        AtomicRmw { op, args, .. } => write!(w, " {}, {}, {}", op, args[0], args[1]),
        AtomicCas { args, .. } => write!(w, " {}, {}, {}", args[0], args[1], args[2]),
        LoadNoOffset { flags, arg, .. } => write!(w, "{} {}", flags, arg),
        StoreNoOffset { flags, args, .. } => write!(w, "{} {}, {}", flags, args[0], args[1]),
        Unary { arg, .. } => write!(w, " {}", arg),
        UnaryImm { imm, .. } => write!(w, " {}", imm),
        UnaryIeee32 { imm, .. } => write!(w, " {}", imm),
        UnaryIeee64 { imm, .. } => write!(w, " {}", imm),
        UnaryBool { imm, .. } => write!(w, " {}", imm),
        UnaryGlobalValue { global_value, .. } => write!(w, " {}", global_value),
        UnaryConst {
            constant_handle, ..
        } => write!(w, " {}", constant_handle),
        Binary { args, .. } => write!(w, " {}, {}", args[0], args[1]),
        BinaryImm8 { arg, imm, .. } => write!(w, " {}, {}", arg, imm),
        BinaryImm64 { arg, imm, .. } => write!(w, " {}, {}", arg, imm),
        Ternary { args, .. } => write!(w, " {}, {}, {}", args[0], args[1], args[2]),
        MultiAry { ref args, .. } => {
            if args.is_empty() {
                write!(w, "")
            } else {
                write!(w, " {}", DisplayValues(args.as_slice(pool)))
            }
        }
        NullAry { .. } => write!(w, " "),
        TernaryImm8 { imm, args, .. } => write!(w, " {}, {}, {}", args[0], args[1], imm),
        Shuffle { imm, args, .. } => {
            let data = dfg.immediates.get(imm).expect(
                "Expected the shuffle mask to already be inserted into the immediates table",
            );
            write!(w, " {}, {}, {}", args[0], args[1], data)
        }
        IntCompare { cond, args, .. } => write!(w, " {} {}, {}", cond, args[0], args[1]),
        IntCompareImm { cond, arg, imm, .. } => write!(w, " {} {}, {}", cond, arg, imm),
        IntCond { cond, arg, .. } => write!(w, " {} {}", cond, arg),
        FloatCompare { cond, args, .. } => write!(w, " {} {}, {}", cond, args[0], args[1]),
        FloatCond { cond, arg, .. } => write!(w, " {} {}", cond, arg),
        IntSelect { cond, args, .. } => {
            write!(w, " {} {}, {}, {}", cond, args[0], args[1], args[2])
        }
        Jump {
            destination,
            ref args,
            ..
        } => {
            write!(w, " {}", destination)?;
            write_block_args(w, args.as_slice(pool))
        }
        Branch {
            destination,
            ref args,
            ..
        } => {
            let args = args.as_slice(pool);
            write!(w, " {}, {}", args[0], destination)?;
            write_block_args(w, &args[1..])
        }
        BranchInt {
            cond,
            destination,
            ref args,
            ..
        } => {
            let args = args.as_slice(pool);
            write!(w, " {} {}, {}", cond, args[0], destination)?;
            write_block_args(w, &args[1..])
        }
        BranchFloat {
            cond,
            destination,
            ref args,
            ..
        } => {
            let args = args.as_slice(pool);
            write!(w, " {} {}, {}", cond, args[0], destination)?;
            write_block_args(w, &args[1..])
        }
        BranchIcmp {
            cond,
            destination,
            ref args,
            ..
        } => {
            let args = args.as_slice(pool);
            write!(w, " {} {}, {}, {}", cond, args[0], args[1], destination)?;
            write_block_args(w, &args[2..])
        }
        BranchTable {
            arg,
            destination,
            table,
            ..
        } => write!(w, " {}, {}, {}", arg, destination, table),
        Call {
            func_ref, ref args, ..
        } => write!(w, " {}({})", func_ref, DisplayValues(args.as_slice(pool))),
        CallIndirect {
            sig_ref, ref args, ..
        } => {
            let args = args.as_slice(pool);
            write!(
                w,
                " {}, {}({})",
                sig_ref,
                args[0],
                DisplayValues(&args[1..])
            )
        }
        FuncAddr { func_ref, .. } => write!(w, " {}", func_ref),
        StackLoad {
            stack_slot, offset, ..
        } => write!(w, " {}{}", stack_slot, offset),
        StackStore {
            arg,
            stack_slot,
            offset,
            ..
        } => write!(w, " {}, {}{}", arg, stack_slot, offset),
        HeapAddr { heap, arg, imm, .. } => write!(w, " {}, {}, {}", heap, arg, imm),
        TableAddr { table, arg, .. } => write!(w, " {}, {}", table, arg),
        Load {
            flags, arg, offset, ..
        } => write!(w, "{} {}{}", flags, arg, offset),
        LoadComplex {
            flags,
            ref args,
            offset,
            ..
        } => {
            let args = args.as_slice(pool);
            write!(
                w,
                "{} {}{}",
                flags,
                DisplayValuesWithDelimiter(&args, '+'),
                offset
            )
        }
        Store {
            flags,
            args,
            offset,
            ..
        } => write!(w, "{} {}, {}{}", flags, args[0], args[1], offset),
        StoreComplex {
            flags,
            ref args,
            offset,
            ..
        } => {
            let args = args.as_slice(pool);
            write!(
                w,
                "{} {}, {}{}",
                flags,
                args[0],
                DisplayValuesWithDelimiter(&args[1..], '+'),
                offset
            )
        }
        Trap { code, .. } => write!(w, " {}", code),
        CondTrap { arg, code, .. } => write!(w, " {}, {}", arg, code),
        IntCondTrap {
            cond, arg, code, ..
        } => write!(w, " {} {}, {}", cond, arg, code),
        FloatCondTrap {
            cond, arg, code, ..
        } => write!(w, " {} {}, {}", cond, arg, code),
    }
}

/// Write block args using optional parantheses.
fn write_block_args(w: &mut dyn Write, args: &[Value]) -> fmt::Result {
    if args.is_empty() {
        Ok(())
    } else {
        write!(w, "({})", DisplayValues(args))
    }
}

/// Displayable slice of values.
struct DisplayValues<'a>(&'a [Value]);

impl<'a> fmt::Display for DisplayValues<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, val) in self.0.iter().enumerate() {
            if i == 0 {
                write!(f, "{}", val)?;
            } else {
                write!(f, ", {}", val)?;
            }
        }
        Ok(())
    }
}

struct DisplayValuesWithDelimiter<'a>(&'a [Value], char);

impl<'a> fmt::Display for DisplayValuesWithDelimiter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, val) in self.0.iter().enumerate() {
            if i == 0 {
                write!(f, "{}", val)?;
            } else {
                write!(f, "{}{}", self.1, val)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::cursor::{Cursor, CursorPosition, FuncCursor};
    use crate::ir::types;
    use crate::ir::{ExternalName, Function, InstBuilder, StackSlotData, StackSlotKind};
    use alloc::string::ToString;

    #[test]
    fn basic() {
        let mut f = Function::new();
        assert_eq!(f.to_string(), "function u0:0() fast {\n}\n");

        f.name = ExternalName::testcase("foo");
        assert_eq!(f.to_string(), "function %foo() fast {\n}\n");

        f.create_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 4));
        assert_eq!(
            f.to_string(),
            "function %foo() fast {\n    ss0 = explicit_slot 4\n}\n"
        );

        let block = f.dfg.make_block();
        f.layout.append_block(block);
        assert_eq!(
            f.to_string(),
            "function %foo() fast {\n    ss0 = explicit_slot 4\n\nblock0:\n}\n"
        );

        f.dfg.append_block_param(block, types::I8);
        assert_eq!(
            f.to_string(),
            "function %foo() fast {\n    ss0 = explicit_slot 4\n\nblock0(v0: i8):\n}\n"
        );

        f.dfg.append_block_param(block, types::F32.by(4).unwrap());
        assert_eq!(
            f.to_string(),
            "function %foo() fast {\n    ss0 = explicit_slot 4\n\nblock0(v0: i8, v1: f32x4):\n}\n"
        );

        {
            let mut cursor = FuncCursor::new(&mut f);
            cursor.set_position(CursorPosition::After(block));
            cursor.ins().return_(&[])
        };
        assert_eq!(
            f.to_string(),
            "function %foo() fast {\n    ss0 = explicit_slot 4\n\nblock0(v0: i8, v1: f32x4):\n    return\n}\n"
        );
    }

    #[test]
    fn aliases() {
        use crate::ir::InstBuilder;

        let mut func = Function::new();
        {
            let block0 = func.dfg.make_block();
            let mut pos = FuncCursor::new(&mut func);
            pos.insert_block(block0);

            // make some detached values for change_to_alias
            let v0 = pos.func.dfg.append_block_param(block0, types::I32);
            let v1 = pos.func.dfg.append_block_param(block0, types::I32);
            let v2 = pos.func.dfg.append_block_param(block0, types::I32);
            pos.func.dfg.detach_block_params(block0);

            // alias to a param--will be printed at beginning of block defining param
            let v3 = pos.func.dfg.append_block_param(block0, types::I32);
            pos.func.dfg.change_to_alias(v0, v3);

            // alias to an alias--should print attached to alias, not ultimate target
            pos.func.dfg.make_value_alias_for_serialization(v0, v2); // v0 <- v2

            // alias to a result--will be printed after instruction producing result
            let _dummy0 = pos.ins().iconst(types::I32, 42);
            let v4 = pos.ins().iadd(v0, v0);
            pos.func.dfg.change_to_alias(v1, v4);
            let _dummy1 = pos.ins().iconst(types::I32, 23);
            let _v7 = pos.ins().iadd(v1, v1);
        }
        assert_eq!(
            func.to_string(),
            "function u0:0() fast {\nblock0(v3: i32):\n    v0 -> v3\n    v2 -> v0\n    v4 = iconst.i32 42\n    v5 = iadd v0, v0\n    v1 -> v5\n    v6 = iconst.i32 23\n    v7 = iadd v1, v1\n}\n"
        );
    }

    #[test]
    fn cold_blocks() {
        let mut func = Function::new();
        {
            let mut pos = FuncCursor::new(&mut func);

            let block0 = pos.func.dfg.make_block();
            pos.insert_block(block0);
            pos.func.layout.set_cold(block0);

            let block1 = pos.func.dfg.make_block();
            pos.insert_block(block1);
            pos.func.dfg.append_block_param(block1, types::I32);
            pos.func.layout.set_cold(block1);
        }

        assert_eq!(
            func.to_string(),
            "function u0:0() fast {\nblock0 cold:\n\nblock1(v0: i32) cold:\n}\n"
        );
    }
}
