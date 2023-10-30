//! Performs dataflow and liveness analysis, including live range construction.

use log::{log_enabled, trace, Level};
use smallvec::{smallvec, SmallVec};
use std::cmp::min;
use std::fmt;

use crate::data_structures::*;
use crate::sparse_set::SparseSet;
use crate::union_find::{ToFromU32, UnionFind};
use crate::Function;
use crate::{analysis_control_flow::CFGInfo, analysis_main::DepthBasedFrequencies};

//===========================================================================//
//                                                                           //
// DATA FLOW AND LIVENESS ANALYSIS                                           //
//                                                                           //
//===========================================================================//

//=============================================================================
// Data flow analysis: extraction and sanitization of reg-use information: low
// level interface

// === The meaning of "sanitization" ===
//
// The meaning of "sanitization" is as follows.  Incoming virtual-registerised
// code may mention a mixture of virtual and real registers.  Those real
// registers may include some which aren't available for the allocators to
// use.  Rather than scatter ad-hoc logic all over the analysis phase and the
// allocators, we simply remove all non-available real registers from the
// per-instruction use/def/mod sets.  The effect is that, after this point, we
// can operate on the assumption that any register we come across is either a
// virtual register or a real register available to the allocator.
//
// A real register is available to the allocator iff its index number is less
// than `RealRegUniverse.allocable`.
//
// Furthermore, it is not allowed that any incoming instruction mentions one
// of the per-class scratch registers listed in
// `RealRegUniverse.allocable_by_class[..].suggested_scratch` in either a use
// or mod role.  Sanitisation will also detect this case and return an error.
// Mentions of a scratch register in a def role are tolerated; however, since
// no instruction may use or modify a scratch register, all such writes are
// dead..
//
// In all of the above, "mentions" of a real register really means "uses,
// defines or modifications of said register".  It doesn't matter whether the
// instruction explicitly mentions the register or whether it is an implicit
// mention (eg, %cl in x86 shift-by-a-variable-amount instructions).  In other
// words, a "mention" is any use, def or mod as detected by the client's
// `get_regs` routine.

// === Filtering of register groups in `RegVec`s ===
//
// Filtering on a group is done by leaving the start point unchanged, sliding
// back retained registers to fill the holes from non-retained registers, and
// reducing the group length accordingly.  The effect is to effectively "leak"
// some registers in the group, but that's not a problem.
//
// Extraction of register usages for the whole function is done by
// `get_sanitized_reg_uses_for_func`.  For each instruction, their used,
// defined and modified register sets are acquired by calling the client's
// `get_regs` function.  Then each of those three sets are cleaned up as
// follows:
//
// (1) duplicates are removed (after which they really are sets)
//
// (2) any registers in the modified set are removed from the used and defined
//     sets.  This enforces the invariant that
//    `intersect(modified, union(used, defined))` is the empty set.  Live range
//    fragment computation (get_range_frags_for_block) depends on this property.
//
// (3) real registers unavailable to the allocator are removed, per the
//     abovementioned sanitization rules.

// ==== LOCAL FN ====
// Given a register group in `regs[start, +len)`, remove duplicates from the
// group.  The new group size is written to `*len`.
#[inline(never)]
fn remove_dups_from_group(regs: &mut Vec<Reg>, start: u32, len: &mut u8) {
    // First sort the group, to facilitate de-duplication.
    regs[start as usize..start as usize + *len as usize].sort_unstable();

    // Now make a compacting pass over the group.  'rd' = read point in the
    // group, 'wr' = write point in the group.
    let mut wr = start as usize;
    for rd in start as usize..start as usize + *len as usize {
        let reg = regs[rd];
        if rd == start as usize || regs[rd - 1] != reg {
            // It's not a duplicate.
            if wr != rd {
                regs[wr] = reg;
            }
            wr += 1;
        }
    }

    let new_len_usize = wr - start as usize;
    assert!(new_len_usize <= *len as usize);
    // This narrowing is safe because the old `len` fitted in 8 bits.
    *len = new_len_usize as u8;
}

// ==== LOCAL FN ====
// Remove from `group[group_start, +group_len)` any registers mentioned in
// `mods[mods_start, +mods_len)`, and update `*group_len` accordingly.
#[inline(never)]
fn remove_mods_from_group(
    group: &mut Vec<Reg>,
    group_start: u32,
    group_len: &mut u8,
    mods: &Vec<Reg>,
    mods_start: u32,
    mods_len: u8,
) {
    let mut wr = group_start as usize;
    for rd in group_start as usize..group_start as usize + *group_len as usize {
        let reg = group[rd];
        // Only retain `reg` if it is not mentioned in `mods[mods_start, +mods_len)`
        let mut retain = true;
        for i in mods_start as usize..mods_start as usize + mods_len as usize {
            if reg == mods[i] {
                retain = false;
                break;
            }
        }
        if retain {
            if wr != rd {
                group[wr] = reg;
            }
            wr += 1;
        }
    }
    let new_group_len_usize = wr - group_start as usize;
    assert!(new_group_len_usize <= *group_len as usize);
    // This narrowing is safe because the old `group_len` fitted in 8 bits.
    *group_len = new_group_len_usize as u8;
}

// ==== EXPORTED FN ====
// For instruction `inst`, add the register uses to the ends of `reg_vecs`,
// and write bounds information into `bounds`.  The register uses are raw
// (unsanitized) but they are guaranteed to be duplicate-free and also to have
// no `mod` mentions in the `use` or `def` groups.  That is, cleanups (1) and
// (2) above have been done.
#[inline(never)]
pub fn add_raw_reg_vecs_for_insn<F: Function>(
    inst: &F::Inst,
    reg_vecs: &mut RegVecs,
    bounds: &mut RegVecBounds,
) {
    bounds.uses_start = reg_vecs.uses.len() as u32;
    bounds.defs_start = reg_vecs.defs.len() as u32;
    bounds.mods_start = reg_vecs.mods.len() as u32;

    let mut collector = RegUsageCollector::new(reg_vecs);
    F::get_regs(inst, &mut collector);

    let uses_len = collector.reg_vecs.uses.len() as u32 - bounds.uses_start;
    let defs_len = collector.reg_vecs.defs.len() as u32 - bounds.defs_start;
    let mods_len = collector.reg_vecs.mods.len() as u32 - bounds.mods_start;

    // This assertion is important -- the cleanup logic also depends on it.
    assert!((uses_len | defs_len | mods_len) < 256);
    bounds.uses_len = uses_len as u8;
    bounds.defs_len = defs_len as u8;
    bounds.mods_len = mods_len as u8;

    // First, de-dup the three new groups.
    if bounds.uses_len > 0 {
        remove_dups_from_group(
            &mut collector.reg_vecs.uses,
            bounds.uses_start,
            &mut bounds.uses_len,
        );
    }
    if bounds.defs_len > 0 {
        remove_dups_from_group(
            &mut collector.reg_vecs.defs,
            bounds.defs_start,
            &mut bounds.defs_len,
        );
    }
    if bounds.mods_len > 0 {
        remove_dups_from_group(
            &mut collector.reg_vecs.mods,
            bounds.mods_start,
            &mut bounds.mods_len,
        );
    }

    // And finally, remove modified registers from the set of used and defined
    // registers, so we don't have to make the client do so.
    if bounds.mods_len > 0 {
        if bounds.uses_len > 0 {
            remove_mods_from_group(
                &mut collector.reg_vecs.uses,
                bounds.uses_start,
                &mut bounds.uses_len,
                &collector.reg_vecs.mods,
                bounds.mods_start,
                bounds.mods_len,
            );
        }
        if bounds.defs_len > 0 {
            remove_mods_from_group(
                &mut collector.reg_vecs.defs,
                bounds.defs_start,
                &mut bounds.defs_len,
                &collector.reg_vecs.mods,
                bounds.mods_start,
                bounds.mods_len,
            );
        }
    }
}

// ==== LOCAL FN ====
// This is the fundamental keep-or-don't-keep? predicate for sanitization.  To
// do this exactly right we also need to know whether the register is
// mentioned in a def role (as opposed to a use or mod role).  Note that this
// function can fail, and the error must be propagated.
#[inline(never)]
fn sanitize_should_retain_reg(
    reg_universe: &RealRegUniverse,
    reg: Reg,
    reg_is_defd: bool,
) -> Result<bool, RealReg> {
    // Retain all virtual regs.
    if reg.is_virtual() {
        return Ok(true);
    }

    // So it's a RealReg.
    let rreg_ix = reg.get_index();

    // Check that this RealReg is mentioned in the universe.
    if rreg_ix >= reg_universe.regs.len() {
        // This is a serious error which should be investigated.  It means the
        // client gave us an instruction which mentions a RealReg which isn't
        // listed in the RealRegUniverse it gave us.  That's not allowed.
        return Err(reg.as_real_reg().unwrap());
    }

    // Discard all real regs that aren't available to the allocator.
    if rreg_ix >= reg_universe.allocable {
        return Ok(false);
    }

    // It isn't allowed for the client to give us an instruction which reads or
    // modifies one of the scratch registers.  It is however allowed to write a
    // scratch register.
    for reg_info in &reg_universe.allocable_by_class {
        if let Some(reg_info) = reg_info {
            if let Some(scratch_idx) = &reg_info.suggested_scratch {
                let scratch_reg = reg_universe.regs[*scratch_idx].0;
                if reg.to_real_reg() == scratch_reg {
                    if !reg_is_defd {
                        // This is an error (on the part of the client).
                        return Err(reg.as_real_reg().unwrap());
                    }
                }
            }
        }
    }

    // `reg` is mentioned in the universe, is available to the allocator, and if
    // it is one of the scratch regs, it is only written, not read or modified.
    Ok(true)
}
// END helper fn

// ==== LOCAL FN ====
// Given a register group in `regs[start, +len)`, sanitize the group.  To do
// this exactly right we also need to know whether the registers in the group
// are mentioned in def roles (as opposed to use or mod roles).  Sanitisation
// can fail, in which case we must propagate the error.  If it is successful,
// the new group size is written to `*len`.
#[inline(never)]
fn sanitize_group(
    reg_universe: &RealRegUniverse,
    regs: &mut Vec<Reg>,
    start: u32,
    len: &mut u8,
    is_def_group: bool,
) -> Result<(), RealReg> {
    // Make a single compacting pass over the group.  'rd' = read point in the
    // group, 'wr' = write point in the group.
    let mut wr = start as usize;
    for rd in start as usize..start as usize + *len as usize {
        let reg = regs[rd];
        // This call can fail:
        if sanitize_should_retain_reg(reg_universe, reg, is_def_group)? {
            if wr != rd {
                regs[wr] = reg;
            }
            wr += 1;
        }
    }

    let new_len_usize = wr - start as usize;
    assert!(new_len_usize <= *len as usize);
    // This narrowing is safe because the old `len` fitted in 8 bits.
    *len = new_len_usize as u8;
    Ok(())
}

// ==== LOCAL FN ====
// For instruction `inst`, add the fully cleaned-up register uses to the ends
// of `reg_vecs`, and write bounds information into `bounds`.  Cleanups (1)
// (2) and (3) mentioned above have been done.  Note, this can fail, and the
// error must be propagated.
#[inline(never)]
fn add_san_reg_vecs_for_insn<F: Function>(
    inst: &F::Inst,
    reg_universe: &RealRegUniverse,
    reg_vecs: &mut RegVecs,
    bounds: &mut RegVecBounds,
) -> Result<(), RealReg> {
    // Get the raw reg usages.  These will be dup-free and mod-cleaned-up
    // (meaning cleanups (1) and (3) have been done).
    add_raw_reg_vecs_for_insn::<F>(inst, reg_vecs, bounds);

    // Finally and sanitize them.  Any errors from sanitization are propagated.
    if bounds.uses_len > 0 {
        sanitize_group(
            &reg_universe,
            &mut reg_vecs.uses,
            bounds.uses_start,
            &mut bounds.uses_len,
            /*is_def_group=*/ false,
        )?;
    }
    if bounds.defs_len > 0 {
        sanitize_group(
            &reg_universe,
            &mut reg_vecs.defs,
            bounds.defs_start,
            &mut bounds.defs_len,
            /*is_def_group=*/ true,
        )?;
    }
    if bounds.mods_len > 0 {
        sanitize_group(
            &reg_universe,
            &mut reg_vecs.mods,
            bounds.mods_start,
            &mut bounds.mods_len,
            /*is_def_group=*/ false,
        )?;
    }

    Ok(())
}

// ==== MAIN FN ====
#[inline(never)]
pub fn get_sanitized_reg_uses_for_func<F: Function>(
    func: &F,
    reg_universe: &RealRegUniverse,
) -> Result<RegVecsAndBounds, RealReg> {
    // These are modified by the per-insn loop.
    let mut reg_vecs = RegVecs::new(false);
    let mut bounds_vec = TypedIxVec::<InstIx, RegVecBounds>::new();
    bounds_vec.reserve(func.insns().len());

    // For each insn, add their register uses to the ends of the 3 vectors in
    // `reg_vecs`, and create an admin entry to describe the 3 new groups.  Any
    // errors from sanitization are propagated.
    for insn in func.insns() {
        let mut bounds = RegVecBounds::new();
        add_san_reg_vecs_for_insn::<F>(insn, &reg_universe, &mut reg_vecs, &mut bounds)?;

        bounds_vec.push(bounds);
    }

    assert!(!reg_vecs.is_sanitized());
    reg_vecs.set_sanitized(true);

    if log_enabled!(Level::Trace) {
        let show_reg = |r: Reg| {
            if r.is_real() {
                reg_universe.regs[r.get_index()].1.clone()
            } else {
                format!("{:?}", r).to_string()
            }
        };
        let show_regs = |r_vec: &[Reg]| {
            let mut s = "".to_string();
            for r in r_vec {
                s = s + &show_reg(*r) + &" ".to_string();
            }
            s
        };

        for i in 0..bounds_vec.len() {
            let iix = InstIx::new(i);
            let s_use = show_regs(
                &reg_vecs.uses[bounds_vec[iix].uses_start as usize
                    ..bounds_vec[iix].uses_start as usize + bounds_vec[iix].uses_len as usize],
            );
            let s_mod = show_regs(
                &reg_vecs.mods[bounds_vec[iix].mods_start as usize
                    ..bounds_vec[iix].mods_start as usize + bounds_vec[iix].mods_len as usize],
            );
            let s_def = show_regs(
                &reg_vecs.defs[bounds_vec[iix].defs_start as usize
                    ..bounds_vec[iix].defs_start as usize + bounds_vec[iix].defs_len as usize],
            );
            trace!(
                "{:?}  SAN_RU: use {{ {}}} mod {{ {}}} def {{ {}}}",
                iix,
                s_use,
                s_mod,
                s_def
            );
        }
    }

    Ok(RegVecsAndBounds::new(reg_vecs, bounds_vec))
}
// END main function

//=============================================================================
// Data flow analysis: extraction and sanitization of reg-use information:
// convenience interface

// ==== EXPORTED ====
#[inline(always)]
pub fn does_inst_use_def_or_mod_reg(
    rvb: &RegVecsAndBounds,
    iix: InstIx,
    reg: Reg,
) -> (/*uses*/ bool, /*defs*/ bool, /*mods*/ bool) {
    let bounds = &rvb.bounds[iix];
    let vecs = &rvb.vecs;
    let mut uses = false;
    let mut defs = false;
    let mut mods = false;
    // Since each group of registers is in order and duplicate-free (as a result
    // of `remove_dups_from_group`), we could in theory binary-search here.  But
    // it'd almost certainly be a net loss; the group sizes are very small,
    // often zero.
    for i in bounds.uses_start as usize..bounds.uses_start as usize + bounds.uses_len as usize {
        if vecs.uses[i] == reg {
            uses = true;
            break;
        }
    }
    for i in bounds.defs_start as usize..bounds.defs_start as usize + bounds.defs_len as usize {
        if vecs.defs[i] == reg {
            defs = true;
            break;
        }
    }
    for i in bounds.mods_start as usize..bounds.mods_start as usize + bounds.mods_len as usize {
        if vecs.mods[i] == reg {
            mods = true;
            break;
        }
    }
    (uses, defs, mods)
}

// ==== EXPORTED ====
// This is slow, really slow.  Don't use it on critical paths.  This applies
// `get_regs` to `inst`, performs cleanups (1) and (2), but does not sanitize
// the results.  The results are wrapped up as Sets for convenience.
// JRS 2020Apr09: remove this if no further use for it appears soon.
#[allow(dead_code)]
#[inline(never)]
pub fn get_raw_reg_sets_for_insn<F: Function>(inst: &F::Inst) -> RegSets {
    let mut reg_vecs = RegVecs::new(false);
    let mut bounds = RegVecBounds::new();

    add_raw_reg_vecs_for_insn::<F>(inst, &mut reg_vecs, &mut bounds);

    // Make up a fake RegVecsAndBounds for just this insn, so we can hand it to
    // RegVecsAndBounds::get_reg_sets_for_iix.
    let mut single_insn_bounds = TypedIxVec::<InstIx, RegVecBounds>::new();
    single_insn_bounds.push(bounds);

    assert!(!reg_vecs.is_sanitized());
    let single_insn_rvb = RegVecsAndBounds::new(reg_vecs, single_insn_bounds);
    single_insn_rvb.get_reg_sets_for_iix(InstIx::new(0))
}

// ==== EXPORTED ====
// This is even slower.  This applies `get_regs` to `inst`, performs cleanups
// (1) (2) and (3).  The results are wrapped up as Sets for convenience.  Note
// this function can fail.
#[inline(never)]
pub fn get_san_reg_sets_for_insn<F: Function>(
    inst: &F::Inst,
    reg_universe: &RealRegUniverse,
) -> Result<RegSets, RealReg> {
    let mut reg_vecs = RegVecs::new(false);
    let mut bounds = RegVecBounds::new();

    add_san_reg_vecs_for_insn::<F>(inst, &reg_universe, &mut reg_vecs, &mut bounds)?;

    // Make up a fake RegVecsAndBounds for just this insn, so we can hand it to
    // RegVecsAndBounds::get_reg_sets_for_iix.
    let mut single_insn_bounds = TypedIxVec::<InstIx, RegVecBounds>::new();
    single_insn_bounds.push(bounds);

    assert!(!reg_vecs.is_sanitized());
    reg_vecs.set_sanitized(true);
    let single_insn_rvb = RegVecsAndBounds::new(reg_vecs, single_insn_bounds);
    Ok(single_insn_rvb.get_reg_sets_for_iix(InstIx::new(0)))
}

//=============================================================================
// Data flow analysis: calculation of per-block register def and use sets

// Returned TypedIxVecs contain one element per block
#[inline(never)]
pub fn calc_def_and_use<F: Function>(
    func: &F,
    rvb: &RegVecsAndBounds,
    univ: &RealRegUniverse,
) -> (
    TypedIxVec<BlockIx, SparseSet<Reg>>,
    TypedIxVec<BlockIx, SparseSet<Reg>>,
) {
    trace!("    calc_def_and_use: begin");
    assert!(rvb.is_sanitized());
    let mut def_sets = TypedIxVec::new();
    let mut use_sets = TypedIxVec::new();
    for b in func.blocks() {
        let mut def = SparseSet::empty();
        let mut uce = SparseSet::empty();
        for iix in func.block_insns(b) {
            let bounds_for_iix = &rvb.bounds[iix];
            // Add to `uce`, any registers for which the first event in this block
            // is a read.  Dealing with the "first event" constraint is a bit
            // tricky.  In the next two loops, `u` and `m` is used (either read or
            // modified) by the instruction.  Whether or not we should consider it
            // live-in for the block depends on whether it was been written earlier
            // in the block.  We can determine that by checking whether it is
            // already in the def set for the block.
            // FIXME: isn't thus just:
            //   uce union= (regs_u minus def)   followed by
            //   uce union= (regs_m minus def)
            for i in bounds_for_iix.uses_start as usize
                ..bounds_for_iix.uses_start as usize + bounds_for_iix.uses_len as usize
            {
                let u = rvb.vecs.uses[i];
                if !def.contains(u) {
                    uce.insert(u);
                }
            }
            for i in bounds_for_iix.mods_start as usize
                ..bounds_for_iix.mods_start as usize + bounds_for_iix.mods_len as usize
            {
                let m = rvb.vecs.mods[i];
                if !def.contains(m) {
                    uce.insert(m);
                }
            }

            // Now add to `def`, all registers written by the instruction.
            // This is simpler.
            // FIXME: isn't this just: def union= (regs_d union regs_m) ?
            for i in bounds_for_iix.defs_start as usize
                ..bounds_for_iix.defs_start as usize + bounds_for_iix.defs_len as usize
            {
                let d = rvb.vecs.defs[i];
                def.insert(d);
            }
            for i in bounds_for_iix.mods_start as usize
                ..bounds_for_iix.mods_start as usize + bounds_for_iix.mods_len as usize
            {
                let m = rvb.vecs.mods[i];
                def.insert(m);
            }
        }
        def_sets.push(def);
        use_sets.push(uce);
    }

    assert!(def_sets.len() == use_sets.len());

    if log_enabled!(Level::Trace) {
        let mut n = 0;
        trace!("");
        for (def_set, use_set) in def_sets.iter().zip(use_sets.iter()) {
            let mut first = true;
            let mut defs_str = "".to_string();
            for def in def_set.to_vec() {
                if !first {
                    defs_str = defs_str + &" ".to_string();
                }
                first = false;
                defs_str = defs_str + &def.show_with_rru(univ);
            }
            first = true;
            let mut uses_str = "".to_string();
            for uce in use_set.to_vec() {
                if !first {
                    uses_str = uses_str + &" ".to_string();
                }
                first = false;
                uses_str = uses_str + &uce.show_with_rru(univ);
            }
            trace!(
                "{:<3?}   def {{{}}}  use {{{}}}",
                BlockIx::new(n),
                defs_str,
                uses_str
            );
            n += 1;
        }
    }

    trace!("    calc_def_and_use: end");
    (def_sets, use_sets)
}

//=============================================================================
// Data flow analysis: computation of per-block register live-in and live-out
// sets

// Returned vectors contain one element per block
#[inline(never)]
pub fn calc_livein_and_liveout<F: Function>(
    func: &F,
    def_sets_per_block: &TypedIxVec<BlockIx, SparseSet<Reg>>,
    use_sets_per_block: &TypedIxVec<BlockIx, SparseSet<Reg>>,
    cfg_info: &CFGInfo,
    univ: &RealRegUniverse,
) -> (
    TypedIxVec<BlockIx, SparseSet<Reg>>,
    TypedIxVec<BlockIx, SparseSet<Reg>>,
) {
    trace!("    calc_livein_and_liveout: begin");
    let num_blocks = func.blocks().len() as u32;
    let empty = SparseSet::<Reg>::empty();

    let mut num_evals = 0;
    let mut liveouts = TypedIxVec::<BlockIx, SparseSet<Reg>>::new();
    liveouts.resize(num_blocks, empty.clone());

    // Initialise the work queue so as to do a reverse preorder traversal
    // through the graph, after which blocks are re-evaluated on demand.
    let mut work_queue = Queue::<BlockIx>::new();
    for i in 0..num_blocks {
        // block_ix travels in "reverse preorder"
        let block_ix = cfg_info.pre_ord[(num_blocks - 1 - i) as usize];
        work_queue.push_back(block_ix);
    }

    // in_queue is an optimisation -- this routine works fine without it.  in_queue is
    // used to avoid inserting duplicate work items in work_queue.  This avoids some
    // number of duplicate re-evaluations and gets us to a fixed point faster.
    // Very roughly, it reduces the number of evaluations per block from around
    // 3 to around 2.
    let mut in_queue = Vec::<bool>::new();
    in_queue.resize(num_blocks as usize, true);

    while let Some(block_ix) = work_queue.pop_front() {
        let i = block_ix.get() as usize;
        assert!(in_queue[i]);
        in_queue[i] = false;

        // Compute a new value for liveouts[block_ix]
        let mut set = SparseSet::<Reg>::empty();
        for block_j_ix in cfg_info.succ_map[block_ix].iter() {
            let mut live_in_j = liveouts[*block_j_ix].clone();
            live_in_j.remove(&def_sets_per_block[*block_j_ix]);
            live_in_j.union(&use_sets_per_block[*block_j_ix]);
            set.union(&live_in_j);
        }
        num_evals += 1;

        if !set.equals(&liveouts[block_ix]) {
            liveouts[block_ix] = set;
            // Add `block_ix`'s predecessors to the work queue, since their
            // liveout values might be affected.
            for block_j_ix in cfg_info.pred_map[block_ix].iter() {
                let j = block_j_ix.get() as usize;
                if !in_queue[j] {
                    work_queue.push_back(*block_j_ix);
                    in_queue[j] = true;
                }
            }
        }
    }

    // The liveout values are done, but we need to compute the liveins
    // too.
    let mut liveins = TypedIxVec::<BlockIx, SparseSet<Reg>>::new();
    liveins.resize(num_blocks, empty.clone());
    for block_ix in BlockIx::new(0).dotdot(BlockIx::new(num_blocks)) {
        let mut live_in = liveouts[block_ix].clone();
        live_in.remove(&def_sets_per_block[block_ix]);
        live_in.union(&use_sets_per_block[block_ix]);
        liveins[block_ix] = live_in;
    }

    if false {
        let mut sum_card_live_in = 0;
        let mut sum_card_live_out = 0;
        for bix in BlockIx::new(0).dotdot(BlockIx::new(num_blocks)) {
            sum_card_live_in += liveins[bix].card();
            sum_card_live_out += liveouts[bix].card();
        }
        println!(
            "QQQQ calc_LI/LO: num_evals {}, tot LI {}, tot LO {}",
            num_evals, sum_card_live_in, sum_card_live_out
        );
    }

    let ratio: f32 = (num_evals as f32) / ((if num_blocks == 0 { 1 } else { num_blocks }) as f32);
    trace!(
        "    calc_livein_and_liveout:   {} blocks, {} evals ({:<.2} per block)",
        num_blocks,
        num_evals,
        ratio
    );

    if log_enabled!(Level::Trace) {
        let mut n = 0;
        trace!("");
        for (livein, liveout) in liveins.iter().zip(liveouts.iter()) {
            let mut first = true;
            let mut li_str = "".to_string();
            for li in livein.to_vec() {
                if !first {
                    li_str = li_str + &" ".to_string();
                }
                first = false;
                li_str = li_str + &li.show_with_rru(univ);
            }
            first = true;
            let mut lo_str = "".to_string();
            for lo in liveout.to_vec() {
                if !first {
                    lo_str = lo_str + &" ".to_string();
                }
                first = false;
                lo_str = lo_str + &lo.show_with_rru(univ);
            }
            trace!(
                "{:<3?}   livein {{{}}}  liveout {{{}}}",
                BlockIx::new(n),
                li_str,
                lo_str
            );
            n += 1;
        }
    }

    trace!("    calc_livein_and_liveout: end");
    (liveins, liveouts)
}

//=============================================================================
// Computation of RangeFrags (Live Range Fragments), aggregated per register.
// This does not produce complete live ranges.  That is done later, by
// `merge_range_frags` below, using the information computed in this section
// by `get_range_frags`.

// This is surprisingly complex, in part because of the need to correctly
// handle (1) live-in and live-out Regs, (2) dead writes, and (3) instructions
// that modify registers rather than merely reading or writing them.

/// A ProtoRangeFrag carries information about a [write .. read] range, within a Block, which
/// we will later turn into a fully-fledged RangeFrag.  It basically records the first and
/// last-known InstPoints for appearances of a Reg.
///
/// ProtoRangeFrag also keeps count of the number of appearances of the Reg to which it
/// pertains, using `uses`.  The counts get rolled into the resulting RangeFrags, and later are
/// used to calculate spill costs.
///
/// The running state of this function is a map from Reg to ProtoRangeFrag.  Only Regs that
/// actually appear in the Block (or are live-in to it) are mapped.  This has the advantage of
/// economy, since most Regs will not appear in (or be live-in to) most Blocks.
#[derive(Clone)]
struct ProtoRangeFrag {
    /// The InstPoint in this Block at which the associated Reg most recently became live (when
    /// moving forwards though the Block).  If this value is the first InstPoint for the Block
    /// (the U point for the Block's lowest InstIx), that indicates the associated Reg is
    /// live-in to the Block.
    first: InstPoint,

    /// This is the InstPoint which is the end point (most recently observed read, in general)
    /// for the current RangeFrag under construction.  In general we will move `last` forwards
    /// as we discover reads of the associated Reg.  If this is the last InstPoint for the
    /// Block (the D point for the Block's highest InstInx), that indicates that the associated
    /// reg is live-out from the Block.
    last: InstPoint,

    /// Number of mentions of the associated Reg in this ProtoRangeFrag.
    num_mentions: u16,
}

impl fmt::Debug for ProtoRangeFrag {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "{:?}x {:?} - {:?}",
            self.num_mentions, self.first, self.last
        )
    }
}

// `fn get_range_frags` and `fn get_range_frags_for_block` below work with two vectors,
// `out_map` and `state`, that are indexed by register.  This allows them to entirely avoid the
// use of hash-based `Map`s.  However, it does give a problem in that real and virtual registers
// occupy separate, zero-based index spaces.  To solve this, we map `Reg`s to a "unified index
// space" as follows:
//
// a `RealReg` is mapped to its `.get_index()` value
//
// a `VirtualReg` is mapped to its `.get_index()` value + the number of real registers
//
// To make this not too inconvenient, `fn reg_to_reg_ix` and `fn reg_ix_to_reg` convert `Reg`s
// to and from the unified index space.  This has the annoying side effect that reconstructing a
// `Reg` from an index space value requires having available both the register universe, and a
// table specifying the class for each virtual register.
//
// Really, we ought to rework the `Reg`/`RealReg`/`VirtualReg` abstractions, so as to (1) impose
// a single index space for both register kinds, and (2) so as to separate the concepts of the
// register index from the `Reg` itself.  This second point would have the additional benefit of
// making it feasible to represent sets of registers using bit sets.

#[inline(always)]
pub(crate) fn reg_to_reg_ix(num_real_regs: u32, r: Reg) -> u32 {
    if r.is_real() {
        r.get_index_u32()
    } else {
        num_real_regs + r.get_index_u32()
    }
}

#[inline(always)]
pub(crate) fn reg_ix_to_reg(
    reg_universe: &RealRegUniverse,
    vreg_classes: &Vec</*vreg index,*/ RegClass>,
    reg_ix: u32,
) -> Reg {
    let reg_ix = reg_ix as usize;
    let num_real_regs = reg_universe.regs.len();
    if reg_ix < num_real_regs {
        reg_universe.regs[reg_ix].0.to_reg()
    } else {
        let vreg_ix = reg_ix - num_real_regs;
        Reg::new_virtual(vreg_classes[vreg_ix], vreg_ix as u32)
    }
}

// HELPER FUNCTION
// Add to `out_map`, a binding from `reg` to the frags-and-metrics pair specified by `frag` and
// `frag_metrics`.  As a space-saving optimisation, make some attempt to avoid creating
// duplicate entries in `out_frags` and `out_frag_metrics`.
#[inline(always)]
fn emit_range_frag(
    out_map: &mut Vec</*rreg index, then vreg index, */ SmallVec<[RangeFragIx; 8]>>,
    out_frags: &mut TypedIxVec<RangeFragIx, RangeFrag>,
    out_frag_metrics: &mut TypedIxVec<RangeFragIx, RangeFragMetrics>,
    num_real_regs: u32,
    reg: Reg,
    frag: &RangeFrag,
    frag_metrics: &RangeFragMetrics,
) {
    debug_assert!(out_frags.len() == out_frag_metrics.len());

    // Allocate a new RangeFragIx for `frag`, except, make some minimal effort to avoid huge
    // numbers of duplicates by inspecting the previous two entries, and using them if
    // possible.
    let mut new_fix = None;

    let num_out_frags = out_frags.len();
    if num_out_frags >= 2 {
        let back_0 = RangeFragIx::new(num_out_frags - 1);
        let back_1 = RangeFragIx::new(num_out_frags - 2);
        if out_frags[back_0] == *frag && out_frag_metrics[back_0] == *frag_metrics {
            new_fix = Some(back_0);
        } else if out_frags[back_1] == *frag && out_frag_metrics[back_1] == *frag_metrics {
            new_fix = Some(back_1);
        }
    }

    let new_fix = match new_fix {
        Some(fix) => fix,
        None => {
            // We can't look back or there was no match; create a new one.
            out_frags.push(frag.clone());
            out_frag_metrics.push(frag_metrics.clone());
            RangeFragIx::new(out_frags.len() as u32 - 1)
        }
    };

    // And use the new RangeFragIx.
    out_map[reg_to_reg_ix(num_real_regs, reg) as usize].push(new_fix);
}

/// Calculate all the RangeFrags for `bix`.  Add them to `out_frags` and corresponding metrics
/// data to `out_frag_metrics`.  Add to `out_map`, the associated RangeFragIxs, segregated by
/// Reg.  `bix`, `livein`, `liveout` and `rvb` are expected to be valid in the context of the
/// Func `f` (duh!).
#[inline(never)]
fn get_range_frags_for_block<F: Function>(
    // Constants
    func: &F,
    rvb: &RegVecsAndBounds,
    reg_universe: &RealRegUniverse,
    vreg_classes: &Vec</*vreg index,*/ RegClass>,
    bix: BlockIx,
    livein: &SparseSet<Reg>,
    liveout: &SparseSet<Reg>,
    // Preallocated storage for use in this function.  They do not carry any useful information
    // in between calls here.
    visited: &mut Vec<u32>,
    state: &mut Vec</*rreg index, then vreg index, */ Option<ProtoRangeFrag>>,
    // These accumulate the results of RangeFrag/RangeFragMetrics across multiple calls here.
    out_map: &mut Vec</*rreg index, then vreg index, */ SmallVec<[RangeFragIx; 8]>>,
    out_frags: &mut TypedIxVec<RangeFragIx, RangeFrag>,
    out_frag_metrics: &mut TypedIxVec<RangeFragIx, RangeFragMetrics>,
) {
    #[inline(always)]
    fn plus1(n: u16) -> u16 {
        if n == 0xFFFFu16 {
            n
        } else {
            n + 1
        }
    }

    // Invariants for the preallocated storage:
    //
    // * `visited` is always irrelevant (and cleared) at the start
    //
    // * `state` always has size (# real regs + # virtual regs).  However, all its entries
    // should be `None` in between calls here.

    // We use `visited` to keep track of which `state` entries need processing at the end of
    // this function.  Since `state` is indexed by unified-reg-index, it follows that `visited`
    // is a vector of unified-reg-indices.  We add an entry to `visited` whenever we change a
    // `state` entry from `None` to `Some`.  This guarantees that we can find all the `Some`
    // `state` entries at the end of the function, change them back to `None`, and emit the
    // corresponding fragment.
    visited.clear();

    // Some handy constants.
    assert!(func.block_insns(bix).len() >= 1);
    let first_iix_in_block = func.block_insns(bix).first();
    let last_iix_in_block = func.block_insns(bix).last();
    let first_pt_in_block = InstPoint::new_use(first_iix_in_block);
    let last_pt_in_block = InstPoint::new_def(last_iix_in_block);
    let num_real_regs = reg_universe.regs.len() as u32;

    // First, set up `state` as if all of `livein` had been written just prior to the block.
    for r in livein.iter() {
        let r_state_ix = reg_to_reg_ix(num_real_regs, *r) as usize;
        debug_assert!(state[r_state_ix].is_none());
        state[r_state_ix] = Some(ProtoRangeFrag {
            num_mentions: 0,
            first: first_pt_in_block,
            last: first_pt_in_block,
        });
        visited.push(r_state_ix as u32);
    }

    // Now visit each instruction in turn, examining first the registers it reads, then those it
    // modifies, and finally those it writes.
    for iix in func.block_insns(bix) {
        let bounds_for_iix = &rvb.bounds[iix];

        // Examine reads.  This is pretty simple.  They simply extend an existing ProtoRangeFrag
        // to the U point of the reading insn.
        for i in
            bounds_for_iix.uses_start..bounds_for_iix.uses_start + bounds_for_iix.uses_len as u32
        {
            let r = rvb.vecs.uses[i as usize];
            let r_state_ix = reg_to_reg_ix(num_real_regs, r) as usize;
            match &mut state[r_state_ix] {
                // First event for `r` is a read, but it's not listed in `livein`, since otherwise
                // `state` would have an entry for it.
                None => panic!("get_range_frags_for_block: fail #1"),
                Some(ref mut pf) => {
                    // This the first or subsequent read after a write.  Note that the "write" can
                    // be either a real write, or due to the fact that `r` is listed in `livein`.
                    // We don't care here.
                    pf.num_mentions = plus1(pf.num_mentions);
                    let new_last = InstPoint::new_use(iix);
                    debug_assert!(pf.last <= new_last);
                    pf.last = new_last;
                }
            }
        }

        // Examine modifies.  These are handled almost identically to reads, except that they
        // extend an existing ProtoRangeFrag down to the D point of the modifying insn.
        for i in
            bounds_for_iix.mods_start..bounds_for_iix.mods_start + bounds_for_iix.mods_len as u32
        {
            let r = &rvb.vecs.mods[i as usize];
            let r_state_ix = reg_to_reg_ix(num_real_regs, *r) as usize;
            match &mut state[r_state_ix] {
                // First event for `r` is a read (really, since this insn modifies `r`), but it's
                // not listed in `livein`, since otherwise `state` would have an entry for it.
                None => panic!("get_range_frags_for_block: fail #2"),
                Some(ref mut pf) => {
                    // This the first or subsequent modify after a write.  Add two to the
                    // mentions count, as that reflects the implied spill cost increment more
                    // accurately than just adding one: if we spill the live range in which this
                    // ends up, we'll generate both a reload and a spill instruction.
                    pf.num_mentions = plus1(plus1(pf.num_mentions));
                    let new_last = InstPoint::new_def(iix);
                    debug_assert!(pf.last <= new_last);
                    pf.last = new_last;
                }
            }
        }

        // Examine writes (but not writes implied by modifies).  The general idea is that a
        // write causes us to terminate and emit the existing ProtoRangeFrag, if any, and start
        // a new frag.
        for i in
            bounds_for_iix.defs_start..bounds_for_iix.defs_start + bounds_for_iix.defs_len as u32
        {
            let r = &rvb.vecs.defs[i as usize];
            let r_state_ix = reg_to_reg_ix(num_real_regs, *r) as usize;
            match &mut state[r_state_ix] {
                // First mention of a Reg we've never heard of before.  Start a new
                // ProtoRangeFrag for it and keep going.
                None => {
                    let new_pt = InstPoint::new_def(iix);
                    let new_pf = ProtoRangeFrag {
                        num_mentions: 1,
                        first: new_pt,
                        last: new_pt,
                    };
                    state[r_state_ix] = Some(new_pf);
                    visited.push(r_state_ix as u32);
                }

                // There's already a ProtoRangeFrag for `r`.  This write will start a new one,
                // so emit the existing one and note this write.
                Some(ProtoRangeFrag {
                    ref mut num_mentions,
                    ref mut first,
                    ref mut last,
                }) => {
                    if first == last {
                        debug_assert!(*num_mentions == 1);
                    }

                    let (frag, frag_metrics) =
                        RangeFrag::new_with_metrics(func, bix, *first, *last, *num_mentions);
                    emit_range_frag(
                        out_map,
                        out_frags,
                        out_frag_metrics,
                        num_real_regs,
                        *r,
                        &frag,
                        &frag_metrics,
                    );
                    let new_pt = InstPoint::new_def(iix);
                    // Reuse the previous entry for this new definition of the same vreg.
                    *num_mentions = 1;
                    *first = new_pt;
                    *last = new_pt;
                }
            }
        }
    }

    // We are at the end of the block.  We still have to deal with live-out Regs.  We must also
    // deal with ProtoRangeFrags in `state` that are for registers not listed as live-out.

    // Deal with live-out Regs.  Treat each one as if it is read just after the block.
    for r in liveout.iter() {
        let r_state_ix = reg_to_reg_ix(num_real_regs, *r) as usize;
        let state_elem_p = &mut state[r_state_ix];
        match state_elem_p {
            // This can't happen.  `r` is in `liveout`, but this implies that it is neither
            // defined in the block nor present in `livein`.
            None => panic!("get_range_frags_for_block: fail #3"),
            Some(ref pf) => {
                // `r` is written (or modified), either literally or by virtue of being present
                // in `livein`, and may or may not subsequently be read -- we don't care,
                // because it must be read "after" the block.  Create a `LiveOut` or `Thru` frag
                // accordingly.
                let (frag, frag_metrics) = RangeFrag::new_with_metrics(
                    func,
                    bix,
                    pf.first,
                    last_pt_in_block,
                    pf.num_mentions,
                );
                emit_range_frag(
                    out_map,
                    out_frags,
                    out_frag_metrics,
                    num_real_regs,
                    *r,
                    &frag,
                    &frag_metrics,
                );
                // Remove the entry from `state` so that the following loop doesn't process it
                // again.
                *state_elem_p = None;
            }
        }
    }

    // Finally, round up any remaining ProtoRangeFrags left in `state`.  This is what `visited`
    // is used for.
    for r_state_ix in visited {
        let state_elem_p = &mut state[*r_state_ix as usize];
        match state_elem_p {
            None => {}
            Some(pf) => {
                if pf.first == pf.last {
                    debug_assert!(pf.num_mentions == 1);
                }
                let (frag, frag_metrics) =
                    RangeFrag::new_with_metrics(func, bix, pf.first, pf.last, pf.num_mentions);
                let r = reg_ix_to_reg(reg_universe, vreg_classes, *r_state_ix);
                emit_range_frag(
                    out_map,
                    out_frags,
                    out_frag_metrics,
                    num_real_regs,
                    r,
                    &frag,
                    &frag_metrics,
                );
                // Maintain invariant that all `state` entries are `None` in between calls to
                // this function.
                *state_elem_p = None;
            }
        }
    }
}

#[inline(never)]
pub fn get_range_frags<F: Function>(
    func: &F,
    rvb: &RegVecsAndBounds,
    reg_universe: &RealRegUniverse,
    livein_sets_per_block: &TypedIxVec<BlockIx, SparseSet<Reg>>,
    liveout_sets_per_block: &TypedIxVec<BlockIx, SparseSet<Reg>>,
) -> (
    Vec</*rreg index, then vreg index, */ SmallVec<[RangeFragIx; 8]>>,
    TypedIxVec<RangeFragIx, RangeFrag>,
    TypedIxVec<RangeFragIx, RangeFragMetrics>,
    Vec</*vreg index,*/ RegClass>,
) {
    trace!("    get_range_frags: begin");
    assert!(livein_sets_per_block.len() == func.blocks().len() as u32);
    assert!(liveout_sets_per_block.len() == func.blocks().len() as u32);
    assert!(rvb.is_sanitized());

    // In order that we can work with unified-reg-indices (see comments above), we need to know
    // the `RegClass` for each virtual register.  That info is collected here.
    let mut vreg_classes = vec![RegClass::INVALID; func.get_num_vregs()];
    for r in rvb
        .vecs
        .uses
        .iter()
        .chain(rvb.vecs.defs.iter())
        .chain(rvb.vecs.mods.iter())
    {
        if r.is_real() {
            continue;
        }
        let r_ix = r.get_index();
        // rustc 1.43.0 appears to have problems avoiding duplicate bounds checks for
        // `vreg_classes[r_ix]`; hence give it a helping hand here.
        let vreg_classes_ptr = &mut vreg_classes[r_ix];
        if *vreg_classes_ptr == RegClass::INVALID {
            *vreg_classes_ptr = r.get_class();
        } else {
            assert_eq!(*vreg_classes_ptr, r.get_class());
        }
    }

    let num_real_regs = reg_universe.regs.len();
    let num_virtual_regs = vreg_classes.len();
    let num_regs = num_real_regs + num_virtual_regs;

    // A state variable that's reused across calls to `get_range_frags_for_block`.  When not in
    // a call to `get_range_frags_for_block`, all entries should be `None`.
    let mut state = Vec::</*rreg index, then vreg index, */ Option<ProtoRangeFrag>>::new();
    state.resize(num_regs, None);

    // Scratch storage needed by `get_range_frags_for_block`.  Doesn't carry any useful info in
    // between calls.  Start it off not-quite-empty since it will always get used at least a
    // bit.
    let mut visited = Vec::<u32>::with_capacity(32);

    // `RangeFrag`/`RangeFragMetrics` are collected across multiple calls to
    // `get_range_frag_for_blocks` in these three vectors.  In other words, they collect the
    // overall results for this function.
    let mut result_frags = TypedIxVec::<RangeFragIx, RangeFrag>::new();
    let mut result_frag_metrics = TypedIxVec::<RangeFragIx, RangeFragMetrics>::new();
    let mut result_map =
        Vec::</*rreg index, then vreg index, */ SmallVec<[RangeFragIx; 8]>>::default();
    result_map.resize(num_regs, smallvec![]);

    for bix in func.blocks() {
        get_range_frags_for_block(
            func,
            rvb,
            reg_universe,
            &vreg_classes,
            bix,
            &livein_sets_per_block[bix],
            &liveout_sets_per_block[bix],
            &mut visited,
            &mut state,
            &mut result_map,
            &mut result_frags,
            &mut result_frag_metrics,
        );
    }

    assert!(state.len() == num_regs);
    assert!(result_map.len() == num_regs);
    assert!(vreg_classes.len() == num_virtual_regs);
    // This is pretty cheap (once per fn) and any failure will be catastrophic since it means we
    // may have forgotten some live range fragments.  Hence `assert!` and not `debug_assert!`.
    for state_elem in &state {
        assert!(state_elem.is_none());
    }

    if log_enabled!(Level::Trace) {
        trace!("");
        let mut n = 0;
        for frag in result_frags.iter() {
            trace!("{:<3?}   {:?}", RangeFragIx::new(n), frag);
            n += 1;
        }

        trace!("");
        for (reg_ix, frag_ixs) in result_map.iter().enumerate() {
            if frag_ixs.len() == 0 {
                continue;
            }
            let reg = reg_ix_to_reg(reg_universe, &vreg_classes, reg_ix as u32);
            trace!(
                "frags for {}   {:?}",
                reg.show_with_rru(reg_universe),
                frag_ixs
            );
        }
    }

    trace!("    get_range_frags: end");
    assert!(result_frags.len() == result_frag_metrics.len());
    (result_map, result_frags, result_frag_metrics, vreg_classes)
}

//=============================================================================
// Auxiliary tasks involved in creating a single VirtualRange from its
// constituent RangeFragIxs:
//
// * The RangeFragIxs we are given here are purely within single blocks.
//   Here, we "compress" them, that is, merge those pairs that flow from one
//   block into the the one that immediately follows it in the instruction
//   stream.  This does not imply anything about control flow; it is purely a
//   scheme for reducing the total number of fragments that need to be dealt
//   with during interference detection (later on).
//
// * Computation of metrics for the VirtualRange.  This is done by examining
//   metrics of the individual fragments, and must be done before they are
//   compressed.

// HELPER FUNCTION
// Does `frag1` describe some range of instructions that is followed
// immediately by `frag2` ?  Note that this assumes (and checks) that there
// are no spill or reload ranges in play at this point; there should not be.
// Note also, this is very conservative: it only merges the case where the two
// ranges are separated by a block boundary.  From measurements, it appears that
// this is the only case where merging is actually a win, though.
fn frags_are_mergeable(
    frag1: &RangeFrag,
    frag1metrics: &RangeFragMetrics,
    frag2: &RangeFrag,
    frag2metrics: &RangeFragMetrics,
) -> bool {
    assert!(frag1.first.pt().is_use_or_def());
    assert!(frag1.last.pt().is_use_or_def());
    assert!(frag2.first.pt().is_use_or_def());
    assert!(frag2.last.pt().is_use_or_def());

    if frag1metrics.bix != frag2metrics.bix
        && frag1.last.iix().plus(1) == frag2.first.iix()
        && frag1.last.pt() == Point::Def
        && frag2.first.pt() == Point::Use
    {
        assert!(
            frag1metrics.kind == RangeFragKind::LiveOut || frag1metrics.kind == RangeFragKind::Thru
        );
        assert!(
            frag2metrics.kind == RangeFragKind::LiveIn || frag2metrics.kind == RangeFragKind::Thru
        );
        return true;
    }

    false
}

// HELPER FUNCTION
// Create a compressed version of the fragments listed in `sorted_frag_ixs`,
// taking the opportunity to dereference them (look them up in `frag_env`) in
// the process.  Assumes that `sorted_frag_ixs` is indeed ordered so that the
// dereferenced frag sequence is in instruction order.
#[inline(never)]
fn deref_and_compress_sorted_range_frag_ixs(
    stats_num_vfrags_uncompressed: &mut usize,
    stats_num_vfrags_compressed: &mut usize,
    sorted_frag_ixs: &SortedRangeFragIxs,
    frag_env: &TypedIxVec<RangeFragIx, RangeFrag>,
    frag_metrics_env: &TypedIxVec<RangeFragIx, RangeFragMetrics>,
) -> SortedRangeFrags {
    let mut res = SortedRangeFrags::empty();

    let num_frags = sorted_frag_ixs.len();
    *stats_num_vfrags_uncompressed += num_frags;

    if num_frags == 1 {
        // Nothing we can do.  Shortcut.
        res.push(frag_env[sorted_frag_ixs[0]].clone());
        *stats_num_vfrags_compressed += 1;
        return res;
    }

    // BEGIN merge this frag sequence as much as possible
    assert!(num_frags > 1);

    let mut s = 0; // start point of current group
    let mut e = 0; // end point of current group
    loop {
        if s >= num_frags {
            break;
        }
        while e + 1 < num_frags
            && frags_are_mergeable(
                &frag_env[sorted_frag_ixs[e]],
                &frag_metrics_env[sorted_frag_ixs[e]],
                &frag_env[sorted_frag_ixs[e + 1]],
                &frag_metrics_env[sorted_frag_ixs[e + 1]],
            )
        {
            e += 1;
        }
        // s to e inclusive is a maximal group
        // emit (s, e)
        if s == e {
            // Can't compress this one
            res.push(frag_env[sorted_frag_ixs[s]].clone());
        } else {
            let compressed_frag = RangeFrag {
                first: frag_env[sorted_frag_ixs[s]].first,
                last: frag_env[sorted_frag_ixs[e]].last,
            };
            res.push(compressed_frag);
        }
        // move on
        s = e + 1;
        e = s;
    }
    // END merge this frag sequence as much as possible

    *stats_num_vfrags_compressed += res.len();
    res
}

// HELPER FUNCTION
// Computes the `size`, `total_cost` and `spill_cost` values for a
// VirtualRange, while being very careful to avoid overflow.
fn calc_virtual_range_metrics(
    sorted_frag_ixs: &SortedRangeFragIxs,
    frag_env: &TypedIxVec<RangeFragIx, RangeFrag>,
    frag_metrics_env: &TypedIxVec<RangeFragIx, RangeFragMetrics>,
    estimated_frequencies: &DepthBasedFrequencies,
) -> (u16, u32, SpillCost) {
    assert!(frag_env.len() == frag_metrics_env.len());

    let mut tot_size: u32 = 0;
    let mut tot_cost: u32 = 0;

    for fix in sorted_frag_ixs.iter() {
        let frag = &frag_env[*fix];
        let frag_metrics = &frag_metrics_env[*fix];

        // Add on the size of this fragment, but make sure we can't
        // overflow a u32 no matter how many fragments there are.
        let mut frag_size: u32 = frag.last.iix().get() - frag.first.iix().get() + 1;
        frag_size = min(frag_size, 0xFFFFu32);
        tot_size += frag_size;
        tot_size = min(tot_size, 0xFFFFu32);

        // Here, tot_size <= 0xFFFF.  frag.count is u16.  estFreq[] is u32.
        // We must be careful not to overflow tot_cost, which is u32.
        let mut new_tot_cost: u64 = frag_metrics.count as u64; // at max 16 bits
        new_tot_cost *= estimated_frequencies.cost(frag_metrics.bix) as u64; // at max 48 bits
        new_tot_cost += tot_cost as u64; // at max 48 bits + epsilon
        new_tot_cost = min(new_tot_cost, 0xFFFF_FFFFu64);

        // Hence this is safe.
        tot_cost = new_tot_cost as u32;
    }

    debug_assert!(tot_size <= 0xFFFF);
    let size = tot_size as u16;
    let total_cost = tot_cost;

    // Divide tot_cost by the total length, so as to increase the apparent
    // spill cost of short LRs.  This is so as to give the advantage to
    // short LRs in competition for registers.  This seems a bit of a hack
    // to me, but hey ..
    debug_assert!(tot_size >= 1);
    let spill_cost = SpillCost::finite(tot_cost as f32 / tot_size as f32);

    (size, total_cost, spill_cost)
}

// MAIN FUNCTION in this section
#[inline(never)]
fn create_and_add_range(
    stats_num_vfrags_uncompressed: &mut usize,
    stats_num_vfrags_compressed: &mut usize,
    result_real: &mut TypedIxVec<RealRangeIx, RealRange>,
    result_virtual: &mut TypedIxVec<VirtualRangeIx, VirtualRange>,
    reg: Reg,
    sorted_frag_ixs: SortedRangeFragIxs,
    frag_env: &TypedIxVec<RangeFragIx, RangeFrag>,
    frag_metrics_env: &TypedIxVec<RangeFragIx, RangeFragMetrics>,
    estimated_frequencies: &DepthBasedFrequencies,
) {
    if reg.is_virtual() {
        // First, compute the VirtualRange metrics.  This has to be done
        // before fragment compression.
        let (size, total_cost, spill_cost) = calc_virtual_range_metrics(
            &sorted_frag_ixs,
            frag_env,
            frag_metrics_env,
            estimated_frequencies,
        );

        // Now it's safe to compress the fragments.
        let sorted_frags = deref_and_compress_sorted_range_frag_ixs(
            stats_num_vfrags_uncompressed,
            stats_num_vfrags_compressed,
            &sorted_frag_ixs,
            frag_env,
            frag_metrics_env,
        );

        result_virtual.push(VirtualRange {
            vreg: reg.to_virtual_reg(),
            rreg: None,
            sorted_frags,
            is_ref: false, // analysis_reftypes.rs may later change this
            size,
            total_cost,
            spill_cost,
        });
    } else {
        result_real.push(RealRange {
            rreg: reg.to_real_reg(),
            sorted_frags: sorted_frag_ixs,
            is_ref: false, // analysis_reftypes.rs may later change this
        });
    }
}

//=============================================================================
// Merging of RangeFrags, producing the final LRs, including metrication and
// compression

// We need this in order to construct a UnionFind<usize>.
impl ToFromU32 for usize {
    // 64 bit
    #[cfg(target_pointer_width = "64")]
    fn to_u32(x: usize) -> u32 {
        if x < 0x1_0000_0000usize {
            x as u32
        } else {
            panic!("impl ToFromU32 for usize: to_u32: out of range")
        }
    }
    #[cfg(target_pointer_width = "64")]
    fn from_u32(x: u32) -> usize {
        x as usize
    }
    // 32 bit
    #[cfg(target_pointer_width = "32")]
    fn to_u32(x: usize) -> u32 {
        x as u32
    }
    #[cfg(target_pointer_width = "32")]
    fn from_u32(x: u32) -> usize {
        x as usize
    }
}

#[inline(never)]
pub(crate) fn merge_range_frags(
    frag_ix_vec_per_reg: &Vec</*rreg index, then vreg index, */ SmallVec<[RangeFragIx; 8]>>,
    frag_env: &TypedIxVec<RangeFragIx, RangeFrag>,
    frag_metrics_env: &TypedIxVec<RangeFragIx, RangeFragMetrics>,
    estimated_frequencies: &DepthBasedFrequencies,
    cfg_info: &CFGInfo,
    reg_universe: &RealRegUniverse,
    vreg_classes: &Vec</*vreg index,*/ RegClass>,
) -> (
    TypedIxVec<RealRangeIx, RealRange>,
    TypedIxVec<VirtualRangeIx, VirtualRange>,
) {
    assert!(frag_env.len() == frag_metrics_env.len());
    let mut stats_num_total_incoming_frags = 0;
    let mut stats_num_total_incoming_regs = 0;
    for all_frag_ixs_for_reg in frag_ix_vec_per_reg {
        stats_num_total_incoming_frags += all_frag_ixs_for_reg.len();
        if all_frag_ixs_for_reg.len() > 0 {
            stats_num_total_incoming_regs += 1;
        }
    }
    trace!("    merge_range_frags: begin");
    trace!("      in: {} in frag_env", frag_env.len());
    trace!(
        "      in: {} regs containing in total {} frags",
        stats_num_total_incoming_regs,
        stats_num_total_incoming_frags
    );

    let mut stats_num_single_grps = 0;
    let mut stats_num_local_frags = 0;

    let mut stats_num_multi_grps_small = 0;
    let mut stats_num_multi_grps_large = 0;
    let mut stats_size_multi_grps_small = 0;
    let mut stats_size_multi_grps_large = 0;

    let mut stats_num_vfrags_uncompressed = 0;
    let mut stats_num_vfrags_compressed = 0;

    let mut result_real = TypedIxVec::<RealRangeIx, RealRange>::new();
    let mut result_virtual = TypedIxVec::<VirtualRangeIx, VirtualRange>::new();

    // BEGIN per_reg_loop
    for (reg_ix, all_frag_ixs_for_reg) in frag_ix_vec_per_reg.iter().enumerate() {
        let n_frags_for_this_reg = all_frag_ixs_for_reg.len();

        // The reg might never have been mentioned at all, especially if it's a real reg.
        if n_frags_for_this_reg == 0 {
            continue;
        }

        let reg_ix = reg_ix as u32;
        let reg = reg_ix_to_reg(reg_universe, vreg_classes, reg_ix);

        // Do some shortcutting.  First off, if there's only one frag for this reg, we can directly
        // give it its own live range, and have done.
        if n_frags_for_this_reg == 1 {
            create_and_add_range(
                &mut stats_num_vfrags_uncompressed,
                &mut stats_num_vfrags_compressed,
                &mut result_real,
                &mut result_virtual,
                reg,
                SortedRangeFragIxs::unit(all_frag_ixs_for_reg[0], frag_env),
                frag_env,
                frag_metrics_env,
                estimated_frequencies,
            );
            stats_num_single_grps += 1;
            continue;
        }

        // BEGIN merge `all_frag_ixs_for_reg` entries as much as possible.
        //
        // but .. if we come across independents (RangeKind::Local), pull them out immediately.

        // Try to avoid heap allocation if at all possible.  Up to 100 entries are very
        // common, so this is sized large to be effective.  Each entry is definitely
        // 16 bytes at most, so this will use 4KB stack at most, which is reasonable.
        let mut triples = SmallVec::<[(RangeFragIx, RangeFragKind, BlockIx); 256]>::new();

        // Create `triples`.  We will use it to guide the merging phase, but it is immutable there.
        for fix in all_frag_ixs_for_reg {
            let frag_metrics = &frag_metrics_env[*fix];

            if frag_metrics.kind == RangeFragKind::Local {
                // This frag is Local (standalone).  Give it its own Range and move on.  This is an
                // optimisation, but it's also necessary: the main fragment-merging logic below
                // relies on the fact that the fragments it is presented with are all either
                // LiveIn, LiveOut or Thru.
                create_and_add_range(
                    &mut stats_num_vfrags_uncompressed,
                    &mut stats_num_vfrags_compressed,
                    &mut result_real,
                    &mut result_virtual,
                    reg,
                    SortedRangeFragIxs::unit(*fix, frag_env),
                    frag_env,
                    frag_metrics_env,
                    estimated_frequencies,
                );
                stats_num_local_frags += 1;
                continue;
            }

            // This frag isn't Local (standalone) so we have to process it the slow way.
            triples.push((*fix, frag_metrics.kind, frag_metrics.bix));
        }

        let triples_len = triples.len();

        // This is the core of the merging algorithm.
        //
        // For each ix@(fix, kind, bix) in `triples` (order unimportant):
        //
        // (1) "Merge with blocks that are live 'downstream' from here":
        //     if fix is live-out or live-through:
        //        for b in succs[bix]
        //           for each ix2@(fix2, kind2, bix2) in `triples`
        //              if bix2 == b && kind2 is live-in or live-through:
        //                  merge(ix, ix2)
        //
        // (2) "Merge with blocks that are live 'upstream' from here":
        //     if fix is live-in or live-through:
        //        for b in preds[bix]
        //           for each ix2@(fix2, kind2, bix2) in `triples`
        //              if bix2 == b && kind2 is live-out or live-through:
        //                  merge(ix, ix2)
        //
        // `triples` remains unchanged.  The equivalence class info is accumulated
        // in `eclasses_uf` instead.  `eclasses_uf` entries are indices into
        // `triples`.
        //
        // Now, you might think it necessary to do both (1) and (2).  But no, they
        // are mutually redundant, since if two blocks are connected by a live
        // flow from one to the other, then they are also connected in the other
        // direction.  Hence checking one of the directions is enough.
        let mut eclasses_uf = UnionFind::<usize>::new(triples_len);

        // We have two schemes for group merging, one of which is N^2 in the
        // length of triples, the other is N-log-N, but with higher constant
        // factors.  Some experimentation with the bz2 test on a Cortex A57 puts
        // the optimal crossover point between 200 and 300; it's not critical.
        // Having this protects us against bad behaviour for huge inputs whilst
        // still being fast for small inputs.
        if triples_len <= 250 {
            // The simple way, which is N^2 in the length of `triples`.
            for (ix, (_fix, kind, bix)) in triples.iter().enumerate() {
                // Deal with liveness flows outbound from `fix`. Meaning, (1) above.
                if *kind == RangeFragKind::LiveOut || *kind == RangeFragKind::Thru {
                    for b in cfg_info.succ_map[*bix].iter() {
                        // Visit all entries in `triples` that are for `b`.
                        for (ix2, (_fix2, kind2, bix2)) in triples.iter().enumerate() {
                            if *bix2 != *b || *kind2 == RangeFragKind::LiveOut {
                                continue;
                            }
                            debug_assert!(
                                *kind2 == RangeFragKind::LiveIn || *kind2 == RangeFragKind::Thru
                            );
                            // Now we know that liveness for this reg "flows" from `triples[ix]` to
                            // `triples[ix2]`.  So those two frags must be part of the same live
                            // range.  Note this.
                            if ix != ix2 {
                                eclasses_uf.union(ix, ix2); // Order of args irrelevant
                            }
                        }
                    }
                }
            } // outermost iteration over `triples`

            stats_num_multi_grps_small += 1;
            stats_size_multi_grps_small += triples_len;
        } else {
            // The more complex way, which is N-log-N in the length of `triples`.  This is the same
            // as the simple way, except that the innermost loop, which is a linear search in
            // `triples` to find entries for some block `b`, is replaced by a binary search.  This
            // means that `triples` first needs to be sorted by block index.
            triples.sort_unstable_by_key(|(_, _, bix)| *bix);

            for (ix, (_fix, kind, bix)) in triples.iter().enumerate() {
                // Deal with liveness flows outbound from `fix`.  Meaning, (1) above.
                if *kind == RangeFragKind::LiveOut || *kind == RangeFragKind::Thru {
                    for b in cfg_info.succ_map[*bix].iter() {
                        // Visit all entries in `triples` that are for `b`.  Binary search
                        // `triples` to find the lowest-indexed entry for `b`.
                        let mut ix_left = 0;
                        let mut ix_right = triples_len;
                        while ix_left < ix_right {
                            let m = (ix_left + ix_right) >> 1;
                            if triples[m].2 < *b {
                                ix_left = m + 1;
                            } else {
                                ix_right = m;
                            }
                        }

                        // It might be that there is no block for `b` in the sequence.  That's
                        // legit; it just means that block `bix` jumps to a successor where the
                        // associated register isn't live-in/thru.  A failure to find `b` can be
                        // indicated one of two ways:
                        //
                        // * ix_left == triples_len
                        // * ix_left < triples_len and b < triples[ix_left].b
                        //
                        // In both cases I *think* the 'loop_over_entries_for_b below will not do
                        // anything.  But this is all a bit hairy, so let's convert the second
                        // variant into the first, so as to make it obvious that the loop won't do
                        // anything.

                        // ix_left now holds the lowest index of any `triples` entry for block `b`.
                        // Assert this.
                        if ix_left < triples_len && *b < triples[ix_left].2 {
                            ix_left = triples_len;
                        }
                        if ix_left < triples_len {
                            assert!(ix_left == 0 || triples[ix_left - 1].2 < *b);
                        }

                        // ix2 plays the same role as in the quadratic version.  ix_left and
                        // ix_right are not used after this point.
                        let mut ix2 = ix_left;
                        loop {
                            let (_fix2, kind2, bix2) = match triples.get(ix2) {
                                None => break,
                                Some(triple) => *triple,
                            };
                            if *b < bix2 {
                                // We've come to the end of the sequence of `b`-blocks.
                                break;
                            }
                            debug_assert!(*b == bix2);
                            if kind2 == RangeFragKind::LiveOut {
                                ix2 += 1;
                                continue;
                            }
                            // Now we know that liveness for this reg "flows" from `triples[ix]` to
                            // `triples[ix2]`.  So those two frags must be part of the same live
                            // range.  Note this.
                            eclasses_uf.union(ix, ix2);
                            ix2 += 1;
                        }

                        if ix2 + 1 < triples_len {
                            debug_assert!(*b < triples[ix2 + 1].2);
                        }
                    }
                }
            }

            stats_num_multi_grps_large += 1;
            stats_size_multi_grps_large += triples_len;
        }

        // Now `eclasses_uf` contains the results of the merging-search.  Visit each of its
        // equivalence classes in turn, and convert each into a virtual or real live range as
        // appropriate.
        let eclasses = eclasses_uf.get_equiv_classes();
        for leader_triple_ix in eclasses.equiv_class_leaders_iter() {
            // `leader_triple_ix` is an eclass leader.  Enumerate the whole eclass.
            let mut frag_ixs = SmallVec::<[RangeFragIx; 4]>::new();
            for triple_ix in eclasses.equiv_class_elems_iter(leader_triple_ix) {
                frag_ixs.push(triples[triple_ix].0 /*first field is frag ix*/);
            }
            let sorted_frags = SortedRangeFragIxs::new(frag_ixs, &frag_env);
            create_and_add_range(
                &mut stats_num_vfrags_uncompressed,
                &mut stats_num_vfrags_compressed,
                &mut result_real,
                &mut result_virtual,
                reg,
                sorted_frags,
                frag_env,
                frag_metrics_env,
                estimated_frequencies,
            );
        }
        // END merge `all_frag_ixs_for_reg` entries as much as possible
    } // END per reg loop

    trace!("      in: {} single groups", stats_num_single_grps);
    trace!(
        "      in: {} local frags in multi groups",
        stats_num_local_frags
    );
    trace!(
        "      in: {} small multi groups, {} small multi group total size",
        stats_num_multi_grps_small,
        stats_size_multi_grps_small
    );
    trace!(
        "      in: {} large multi groups, {} large multi group total size",
        stats_num_multi_grps_large,
        stats_size_multi_grps_large
    );
    trace!(
        "      out: {} VLRs, {} RLRs",
        result_virtual.len(),
        result_real.len()
    );
    trace!(
        "      compress vfrags: in {}, out {}",
        stats_num_vfrags_uncompressed,
        stats_num_vfrags_compressed
    );
    trace!("    merge_range_frags: end");

    (result_real, result_virtual)
}

//=============================================================================
// Auxiliary activities that mostly fall under the category "dataflow analysis", but are not
// part of the main dataflow analysis pipeline.

/// Dataflow and liveness together create vectors of VirtualRanges and RealRanges.  These define
/// (amongst other things) mappings from VirtualRanges to VirtualRegs and from RealRanges to
/// RealRegs.  However, we often need the inverse mappings: from VirtualRegs to (sets of
/// VirtualRanges) and from RealRegs to (sets of) RealRanges.  This function computes those inverse
/// mappings.  They are used by BT's coalescing analysis, and for the dataflow analysis that
/// supports reftype handling.
#[inline(never)]
pub(crate) fn compute_reg_to_ranges_maps<F: Function>(
    func: &F,
    univ: &RealRegUniverse,
    rlr_env: &TypedIxVec<RealRangeIx, RealRange>,
    vlr_env: &TypedIxVec<VirtualRangeIx, VirtualRange>,
) -> RegToRangesMaps {
    // Arbitrary, but chosen after quite some profiling, so as to minimise both instruction count
    // and number of `malloc` calls.  Don't mess with this without first collecting comprehensive
    // measurements.  Note that if you set this above 255, the type of `r/vreg_approx_frag_counts`
    // below will need to change accordingly.
    const MANY_FRAGS_THRESH: u8 = 200;

    // Adds `to_add` to `*counter`, taking care not to overflow it in the process.
    let add_u8_usize_saturate_to_u8 = |counter: &mut u8, to_add: usize| {
        *counter = counter.saturating_add(usize::min(to_add, 0xff) as u8);
    };

    // We have in hand the virtual live ranges.  Each of these carries its associated vreg.  So in
    // effect we have a VLR -> VReg mapping.  We now invert that, so as to generate a mapping from
    // VRegs to their containing VLRs.
    //
    // Note that multiple VLRs may map to the same VReg.  So the inverse mapping will actually be
    // from VRegs to a set of VLRs.  In most cases, we expect the virtual-registerised-code given
    // to this allocator to be derived from SSA, in which case each VReg will have only one VLR.
    // So in this case, the cost of first creating the mapping, and then looking up all the VRegs
    // in moves in it, will have cost linear in the size of the input function.
    //
    // NB re the SmallVec.  That has set semantics (no dups).

    let num_vregs = func.get_num_vregs();
    let num_rregs = univ.allocable;

    let mut vreg_approx_frag_counts = vec![0u8; num_vregs];
    let mut vreg_to_vlrs_map = vec![SmallVec::<[VirtualRangeIx; 3]>::new(); num_vregs];
    for (vlr, vlr_ix) in vlr_env.iter().zip(0..) {
        // Now we know that there's a VLR `vlr` that is for VReg `vreg`.  Update the inverse
        // mapping accordingly.  We know we are stepping sequentially through the VLR (index)
        // space, so we'll never see the same VLRIx twice.  Hence there's no need to check for dups
        // when adding a VLR index to an existing binding for a VReg.
        //
        // If this array-indexing fails, it means the client's `.get_num_vregs()` function claims
        // there are fewer virtual regs than we actually observe in the code it gave us.  So it's a
        // bug in the client.
        let vreg_index = vlr.vreg.get_index();
        vreg_to_vlrs_map[vreg_index].push(VirtualRangeIx::new(vlr_ix));

        let vlr_num_frags = vlr.sorted_frags.len();
        add_u8_usize_saturate_to_u8(&mut vreg_approx_frag_counts[vreg_index], vlr_num_frags);
    }

    // Same for the real live ranges.
    let mut rreg_approx_frag_counts = vec![0u8; num_rregs];
    let mut rreg_to_rlrs_map = vec![SmallVec::<[RealRangeIx; 6]>::new(); num_rregs];
    for (rlr, rlr_ix) in rlr_env.iter().zip(0..) {
        // If this array-indexing fails, it means something has gone wrong with sanitisation of
        // real registers -- that should ensure that we never see a real register with an index
        // greater than `univ.allocable`.  So it's a bug in the allocator's analysis phases.
        let rreg_index = rlr.rreg.get_index();
        rreg_to_rlrs_map[rreg_index].push(RealRangeIx::new(rlr_ix));

        let rlr_num_frags = rlr.sorted_frags.len();
        add_u8_usize_saturate_to_u8(&mut rreg_approx_frag_counts[rreg_index], rlr_num_frags);
    }

    // Create sets indicating which regs have "many" live ranges.  Hopefully very few. Since the
    // `push`ed-in values are supplied by the `zip(0..)` iterator, they are guaranteed
    // duplicate-free, as required by the definition of `RegToRangesMaps`.
    let mut vregs_with_many_frags = Vec::<u32 /*VirtualReg index*/>::with_capacity(16);
    for (count, vreg_ix) in vreg_approx_frag_counts.iter().zip(0..) {
        if *count >= MANY_FRAGS_THRESH {
            vregs_with_many_frags.push(vreg_ix);
        }
    }

    let mut rregs_with_many_frags = Vec::<u32 /*RealReg index*/>::with_capacity(64);
    for (count, rreg_ix) in rreg_approx_frag_counts.iter().zip(0..) {
        if *count >= MANY_FRAGS_THRESH {
            rregs_with_many_frags.push(rreg_ix);
        }
    }

    RegToRangesMaps {
        rreg_to_rlrs_map,
        vreg_to_vlrs_map,
        rregs_with_many_frags,
        vregs_with_many_frags,
        many_frags_thresh: MANY_FRAGS_THRESH as usize,
    }
}

/// Collect info about registers that are connected by moves.
#[inline(never)]
pub(crate) fn collect_move_info<F: Function>(
    func: &F,
    reg_vecs_and_bounds: &RegVecsAndBounds,
    estimated_frequency: &DepthBasedFrequencies,
) -> MoveInfo {
    let mut moves = Vec::new();
    for b in func.blocks() {
        let block_estimated_frequency = estimated_frequency.cost(b);
        for inst_ix in func.block_insns(b) {
            if let Some((dst, src)) = func.is_move(func.get_insn(inst_ix)) {
                let iix_bounds = &reg_vecs_and_bounds.bounds[inst_ix];
                // It might seem strange to assert that `defs_len` and/or `uses_len` is <= 1 rather
                // than == 1.  The reason is that either or even both registers might be ones which
                // are not available to the allocator.  Hence they will have been removed by the
                // sanitisation machinery before we get to this point.  If either is missing, we
                // unfortunately can't coalesce the move away, and just have to live with it.
                //
                // If any of the following five assertions fail, the client's `is_move` is probably
                // lying to us.
                assert!(iix_bounds.uses_len <= 1);
                assert!(iix_bounds.defs_len <= 1);
                assert!(iix_bounds.mods_len == 0);
                if iix_bounds.uses_len == 1 && iix_bounds.defs_len == 1 {
                    let reg_vecs = &reg_vecs_and_bounds.vecs;
                    assert!(reg_vecs.uses[iix_bounds.uses_start as usize] == src);
                    assert!(reg_vecs.defs[iix_bounds.defs_start as usize] == dst.to_reg());
                    moves.push(MoveInfoElem {
                        dst: dst.to_reg(),
                        src,
                        iix: inst_ix,
                        est_freq: block_estimated_frequency,
                    });
                }
            }
        }
    }
    MoveInfo::new(moves)
}
