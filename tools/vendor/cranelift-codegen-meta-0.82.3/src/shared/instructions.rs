#![allow(non_snake_case)]

use crate::cdsl::instructions::{
    AllInstructions, InstructionBuilder as Inst, InstructionGroupBuilder,
};
use crate::cdsl::operands::Operand;
use crate::cdsl::types::{LaneType, ValueType};
use crate::cdsl::typevar::{Interval, TypeSetBuilder, TypeVar};
use crate::shared::formats::Formats;
use crate::shared::types;
use crate::shared::{entities::EntityRefs, immediates::Immediates};

#[inline(never)]
fn define_control_flow(
    ig: &mut InstructionGroupBuilder,
    formats: &Formats,
    imm: &Immediates,
    entities: &EntityRefs,
) {
    let block = &Operand::new("block", &entities.block).with_doc("Destination basic block");
    let args = &Operand::new("args", &entities.varargs).with_doc("block arguments");

    ig.push(
        Inst::new(
            "jump",
            r#"
        Jump.

        Unconditionally jump to a basic block, passing the specified
        block arguments. The number and types of arguments must match the
        destination block.
        "#,
            &formats.jump,
        )
        .operands_in(vec![block, args])
        .is_terminator(true)
        .is_branch(true),
    );

    let Testable = &TypeVar::new(
        "Testable",
        "A scalar boolean or integer type",
        TypeSetBuilder::new()
            .ints(Interval::All)
            .bools(Interval::All)
            .build(),
    );

    {
        let c = &Operand::new("c", Testable).with_doc("Controlling value to test");

        ig.push(
            Inst::new(
                "brz",
                r#"
        Branch when zero.

        If ``c`` is a `b1` value, take the branch when ``c`` is false. If
        ``c`` is an integer value, take the branch when ``c = 0``.
        "#,
                &formats.branch,
            )
            .operands_in(vec![c, block, args])
            .is_branch(true),
        );

        ig.push(
            Inst::new(
                "brnz",
                r#"
        Branch when non-zero.

        If ``c`` is a `b1` value, take the branch when ``c`` is true. If
        ``c`` is an integer value, take the branch when ``c != 0``.
        "#,
                &formats.branch,
            )
            .operands_in(vec![c, block, args])
            .is_branch(true),
        );
    }

    let iB = &TypeVar::new(
        "iB",
        "A scalar integer type",
        TypeSetBuilder::new().ints(Interval::All).build(),
    );
    let iflags: &TypeVar = &ValueType::Special(types::Flag::IFlags.into()).into();
    let fflags: &TypeVar = &ValueType::Special(types::Flag::FFlags.into()).into();

    {
        let Cond = &Operand::new("Cond", &imm.intcc);
        let x = &Operand::new("x", iB);
        let y = &Operand::new("y", iB);

        ig.push(
            Inst::new(
                "br_icmp",
                r#"
        Compare scalar integers and branch.

        Compare ``x`` and ``y`` in the same way as the `icmp` instruction
        and take the branch if the condition is true:

        ```text
            br_icmp ugt v1, v2, block4(v5, v6)
        ```

        is semantically equivalent to:

        ```text
            v10 = icmp ugt, v1, v2
            brnz v10, block4(v5, v6)
        ```

        Some RISC architectures like MIPS and RISC-V provide instructions that
        implement all or some of the condition codes. The instruction can also
        be used to represent *macro-op fusion* on architectures like Intel's.
        "#,
                &formats.branch_icmp,
            )
            .operands_in(vec![Cond, x, y, block, args])
            .is_branch(true),
        );

        let f = &Operand::new("f", iflags);

        ig.push(
            Inst::new(
                "brif",
                r#"
        Branch when condition is true in integer CPU flags.
        "#,
                &formats.branch_int,
            )
            .operands_in(vec![Cond, f, block, args])
            .is_branch(true),
        );
    }

    {
        let Cond = &Operand::new("Cond", &imm.floatcc);

        let f = &Operand::new("f", fflags);

        ig.push(
            Inst::new(
                "brff",
                r#"
        Branch when condition is true in floating point CPU flags.
        "#,
                &formats.branch_float,
            )
            .operands_in(vec![Cond, f, block, args])
            .is_branch(true),
        );
    }

    {
        let x = &Operand::new("x", iB).with_doc("index into jump table");
        let JT = &Operand::new("JT", &entities.jump_table);

        ig.push(
            Inst::new(
                "br_table",
                r#"
        Indirect branch via jump table.

        Use ``x`` as an unsigned index into the jump table ``JT``. If a jump
        table entry is found, branch to the corresponding block. If no entry was
        found or the index is out-of-bounds, branch to the given default block.

        Note that this branch instruction can't pass arguments to the targeted
        blocks. Split critical edges as needed to work around this.

        Do not confuse this with "tables" in WebAssembly. ``br_table`` is for
        jump tables with destinations within the current function only -- think
        of a ``match`` in Rust or a ``switch`` in C.  If you want to call a
        function in a dynamic library, that will typically use
        ``call_indirect``.
        "#,
                &formats.branch_table,
            )
            .operands_in(vec![x, block, JT])
            .is_terminator(true)
            .is_branch(true),
        );
    }

    let iAddr = &TypeVar::new(
        "iAddr",
        "An integer address type",
        TypeSetBuilder::new().ints(32..64).refs(32..64).build(),
    );

    ig.push(
        Inst::new(
            "debugtrap",
            r#"
    Encodes an assembly debug trap.
    "#,
            &formats.nullary,
        )
        .other_side_effects(true)
        .can_load(true)
        .can_store(true),
    );

    {
        let code = &Operand::new("code", &imm.trapcode);
        ig.push(
            Inst::new(
                "trap",
                r#"
        Terminate execution unconditionally.
        "#,
                &formats.trap,
            )
            .operands_in(vec![code])
            .can_trap(true)
            .is_terminator(true),
        );

        let c = &Operand::new("c", Testable).with_doc("Controlling value to test");
        ig.push(
            Inst::new(
                "trapz",
                r#"
        Trap when zero.

        if ``c`` is non-zero, execution continues at the following instruction.
        "#,
                &formats.cond_trap,
            )
            .operands_in(vec![c, code])
            .can_trap(true),
        );

        ig.push(
            Inst::new(
                "resumable_trap",
                r#"
        A resumable trap.

        This instruction allows non-conditional traps to be used as non-terminal instructions.
        "#,
                &formats.trap,
            )
            .operands_in(vec![code])
            .can_trap(true),
        );

        let c = &Operand::new("c", Testable).with_doc("Controlling value to test");
        ig.push(
            Inst::new(
                "trapnz",
                r#"
        Trap when non-zero.

        If ``c`` is zero, execution continues at the following instruction.
        "#,
                &formats.cond_trap,
            )
            .operands_in(vec![c, code])
            .can_trap(true),
        );

        ig.push(
            Inst::new(
                "resumable_trapnz",
                r#"
        A resumable trap to be called when the passed condition is non-zero.

        If ``c`` is zero, execution continues at the following instruction.
        "#,
                &formats.cond_trap,
            )
            .operands_in(vec![c, code])
            .can_trap(true),
        );

        let Cond = &Operand::new("Cond", &imm.intcc);
        let f = &Operand::new("f", iflags);
        ig.push(
            Inst::new(
                "trapif",
                r#"
        Trap when condition is true in integer CPU flags.
        "#,
                &formats.int_cond_trap,
            )
            .operands_in(vec![Cond, f, code])
            .can_trap(true),
        );

        let Cond = &Operand::new("Cond", &imm.floatcc);
        let f = &Operand::new("f", fflags);
        let code = &Operand::new("code", &imm.trapcode);
        ig.push(
            Inst::new(
                "trapff",
                r#"
        Trap when condition is true in floating point CPU flags.
        "#,
                &formats.float_cond_trap,
            )
            .operands_in(vec![Cond, f, code])
            .can_trap(true),
        );
    }

    let rvals = &Operand::new("rvals", &entities.varargs).with_doc("return values");
    ig.push(
        Inst::new(
            "return",
            r#"
        Return from the function.

        Unconditionally transfer control to the calling function, passing the
        provided return values. The list of return values must match the
        function signature's return types.
        "#,
            &formats.multiary,
        )
        .operands_in(vec![rvals])
        .is_return(true)
        .is_terminator(true),
    );

    let rvals = &Operand::new("rvals", &entities.varargs).with_doc("return values");
    ig.push(
        Inst::new(
            "fallthrough_return",
            r#"
        Return from the function by fallthrough.

        This is a specialized instruction for use where one wants to append
        a custom epilogue, which will then perform the real return. This
        instruction has no encoding.
        "#,
            &formats.multiary,
        )
        .operands_in(vec![rvals])
        .is_return(true)
        .is_terminator(true),
    );

    let FN = &Operand::new("FN", &entities.func_ref)
        .with_doc("function to call, declared by `function`");
    let args = &Operand::new("args", &entities.varargs).with_doc("call arguments");
    let rvals = &Operand::new("rvals", &entities.varargs).with_doc("return values");
    ig.push(
        Inst::new(
            "call",
            r#"
        Direct function call.

        Call a function which has been declared in the preamble. The argument
        types must match the function's signature.
        "#,
            &formats.call,
        )
        .operands_in(vec![FN, args])
        .operands_out(vec![rvals])
        .is_call(true),
    );

    let SIG = &Operand::new("SIG", &entities.sig_ref).with_doc("function signature");
    let callee = &Operand::new("callee", iAddr).with_doc("address of function to call");
    let args = &Operand::new("args", &entities.varargs).with_doc("call arguments");
    let rvals = &Operand::new("rvals", &entities.varargs).with_doc("return values");
    ig.push(
        Inst::new(
            "call_indirect",
            r#"
        Indirect function call.

        Call the function pointed to by `callee` with the given arguments. The
        called function must match the specified signature.

        Note that this is different from WebAssembly's ``call_indirect``; the
        callee is a native address, rather than a table index. For WebAssembly,
        `table_addr` and `load` are used to obtain a native address
        from a table.
        "#,
            &formats.call_indirect,
        )
        .operands_in(vec![SIG, callee, args])
        .operands_out(vec![rvals])
        .is_call(true),
    );

    let FN = &Operand::new("FN", &entities.func_ref)
        .with_doc("function to call, declared by `function`");
    let addr = &Operand::new("addr", iAddr);
    ig.push(
        Inst::new(
            "func_addr",
            r#"
        Get the address of a function.

        Compute the absolute address of a function declared in the preamble.
        The returned address can be used as a ``callee`` argument to
        `call_indirect`. This is also a method for calling functions that
        are too far away to be addressable by a direct `call`
        instruction.
        "#,
            &formats.func_addr,
        )
        .operands_in(vec![FN])
        .operands_out(vec![addr]),
    );
}

#[inline(never)]
fn define_simd_lane_access(
    ig: &mut InstructionGroupBuilder,
    formats: &Formats,
    imm: &Immediates,
    _: &EntityRefs,
) {
    let TxN = &TypeVar::new(
        "TxN",
        "A SIMD vector type",
        TypeSetBuilder::new()
            .ints(Interval::All)
            .floats(Interval::All)
            .bools(Interval::All)
            .simd_lanes(Interval::All)
            .includes_scalars(false)
            .build(),
    );

    let x = &Operand::new("x", &TxN.lane_of()).with_doc("Value to splat to all lanes");
    let a = &Operand::new("a", TxN);

    ig.push(
        Inst::new(
            "splat",
            r#"
        Vector splat.

        Return a vector whose lanes are all ``x``.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    let I8x16 = &TypeVar::new(
        "I8x16",
        "A SIMD vector type consisting of 16 lanes of 8-bit integers",
        TypeSetBuilder::new()
            .ints(8..8)
            .simd_lanes(16..16)
            .includes_scalars(false)
            .build(),
    );
    let x = &Operand::new("x", I8x16).with_doc("Vector to modify by re-arranging lanes");
    let y = &Operand::new("y", I8x16).with_doc("Mask for re-arranging lanes");

    ig.push(
        Inst::new(
            "swizzle",
            r#"
        Vector swizzle.

        Returns a new vector with byte-width lanes selected from the lanes of the first input
        vector ``x`` specified in the second input vector ``s``. The indices ``i`` in range
        ``[0, 15]`` select the ``i``-th element of ``x``. For indices outside of the range the
        resulting lane is 0. Note that this operates on byte-width lanes.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    let x = &Operand::new("x", TxN).with_doc("The vector to modify");
    let y = &Operand::new("y", &TxN.lane_of()).with_doc("New lane value");
    let Idx = &Operand::new("Idx", &imm.uimm8).with_doc("Lane index");

    ig.push(
        Inst::new(
            "insertlane",
            r#"
        Insert ``y`` as lane ``Idx`` in x.

        The lane index, ``Idx``, is an immediate value, not an SSA value. It
        must indicate a valid lane index for the type of ``x``.
        "#,
            &formats.ternary_imm8,
        )
        .operands_in(vec![x, y, Idx])
        .operands_out(vec![a]),
    );

    let x = &Operand::new("x", TxN);
    let a = &Operand::new("a", &TxN.lane_of());

    ig.push(
        Inst::new(
            "extractlane",
            r#"
        Extract lane ``Idx`` from ``x``.

        The lane index, ``Idx``, is an immediate value, not an SSA value. It
        must indicate a valid lane index for the type of ``x``. Note that the upper bits of ``a``
        may or may not be zeroed depending on the ISA but the type system should prevent using
        ``a`` as anything other than the extracted value.
        "#,
            &formats.binary_imm8,
        )
        .operands_in(vec![x, Idx])
        .operands_out(vec![a]),
    );
}

#[inline(never)]
fn define_simd_arithmetic(
    ig: &mut InstructionGroupBuilder,
    formats: &Formats,
    _: &Immediates,
    _: &EntityRefs,
) {
    let Int = &TypeVar::new(
        "Int",
        "A scalar or vector integer type",
        TypeSetBuilder::new()
            .ints(Interval::All)
            .simd_lanes(Interval::All)
            .build(),
    );

    let a = &Operand::new("a", Int);
    let x = &Operand::new("x", Int);
    let y = &Operand::new("y", Int);

    ig.push(
        Inst::new(
            "imin",
            r#"
        Signed integer minimum.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "umin",
            r#"
        Unsigned integer minimum.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "imax",
            r#"
        Signed integer maximum.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "umax",
            r#"
        Unsigned integer maximum.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    let IxN = &TypeVar::new(
        "IxN",
        "A SIMD vector type containing integers",
        TypeSetBuilder::new()
            .ints(Interval::All)
            .simd_lanes(Interval::All)
            .includes_scalars(false)
            .build(),
    );

    let a = &Operand::new("a", IxN);
    let x = &Operand::new("x", IxN);
    let y = &Operand::new("y", IxN);

    ig.push(
        Inst::new(
            "avg_round",
            r#"
        Unsigned average with rounding: `a := (x + y + 1) // 2`
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "uadd_sat",
            r#"
        Add with unsigned saturation.

        This is similar to `iadd` but the operands are interpreted as unsigned integers and their
        summed result, instead of wrapping, will be saturated to the highest unsigned integer for
        the controlling type (e.g. `0xFF` for i8).
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "sadd_sat",
            r#"
        Add with signed saturation.

        This is similar to `iadd` but the operands are interpreted as signed integers and their
        summed result, instead of wrapping, will be saturated to the lowest or highest
        signed integer for the controlling type (e.g. `0x80` or `0x7F` for i8). For example,
        since an `sadd_sat.i8` of `0x70` and `0x70` is greater than `0x7F`, the result will be
        clamped to `0x7F`.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "usub_sat",
            r#"
        Subtract with unsigned saturation.

        This is similar to `isub` but the operands are interpreted as unsigned integers and their
        difference, instead of wrapping, will be saturated to the lowest unsigned integer for
        the controlling type (e.g. `0x00` for i8).
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "ssub_sat",
            r#"
        Subtract with signed saturation.

        This is similar to `isub` but the operands are interpreted as signed integers and their
        difference, instead of wrapping, will be saturated to the lowest or highest
        signed integer for the controlling type (e.g. `0x80` or `0x7F` for i8).
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );
}

#[allow(clippy::many_single_char_names)]
pub(crate) fn define(
    all_instructions: &mut AllInstructions,
    formats: &Formats,
    imm: &Immediates,
    entities: &EntityRefs,
) {
    let mut ig = InstructionGroupBuilder::new(all_instructions);

    define_control_flow(&mut ig, formats, imm, entities);
    define_simd_lane_access(&mut ig, formats, imm, entities);
    define_simd_arithmetic(&mut ig, formats, imm, entities);

    // Operand kind shorthands.
    let iflags: &TypeVar = &ValueType::Special(types::Flag::IFlags.into()).into();
    let fflags: &TypeVar = &ValueType::Special(types::Flag::FFlags.into()).into();

    let b1: &TypeVar = &ValueType::from(LaneType::from(types::Bool::B1)).into();
    let f32_: &TypeVar = &ValueType::from(LaneType::from(types::Float::F32)).into();
    let f64_: &TypeVar = &ValueType::from(LaneType::from(types::Float::F64)).into();

    // Starting definitions.
    let Int = &TypeVar::new(
        "Int",
        "A scalar or vector integer type",
        TypeSetBuilder::new()
            .ints(Interval::All)
            .simd_lanes(Interval::All)
            .build(),
    );

    let Bool = &TypeVar::new(
        "Bool",
        "A scalar or vector boolean type",
        TypeSetBuilder::new()
            .bools(Interval::All)
            .simd_lanes(Interval::All)
            .build(),
    );

    let iB = &TypeVar::new(
        "iB",
        "A scalar integer type",
        TypeSetBuilder::new().ints(Interval::All).build(),
    );

    let iAddr = &TypeVar::new(
        "iAddr",
        "An integer address type",
        TypeSetBuilder::new().ints(32..64).refs(32..64).build(),
    );

    let Ref = &TypeVar::new(
        "Ref",
        "A scalar reference type",
        TypeSetBuilder::new().refs(Interval::All).build(),
    );

    let Testable = &TypeVar::new(
        "Testable",
        "A scalar boolean or integer type",
        TypeSetBuilder::new()
            .ints(Interval::All)
            .bools(Interval::All)
            .build(),
    );

    let TxN = &TypeVar::new(
        "TxN",
        "A SIMD vector type",
        TypeSetBuilder::new()
            .ints(Interval::All)
            .floats(Interval::All)
            .bools(Interval::All)
            .simd_lanes(Interval::All)
            .includes_scalars(false)
            .build(),
    );
    let Any = &TypeVar::new(
        "Any",
        "Any integer, float, boolean, or reference scalar or vector type",
        TypeSetBuilder::new()
            .ints(Interval::All)
            .floats(Interval::All)
            .bools(Interval::All)
            .refs(Interval::All)
            .simd_lanes(Interval::All)
            .includes_scalars(true)
            .build(),
    );

    let AnyTo = &TypeVar::copy_from(Any, "AnyTo".to_string());

    let Mem = &TypeVar::new(
        "Mem",
        "Any type that can be stored in memory",
        TypeSetBuilder::new()
            .ints(Interval::All)
            .floats(Interval::All)
            .simd_lanes(Interval::All)
            .refs(Interval::All)
            .build(),
    );

    let MemTo = &TypeVar::copy_from(Mem, "MemTo".to_string());

    let addr = &Operand::new("addr", iAddr);

    let SS = &Operand::new("SS", &entities.stack_slot);
    let Offset = &Operand::new("Offset", &imm.offset32).with_doc("Byte offset from base address");
    let x = &Operand::new("x", Mem).with_doc("Value to be stored");
    let a = &Operand::new("a", Mem).with_doc("Value loaded");
    let p = &Operand::new("p", iAddr);
    let MemFlags = &Operand::new("MemFlags", &imm.memflags);
    let args = &Operand::new("args", &entities.varargs).with_doc("Address arguments");

    ig.push(
        Inst::new(
            "load",
            r#"
        Load from memory at ``p + Offset``.

        This is a polymorphic instruction that can load any value type which
        has a memory representation.
        "#,
            &formats.load,
        )
        .operands_in(vec![MemFlags, p, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    ig.push(
        Inst::new(
            "load_complex",
            r#"
        Load from memory at ``sum(args) + Offset``.

        This is a polymorphic instruction that can load any value type which
        has a memory representation.
        "#,
            &formats.load_complex,
        )
        .operands_in(vec![MemFlags, args, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    ig.push(
        Inst::new(
            "store",
            r#"
        Store ``x`` to memory at ``p + Offset``.

        This is a polymorphic instruction that can store any value type with a
        memory representation.
        "#,
            &formats.store,
        )
        .operands_in(vec![MemFlags, x, p, Offset])
        .can_store(true),
    );

    ig.push(
        Inst::new(
            "store_complex",
            r#"
        Store ``x`` to memory at ``sum(args) + Offset``.

        This is a polymorphic instruction that can store any value type with a
        memory representation.
        "#,
            &formats.store_complex,
        )
        .operands_in(vec![MemFlags, x, args, Offset])
        .can_store(true),
    );

    let iExt8 = &TypeVar::new(
        "iExt8",
        "An integer type with more than 8 bits",
        TypeSetBuilder::new().ints(16..64).build(),
    );
    let x = &Operand::new("x", iExt8);
    let a = &Operand::new("a", iExt8);

    ig.push(
        Inst::new(
            "uload8",
            r#"
        Load 8 bits from memory at ``p + Offset`` and zero-extend.

        This is equivalent to ``load.i8`` followed by ``uextend``.
        "#,
            &formats.load,
        )
        .operands_in(vec![MemFlags, p, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    ig.push(
        Inst::new(
            "uload8_complex",
            r#"
        Load 8 bits from memory at ``sum(args) + Offset`` and zero-extend.

        This is equivalent to ``load.i8`` followed by ``uextend``.
        "#,
            &formats.load_complex,
        )
        .operands_in(vec![MemFlags, args, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    ig.push(
        Inst::new(
            "sload8",
            r#"
        Load 8 bits from memory at ``p + Offset`` and sign-extend.

        This is equivalent to ``load.i8`` followed by ``sextend``.
        "#,
            &formats.load,
        )
        .operands_in(vec![MemFlags, p, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    ig.push(
        Inst::new(
            "sload8_complex",
            r#"
        Load 8 bits from memory at ``sum(args) + Offset`` and sign-extend.

        This is equivalent to ``load.i8`` followed by ``sextend``.
        "#,
            &formats.load_complex,
        )
        .operands_in(vec![MemFlags, args, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    ig.push(
        Inst::new(
            "istore8",
            r#"
        Store the low 8 bits of ``x`` to memory at ``p + Offset``.

        This is equivalent to ``ireduce.i8`` followed by ``store.i8``.
        "#,
            &formats.store,
        )
        .operands_in(vec![MemFlags, x, p, Offset])
        .can_store(true),
    );

    ig.push(
        Inst::new(
            "istore8_complex",
            r#"
        Store the low 8 bits of ``x`` to memory at ``sum(args) + Offset``.

        This is equivalent to ``ireduce.i8`` followed by ``store.i8``.
        "#,
            &formats.store_complex,
        )
        .operands_in(vec![MemFlags, x, args, Offset])
        .can_store(true),
    );

    let iExt16 = &TypeVar::new(
        "iExt16",
        "An integer type with more than 16 bits",
        TypeSetBuilder::new().ints(32..64).build(),
    );
    let x = &Operand::new("x", iExt16);
    let a = &Operand::new("a", iExt16);

    ig.push(
        Inst::new(
            "uload16",
            r#"
        Load 16 bits from memory at ``p + Offset`` and zero-extend.

        This is equivalent to ``load.i16`` followed by ``uextend``.
        "#,
            &formats.load,
        )
        .operands_in(vec![MemFlags, p, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    ig.push(
        Inst::new(
            "uload16_complex",
            r#"
        Load 16 bits from memory at ``sum(args) + Offset`` and zero-extend.

        This is equivalent to ``load.i16`` followed by ``uextend``.
        "#,
            &formats.load_complex,
        )
        .operands_in(vec![MemFlags, args, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    ig.push(
        Inst::new(
            "sload16",
            r#"
        Load 16 bits from memory at ``p + Offset`` and sign-extend.

        This is equivalent to ``load.i16`` followed by ``sextend``.
        "#,
            &formats.load,
        )
        .operands_in(vec![MemFlags, p, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    ig.push(
        Inst::new(
            "sload16_complex",
            r#"
        Load 16 bits from memory at ``sum(args) + Offset`` and sign-extend.

        This is equivalent to ``load.i16`` followed by ``sextend``.
        "#,
            &formats.load_complex,
        )
        .operands_in(vec![MemFlags, args, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    ig.push(
        Inst::new(
            "istore16",
            r#"
        Store the low 16 bits of ``x`` to memory at ``p + Offset``.

        This is equivalent to ``ireduce.i16`` followed by ``store.i16``.
        "#,
            &formats.store,
        )
        .operands_in(vec![MemFlags, x, p, Offset])
        .can_store(true),
    );

    ig.push(
        Inst::new(
            "istore16_complex",
            r#"
        Store the low 16 bits of ``x`` to memory at ``sum(args) + Offset``.

        This is equivalent to ``ireduce.i16`` followed by ``store.i16``.
        "#,
            &formats.store_complex,
        )
        .operands_in(vec![MemFlags, x, args, Offset])
        .can_store(true),
    );

    let iExt32 = &TypeVar::new(
        "iExt32",
        "An integer type with more than 32 bits",
        TypeSetBuilder::new().ints(64..64).build(),
    );
    let x = &Operand::new("x", iExt32);
    let a = &Operand::new("a", iExt32);

    ig.push(
        Inst::new(
            "uload32",
            r#"
        Load 32 bits from memory at ``p + Offset`` and zero-extend.

        This is equivalent to ``load.i32`` followed by ``uextend``.
        "#,
            &formats.load,
        )
        .operands_in(vec![MemFlags, p, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    ig.push(
        Inst::new(
            "uload32_complex",
            r#"
        Load 32 bits from memory at ``sum(args) + Offset`` and zero-extend.

        This is equivalent to ``load.i32`` followed by ``uextend``.
        "#,
            &formats.load_complex,
        )
        .operands_in(vec![MemFlags, args, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    ig.push(
        Inst::new(
            "sload32",
            r#"
        Load 32 bits from memory at ``p + Offset`` and sign-extend.

        This is equivalent to ``load.i32`` followed by ``sextend``.
        "#,
            &formats.load,
        )
        .operands_in(vec![MemFlags, p, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    ig.push(
        Inst::new(
            "sload32_complex",
            r#"
        Load 32 bits from memory at ``sum(args) + Offset`` and sign-extend.

        This is equivalent to ``load.i32`` followed by ``sextend``.
        "#,
            &formats.load_complex,
        )
        .operands_in(vec![MemFlags, args, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    ig.push(
        Inst::new(
            "istore32",
            r#"
        Store the low 32 bits of ``x`` to memory at ``p + Offset``.

        This is equivalent to ``ireduce.i32`` followed by ``store.i32``.
        "#,
            &formats.store,
        )
        .operands_in(vec![MemFlags, x, p, Offset])
        .can_store(true),
    );

    ig.push(
        Inst::new(
            "istore32_complex",
            r#"
        Store the low 32 bits of ``x`` to memory at ``sum(args) + Offset``.

        This is equivalent to ``ireduce.i32`` followed by ``store.i32``.
        "#,
            &formats.store_complex,
        )
        .operands_in(vec![MemFlags, x, args, Offset])
        .can_store(true),
    );

    let I16x8 = &TypeVar::new(
        "I16x8",
        "A SIMD vector with exactly 8 lanes of 16-bit values",
        TypeSetBuilder::new()
            .ints(16..16)
            .simd_lanes(8..8)
            .includes_scalars(false)
            .build(),
    );
    let a = &Operand::new("a", I16x8).with_doc("Value loaded");

    ig.push(
        Inst::new(
            "uload8x8",
            r#"
        Load an 8x8 vector (64 bits) from memory at ``p + Offset`` and zero-extend into an i16x8
        vector.
        "#,
            &formats.load,
        )
        .operands_in(vec![MemFlags, p, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    ig.push(
        Inst::new(
            "uload8x8_complex",
            r#"
        Load an 8x8 vector (64 bits) from memory at ``sum(args) + Offset`` and zero-extend into an
        i16x8 vector.
        "#,
            &formats.load_complex,
        )
        .operands_in(vec![MemFlags, args, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    ig.push(
        Inst::new(
            "sload8x8",
            r#"
        Load an 8x8 vector (64 bits) from memory at ``p + Offset`` and sign-extend into an i16x8
        vector.
        "#,
            &formats.load,
        )
        .operands_in(vec![MemFlags, p, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    ig.push(
        Inst::new(
            "sload8x8_complex",
            r#"
        Load an 8x8 vector (64 bits) from memory at ``sum(args) + Offset`` and sign-extend into an
        i16x8 vector.
        "#,
            &formats.load_complex,
        )
        .operands_in(vec![MemFlags, args, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    let I32x4 = &TypeVar::new(
        "I32x4",
        "A SIMD vector with exactly 4 lanes of 32-bit values",
        TypeSetBuilder::new()
            .ints(32..32)
            .simd_lanes(4..4)
            .includes_scalars(false)
            .build(),
    );
    let a = &Operand::new("a", I32x4).with_doc("Value loaded");

    ig.push(
        Inst::new(
            "uload16x4",
            r#"
        Load a 16x4 vector (64 bits) from memory at ``p + Offset`` and zero-extend into an i32x4
        vector.
        "#,
            &formats.load,
        )
        .operands_in(vec![MemFlags, p, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    ig.push(
        Inst::new(
            "uload16x4_complex",
            r#"
        Load a 16x4 vector (64 bits) from memory at ``sum(args) + Offset`` and zero-extend into an
        i32x4 vector.
        "#,
            &formats.load_complex,
        )
        .operands_in(vec![MemFlags, args, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    ig.push(
        Inst::new(
            "sload16x4",
            r#"
        Load a 16x4 vector (64 bits) from memory at ``p + Offset`` and sign-extend into an i32x4
        vector.
        "#,
            &formats.load,
        )
        .operands_in(vec![MemFlags, p, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    ig.push(
        Inst::new(
            "sload16x4_complex",
            r#"
        Load a 16x4 vector (64 bits) from memory at ``sum(args) + Offset`` and sign-extend into an
        i32x4 vector.
        "#,
            &formats.load_complex,
        )
        .operands_in(vec![MemFlags, args, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    let I64x2 = &TypeVar::new(
        "I64x2",
        "A SIMD vector with exactly 2 lanes of 64-bit values",
        TypeSetBuilder::new()
            .ints(64..64)
            .simd_lanes(2..2)
            .includes_scalars(false)
            .build(),
    );
    let a = &Operand::new("a", I64x2).with_doc("Value loaded");

    ig.push(
        Inst::new(
            "uload32x2",
            r#"
        Load an 32x2 vector (64 bits) from memory at ``p + Offset`` and zero-extend into an i64x2
        vector.
        "#,
            &formats.load,
        )
        .operands_in(vec![MemFlags, p, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    ig.push(
        Inst::new(
            "uload32x2_complex",
            r#"
        Load a 32x2 vector (64 bits) from memory at ``sum(args) + Offset`` and zero-extend into an
        i64x2 vector.
        "#,
            &formats.load_complex,
        )
        .operands_in(vec![MemFlags, args, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    ig.push(
        Inst::new(
            "sload32x2",
            r#"
        Load a 32x2 vector (64 bits) from memory at ``p + Offset`` and sign-extend into an i64x2
        vector.
        "#,
            &formats.load,
        )
        .operands_in(vec![MemFlags, p, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    ig.push(
        Inst::new(
            "sload32x2_complex",
            r#"
        Load a 32x2 vector (64 bits) from memory at ``sum(args) + Offset`` and sign-extend into an
        i64x2 vector.
        "#,
            &formats.load_complex,
        )
        .operands_in(vec![MemFlags, args, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    let x = &Operand::new("x", Mem).with_doc("Value to be stored");
    let a = &Operand::new("a", Mem).with_doc("Value loaded");
    let Offset =
        &Operand::new("Offset", &imm.offset32).with_doc("In-bounds offset into stack slot");

    ig.push(
        Inst::new(
            "stack_load",
            r#"
        Load a value from a stack slot at the constant offset.

        This is a polymorphic instruction that can load any value type which
        has a memory representation.

        The offset is an immediate constant, not an SSA value. The memory
        access cannot go out of bounds, i.e.
        `sizeof(a) + Offset <= sizeof(SS)`.
        "#,
            &formats.stack_load,
        )
        .operands_in(vec![SS, Offset])
        .operands_out(vec![a])
        .can_load(true),
    );

    ig.push(
        Inst::new(
            "stack_store",
            r#"
        Store a value to a stack slot at a constant offset.

        This is a polymorphic instruction that can store any value type with a
        memory representation.

        The offset is an immediate constant, not an SSA value. The memory
        access cannot go out of bounds, i.e.
        `sizeof(a) + Offset <= sizeof(SS)`.
        "#,
            &formats.stack_store,
        )
        .operands_in(vec![x, SS, Offset])
        .can_store(true),
    );

    ig.push(
        Inst::new(
            "stack_addr",
            r#"
        Get the address of a stack slot.

        Compute the absolute address of a byte in a stack slot. The offset must
        refer to a byte inside the stack slot:
        `0 <= Offset < sizeof(SS)`.
        "#,
            &formats.stack_load,
        )
        .operands_in(vec![SS, Offset])
        .operands_out(vec![addr]),
    );

    let GV = &Operand::new("GV", &entities.global_value);

    ig.push(
        Inst::new(
            "global_value",
            r#"
        Compute the value of global GV.
        "#,
            &formats.unary_global_value,
        )
        .operands_in(vec![GV])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "symbol_value",
            r#"
        Compute the value of global GV, which is a symbolic value.
        "#,
            &formats.unary_global_value,
        )
        .operands_in(vec![GV])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "tls_value",
            r#"
        Compute the value of global GV, which is a TLS (thread local storage) value.
        "#,
            &formats.unary_global_value,
        )
        .operands_in(vec![GV])
        .operands_out(vec![a]),
    );

    let HeapOffset = &TypeVar::new(
        "HeapOffset",
        "An unsigned heap offset",
        TypeSetBuilder::new().ints(32..64).build(),
    );

    let H = &Operand::new("H", &entities.heap);
    let p = &Operand::new("p", HeapOffset);
    let Size = &Operand::new("Size", &imm.uimm32).with_doc("Size in bytes");

    ig.push(
        Inst::new(
            "heap_addr",
            r#"
        Bounds check and compute absolute address of heap memory.

        Verify that the offset range ``p .. p + Size - 1`` is in bounds for the
        heap H, and generate an absolute address that is safe to dereference.

        1. If ``p + Size`` is not greater than the heap bound, return an
           absolute address corresponding to a byte offset of ``p`` from the
           heap's base address.
        2. If ``p + Size`` is greater than the heap bound, generate a trap.
        "#,
            &formats.heap_addr,
        )
        .operands_in(vec![H, p, Size])
        .operands_out(vec![addr]),
    );

    // Note this instruction is marked as having other side-effects, so GVN won't try to hoist it,
    // which would result in it being subject to spilling. While not hoisting would generally hurt
    // performance, since a computed value used many times may need to be regenerated before each
    // use, it is not the case here: this instruction doesn't generate any code.  That's because,
    // by definition the pinned register is never used by the register allocator, but is written to
    // and read explicitly and exclusively by set_pinned_reg and get_pinned_reg.
    ig.push(
        Inst::new(
            "get_pinned_reg",
            r#"
            Gets the content of the pinned register, when it's enabled.
        "#,
            &formats.nullary,
        )
        .operands_out(vec![addr])
        .other_side_effects(true),
    );

    ig.push(
        Inst::new(
            "set_pinned_reg",
            r#"
        Sets the content of the pinned register, when it's enabled.
        "#,
            &formats.unary,
        )
        .operands_in(vec![addr])
        .other_side_effects(true),
    );

    let TableOffset = &TypeVar::new(
        "TableOffset",
        "An unsigned table offset",
        TypeSetBuilder::new().ints(32..64).build(),
    );
    let T = &Operand::new("T", &entities.table);
    let p = &Operand::new("p", TableOffset);
    let Offset =
        &Operand::new("Offset", &imm.offset32).with_doc("Byte offset from element address");

    ig.push(
        Inst::new(
            "table_addr",
            r#"
        Bounds check and compute absolute address of a table entry.

        Verify that the offset ``p`` is in bounds for the table T, and generate
        an absolute address that is safe to dereference.

        ``Offset`` must be less than the size of a table element.

        1. If ``p`` is not greater than the table bound, return an absolute
           address corresponding to a byte offset of ``p`` from the table's
           base address.
        2. If ``p`` is greater than the table bound, generate a trap.
        "#,
            &formats.table_addr,
        )
        .operands_in(vec![T, p, Offset])
        .operands_out(vec![addr]),
    );

    let N = &Operand::new("N", &imm.imm64);
    let a = &Operand::new("a", Int).with_doc("A constant integer scalar or vector value");

    ig.push(
        Inst::new(
            "iconst",
            r#"
        Integer constant.

        Create a scalar integer SSA value with an immediate constant value, or
        an integer vector where all the lanes have the same value.
        "#,
            &formats.unary_imm,
        )
        .operands_in(vec![N])
        .operands_out(vec![a]),
    );

    let N = &Operand::new("N", &imm.ieee32);
    let a = &Operand::new("a", f32_).with_doc("A constant f32 scalar value");

    ig.push(
        Inst::new(
            "f32const",
            r#"
        Floating point constant.

        Create a `f32` SSA value with an immediate constant value.
        "#,
            &formats.unary_ieee32,
        )
        .operands_in(vec![N])
        .operands_out(vec![a]),
    );

    let N = &Operand::new("N", &imm.ieee64);
    let a = &Operand::new("a", f64_).with_doc("A constant f64 scalar value");

    ig.push(
        Inst::new(
            "f64const",
            r#"
        Floating point constant.

        Create a `f64` SSA value with an immediate constant value.
        "#,
            &formats.unary_ieee64,
        )
        .operands_in(vec![N])
        .operands_out(vec![a]),
    );

    let N = &Operand::new("N", &imm.boolean);
    let a = &Operand::new("a", Bool).with_doc("A constant boolean scalar or vector value");

    ig.push(
        Inst::new(
            "bconst",
            r#"
        Boolean constant.

        Create a scalar boolean SSA value with an immediate constant value, or
        a boolean vector where all the lanes have the same value.
        "#,
            &formats.unary_bool,
        )
        .operands_in(vec![N])
        .operands_out(vec![a]),
    );

    let N = &Operand::new("N", &imm.pool_constant)
        .with_doc("The 16 immediate bytes of a 128-bit vector");
    let a = &Operand::new("a", TxN).with_doc("A constant vector value");

    ig.push(
        Inst::new(
            "vconst",
            r#"
        SIMD vector constant.

        Construct a vector with the given immediate bytes.
        "#,
            &formats.unary_const,
        )
        .operands_in(vec![N])
        .operands_out(vec![a]),
    );

    let constant =
        &Operand::new("constant", &imm.pool_constant).with_doc("A constant in the constant pool");
    let address = &Operand::new("address", iAddr);
    ig.push(
        Inst::new(
            "const_addr",
            r#"
        Calculate the base address of a value in the constant pool.
        "#,
            &formats.unary_const,
        )
        .operands_in(vec![constant])
        .operands_out(vec![address]),
    );

    let mask = &Operand::new("mask", &imm.uimm128)
        .with_doc("The 16 immediate bytes used for selecting the elements to shuffle");
    let Tx16 = &TypeVar::new(
        "Tx16",
        "A SIMD vector with exactly 16 lanes of 8-bit values; eventually this may support other \
         lane counts and widths",
        TypeSetBuilder::new()
            .ints(8..8)
            .bools(8..8)
            .simd_lanes(16..16)
            .includes_scalars(false)
            .build(),
    );
    let a = &Operand::new("a", Tx16).with_doc("A vector value");
    let b = &Operand::new("b", Tx16).with_doc("A vector value");

    ig.push(
        Inst::new(
            "shuffle",
            r#"
        SIMD vector shuffle.

        Shuffle two vectors using the given immediate bytes. For each of the 16 bytes of the
        immediate, a value i of 0-15 selects the i-th element of the first vector and a value i of
        16-31 selects the (i-16)th element of the second vector. Immediate values outside of the
        0-31 range place a 0 in the resulting vector lane.
        "#,
            &formats.shuffle,
        )
        .operands_in(vec![a, b, mask])
        .operands_out(vec![a]),
    );

    let a = &Operand::new("a", Ref).with_doc("A constant reference null value");

    ig.push(
        Inst::new(
            "null",
            r#"
        Null constant value for reference types.

        Create a scalar reference SSA value with a constant null value.
        "#,
            &formats.nullary,
        )
        .operands_out(vec![a]),
    );

    ig.push(Inst::new(
        "nop",
        r#"
        Just a dummy instruction.

        Note: this doesn't compile to a machine code nop.
        "#,
        &formats.nullary,
    ));

    let c = &Operand::new("c", Testable).with_doc("Controlling value to test");
    let x = &Operand::new("x", Any).with_doc("Value to use when `c` is true");
    let y = &Operand::new("y", Any).with_doc("Value to use when `c` is false");
    let a = &Operand::new("a", Any);

    ig.push(
        Inst::new(
            "select",
            r#"
        Conditional select.

        This instruction selects whole values. Use `vselect` for
        lane-wise selection.
        "#,
            &formats.ternary,
        )
        .operands_in(vec![c, x, y])
        .operands_out(vec![a]),
    );

    let cc = &Operand::new("cc", &imm.intcc).with_doc("Controlling condition code");
    let flags = &Operand::new("flags", iflags).with_doc("The machine's flag register");

    ig.push(
        Inst::new(
            "selectif",
            r#"
        Conditional select, dependent on integer condition codes.
        "#,
            &formats.int_select,
        )
        .operands_in(vec![cc, flags, x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "selectif_spectre_guard",
            r#"
            Conditional select intended for Spectre guards.

            This operation is semantically equivalent to a selectif instruction.
            However, it is guaranteed to not be removed or otherwise altered by any
            optimization pass, and is guaranteed to result in a conditional-move
            instruction, not a branch-based lowering.  As such, it is suitable
            for use when producing Spectre guards. For example, a bounds-check
            may guard against unsafe speculation past a bounds-check conditional
            branch by passing the address or index to be accessed through a
            conditional move, also gated on the same condition. Because no
            Spectre-vulnerable processors are known to perform speculation on
            conditional move instructions, this is guaranteed to pick the
            correct input. If the selected input in case of overflow is a "safe"
            value, for example a null pointer that causes an exception in the
            speculative path, this ensures that no Spectre vulnerability will
            exist.
            "#,
            &formats.int_select,
        )
        .operands_in(vec![cc, flags, x, y])
        .operands_out(vec![a])
        .other_side_effects(true),
    );

    let c = &Operand::new("c", Any).with_doc("Controlling value to test");
    ig.push(
        Inst::new(
            "bitselect",
            r#"
        Conditional select of bits.

        For each bit in `c`, this instruction selects the corresponding bit from `x` if the bit
        in `c` is 1 and the corresponding bit from `y` if the bit in `c` is 0. See also:
        `select`, `vselect`.
        "#,
            &formats.ternary,
        )
        .operands_in(vec![c, x, y])
        .operands_out(vec![a]),
    );

    let x = &Operand::new("x", Any);

    ig.push(
        Inst::new(
            "copy",
            r#"
        Register-register copy.

        This instruction copies its input, preserving the value type.

        A pure SSA-form program does not need to copy values, but this
        instruction is useful for representing intermediate stages during
        instruction transformations, and the register allocator needs a way of
        representing register copies.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "ifcmp_sp",
            r#"
    Compare ``addr`` with the stack pointer and set the CPU flags.

    This is like `ifcmp` where ``addr`` is the LHS operand and the stack
    pointer is the RHS.
    "#,
            &formats.unary,
        )
        .operands_in(vec![addr])
        .operands_out(vec![flags]),
    );

    let x = &Operand::new("x", TxN).with_doc("Vector to split");
    let lo = &Operand::new("lo", &TxN.half_vector()).with_doc("Low-numbered lanes of `x`");
    let hi = &Operand::new("hi", &TxN.half_vector()).with_doc("High-numbered lanes of `x`");

    ig.push(
        Inst::new(
            "vsplit",
            r#"
        Split a vector into two halves.

        Split the vector `x` into two separate values, each containing half of
        the lanes from ``x``. The result may be two scalars if ``x`` only had
        two lanes.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![lo, hi]),
    );

    let Any128 = &TypeVar::new(
        "Any128",
        "Any scalar or vector type with as most 128 lanes",
        TypeSetBuilder::new()
            .ints(Interval::All)
            .floats(Interval::All)
            .bools(Interval::All)
            .simd_lanes(1..128)
            .includes_scalars(true)
            .build(),
    );

    let x = &Operand::new("x", Any128).with_doc("Low-numbered lanes");
    let y = &Operand::new("y", Any128).with_doc("High-numbered lanes");
    let a = &Operand::new("a", &Any128.double_vector()).with_doc("Concatenation of `x` and `y`");

    ig.push(
        Inst::new(
            "vconcat",
            r#"
        Vector concatenation.

        Return a vector formed by concatenating ``x`` and ``y``. The resulting
        vector type has twice as many lanes as each of the inputs. The lanes of
        ``x`` appear as the low-numbered lanes, and the lanes of ``y`` become
        the high-numbered lanes of ``a``.

        It is possible to form a vector by concatenating two scalars.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    let c = &Operand::new("c", &TxN.as_bool()).with_doc("Controlling vector");
    let x = &Operand::new("x", TxN).with_doc("Value to use where `c` is true");
    let y = &Operand::new("y", TxN).with_doc("Value to use where `c` is false");
    let a = &Operand::new("a", TxN);

    ig.push(
        Inst::new(
            "vselect",
            r#"
        Vector lane select.

        Select lanes from ``x`` or ``y`` controlled by the lanes of the boolean
        vector ``c``.
        "#,
            &formats.ternary,
        )
        .operands_in(vec![c, x, y])
        .operands_out(vec![a]),
    );

    let s = &Operand::new("s", b1);

    ig.push(
        Inst::new(
            "vany_true",
            r#"
        Reduce a vector to a scalar boolean.

        Return a scalar boolean true if any lane in ``a`` is non-zero, false otherwise.
        "#,
            &formats.unary,
        )
        .operands_in(vec![a])
        .operands_out(vec![s]),
    );

    ig.push(
        Inst::new(
            "vall_true",
            r#"
        Reduce a vector to a scalar boolean.

        Return a scalar boolean true if all lanes in ``i`` are non-zero, false otherwise.
        "#,
            &formats.unary,
        )
        .operands_in(vec![a])
        .operands_out(vec![s]),
    );

    let a = &Operand::new("a", TxN);
    let x = &Operand::new("x", Int);

    ig.push(
        Inst::new(
            "vhigh_bits",
            r#"
        Reduce a vector to a scalar integer.

        Return a scalar integer, consisting of the concatenation of the most significant bit
        of each lane of ``a``.
        "#,
            &formats.unary,
        )
        .operands_in(vec![a])
        .operands_out(vec![x]),
    );

    let a = &Operand::new("a", &Int.as_bool());
    let Cond = &Operand::new("Cond", &imm.intcc);
    let x = &Operand::new("x", Int);
    let y = &Operand::new("y", Int);

    ig.push(
        Inst::new(
            "icmp",
            r#"
        Integer comparison.

        The condition code determines if the operands are interpreted as signed
        or unsigned integers.

        | Signed | Unsigned | Condition             |
        |--------|----------|-----------------------|
        | eq     | eq       | Equal                 |
        | ne     | ne       | Not equal             |
        | slt    | ult      | Less than             |
        | sge    | uge      | Greater than or equal |
        | sgt    | ugt      | Greater than          |
        | sle    | ule      | Less than or equal    |
        | of     | *        | Overflow              |
        | nof    | *        | No Overflow           |

        \* The unsigned version of overflow condition for add has ISA-specific semantics and thus
        has been kept as a method on the TargetIsa trait as
        [unsigned_add_overflow_condition][crate::isa::TargetIsa::unsigned_add_overflow_condition].

        When this instruction compares integer vectors, it returns a boolean
        vector of lane-wise comparisons.
        "#,
            &formats.int_compare,
        )
        .operands_in(vec![Cond, x, y])
        .operands_out(vec![a]),
    );

    let a = &Operand::new("a", b1);
    let x = &Operand::new("x", iB);
    let Y = &Operand::new("Y", &imm.imm64);

    ig.push(
        Inst::new(
            "icmp_imm",
            r#"
        Compare scalar integer to a constant.

        This is the same as the `icmp` instruction, except one operand is
        an immediate constant.

        This instruction can only compare scalars. Use `icmp` for
        lane-wise vector comparisons.
        "#,
            &formats.int_compare_imm,
        )
        .operands_in(vec![Cond, x, Y])
        .operands_out(vec![a]),
    );

    let f = &Operand::new("f", iflags);
    let x = &Operand::new("x", iB);
    let y = &Operand::new("y", iB);

    ig.push(
        Inst::new(
            "ifcmp",
            r#"
        Compare scalar integers and return flags.

        Compare two scalar integer values and return integer CPU flags
        representing the result.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![f]),
    );

    ig.push(
        Inst::new(
            "ifcmp_imm",
            r#"
        Compare scalar integer to a constant and return flags.

        Like `icmp_imm`, but returns integer CPU flags instead of testing
        a specific condition code.
        "#,
            &formats.binary_imm64,
        )
        .operands_in(vec![x, Y])
        .operands_out(vec![f]),
    );

    let a = &Operand::new("a", Int);
    let x = &Operand::new("x", Int);
    let y = &Operand::new("y", Int);

    ig.push(
        Inst::new(
            "iadd",
            r#"
        Wrapping integer addition: `a := x + y \pmod{2^B}`.

        This instruction does not depend on the signed/unsigned interpretation
        of the operands.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "isub",
            r#"
        Wrapping integer subtraction: `a := x - y \pmod{2^B}`.

        This instruction does not depend on the signed/unsigned interpretation
        of the operands.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "ineg",
            r#"
        Integer negation: `a := -x \pmod{2^B}`.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "iabs",
            r#"
        Integer absolute value with wrapping: `a := |x|`.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "imul",
            r#"
        Wrapping integer multiplication: `a := x y \pmod{2^B}`.

        This instruction does not depend on the signed/unsigned interpretation
        of the operands.

        Polymorphic over all integer types (vector and scalar).
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "umulhi",
            r#"
        Unsigned integer multiplication, producing the high half of a
        double-length result.

        Polymorphic over all integer types (vector and scalar).
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "smulhi",
            r#"
        Signed integer multiplication, producing the high half of a
        double-length result.

        Polymorphic over all integer types (vector and scalar).
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    let I16or32 = &TypeVar::new(
        "I16or32",
        "A scalar or vector integer type with 16- or 32-bit numbers",
        TypeSetBuilder::new().ints(16..32).simd_lanes(4..8).build(),
    );

    let qx = &Operand::new("x", I16or32);
    let qy = &Operand::new("y", I16or32);
    let qa = &Operand::new("a", I16or32);

    ig.push(
        Inst::new(
            "sqmul_round_sat",
            r#"
        Fixed-point multiplication of numbers in the QN format, where N + 1
        is the number bitwidth:
        `a := signed_saturate((x * y + 1 << (Q - 1)) >> Q)`

        Polymorphic over all integer types (scalar and vector) with 16- or
        32-bit numbers.
        "#,
            &formats.binary,
        )
        .operands_in(vec![qx, qy])
        .operands_out(vec![qa]),
    );

    ig.push(
        Inst::new(
            "udiv",
            r#"
        Unsigned integer division: `a := \lfloor {x \over y} \rfloor`.

        This operation traps if the divisor is zero.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a])
        .can_trap(true),
    );

    ig.push(
        Inst::new(
            "sdiv",
            r#"
        Signed integer division rounded toward zero: `a := sign(xy)
        \lfloor {|x| \over |y|}\rfloor`.

        This operation traps if the divisor is zero, or if the result is not
        representable in `B` bits two's complement. This only happens
        when `x = -2^{B-1}, y = -1`.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a])
        .can_trap(true),
    );

    ig.push(
        Inst::new(
            "urem",
            r#"
        Unsigned integer remainder.

        This operation traps if the divisor is zero.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a])
        .can_trap(true),
    );

    ig.push(
        Inst::new(
            "srem",
            r#"
        Signed integer remainder. The result has the sign of the dividend.

        This operation traps if the divisor is zero.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a])
        .can_trap(true),
    );

    let a = &Operand::new("a", iB);
    let x = &Operand::new("x", iB);
    let Y = &Operand::new("Y", &imm.imm64);

    ig.push(
        Inst::new(
            "iadd_imm",
            r#"
        Add immediate integer.

        Same as `iadd`, but one operand is an immediate constant.

        Polymorphic over all scalar integer types, but does not support vector
        types.
        "#,
            &formats.binary_imm64,
        )
        .operands_in(vec![x, Y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "imul_imm",
            r#"
        Integer multiplication by immediate constant.

        Polymorphic over all scalar integer types, but does not support vector
        types.
        "#,
            &formats.binary_imm64,
        )
        .operands_in(vec![x, Y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "udiv_imm",
            r#"
        Unsigned integer division by an immediate constant.

        This operation traps if the divisor is zero.
        "#,
            &formats.binary_imm64,
        )
        .operands_in(vec![x, Y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "sdiv_imm",
            r#"
        Signed integer division by an immediate constant.

        This operation traps if the divisor is zero, or if the result is not
        representable in `B` bits two's complement. This only happens
        when `x = -2^{B-1}, Y = -1`.
        "#,
            &formats.binary_imm64,
        )
        .operands_in(vec![x, Y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "urem_imm",
            r#"
        Unsigned integer remainder with immediate divisor.

        This operation traps if the divisor is zero.
        "#,
            &formats.binary_imm64,
        )
        .operands_in(vec![x, Y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "srem_imm",
            r#"
        Signed integer remainder with immediate divisor.

        This operation traps if the divisor is zero.
        "#,
            &formats.binary_imm64,
        )
        .operands_in(vec![x, Y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "irsub_imm",
            r#"
        Immediate reverse wrapping subtraction: `a := Y - x \pmod{2^B}`.

        Also works as integer negation when `Y = 0`. Use `iadd_imm`
        with a negative immediate operand for the reverse immediate
        subtraction.

        Polymorphic over all scalar integer types, but does not support vector
        types.
        "#,
            &formats.binary_imm64,
        )
        .operands_in(vec![x, Y])
        .operands_out(vec![a]),
    );

    let a = &Operand::new("a", iB);
    let x = &Operand::new("x", iB);
    let y = &Operand::new("y", iB);

    let c_in = &Operand::new("c_in", b1).with_doc("Input carry flag");
    let c_out = &Operand::new("c_out", b1).with_doc("Output carry flag");
    let b_in = &Operand::new("b_in", b1).with_doc("Input borrow flag");
    let b_out = &Operand::new("b_out", b1).with_doc("Output borrow flag");

    let c_if_in = &Operand::new("c_in", iflags);
    let c_if_out = &Operand::new("c_out", iflags);
    let b_if_in = &Operand::new("b_in", iflags);
    let b_if_out = &Operand::new("b_out", iflags);

    ig.push(
        Inst::new(
            "iadd_cin",
            r#"
        Add integers with carry in.

        Same as `iadd` with an additional carry input. Computes:

        ```text
            a = x + y + c_{in} \pmod 2^B
        ```

        Polymorphic over all scalar integer types, but does not support vector
        types.
        "#,
            &formats.ternary,
        )
        .operands_in(vec![x, y, c_in])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "iadd_ifcin",
            r#"
        Add integers with carry in.

        Same as `iadd` with an additional carry flag input. Computes:

        ```text
            a = x + y + c_{in} \pmod 2^B
        ```

        Polymorphic over all scalar integer types, but does not support vector
        types.
        "#,
            &formats.ternary,
        )
        .operands_in(vec![x, y, c_if_in])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "iadd_cout",
            r#"
        Add integers with carry out.

        Same as `iadd` with an additional carry output.

        ```text
            a &= x + y \pmod 2^B \\
            c_{out} &= x+y >= 2^B
        ```

        Polymorphic over all scalar integer types, but does not support vector
        types.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a, c_out]),
    );

    ig.push(
        Inst::new(
            "iadd_ifcout",
            r#"
        Add integers with carry out.

        Same as `iadd` with an additional carry flag output.

        ```text
            a &= x + y \pmod 2^B \\
            c_{out} &= x+y >= 2^B
        ```

        Polymorphic over all scalar integer types, but does not support vector
        types.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a, c_if_out]),
    );

    ig.push(
        Inst::new(
            "iadd_carry",
            r#"
        Add integers with carry in and out.

        Same as `iadd` with an additional carry input and output.

        ```text
            a &= x + y + c_{in} \pmod 2^B \\
            c_{out} &= x + y + c_{in} >= 2^B
        ```

        Polymorphic over all scalar integer types, but does not support vector
        types.
        "#,
            &formats.ternary,
        )
        .operands_in(vec![x, y, c_in])
        .operands_out(vec![a, c_out]),
    );

    ig.push(
        Inst::new(
            "iadd_ifcarry",
            r#"
        Add integers with carry in and out.

        Same as `iadd` with an additional carry flag input and output.

        ```text
            a &= x + y + c_{in} \pmod 2^B \\
            c_{out} &= x + y + c_{in} >= 2^B
        ```

        Polymorphic over all scalar integer types, but does not support vector
        types.
        "#,
            &formats.ternary,
        )
        .operands_in(vec![x, y, c_if_in])
        .operands_out(vec![a, c_if_out]),
    );

    ig.push(
        Inst::new(
            "isub_bin",
            r#"
        Subtract integers with borrow in.

        Same as `isub` with an additional borrow flag input. Computes:

        ```text
            a = x - (y + b_{in}) \pmod 2^B
        ```

        Polymorphic over all scalar integer types, but does not support vector
        types.
        "#,
            &formats.ternary,
        )
        .operands_in(vec![x, y, b_in])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "isub_ifbin",
            r#"
        Subtract integers with borrow in.

        Same as `isub` with an additional borrow flag input. Computes:

        ```text
            a = x - (y + b_{in}) \pmod 2^B
        ```

        Polymorphic over all scalar integer types, but does not support vector
        types.
        "#,
            &formats.ternary,
        )
        .operands_in(vec![x, y, b_if_in])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "isub_bout",
            r#"
        Subtract integers with borrow out.

        Same as `isub` with an additional borrow flag output.

        ```text
            a &= x - y \pmod 2^B \\
            b_{out} &= x < y
        ```

        Polymorphic over all scalar integer types, but does not support vector
        types.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a, b_out]),
    );

    ig.push(
        Inst::new(
            "isub_ifbout",
            r#"
        Subtract integers with borrow out.

        Same as `isub` with an additional borrow flag output.

        ```text
            a &= x - y \pmod 2^B \\
            b_{out} &= x < y
        ```

        Polymorphic over all scalar integer types, but does not support vector
        types.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a, b_if_out]),
    );

    ig.push(
        Inst::new(
            "isub_borrow",
            r#"
        Subtract integers with borrow in and out.

        Same as `isub` with an additional borrow flag input and output.

        ```text
            a &= x - (y + b_{in}) \pmod 2^B \\
            b_{out} &= x < y + b_{in}
        ```

        Polymorphic over all scalar integer types, but does not support vector
        types.
        "#,
            &formats.ternary,
        )
        .operands_in(vec![x, y, b_in])
        .operands_out(vec![a, b_out]),
    );

    ig.push(
        Inst::new(
            "isub_ifborrow",
            r#"
        Subtract integers with borrow in and out.

        Same as `isub` with an additional borrow flag input and output.

        ```text
            a &= x - (y + b_{in}) \pmod 2^B \\
            b_{out} &= x < y + b_{in}
        ```

        Polymorphic over all scalar integer types, but does not support vector
        types.
        "#,
            &formats.ternary,
        )
        .operands_in(vec![x, y, b_if_in])
        .operands_out(vec![a, b_if_out]),
    );

    let bits = &TypeVar::new(
        "bits",
        "Any integer, float, or boolean scalar or vector type",
        TypeSetBuilder::new()
            .ints(Interval::All)
            .floats(Interval::All)
            .bools(Interval::All)
            .simd_lanes(Interval::All)
            .includes_scalars(true)
            .build(),
    );
    let x = &Operand::new("x", bits);
    let y = &Operand::new("y", bits);
    let a = &Operand::new("a", bits);

    ig.push(
        Inst::new(
            "band",
            r#"
        Bitwise and.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "bor",
            r#"
        Bitwise or.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "bxor",
            r#"
        Bitwise xor.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "bnot",
            r#"
        Bitwise not.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "band_not",
            r#"
        Bitwise and not.

        Computes `x & ~y`.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "bor_not",
            r#"
        Bitwise or not.

        Computes `x | ~y`.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "bxor_not",
            r#"
        Bitwise xor not.

        Computes `x ^ ~y`.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    let x = &Operand::new("x", iB);
    let Y = &Operand::new("Y", &imm.imm64);
    let a = &Operand::new("a", iB);

    ig.push(
        Inst::new(
            "band_imm",
            r#"
        Bitwise and with immediate.

        Same as `band`, but one operand is an immediate constant.

        Polymorphic over all scalar integer types, but does not support vector
        types.
        "#,
            &formats.binary_imm64,
        )
        .operands_in(vec![x, Y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "bor_imm",
            r#"
        Bitwise or with immediate.

        Same as `bor`, but one operand is an immediate constant.

        Polymorphic over all scalar integer types, but does not support vector
        types.
        "#,
            &formats.binary_imm64,
        )
        .operands_in(vec![x, Y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "bxor_imm",
            r#"
        Bitwise xor with immediate.

        Same as `bxor`, but one operand is an immediate constant.

        Polymorphic over all scalar integer types, but does not support vector
        types.
        "#,
            &formats.binary_imm64,
        )
        .operands_in(vec![x, Y])
        .operands_out(vec![a]),
    );

    let x = &Operand::new("x", Int).with_doc("Scalar or vector value to shift");
    let y = &Operand::new("y", iB).with_doc("Number of bits to shift");
    let Y = &Operand::new("Y", &imm.imm64);
    let a = &Operand::new("a", Int);

    ig.push(
        Inst::new(
            "rotl",
            r#"
        Rotate left.

        Rotate the bits in ``x`` by ``y`` places.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "rotr",
            r#"
        Rotate right.

        Rotate the bits in ``x`` by ``y`` places.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "rotl_imm",
            r#"
        Rotate left by immediate.
        "#,
            &formats.binary_imm64,
        )
        .operands_in(vec![x, Y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "rotr_imm",
            r#"
        Rotate right by immediate.
        "#,
            &formats.binary_imm64,
        )
        .operands_in(vec![x, Y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "ishl",
            r#"
        Integer shift left. Shift the bits in ``x`` towards the MSB by ``y``
        places. Shift in zero bits to the LSB.

        The shift amount is masked to the size of ``x``.

        When shifting a B-bits integer type, this instruction computes:

        ```text
            s &:= y \pmod B,
            a &:= x \cdot 2^s \pmod{2^B}.
        ```
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "ushr",
            r#"
        Unsigned shift right. Shift bits in ``x`` towards the LSB by ``y``
        places, shifting in zero bits to the MSB. Also called a *logical
        shift*.

        The shift amount is masked to the size of the register.

        When shifting a B-bits integer type, this instruction computes:

        ```text
            s &:= y \pmod B,
            a &:= \lfloor x \cdot 2^{-s} \rfloor.
        ```
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "sshr",
            r#"
        Signed shift right. Shift bits in ``x`` towards the LSB by ``y``
        places, shifting in sign bits to the MSB. Also called an *arithmetic
        shift*.

        The shift amount is masked to the size of the register.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "ishl_imm",
            r#"
        Integer shift left by immediate.

        The shift amount is masked to the size of ``x``.
        "#,
            &formats.binary_imm64,
        )
        .operands_in(vec![x, Y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "ushr_imm",
            r#"
        Unsigned shift right by immediate.

        The shift amount is masked to the size of the register.
        "#,
            &formats.binary_imm64,
        )
        .operands_in(vec![x, Y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "sshr_imm",
            r#"
        Signed shift right by immediate.

        The shift amount is masked to the size of the register.
        "#,
            &formats.binary_imm64,
        )
        .operands_in(vec![x, Y])
        .operands_out(vec![a]),
    );

    let x = &Operand::new("x", iB);
    let a = &Operand::new("a", iB);

    ig.push(
        Inst::new(
            "bitrev",
            r#"
        Reverse the bits of a integer.

        Reverses the bits in ``x``.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "clz",
            r#"
        Count leading zero bits.

        Starting from the MSB in ``x``, count the number of zero bits before
        reaching the first one bit. When ``x`` is zero, returns the size of x
        in bits.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "cls",
            r#"
        Count leading sign bits.

        Starting from the MSB after the sign bit in ``x``, count the number of
        consecutive bits identical to the sign bit. When ``x`` is 0 or -1,
        returns one less than the size of x in bits.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "ctz",
            r#"
        Count trailing zeros.

        Starting from the LSB in ``x``, count the number of zero bits before
        reaching the first one bit. When ``x`` is zero, returns the size of x
        in bits.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    let x = &Operand::new("x", Int);
    let a = &Operand::new("a", Int);

    ig.push(
        Inst::new(
            "popcnt",
            r#"
        Population count

        Count the number of one bits in ``x``.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    let Float = &TypeVar::new(
        "Float",
        "A scalar or vector floating point number",
        TypeSetBuilder::new()
            .floats(Interval::All)
            .simd_lanes(Interval::All)
            .build(),
    );
    let Cond = &Operand::new("Cond", &imm.floatcc);
    let x = &Operand::new("x", Float);
    let y = &Operand::new("y", Float);
    let a = &Operand::new("a", &Float.as_bool());

    ig.push(
        Inst::new(
            "fcmp",
            r#"
        Floating point comparison.

        Two IEEE 754-2008 floating point numbers, `x` and `y`, relate to each
        other in exactly one of four ways:

        ```text
        == ==========================================
        UN Unordered when one or both numbers is NaN.
        EQ When `x = y`. (And `0.0 = -0.0`).
        LT When `x < y`.
        GT When `x > y`.
        == ==========================================
        ```

        The 14 `floatcc` condition codes each correspond to a subset of
        the four relations, except for the empty set which would always be
        false, and the full set which would always be true.

        The condition codes are divided into 7 'ordered' conditions which don't
        include UN, and 7 unordered conditions which all include UN.

        ```text
        +-------+------------+---------+------------+-------------------------+
        |Ordered             |Unordered             |Condition                |
        +=======+============+=========+============+=========================+
        |ord    |EQ | LT | GT|uno      |UN          |NaNs absent / present.   |
        +-------+------------+---------+------------+-------------------------+
        |eq     |EQ          |ueq      |UN | EQ     |Equal                    |
        +-------+------------+---------+------------+-------------------------+
        |one    |LT | GT     |ne       |UN | LT | GT|Not equal                |
        +-------+------------+---------+------------+-------------------------+
        |lt     |LT          |ult      |UN | LT     |Less than                |
        +-------+------------+---------+------------+-------------------------+
        |le     |LT | EQ     |ule      |UN | LT | EQ|Less than or equal       |
        +-------+------------+---------+------------+-------------------------+
        |gt     |GT          |ugt      |UN | GT     |Greater than             |
        +-------+------------+---------+------------+-------------------------+
        |ge     |GT | EQ     |uge      |UN | GT | EQ|Greater than or equal    |
        +-------+------------+---------+------------+-------------------------+
        ```

        The standard C comparison operators, `<, <=, >, >=`, are all ordered,
        so they are false if either operand is NaN. The C equality operator,
        `==`, is ordered, and since inequality is defined as the logical
        inverse it is *unordered*. They map to the `floatcc` condition
        codes as follows:

        ```text
        ==== ====== ============
        C    `Cond` Subset
        ==== ====== ============
        `==` eq     EQ
        `!=` ne     UN | LT | GT
        `<`  lt     LT
        `<=` le     LT | EQ
        `>`  gt     GT
        `>=` ge     GT | EQ
        ==== ====== ============
        ```

        This subset of condition codes also corresponds to the WebAssembly
        floating point comparisons of the same name.

        When this instruction compares floating point vectors, it returns a
        boolean vector with the results of lane-wise comparisons.
        "#,
            &formats.float_compare,
        )
        .operands_in(vec![Cond, x, y])
        .operands_out(vec![a]),
    );

    let f = &Operand::new("f", fflags);

    ig.push(
        Inst::new(
            "ffcmp",
            r#"
        Floating point comparison returning flags.

        Compares two numbers like `fcmp`, but returns floating point CPU
        flags instead of testing a specific condition.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![f]),
    );

    let x = &Operand::new("x", Float);
    let y = &Operand::new("y", Float);
    let z = &Operand::new("z", Float);
    let a = &Operand::new("a", Float).with_doc("Result of applying operator to each lane");

    ig.push(
        Inst::new(
            "fadd",
            r#"
        Floating point addition.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "fsub",
            r#"
        Floating point subtraction.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "fmul",
            r#"
        Floating point multiplication.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "fdiv",
            r#"
        Floating point division.

        Unlike the integer division instructions ` and
        `udiv`, this can't trap. Division by zero is infinity or
        NaN, depending on the dividend.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "sqrt",
            r#"
        Floating point square root.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "fma",
            r#"
        Floating point fused multiply-and-add.

        Computes `a := xy+z` without any intermediate rounding of the
        product.
        "#,
            &formats.ternary,
        )
        .operands_in(vec![x, y, z])
        .operands_out(vec![a]),
    );

    let a = &Operand::new("a", Float).with_doc("``x`` with its sign bit inverted");

    ig.push(
        Inst::new(
            "fneg",
            r#"
        Floating point negation.

        Note that this is a pure bitwise operation.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    let a = &Operand::new("a", Float).with_doc("``x`` with its sign bit cleared");

    ig.push(
        Inst::new(
            "fabs",
            r#"
        Floating point absolute value.

        Note that this is a pure bitwise operation.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    let a = &Operand::new("a", Float).with_doc("``x`` with its sign bit changed to that of ``y``");

    ig.push(
        Inst::new(
            "fcopysign",
            r#"
        Floating point copy sign.

        Note that this is a pure bitwise operation. The sign bit from ``y`` is
        copied to the sign bit of ``x``.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    let a = &Operand::new("a", Float).with_doc("The smaller of ``x`` and ``y``");

    ig.push(
        Inst::new(
            "fmin",
            r#"
        Floating point minimum, propagating NaNs using the WebAssembly rules.

        If either operand is NaN, this returns NaN with an unspecified sign. Furthermore, if
        each input NaN consists of a mantissa whose most significant bit is 1 and the rest is
        0, then the output has the same form. Otherwise, the output mantissa's most significant
        bit is 1 and the rest is unspecified.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "fmin_pseudo",
            r#"
        Floating point pseudo-minimum, propagating NaNs.  This behaves differently from ``fmin``.
        See <https://github.com/WebAssembly/simd/pull/122> for background.

        The behaviour is defined as ``fmin_pseudo(a, b) = (b < a) ? b : a``, and the behaviour
        for zero or NaN inputs follows from the behaviour of ``<`` with such inputs.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    let a = &Operand::new("a", Float).with_doc("The larger of ``x`` and ``y``");

    ig.push(
        Inst::new(
            "fmax",
            r#"
        Floating point maximum, propagating NaNs using the WebAssembly rules.

        If either operand is NaN, this returns NaN with an unspecified sign. Furthermore, if
        each input NaN consists of a mantissa whose most significant bit is 1 and the rest is
        0, then the output has the same form. Otherwise, the output mantissa's most significant
        bit is 1 and the rest is unspecified.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "fmax_pseudo",
            r#"
        Floating point pseudo-maximum, propagating NaNs.  This behaves differently from ``fmax``.
        See <https://github.com/WebAssembly/simd/pull/122> for background.

        The behaviour is defined as ``fmax_pseudo(a, b) = (a < b) ? b : a``, and the behaviour
        for zero or NaN inputs follows from the behaviour of ``<`` with such inputs.
        "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    let a = &Operand::new("a", Float).with_doc("``x`` rounded to integral value");

    ig.push(
        Inst::new(
            "ceil",
            r#"
        Round floating point round to integral, towards positive infinity.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "floor",
            r#"
        Round floating point round to integral, towards negative infinity.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "trunc",
            r#"
        Round floating point round to integral, towards zero.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "nearest",
            r#"
        Round floating point round to integral, towards nearest with ties to
        even.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    let a = &Operand::new("a", b1);
    let x = &Operand::new("x", Ref);

    ig.push(
        Inst::new(
            "is_null",
            r#"
        Reference verification.

        The condition code determines if the reference type in question is
        null or not.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    let a = &Operand::new("a", b1);
    let x = &Operand::new("x", Ref);

    ig.push(
        Inst::new(
            "is_invalid",
            r#"
        Reference verification.

        The condition code determines if the reference type in question is
        invalid or not.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    let Cond = &Operand::new("Cond", &imm.intcc);
    let f = &Operand::new("f", iflags);
    let a = &Operand::new("a", b1);

    ig.push(
        Inst::new(
            "trueif",
            r#"
        Test integer CPU flags for a specific condition.

        Check the CPU flags in ``f`` against the ``Cond`` condition code and
        return true when the condition code is satisfied.
        "#,
            &formats.int_cond,
        )
        .operands_in(vec![Cond, f])
        .operands_out(vec![a]),
    );

    let Cond = &Operand::new("Cond", &imm.floatcc);
    let f = &Operand::new("f", fflags);

    ig.push(
        Inst::new(
            "trueff",
            r#"
        Test floating point CPU flags for a specific condition.

        Check the CPU flags in ``f`` against the ``Cond`` condition code and
        return true when the condition code is satisfied.
        "#,
            &formats.float_cond,
        )
        .operands_in(vec![Cond, f])
        .operands_out(vec![a]),
    );

    let x = &Operand::new("x", Mem);
    let a = &Operand::new("a", MemTo).with_doc("Bits of `x` reinterpreted");

    ig.push(
        Inst::new(
            "bitcast",
            r#"
        Reinterpret the bits in `x` as a different type.

        The input and output types must be storable to memory and of the same
        size. A bitcast is equivalent to storing one type and loading the other
        type from the same address.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    let x = &Operand::new("x", Any);
    let a = &Operand::new("a", AnyTo).with_doc("Bits of `x` reinterpreted");

    ig.push(
        Inst::new(
            "raw_bitcast",
            r#"
        Cast the bits in `x` as a different type of the same bit width.

        This instruction does not change the data's representation but allows
        data in registers to be used as different types, e.g. an i32x4 as a
        b8x16. The only constraint on the result `a` is that it can be
        `raw_bitcast` back to the original type. Also, in a raw_bitcast between
        vector types with the same number of lanes, the value of each result
        lane is a raw_bitcast of the corresponding operand lane. TODO there is
        currently no mechanism for enforcing the bit width constraint.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    let a = &Operand::new("a", TxN).with_doc("A vector value");
    let s = &Operand::new("s", &TxN.lane_of()).with_doc("A scalar value");

    ig.push(
        Inst::new(
            "scalar_to_vector",
            r#"
            Copies a scalar value to a vector value.  The scalar is copied into the
            least significant lane of the vector, and all other lanes will be zero.
            "#,
            &formats.unary,
        )
        .operands_in(vec![s])
        .operands_out(vec![a]),
    );

    let Bool = &TypeVar::new(
        "Bool",
        "A scalar or vector boolean type",
        TypeSetBuilder::new()
            .bools(Interval::All)
            .simd_lanes(Interval::All)
            .build(),
    );

    let BoolTo = &TypeVar::new(
        "BoolTo",
        "A smaller boolean type with the same number of lanes",
        TypeSetBuilder::new()
            .bools(Interval::All)
            .simd_lanes(Interval::All)
            .build(),
    );

    let x = &Operand::new("x", Bool);
    let a = &Operand::new("a", BoolTo);

    ig.push(
        Inst::new(
            "breduce",
            r#"
        Convert `x` to a smaller boolean type in the platform-defined way.

        The result type must have the same number of vector lanes as the input,
        and each lane must not have more bits that the input lanes. If the
        input and output types are the same, this is a no-op.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    let BoolTo = &TypeVar::new(
        "BoolTo",
        "A larger boolean type with the same number of lanes",
        TypeSetBuilder::new()
            .bools(Interval::All)
            .simd_lanes(Interval::All)
            .build(),
    );
    let x = &Operand::new("x", Bool);
    let a = &Operand::new("a", BoolTo);

    ig.push(
        Inst::new(
            "bextend",
            r#"
        Convert `x` to a larger boolean type in the platform-defined way.

        The result type must have the same number of vector lanes as the input,
        and each lane must not have fewer bits that the input lanes. If the
        input and output types are the same, this is a no-op.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    let IntTo = &TypeVar::new(
        "IntTo",
        "An integer type with the same number of lanes",
        TypeSetBuilder::new()
            .ints(Interval::All)
            .simd_lanes(Interval::All)
            .build(),
    );
    let x = &Operand::new("x", Bool);
    let a = &Operand::new("a", IntTo);

    ig.push(
        Inst::new(
            "bint",
            r#"
        Convert `x` to an integer.

        True maps to 1 and false maps to 0. The result type must have the same
        number of vector lanes as the input.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "bmask",
            r#"
        Convert `x` to an integer mask.

        True maps to all 1s and false maps to all 0s. The result type must have
        the same number of vector lanes as the input.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    let Int = &TypeVar::new(
        "Int",
        "A scalar or vector integer type",
        TypeSetBuilder::new()
            .ints(Interval::All)
            .simd_lanes(Interval::All)
            .build(),
    );

    let IntTo = &TypeVar::new(
        "IntTo",
        "A smaller integer type with the same number of lanes",
        TypeSetBuilder::new()
            .ints(Interval::All)
            .simd_lanes(Interval::All)
            .build(),
    );
    let x = &Operand::new("x", Int);
    let a = &Operand::new("a", IntTo);

    ig.push(
        Inst::new(
            "ireduce",
            r#"
        Convert `x` to a smaller integer type by dropping high bits.

        Each lane in `x` is converted to a smaller integer type by discarding
        the most significant bits. This is the same as reducing modulo
        `2^n`.

        The result type must have the same number of vector lanes as the input,
        and each lane must not have more bits that the input lanes. If the
        input and output types are the same, this is a no-op.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    let I16or32or64xN = &TypeVar::new(
        "I16or32or64xN",
        "A SIMD vector type containing integer lanes 16, 32, or 64 bits wide",
        TypeSetBuilder::new()
            .ints(16..64)
            .simd_lanes(2..8)
            .includes_scalars(false)
            .build(),
    );

    let x = &Operand::new("x", I16or32or64xN);
    let y = &Operand::new("y", I16or32or64xN);
    let a = &Operand::new("a", &I16or32or64xN.split_lanes());

    ig.push(
        Inst::new(
            "snarrow",
            r#"
        Combine `x` and `y` into a vector with twice the lanes but half the integer width while
        saturating overflowing values to the signed maximum and minimum.

        The lanes will be concatenated after narrowing. For example, when `x` and `y` are `i32x4`
        and `x = [x3, x2, x1, x0]` and `y = [y3, y2, y1, y0]`, then after narrowing the value
        returned is an `i16x8`: `a = [y3', y2', y1', y0', x3', x2', x1', x0']`.
            "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "unarrow",
            r#"
        Combine `x` and `y` into a vector with twice the lanes but half the integer width while
        saturating overflowing values to the unsigned maximum and minimum.

        Note that all input lanes are considered signed: any negative lanes will overflow and be
        replaced with the unsigned minimum, `0x00`.

        The lanes will be concatenated after narrowing. For example, when `x` and `y` are `i32x4`
        and `x = [x3, x2, x1, x0]` and `y = [y3, y2, y1, y0]`, then after narrowing the value
        returned is an `i16x8`: `a = [y3', y2', y1', y0', x3', x2', x1', x0']`.
            "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "uunarrow",
            r#"
        Combine `x` and `y` into a vector with twice the lanes but half the integer width while
        saturating overflowing values to the unsigned maximum and minimum.

        Note that all input lanes are considered unsigned: any negative values will be interpreted as unsigned, overflowing and being replaced with the unsigned maximum.

        The lanes will be concatenated after narrowing. For example, when `x` and `y` are `i32x4`
        and `x = [x3, x2, x1, x0]` and `y = [y3, y2, y1, y0]`, then after narrowing the value
        returned is an `i16x8`: `a = [y3', y2', y1', y0', x3', x2', x1', x0']`.
            "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    let I8or16or32xN = &TypeVar::new(
        "I8or16or32xN",
        "A SIMD vector type containing integer lanes 8, 16, or 32 bits wide.",
        TypeSetBuilder::new()
            .ints(8..32)
            .simd_lanes(4..16)
            .includes_scalars(false)
            .build(),
    );

    let x = &Operand::new("x", I8or16or32xN);
    let a = &Operand::new("a", &I8or16or32xN.merge_lanes());

    ig.push(
        Inst::new(
            "swiden_low",
            r#"
        Widen the low lanes of `x` using signed extension.

        This will double the lane width and halve the number of lanes.
            "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "swiden_high",
            r#"
        Widen the high lanes of `x` using signed extension.

        This will double the lane width and halve the number of lanes.
            "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "uwiden_low",
            r#"
        Widen the low lanes of `x` using unsigned extension.

        This will double the lane width and halve the number of lanes.
            "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "uwiden_high",
            r#"
            Widen the high lanes of `x` using unsigned extension.

            This will double the lane width and halve the number of lanes.
            "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    let x = &Operand::new("x", I8or16or32xN);
    let y = &Operand::new("y", I8or16or32xN);
    let a = &Operand::new("a", I8or16or32xN);

    ig.push(
        Inst::new(
            "iadd_pairwise",
            r#"
        Does lane-wise integer pairwise addition on two operands, putting the
        combined results into a single vector result. Here a pair refers to adjacent
        lanes in a vector, i.e. i*2 + (i*2+1) for i == num_lanes/2. The first operand
        pairwise add results will make up the low half of the resulting vector while
        the second operand pairwise add results will make up the upper half of the
        resulting vector.
            "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    let I16x8 = &TypeVar::new(
        "I16x8",
        "A SIMD vector type containing 8 integer lanes each 16 bits wide.",
        TypeSetBuilder::new()
            .ints(16..16)
            .simd_lanes(8..8)
            .includes_scalars(false)
            .build(),
    );

    let x = &Operand::new("x", I16x8);
    let y = &Operand::new("y", I16x8);
    let a = &Operand::new("a", &I16x8.merge_lanes());

    ig.push(
        Inst::new(
            "widening_pairwise_dot_product_s",
            r#"
        Takes corresponding elements in `x` and `y`, performs a sign-extending length-doubling
        multiplication on them, then adds adjacent pairs of elements to form the result.  For
        example, if the input vectors are `[x3, x2, x1, x0]` and `[y3, y2, y1, y0]`, it produces
        the vector `[r1, r0]`, where `r1 = sx(x3) * sx(y3) + sx(x2) * sx(y2)` and
        `r0 = sx(x1) * sx(y1) + sx(x0) * sx(y0)`, and `sx(n)` sign-extends `n` to twice its width.

        This will double the lane width and halve the number of lanes.  So the resulting
        vector has the same number of bits as `x` and `y` do (individually).

        See <https://github.com/WebAssembly/simd/pull/127> for background info.
            "#,
            &formats.binary,
        )
        .operands_in(vec![x, y])
        .operands_out(vec![a]),
    );

    let IntTo = &TypeVar::new(
        "IntTo",
        "A larger integer type with the same number of lanes",
        TypeSetBuilder::new()
            .ints(Interval::All)
            .simd_lanes(Interval::All)
            .build(),
    );
    let x = &Operand::new("x", Int);
    let a = &Operand::new("a", IntTo);

    ig.push(
        Inst::new(
            "uextend",
            r#"
        Convert `x` to a larger integer type by zero-extending.

        Each lane in `x` is converted to a larger integer type by adding
        zeroes. The result has the same numerical value as `x` when both are
        interpreted as unsigned integers.

        The result type must have the same number of vector lanes as the input,
        and each lane must not have fewer bits that the input lanes. If the
        input and output types are the same, this is a no-op.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "sextend",
            r#"
        Convert `x` to a larger integer type by sign-extending.

        Each lane in `x` is converted to a larger integer type by replicating
        the sign bit. The result has the same numerical value as `x` when both
        are interpreted as signed integers.

        The result type must have the same number of vector lanes as the input,
        and each lane must not have fewer bits that the input lanes. If the
        input and output types are the same, this is a no-op.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    let FloatTo = &TypeVar::new(
        "FloatTo",
        "A scalar or vector floating point number",
        TypeSetBuilder::new()
            .floats(Interval::All)
            .simd_lanes(Interval::All)
            .build(),
    );
    let x = &Operand::new("x", Float);
    let a = &Operand::new("a", FloatTo);

    ig.push(
        Inst::new(
            "fpromote",
            r#"
        Convert `x` to a larger floating point format.

        Each lane in `x` is converted to the destination floating point format.
        This is an exact operation.

        Cranelift currently only supports two floating point formats
        - `f32` and `f64`. This may change in the future.

        The result type must have the same number of vector lanes as the input,
        and the result lanes must not have fewer bits than the input lanes. If
        the input and output types are the same, this is a no-op.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "fdemote",
            r#"
        Convert `x` to a smaller floating point format.

        Each lane in `x` is converted to the destination floating point format
        by rounding to nearest, ties to even.

        Cranelift currently only supports two floating point formats
        - `f32` and `f64`. This may change in the future.

        The result type must have the same number of vector lanes as the input,
        and the result lanes must not have more bits than the input lanes. If
        the input and output types are the same, this is a no-op.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    let F64x2 = &TypeVar::new(
        "F64x2",
        "A SIMD vector type consisting of 2 lanes of 64-bit floats",
        TypeSetBuilder::new()
            .floats(64..64)
            .simd_lanes(2..2)
            .includes_scalars(false)
            .build(),
    );
    let F32x4 = &TypeVar::new(
        "F32x4",
        "A SIMD vector type consisting of 4 lanes of 32-bit floats",
        TypeSetBuilder::new()
            .floats(32..32)
            .simd_lanes(4..4)
            .includes_scalars(false)
            .build(),
    );

    let x = &Operand::new("x", F64x2);
    let a = &Operand::new("a", F32x4);

    ig.push(
        Inst::new(
            "fvdemote",
            r#"
                Convert `x` to a smaller floating point format.

                Each lane in `x` is converted to the destination floating point format
                by rounding to nearest, ties to even.

                Cranelift currently only supports two floating point formats
                - `f32` and `f64`. This may change in the future.

                Fvdemote differs from fdemote in that with fvdemote it targets vectors.
                Fvdemote is constrained to having the input type being F64x2 and the result
                type being F32x4. The result lane that was the upper half of the input lane
                is initialized to zero.
                "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "fvpromote_low",
            r#"
        Converts packed single precision floating point to packed double precision floating point.

        Considering only the lower half of the register, the low lanes in `x` are interpreted as
        single precision floats that are then converted to a double precision floats.

        The result type will have half the number of vector lanes as the input. Fvpromote_low is
        constrained to input F32x4 with a result type of F64x2.
        "#,
            &formats.unary,
        )
        .operands_in(vec![a])
        .operands_out(vec![x]),
    );

    let x = &Operand::new("x", Float);
    let a = &Operand::new("a", IntTo);

    ig.push(
        Inst::new(
            "fcvt_to_uint",
            r#"
        Convert floating point to unsigned integer.

        Each lane in `x` is converted to an unsigned integer by rounding
        towards zero. If `x` is NaN or if the unsigned integral value cannot be
        represented in the result type, this instruction traps.

        The result type must have the same number of vector lanes as the input.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a])
        .can_trap(true),
    );

    ig.push(
        Inst::new(
            "fcvt_to_uint_sat",
            r#"
        Convert floating point to unsigned integer as fcvt_to_uint does, but
        saturates the input instead of trapping. NaN and negative values are
        converted to 0.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "fcvt_to_sint",
            r#"
        Convert floating point to signed integer.

        Each lane in `x` is converted to a signed integer by rounding towards
        zero. If `x` is NaN or if the signed integral value cannot be
        represented in the result type, this instruction traps.

        The result type must have the same number of vector lanes as the input.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a])
        .can_trap(true),
    );

    ig.push(
        Inst::new(
            "fcvt_to_sint_sat",
            r#"
        Convert floating point to signed integer as fcvt_to_sint does, but
        saturates the input instead of trapping. NaN values are converted to 0.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    let x = &Operand::new("x", Int);
    let a = &Operand::new("a", FloatTo);

    ig.push(
        Inst::new(
            "fcvt_from_uint",
            r#"
        Convert unsigned integer to floating point.

        Each lane in `x` is interpreted as an unsigned integer and converted to
        floating point using round to nearest, ties to even.

        The result type must have the same number of vector lanes as the input.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "fcvt_from_sint",
            r#"
        Convert signed integer to floating point.

        Each lane in `x` is interpreted as a signed integer and converted to
        floating point using round to nearest, ties to even.

        The result type must have the same number of vector lanes as the input.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    ig.push(
        Inst::new(
            "fcvt_low_from_sint",
            r#"
        Converts packed signed 32-bit integers to packed double precision floating point.

        Considering only the low half of the register, each lane in `x` is interpreted as a
        signed 32-bit integer that is then converted to a double precision float. This
        instruction differs from fcvt_from_sint in that it converts half the number of lanes
        which are converted to occupy twice the number of bits. No rounding should be needed
        for the resulting float.

        The result type will have half the number of vector lanes as the input.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![a]),
    );

    let WideInt = &TypeVar::new(
        "WideInt",
        "An integer type with lanes from `i16` upwards",
        TypeSetBuilder::new()
            .ints(16..128)
            .simd_lanes(Interval::All)
            .build(),
    );
    let x = &Operand::new("x", WideInt);
    let lo = &Operand::new("lo", &WideInt.half_width()).with_doc("The low bits of `x`");
    let hi = &Operand::new("hi", &WideInt.half_width()).with_doc("The high bits of `x`");

    ig.push(
        Inst::new(
            "isplit",
            r#"
        Split an integer into low and high parts.

        Vectors of integers are split lane-wise, so the results have the same
        number of lanes as the input, but the lanes are half the size.

        Returns the low half of `x` and the high half of `x` as two independent
        values.
        "#,
            &formats.unary,
        )
        .operands_in(vec![x])
        .operands_out(vec![lo, hi]),
    );

    let NarrowInt = &TypeVar::new(
        "NarrowInt",
        "An integer type with lanes type to `i64`",
        TypeSetBuilder::new()
            .ints(8..64)
            .simd_lanes(Interval::All)
            .build(),
    );

    let lo = &Operand::new("lo", NarrowInt);
    let hi = &Operand::new("hi", NarrowInt);
    let a = &Operand::new("a", &NarrowInt.double_width())
        .with_doc("The concatenation of `lo` and `hi`");

    ig.push(
        Inst::new(
            "iconcat",
            r#"
        Concatenate low and high bits to form a larger integer type.

        Vectors of integers are concatenated lane-wise such that the result has
        the same number of lanes as the inputs, but the lanes are twice the
        size.
        "#,
            &formats.binary,
        )
        .operands_in(vec![lo, hi])
        .operands_out(vec![a]),
    );

    // Instructions relating to atomic memory accesses and fences
    let AtomicMem = &TypeVar::new(
        "AtomicMem",
        "Any type that can be stored in memory, which can be used in an atomic operation",
        TypeSetBuilder::new().ints(8..64).build(),
    );
    let x = &Operand::new("x", AtomicMem).with_doc("Value to be atomically stored");
    let a = &Operand::new("a", AtomicMem).with_doc("Value atomically loaded");
    let e = &Operand::new("e", AtomicMem).with_doc("Expected value in CAS");
    let p = &Operand::new("p", iAddr);
    let MemFlags = &Operand::new("MemFlags", &imm.memflags);
    let AtomicRmwOp = &Operand::new("AtomicRmwOp", &imm.atomic_rmw_op);

    ig.push(
        Inst::new(
            "atomic_rmw",
            r#"
        Atomically read-modify-write memory at `p`, with second operand `x`.  The old value is
        returned.  `p` has the type of the target word size, and `x` may be an integer type of
        8, 16, 32 or 64 bits, even on a 32-bit target.  The type of the returned value is the
        same as the type of `x`.  This operation is sequentially consistent and creates
        happens-before edges that order normal (non-atomic) loads and stores.
        "#,
            &formats.atomic_rmw,
        )
        .operands_in(vec![MemFlags, AtomicRmwOp, p, x])
        .operands_out(vec![a])
        .can_load(true)
        .can_store(true)
        .other_side_effects(true),
    );

    ig.push(
        Inst::new(
            "atomic_cas",
            r#"
        Perform an atomic compare-and-swap operation on memory at `p`, with expected value `e`,
        storing `x` if the value at `p` equals `e`.  The old value at `p` is returned,
        regardless of whether the operation succeeds or fails.  `p` has the type of the target
        word size, and `x` and `e` must have the same type and the same size, which may be an
        integer type of 8, 16, 32 or 64 bits, even on a 32-bit target.  The type of the returned
        value is the same as the type of `x` and `e`.  This operation is sequentially
        consistent and creates happens-before edges that order normal (non-atomic) loads and
        stores.
        "#,
            &formats.atomic_cas,
        )
        .operands_in(vec![MemFlags, p, e, x])
        .operands_out(vec![a])
        .can_load(true)
        .can_store(true)
        .other_side_effects(true),
    );

    ig.push(
        Inst::new(
            "atomic_load",
            r#"
        Atomically load from memory at `p`.

        This is a polymorphic instruction that can load any value type which has a memory
        representation.  It should only be used for integer types with 8, 16, 32 or 64 bits.
        This operation is sequentially consistent and creates happens-before edges that order
        normal (non-atomic) loads and stores.
        "#,
            &formats.load_no_offset,
        )
        .operands_in(vec![MemFlags, p])
        .operands_out(vec![a])
        .can_load(true)
        .other_side_effects(true),
    );

    ig.push(
        Inst::new(
            "atomic_store",
            r#"
        Atomically store `x` to memory at `p`.

        This is a polymorphic instruction that can store any value type with a memory
        representation.  It should only be used for integer types with 8, 16, 32 or 64 bits.
        This operation is sequentially consistent and creates happens-before edges that order
        normal (non-atomic) loads and stores.
        "#,
            &formats.store_no_offset,
        )
        .operands_in(vec![MemFlags, x, p])
        .can_store(true)
        .other_side_effects(true),
    );

    ig.push(
        Inst::new(
            "fence",
            r#"
        A memory fence.  This must provide ordering to ensure that, at a minimum, neither loads
        nor stores of any kind may move forwards or backwards across the fence.  This operation
        is sequentially consistent.
        "#,
            &formats.nullary,
        )
        .other_side_effects(true),
    );
}
