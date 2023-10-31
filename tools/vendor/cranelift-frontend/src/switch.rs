use super::HashMap;
use crate::frontend::FunctionBuilder;
use alloc::vec::Vec;
use core::convert::TryFrom;
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::*;

type EntryIndex = u128;

/// Unlike with `br_table`, `Switch` cases may be sparse or non-0-based.
/// They emit efficient code using branches, jump tables, or a combination of both.
///
/// # Example
///
/// ```rust
/// # use cranelift_codegen::ir::types::*;
/// # use cranelift_codegen::ir::{ExternalName, Function, Signature, InstBuilder};
/// # use cranelift_codegen::isa::CallConv;
/// # use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Switch};
/// #
/// # let mut sig = Signature::new(CallConv::SystemV);
/// # let mut fn_builder_ctx = FunctionBuilderContext::new();
/// # let mut func = Function::with_name_signature(ExternalName::user(0, 0), sig);
/// # let mut builder = FunctionBuilder::new(&mut func, &mut fn_builder_ctx);
/// #
/// # let entry = builder.create_block();
/// # builder.switch_to_block(entry);
/// #
/// let block0 = builder.create_block();
/// let block1 = builder.create_block();
/// let block2 = builder.create_block();
/// let fallback = builder.create_block();
///
/// let val = builder.ins().iconst(I32, 1);
///
/// let mut switch = Switch::new();
/// switch.set_entry(0, block0);
/// switch.set_entry(1, block1);
/// switch.set_entry(7, block2);
/// switch.emit(&mut builder, val, fallback);
/// ```
#[derive(Debug, Default)]
pub struct Switch {
    cases: HashMap<EntryIndex, Block>,
}

impl Switch {
    /// Create a new empty switch
    pub fn new() -> Self {
        Self {
            cases: HashMap::new(),
        }
    }

    /// Set a switch entry
    pub fn set_entry(&mut self, index: EntryIndex, block: Block) {
        let prev = self.cases.insert(index, block);
        assert!(
            prev.is_none(),
            "Tried to set the same entry {} twice",
            index
        );
    }

    /// Get a reference to all existing entries
    pub fn entries(&self) -> &HashMap<EntryIndex, Block> {
        &self.cases
    }

    /// Turn the `cases` `HashMap` into a list of `ContiguousCaseRange`s.
    ///
    /// # Postconditions
    ///
    /// * Every entry will be represented.
    /// * The `ContiguousCaseRange`s will not overlap.
    /// * Between two `ContiguousCaseRange`s there will be at least one entry index.
    /// * No `ContiguousCaseRange`s will be empty.
    fn collect_contiguous_case_ranges(self) -> Vec<ContiguousCaseRange> {
        log::trace!("build_contiguous_case_ranges before: {:#?}", self.cases);
        let mut cases = self.cases.into_iter().collect::<Vec<(_, _)>>();
        cases.sort_by_key(|&(index, _)| index);

        let mut contiguous_case_ranges: Vec<ContiguousCaseRange> = vec![];
        let mut last_index = None;
        for (index, block) in cases {
            match last_index {
                None => contiguous_case_ranges.push(ContiguousCaseRange::new(index)),
                Some(last_index) => {
                    if index > last_index + 1 {
                        contiguous_case_ranges.push(ContiguousCaseRange::new(index));
                    }
                }
            }
            contiguous_case_ranges
                .last_mut()
                .unwrap()
                .blocks
                .push(block);
            last_index = Some(index);
        }

        log::trace!(
            "build_contiguous_case_ranges after: {:#?}",
            contiguous_case_ranges
        );

        contiguous_case_ranges
    }

    /// Binary search for the right `ContiguousCaseRange`.
    fn build_search_tree(
        bx: &mut FunctionBuilder,
        val: Value,
        otherwise: Block,
        contiguous_case_ranges: Vec<ContiguousCaseRange>,
    ) -> Vec<(EntryIndex, Block, Vec<Block>)> {
        let mut cases_and_jt_blocks = Vec::new();

        // Avoid allocation in the common case
        if contiguous_case_ranges.len() <= 3 {
            Self::build_search_branches(
                bx,
                val,
                otherwise,
                contiguous_case_ranges,
                &mut cases_and_jt_blocks,
            );
            return cases_and_jt_blocks;
        }

        let mut stack: Vec<(Option<Block>, Vec<ContiguousCaseRange>)> = Vec::new();
        stack.push((None, contiguous_case_ranges));

        while let Some((block, contiguous_case_ranges)) = stack.pop() {
            if let Some(block) = block {
                bx.switch_to_block(block);
            }

            if contiguous_case_ranges.len() <= 3 {
                Self::build_search_branches(
                    bx,
                    val,
                    otherwise,
                    contiguous_case_ranges,
                    &mut cases_and_jt_blocks,
                );
            } else {
                let split_point = contiguous_case_ranges.len() / 2;
                let mut left = contiguous_case_ranges;
                let right = left.split_off(split_point);

                let left_block = bx.create_block();
                let right_block = bx.create_block();

                let first_index = right[0].first_index;
                let should_take_right_side =
                    icmp_imm_u128(bx, IntCC::UnsignedGreaterThanOrEqual, val, first_index);
                bx.ins().brnz(should_take_right_side, right_block, &[]);
                bx.ins().jump(left_block, &[]);

                bx.seal_block(left_block);
                bx.seal_block(right_block);

                stack.push((Some(left_block), left));
                stack.push((Some(right_block), right));
            }
        }

        cases_and_jt_blocks
    }

    /// Linear search for the right `ContiguousCaseRange`.
    fn build_search_branches(
        bx: &mut FunctionBuilder,
        val: Value,
        otherwise: Block,
        contiguous_case_ranges: Vec<ContiguousCaseRange>,
        cases_and_jt_blocks: &mut Vec<(EntryIndex, Block, Vec<Block>)>,
    ) {
        let mut was_branch = false;
        let ins_fallthrough_jump = |was_branch: bool, bx: &mut FunctionBuilder| {
            if was_branch {
                let block = bx.create_block();
                bx.ins().jump(block, &[]);
                bx.seal_block(block);
                bx.switch_to_block(block);
            }
        };
        for ContiguousCaseRange {
            first_index,
            blocks,
        } in contiguous_case_ranges.into_iter().rev()
        {
            match (blocks.len(), first_index) {
                (1, 0) => {
                    ins_fallthrough_jump(was_branch, bx);
                    bx.ins().brz(val, blocks[0], &[]);
                }
                (1, _) => {
                    ins_fallthrough_jump(was_branch, bx);
                    let is_good_val = icmp_imm_u128(bx, IntCC::Equal, val, first_index);
                    bx.ins().brnz(is_good_val, blocks[0], &[]);
                }
                (_, 0) => {
                    // if `first_index` is 0, then `icmp_imm uge val, first_index` is trivially true
                    let jt_block = bx.create_block();
                    bx.ins().jump(jt_block, &[]);
                    bx.seal_block(jt_block);
                    cases_and_jt_blocks.push((first_index, jt_block, blocks));
                    // `jump otherwise` below must not be hit, because the current block has been
                    // filled above. This is the last iteration anyway, as 0 is the smallest
                    // unsigned int, so just return here.
                    return;
                }
                (_, _) => {
                    ins_fallthrough_jump(was_branch, bx);
                    let jt_block = bx.create_block();
                    let is_good_val =
                        icmp_imm_u128(bx, IntCC::UnsignedGreaterThanOrEqual, val, first_index);
                    bx.ins().brnz(is_good_val, jt_block, &[]);
                    bx.seal_block(jt_block);
                    cases_and_jt_blocks.push((first_index, jt_block, blocks));
                }
            }
            was_branch = true;
        }

        bx.ins().jump(otherwise, &[]);
    }

    /// For every item in `cases_and_jt_blocks` this will create a jump table in the specified block.
    fn build_jump_tables(
        bx: &mut FunctionBuilder,
        val: Value,
        otherwise: Block,
        cases_and_jt_blocks: Vec<(EntryIndex, Block, Vec<Block>)>,
    ) {
        for (first_index, jt_block, blocks) in cases_and_jt_blocks.into_iter().rev() {
            // There are currently no 128bit systems supported by rustc, but once we do ensure that
            // we don't silently ignore a part of the jump table for 128bit integers on 128bit systems.
            assert!(
                u32::try_from(blocks.len()).is_ok(),
                "Jump tables bigger than 2^32-1 are not yet supported"
            );

            let mut jt_data = JumpTableData::new();
            for block in blocks {
                jt_data.push_entry(block);
            }
            let jump_table = bx.create_jump_table(jt_data);

            bx.switch_to_block(jt_block);
            let discr = if first_index == 0 {
                val
            } else {
                if let Ok(first_index) = u64::try_from(first_index) {
                    bx.ins().iadd_imm(val, (first_index as i64).wrapping_neg())
                } else {
                    let (lsb, msb) = (first_index as u64, (first_index >> 64) as u64);
                    let lsb = bx.ins().iconst(types::I64, lsb as i64);
                    let msb = bx.ins().iconst(types::I64, msb as i64);
                    let index = bx.ins().iconcat(lsb, msb);
                    bx.ins().isub(val, index)
                }
            };

            let discr = if bx.func.dfg.value_type(discr).bits() > 64 {
                // Check for overflow of cast to u32. This is the max supported jump table entries.
                let new_block = bx.create_block();
                let bigger_than_u32 =
                    bx.ins()
                        .icmp_imm(IntCC::UnsignedGreaterThan, discr, u32::MAX as i64);
                bx.ins().brnz(bigger_than_u32, otherwise, &[]);
                bx.ins().jump(new_block, &[]);
                bx.seal_block(new_block);
                bx.switch_to_block(new_block);

                // Cast to u64, as br_table is not implemented for i128
                bx.ins().isplit(discr).0
            } else {
                discr
            };

            bx.ins().br_table(discr, otherwise, jump_table);
        }
    }

    /// Build the switch
    ///
    /// # Arguments
    ///
    /// * The function builder to emit to
    /// * The value to switch on
    /// * The default block
    pub fn emit(self, bx: &mut FunctionBuilder, val: Value, otherwise: Block) {
        // FIXME icmp(_imm) doesn't have encodings for i8 and i16 on x86(_64) yet
        let val = match bx.func.dfg.value_type(val) {
            types::I8 | types::I16 => bx.ins().uextend(types::I32, val),
            _ => val,
        };

        let contiguous_case_ranges = self.collect_contiguous_case_ranges();
        let cases_and_jt_blocks =
            Self::build_search_tree(bx, val, otherwise, contiguous_case_ranges);
        Self::build_jump_tables(bx, val, otherwise, cases_and_jt_blocks);
    }
}

fn icmp_imm_u128(bx: &mut FunctionBuilder, cond: IntCC, x: Value, y: u128) -> Value {
    if let Ok(index) = u64::try_from(y) {
        bx.ins().icmp_imm(cond, x, index as i64)
    } else {
        let (lsb, msb) = (y as u64, (y >> 64) as u64);
        let lsb = bx.ins().iconst(types::I64, lsb as i64);
        let msb = bx.ins().iconst(types::I64, msb as i64);
        let index = bx.ins().iconcat(lsb, msb);
        bx.ins().icmp(cond, x, index)
    }
}

/// This represents a contiguous range of cases to switch on.
///
/// For example 10 => block1, 11 => block2, 12 => block7 will be represented as:
///
/// ```plain
/// ContiguousCaseRange {
///     first_index: 10,
///     blocks: vec![Block::from_u32(1), Block::from_u32(2), Block::from_u32(7)]
/// }
/// ```
#[derive(Debug)]
struct ContiguousCaseRange {
    /// The entry index of the first case. Eg. 10 when the entry indexes are 10, 11, 12 and 13.
    first_index: EntryIndex,

    /// The blocks to jump to sorted in ascending order of entry index.
    blocks: Vec<Block>,
}

impl ContiguousCaseRange {
    fn new(first_index: EntryIndex) -> Self {
        Self {
            first_index,
            blocks: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::FunctionBuilderContext;
    use alloc::string::ToString;
    use cranelift_codegen::ir::Function;

    macro_rules! setup {
        ($default:expr, [$($index:expr,)*]) => {{
            let mut func = Function::new();
            let mut func_ctx = FunctionBuilderContext::new();
            {
                let mut bx = FunctionBuilder::new(&mut func, &mut func_ctx);
                let block = bx.create_block();
                bx.switch_to_block(block);
                let val = bx.ins().iconst(types::I8, 0);
                let mut switch = Switch::new();
                $(
                    let block = bx.create_block();
                    switch.set_entry($index, block);
                )*
                switch.emit(&mut bx, val, Block::with_number($default).unwrap());
            }
            func
                .to_string()
                .trim_start_matches("function u0:0() fast {\n")
                .trim_end_matches("\n}\n")
                .to_string()
        }};
    }

    #[test]
    fn switch_zero() {
        let func = setup!(0, [0,]);
        assert_eq!(
            func,
            "block0:
    v0 = iconst.i8 0
    v1 = uextend.i32 v0
    brz v1, block1
    jump block0"
        );
    }

    #[test]
    fn switch_single() {
        let func = setup!(0, [1,]);
        assert_eq!(
            func,
            "block0:
    v0 = iconst.i8 0
    v1 = uextend.i32 v0
    v2 = icmp_imm eq v1, 1
    brnz v2, block1
    jump block0"
        );
    }

    #[test]
    fn switch_bool() {
        let func = setup!(0, [0, 1,]);
        assert_eq!(
            func,
            "    jt0 = jump_table [block1, block2]

block0:
    v0 = iconst.i8 0
    v1 = uextend.i32 v0
    jump block3

block3:
    br_table.i32 v1, block0, jt0"
        );
    }

    #[test]
    fn switch_two_gap() {
        let func = setup!(0, [0, 2,]);
        assert_eq!(
            func,
            "block0:
    v0 = iconst.i8 0
    v1 = uextend.i32 v0
    v2 = icmp_imm eq v1, 2
    brnz v2, block2
    jump block3

block3:
    brz.i32 v1, block1
    jump block0"
        );
    }

    #[test]
    fn switch_many() {
        let func = setup!(0, [0, 1, 5, 7, 10, 11, 12,]);
        assert_eq!(
            func,
            "    jt0 = jump_table [block1, block2]
    jt1 = jump_table [block5, block6, block7]

block0:
    v0 = iconst.i8 0
    v1 = uextend.i32 v0
    v2 = icmp_imm uge v1, 7
    brnz v2, block9
    jump block8

block9:
    v3 = icmp_imm.i32 uge v1, 10
    brnz v3, block10
    jump block11

block11:
    v4 = icmp_imm.i32 eq v1, 7
    brnz v4, block4
    jump block0

block8:
    v5 = icmp_imm.i32 eq v1, 5
    brnz v5, block3
    jump block12

block12:
    br_table.i32 v1, block0, jt0

block10:
    v6 = iadd_imm.i32 v1, -10
    br_table v6, block0, jt1"
        );
    }

    #[test]
    fn switch_min_index_value() {
        let func = setup!(0, [::core::i64::MIN as u64 as u128, 1,]);
        assert_eq!(
            func,
            "block0:
    v0 = iconst.i8 0
    v1 = uextend.i32 v0
    v2 = icmp_imm eq v1, 0x8000_0000_0000_0000
    brnz v2, block1
    jump block3

block3:
    v3 = icmp_imm.i32 eq v1, 1
    brnz v3, block2
    jump block0"
        );
    }

    #[test]
    fn switch_max_index_value() {
        let func = setup!(0, [::core::i64::MAX as u64 as u128, 1,]);
        assert_eq!(
            func,
            "block0:
    v0 = iconst.i8 0
    v1 = uextend.i32 v0
    v2 = icmp_imm eq v1, 0x7fff_ffff_ffff_ffff
    brnz v2, block1
    jump block3

block3:
    v3 = icmp_imm.i32 eq v1, 1
    brnz v3, block2
    jump block0"
        )
    }

    #[test]
    fn switch_optimal_codegen() {
        let func = setup!(0, [-1i64 as u64 as u128, 0, 1,]);
        assert_eq!(
            func,
            "    jt0 = jump_table [block2, block3]

block0:
    v0 = iconst.i8 0
    v1 = uextend.i32 v0
    v2 = icmp_imm eq v1, -1
    brnz v2, block1
    jump block4

block4:
    br_table.i32 v1, block0, jt0"
        );
    }

    #[test]
    fn switch_seal_generated_blocks() {
        let cases = &[vec![0, 1, 2], vec![0, 1, 2, 10, 11, 12, 20, 30, 40, 50]];

        for case in cases {
            for typ in &[types::I8, types::I16, types::I32, types::I64, types::I128] {
                eprintln!("Testing {:?} with keys: {:?}", typ, case);
                do_case(case, *typ);
            }
        }

        fn do_case(keys: &[u128], typ: Type) {
            let mut func = Function::new();
            let mut builder_ctx = FunctionBuilderContext::new();
            let mut builder = FunctionBuilder::new(&mut func, &mut builder_ctx);

            let root_block = builder.create_block();
            let default_block = builder.create_block();
            let mut switch = Switch::new();

            let case_blocks = keys
                .iter()
                .map(|key| {
                    let block = builder.create_block();
                    switch.set_entry(*key, block);
                    block
                })
                .collect::<Vec<_>>();

            builder.seal_block(root_block);
            builder.switch_to_block(root_block);

            let val = builder.ins().iconst(typ, 1);
            switch.emit(&mut builder, val, default_block);

            for &block in case_blocks.iter().chain(std::iter::once(&default_block)) {
                builder.seal_block(block);
                builder.switch_to_block(block);
                builder.ins().return_(&[]);
            }

            builder.finalize(); // Will panic if some blocks are not sealed
        }
    }

    #[test]
    fn switch_64bit() {
        let mut func = Function::new();
        let mut func_ctx = FunctionBuilderContext::new();
        {
            let mut bx = FunctionBuilder::new(&mut func, &mut func_ctx);
            let block0 = bx.create_block();
            bx.switch_to_block(block0);
            let val = bx.ins().iconst(types::I64, 0);
            let mut switch = Switch::new();
            let block1 = bx.create_block();
            switch.set_entry(1, block1);
            let block2 = bx.create_block();
            switch.set_entry(0, block2);
            let block3 = bx.create_block();
            switch.emit(&mut bx, val, block3);
        }
        let func = func
            .to_string()
            .trim_start_matches("function u0:0() fast {\n")
            .trim_end_matches("\n}\n")
            .to_string();
        assert_eq!(
            func,
            "    jt0 = jump_table [block2, block1]

block0:
    v0 = iconst.i64 0
    jump block4

block4:
    br_table.i64 v0, block3, jt0"
        );
    }

    #[test]
    fn switch_128bit() {
        let mut func = Function::new();
        let mut func_ctx = FunctionBuilderContext::new();
        {
            let mut bx = FunctionBuilder::new(&mut func, &mut func_ctx);
            let block0 = bx.create_block();
            bx.switch_to_block(block0);
            let val = bx.ins().iconst(types::I128, 0);
            let mut switch = Switch::new();
            let block1 = bx.create_block();
            switch.set_entry(1, block1);
            let block2 = bx.create_block();
            switch.set_entry(0, block2);
            let block3 = bx.create_block();
            switch.emit(&mut bx, val, block3);
        }
        let func = func
            .to_string()
            .trim_start_matches("function u0:0() fast {\n")
            .trim_end_matches("\n}\n")
            .to_string();
        assert_eq!(
            func,
            "    jt0 = jump_table [block2, block1]

block0:
    v0 = iconst.i128 0
    jump block4

block4:
    v1 = icmp_imm.i128 ugt v0, 0xffff_ffff
    brnz v1, block3
    jump block5

block5:
    v2, v3 = isplit.i128 v0
    br_table v2, block3, jt0"
        );
    }
}
