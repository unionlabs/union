//! Analysis in support of copy coalescing for the backtracking allocator.
//!
//! This detects and collects information about all copy coalescing
//! opportunities in the incoming function.  It does not use that information
//! at all -- that is for the main allocation loop and the spill slot allocator
//! to do.
//!
//! Coalescing analysis creates 4 pieces of information:
//!
//! * a map from `VirtualRangeIx` to a set of `Hint`s (see below) which state a
//!   preference for which register that range would prefer to be allocated to.
//!
//! * equivalence class groupings for the virtual ranges.  Two virtual ranges
//!   will be assigned the same equivalence class if there is a move instruction
//!   that transfers a value from one range to the other.  The equivalence
//!   classes created are the transitive closure of this pairwise relation.
//!
//! * a simple mapping from instruction index to bool, indicating those
//! instructions that are moves between virtual registers, and that have been
//! used to construct the equivalence classes above.
//!
//! * a mapping from virtual registers to virtual ranges.  This is really
//!   produced as a side-effect of computing the above three elements, but is
//!   useful in its own right and so is also returned.

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use log::{log_enabled, trace, Level};
use smallvec::{smallvec, SmallVec};

use crate::data_structures::{
    InstIx, InstPoint, Map, MoveInfo, MoveInfoElem, RangeFrag, RangeFragIx, RealRange, RealRangeIx,
    RealReg, RealRegUniverse, RegToRangesMaps, SpillCost, TypedIxVec, VirtualRange, VirtualRangeIx,
    VirtualReg,
};
use crate::union_find::{ToFromU32, UnionFind, UnionFindEquivClasses};
use crate::Function;

//=============================================================================
//
// *** Some important comments about the interaction between this coalescing
// *** analysis, the main allocation loop and the spill slot allocator.
//
// The main allocation loop tries to assign the same register to all the
// VirtualRanges in an equivalence class.  Similarly, the spill slot allocator
// tries to allocate the same spill slot to all the VirtualRanges in an
// equivalence class.  In most cases they are successful, and so the moves
// between those VirtualRanges will later disappear.  However, the complete
// story is not quite so simple.
//
// It is only safe to assign the VirtualRanges in the same equivalence class
// to a single register or spill slot if those VirtualRanges are
// non-overlapping.  That is, if their overall collection of RangeFrags is
// disjoint.  If two such VirtualRanges overlapped, then they could be
// carrying different values, and so they would need separate registers or
// spill slots.
//
// Most of the time, these equivalence classes are indeed internally
// non-overlapping.  But that's just luck -- that's how the input VCode mostly
// is.  The coalescing analysis *doesn't* properly check for overlaps within an
// equivalence class, so it can be the case that the members of an equivalence
// class overlap.  The users of that information -- the main allocation loop
// and the spill slot allocator -- currently check for, and handle, such
// situations.  So the generated allocation is correct.
//
// It does, however, cause imprecision and unnecessary spilling, and, in the
// main allocation loop, slightly increased evictions.
//
// The "proper" fix for all this would be to fix the coalescing analysis so as
// only to build non-internally-overlapping VirtualRange equivalence classes.
// However, that sounds expensive.  Instead there is a half-hearted effort
// made to avoid creating equivalence classes whose elements (VirtualRanges)
// overlap.  This is done by doing an overlap check on two VirtualRanges
// connected by a move, and not merging their equivalence classes if they
// overlap.  That helps, but it doesn't completely avoid the problem because
// there might be overlaps between other members (VirtualRanges) of the
// about-to-be-merged equivalence classes.

//=============================================================================
// Coalescing analysis: Hints
//
// A coalescing hint for a virtual live range.  The u32 is an arbitrary
// "weight" value which indicates a relative strength-of-preference for the
// hint.  It exists because a VLR can have arbitrarily many copy
// instructions at its "boundary", and hence arbitrarily many hints.  Of
// course the allocator core can honour at most one of them, so it needs a
// way to choose between them.  In this implementation, the u32s are simply
// the estimated execution count of the associated copy instruction.
#[derive(Clone)]
pub enum Hint {
    // I would like to have the same real register as some other virtual range.
    SameAs(VirtualRangeIx, u32),
    // I would like to have exactly this real register.
    Exactly(RealReg, u32),
}
fn show_hint(h: &Hint, univ: &RealRegUniverse) -> String {
    match h {
        Hint::SameAs(vlrix, weight) => format!("(SameAs {:?}, weight={})", vlrix, weight),
        Hint::Exactly(rreg, weight) => format!(
            "(Exactly {}, weight={})",
            rreg.to_reg().show_with_rru(&univ),
            weight
        ),
    }
}
impl Hint {
    #[inline(always)]
    fn get_weight(&self) -> u32 {
        match self {
            Hint::SameAs(_vlrix, weight) => *weight,
            Hint::Exactly(_rreg, weight) => *weight,
        }
    }
}

// We need this in order to construct a UnionFind<VirtualRangeIx>.
impl ToFromU32 for VirtualRangeIx {
    fn to_u32(x: VirtualRangeIx) -> u32 {
        x.get()
    }
    fn from_u32(x: u32) -> VirtualRangeIx {
        VirtualRangeIx::new(x)
    }
}

//=============================================================================
// Coalescing analysis: top level function

// This performs coalescing analysis and returns info as a 3-tuple.  Note that
// it also may change the spill costs for some of the VLRs in `vlr_env` to
// better reflect the spill cost situation in the presence of coalescing.
#[inline(never)]
pub(crate) fn do_coalescing_analysis<F: Function>(
    func: &F,
    univ: &RealRegUniverse,
    rlr_env: &TypedIxVec<RealRangeIx, RealRange>,
    vlr_env: &mut TypedIxVec<VirtualRangeIx, VirtualRange>,
    frag_env: &TypedIxVec<RangeFragIx, RangeFrag>,
    reg_to_ranges_maps: &RegToRangesMaps,
    move_info: &MoveInfo,
) -> (
    TypedIxVec<VirtualRangeIx, SmallVec<[Hint; 8]>>,
    UnionFindEquivClasses<VirtualRangeIx>,
    TypedIxVec<InstIx, bool>,
) {
    trace!("");
    trace!("do_coalescing_analysis: begin");

    // This function contains significant additional complexity due to the requirement to handle
    // pathological cases in reasonable time without unduly burdening the common cases.
    //
    // ========================================================================================
    //
    // The core questions that the coalescing analysis asks is:
    //
    //    For an instruction I and a reg V:
    //
    //    * does I start a live range fragment for V?  In other words, is it a "first def of V" ?
    //
    //    * and dually: does I end a live range fragment for V?  IOW, is it a "last use of V" ?
    //
    // V may be a real or virtual register -- we must handle both.  I is invariably a move
    // instruction.  We could ask such questions about other kinds of insns, but we don't care
    // about those.
    //
    // The reason we care about this is as follows.  If we can find some move insn I, which is
    // the last use of some reg V1 and the first def of some other reg V2, then V1 and V2 can at
    // least in principle be allocated to the same real register.
    //
    // Note that the "last" and "first" aspect is critical for correctness.  Consider this:
    //
    //       V1 = ...
    //   I   V2 = V1
    //   *   V2 = V2 - 99
    //   #   V3 = V1 + 47
    //
    // Here, I might be a first def of V2, but it's certainly not the last use of V1, and so if
    // we allocate V1 and V2 to the same real register, the insn marked * will trash the value
    // of V1 while it's still needed at #, and we'll create wrong code.  For the same reason, we
    // can only coalesce out a move if the destination is a first def.
    //
    // The use of names V* in the above example is slightly misleading.  As mentioned, the above
    // criteria apply to both real and virtual registers.  The only limitation is that,
    // obviously, we can't coalesce out a move if both registers involved are real.  But if only
    // one is real then we have at least the possibility to do that.
    //
    // Now to the question of compiler performance.  The simplest way to establish whether (for
    // example) I is a first def of V is to visit all of V's `RangeFrag`s, to see if any of them
    // start at `I.d`.  That can be done by iterating over all of the live ranges that belong to
    // V, and through each `RangeFrag` in each live range.  Hence it's a linear search through
    // V's `RangeFrag`s.
    //
    // For the vast majority of cases, this works well because most regs -- and especially, most
    // virtual regs, in code derived from an SSA precursor -- have short live ranges, and
    // usually only one, and so there are very few `RangeFrag`s to visit.  However, there are
    // cases where a register has many `RangeFrag`s -- we've seen inputs where that number
    // exceeds 100,000 -- in which case a linear search is disastrously slow.
    //
    // To fix this, there is a Plan B scheme for establishing the same facts.  It relies on the
    // observation that the `RangeFrag`s for each register are mutually non-overlapping.  Hence
    // their start points are all unique, so we can park them all in a vector, sort it, and
    // binary search it.  And the same for the end points.  This is the purpose of structs
    // `ManyFragsInfoR` and `ManyFragsInfoV` below.
    //
    // Although this plan keeps us out of performance black holes in pathological cases, it is
    // expensive in a constant-factors sense: it requires dynamic memory allocation for these
    // vectors, and it requires sorting them.  Hence we try to avoid it as much as possible, and
    // route almost all work via the simple linear-search scheme.
    //
    // The linear-vs-binary-search choice is made for each register individually.  Incoming
    // parameter `reg_to_ranges_maps` contains fields `r/vregs_with_many_frags`, and it is only
    // for those that sorted vectors are prepared.  Those vectors are tracked by the maps
    // `r/v_many_map` below.  `reg_to_ranges_maps` also contains field `many_frags_thresh` which
    // tells us what the size threshold actually was, and this is used to opportunistically
    // pre-size the vectors.  It's not required for correctness.
    //
    // All this complexity is bought together in the four closures `doesVRegHaveLastUseAt`,
    // `doesVRegHaveFirstDefAt`, `doesRRegHaveLastUseAt` and `doesRRegHaveFirstDefAt`.  In each
    // case, they first try to resolve the query by binary search, which usually fails, in which
    // case they fall back to a linear search, which will always give a correct result.  In
    // debug builds, if the binary search does produce an answer, it is crosschecked against the
    // linear search result.
    //
    // The duplication in the four closures is undesirable but hard to avoid.  The real- and
    // virtual-reg cases have different types.  Similarly, the first/last cases are slightly
    // different.  If there were a way to guarantee that rustc would inline closures, then it
    // might be worth trying to common them up, on the basis that rustc can inline and
    // specialise, leading back to what we currently have here.  However, in the absence of such
    // a facility, I didn't want to risk it, given that these closures are performance-critical.
    //
    // Finally, note that the coalescing analysis proper involves more than just the above
    // described searches, and one sees the code for the rest of it following the search
    // closures below.  However, the rest of it isn't performance critical, and is not described
    // in this comment.
    //
    // ========================================================================================

    // So, first: for the registers which `reg_to_ranges_maps` tells us have "many" fragments,
    // prepare the binary-search vectors.  This is done first for the real regs and then below
    // for virtual regs.

    struct ManyFragsInfoR {
        sorted_firsts: Vec<(InstPoint, RealRangeIx)>,
        sorted_lasts: Vec<(InstPoint, RealRangeIx)>,
    }
    let r_many_card = reg_to_ranges_maps.rregs_with_many_frags.len();
    let mut r_many_map = Map::<u32 /*RealReg index*/, ManyFragsInfoR>::default();
    r_many_map.reserve(r_many_card);

    for rreg_no in &reg_to_ranges_maps.rregs_with_many_frags {
        // `2 * reg_to_ranges_maps.many_frags_thresh` is clearly a heuristic hack, but we do
        // know for sure that each vector will contain at least
        // `reg_to_ranges_maps.many_frags_thresh` and very likely more.  And that threshold is
        // already quite high, so pre-sizing the vectors at this point avoids quite a number of
        // resize-reallocations later.
        let mut many_frags_info = ManyFragsInfoR {
            sorted_firsts: Vec::with_capacity(2 * reg_to_ranges_maps.many_frags_thresh),
            sorted_lasts: Vec::with_capacity(2 * reg_to_ranges_maps.many_frags_thresh),
        };
        let rlrixs = &reg_to_ranges_maps.rreg_to_rlrs_map[*rreg_no as usize];
        for rlrix in rlrixs {
            for fix in rlr_env[*rlrix].sorted_frags.iter() {
                let frag = &frag_env[*fix];
                many_frags_info.sorted_firsts.push((frag.first, *rlrix));
                many_frags_info.sorted_lasts.push((frag.last, *rlrix));
            }
        }
        many_frags_info
            .sorted_firsts
            .sort_unstable_by_key(|&(point, _)| point);
        many_frags_info
            .sorted_lasts
            .sort_unstable_by_key(|&(point, _)| point);
        debug_assert!(many_frags_info.sorted_firsts.len() == many_frags_info.sorted_lasts.len());
        // Because the RangeFrags for any reg (virtual or real) are non-overlapping, it follows
        // that both the sorted first points and sorted last points contain no duplicates.  (In
        // fact the implied condition (no duplicates) is weaker than the premise
        // (non-overlapping), but this is nevertheless correct.)
        for i in 1..(many_frags_info.sorted_firsts.len()) {
            debug_assert!(
                many_frags_info.sorted_firsts[i - 1].0 < many_frags_info.sorted_firsts[i].0
            );
        }
        for i in 1..(many_frags_info.sorted_lasts.len()) {
            debug_assert!(
                many_frags_info.sorted_lasts[i - 1].0 < many_frags_info.sorted_lasts[i].0
            );
        }
        r_many_map.insert(*rreg_no, many_frags_info);
    }

    // And the same for virtual regs.
    struct ManyFragsInfoV {
        sorted_firsts: Vec<(InstPoint, VirtualRangeIx)>,
        sorted_lasts: Vec<(InstPoint, VirtualRangeIx)>,
    }
    let v_many_card = reg_to_ranges_maps.vregs_with_many_frags.len();
    let mut v_many_map = Map::<u32 /*VirtualReg index*/, ManyFragsInfoV>::default();
    v_many_map.reserve(v_many_card);

    for vreg_no in &reg_to_ranges_maps.vregs_with_many_frags {
        let mut many_frags_info = ManyFragsInfoV {
            sorted_firsts: Vec::with_capacity(2 * reg_to_ranges_maps.many_frags_thresh),
            sorted_lasts: Vec::with_capacity(2 * reg_to_ranges_maps.many_frags_thresh),
        };
        let vlrixs = &reg_to_ranges_maps.vreg_to_vlrs_map[*vreg_no as usize];
        for vlrix in vlrixs {
            for frag in vlr_env[*vlrix].sorted_frags.iter() {
                many_frags_info.sorted_firsts.push((frag.first, *vlrix));
                many_frags_info.sorted_lasts.push((frag.last, *vlrix));
            }
        }
        many_frags_info
            .sorted_firsts
            .sort_unstable_by_key(|&(point, _)| point);
        many_frags_info
            .sorted_lasts
            .sort_unstable_by_key(|&(point, _)| point);
        debug_assert!(many_frags_info.sorted_firsts.len() == many_frags_info.sorted_lasts.len());
        for i in 1..(many_frags_info.sorted_firsts.len()) {
            debug_assert!(
                many_frags_info.sorted_firsts[i - 1].0 < many_frags_info.sorted_firsts[i].0
            );
        }
        for i in 1..(many_frags_info.sorted_lasts.len()) {
            debug_assert!(
                many_frags_info.sorted_lasts[i - 1].0 < many_frags_info.sorted_lasts[i].0
            );
        }
        v_many_map.insert(*vreg_no, many_frags_info);
    }

    // There now follows the abovementioned four (well, actually, eight) closures, which are
    // used to find out whether a real or virtual reg has a last use or first def at some
    // instruction.  This is the central activity of the coalescing analysis -- finding move
    // instructions that are the last def for the src reg and the first def for the dst reg.

    // ---------------- Range checks for VirtualRegs: last use ----------------
    // Checks whether `vreg` has a last use at `iix`.u.

    let doesVRegHaveLastUseAt_LINEAR = |vreg: VirtualReg, iix: InstIx| -> Option<VirtualRangeIx> {
        let point_to_find = InstPoint::new_use(iix);
        let vreg_no = vreg.get_index();
        let vlrixs = &reg_to_ranges_maps.vreg_to_vlrs_map[vreg_no];
        for vlrix in vlrixs {
            for frag in vlr_env[*vlrix].sorted_frags.iter() {
                if frag.last == point_to_find {
                    return Some(*vlrix);
                }
            }
        }
        None
    };

    let doesVRegHaveLastUseAt = |vreg: VirtualReg, iix: InstIx| -> Option<VirtualRangeIx> {
        let point_to_find = InstPoint::new_use(iix);
        let vreg_no = vreg.get_index();
        let mut binary_search_result = None;
        if let Some(ref mfi) = v_many_map.get(&(vreg_no as u32)) {
            match mfi
                .sorted_lasts
                .binary_search_by_key(&point_to_find, |(point, _)| *point)
            {
                Ok(found_at_ix) => binary_search_result = Some(mfi.sorted_lasts[found_at_ix].1),
                Err(_) => {}
            }
        }
        match binary_search_result {
            None => doesVRegHaveLastUseAt_LINEAR(vreg, iix),
            Some(_) => {
                debug_assert!(binary_search_result == doesVRegHaveLastUseAt_LINEAR(vreg, iix));
                binary_search_result
            }
        }
    };

    // ---------------- Range checks for VirtualRegs: first def ----------------
    // Checks whether `vreg` has a first def at `iix`.d.

    let doesVRegHaveFirstDefAt_LINEAR = |vreg: VirtualReg, iix: InstIx| -> Option<VirtualRangeIx> {
        let point_to_find = InstPoint::new_def(iix);
        let vreg_no = vreg.get_index();
        let vlrixs = &reg_to_ranges_maps.vreg_to_vlrs_map[vreg_no];
        for vlrix in vlrixs {
            for frag in vlr_env[*vlrix].sorted_frags.iter() {
                if frag.first == point_to_find {
                    return Some(*vlrix);
                }
            }
        }
        None
    };

    let doesVRegHaveFirstDefAt = |vreg: VirtualReg, iix: InstIx| -> Option<VirtualRangeIx> {
        let point_to_find = InstPoint::new_def(iix);
        let vreg_no = vreg.get_index();
        let mut binary_search_result = None;
        if let Some(ref mfi) = v_many_map.get(&(vreg_no as u32)) {
            match mfi
                .sorted_firsts
                .binary_search_by_key(&point_to_find, |(point, _)| *point)
            {
                Ok(found_at_ix) => binary_search_result = Some(mfi.sorted_firsts[found_at_ix].1),
                Err(_) => {}
            }
        }
        match binary_search_result {
            None => doesVRegHaveFirstDefAt_LINEAR(vreg, iix),
            Some(_) => {
                debug_assert!(binary_search_result == doesVRegHaveFirstDefAt_LINEAR(vreg, iix));
                binary_search_result
            }
        }
    };

    // ---------------- Range checks for RealRegs: last use ----------------
    // Checks whether `rreg` has a last use at `iix`.u.

    let doesRRegHaveLastUseAt_LINEAR = |rreg: RealReg, iix: InstIx| -> Option<RealRangeIx> {
        let point_to_find = InstPoint::new_use(iix);
        let rreg_no = rreg.get_index();
        let rlrixs = &reg_to_ranges_maps.rreg_to_rlrs_map[rreg_no];
        for rlrix in rlrixs {
            let frags = &rlr_env[*rlrix].sorted_frags;
            for fix in frags.iter() {
                let frag = &frag_env[*fix];
                if frag.last == point_to_find {
                    return Some(*rlrix);
                }
            }
        }
        None
    };

    let doesRRegHaveLastUseAt = |rreg: RealReg, iix: InstIx| -> Option<RealRangeIx> {
        let point_to_find = InstPoint::new_use(iix);
        let rreg_no = rreg.get_index();
        let mut binary_search_result = None;
        if let Some(ref mfi) = r_many_map.get(&(rreg_no as u32)) {
            match mfi
                .sorted_lasts
                .binary_search_by_key(&point_to_find, |(point, _)| *point)
            {
                Ok(found_at_ix) => binary_search_result = Some(mfi.sorted_lasts[found_at_ix].1),
                Err(_) => {}
            }
        }
        match binary_search_result {
            None => doesRRegHaveLastUseAt_LINEAR(rreg, iix),
            Some(_) => {
                debug_assert!(binary_search_result == doesRRegHaveLastUseAt_LINEAR(rreg, iix));
                binary_search_result
            }
        }
    };

    // ---------------- Range checks for RealRegs: first def ----------------
    // Checks whether `rreg` has a first def at `iix`.d.

    let doesRRegHaveFirstDefAt_LINEAR = |rreg: RealReg, iix: InstIx| -> Option<RealRangeIx> {
        let point_to_find = InstPoint::new_def(iix);
        let rreg_no = rreg.get_index();
        let rlrixs = &reg_to_ranges_maps.rreg_to_rlrs_map[rreg_no];
        for rlrix in rlrixs {
            let frags = &rlr_env[*rlrix].sorted_frags;
            for fix in frags.iter() {
                let frag = &frag_env[*fix];
                if frag.first == point_to_find {
                    return Some(*rlrix);
                }
            }
        }
        None
    };

    let doesRRegHaveFirstDefAt = |rreg: RealReg, iix: InstIx| -> Option<RealRangeIx> {
        let point_to_find = InstPoint::new_def(iix);
        let rreg_no = rreg.get_index();
        let mut binary_search_result = None;
        if let Some(ref mfi) = r_many_map.get(&(rreg_no as u32)) {
            match mfi
                .sorted_firsts
                .binary_search_by_key(&point_to_find, |(point, _)| *point)
            {
                Ok(found_at_ix) => binary_search_result = Some(mfi.sorted_firsts[found_at_ix].1),
                Err(_) => {}
            }
        }
        match binary_search_result {
            None => doesRRegHaveFirstDefAt_LINEAR(rreg, iix),
            Some(_) => {
                debug_assert!(binary_search_result == doesRRegHaveFirstDefAt_LINEAR(rreg, iix));
                binary_search_result
            }
        }
    };

    // Finally we come to the core logic of the coalescing analysis.  It uses the complex
    // hybrid-search mechanism described extensively above.  The comments above however don't
    // describe any of the logic after this point.

    // RETURNED TO CALLER
    // Hints for each VirtualRange.  Note that the SmallVecs could contain duplicates, I
    // suppose, for example if there are two identical copy insns at different points on the
    // "boundary" for some VLR.  I don't think it matters though since we're going to rank the
    // hints by strength and then choose at most one.
    let mut hints = TypedIxVec::<VirtualRangeIx, SmallVec<[Hint; 8]>>::new();
    hints.resize(vlr_env.len(), smallvec![]);

    // RETURNED TO CALLER
    // A vector that simply records which insns are v-to-v boundary moves, as established by the
    // analysis below.  This info is collected here because (1) the caller (BT) needs to have it
    // and (2) this is the first point at which we can efficiently compute it.
    let mut is_vv_boundary_move = TypedIxVec::<InstIx, bool>::new();
    is_vv_boundary_move.resize(func.insns().len() as u32, false);

    // RETURNED TO CALLER (after finalisation)
    // The virtual-to-virtual equivalence classes we're collecting.
    let mut vlrEquivClassesUF = UnionFind::<VirtualRangeIx>::new(vlr_env.len() as usize);

    // Not returned to caller; for use only in this function.
    // A list of `VirtualRange`s for which the `total_cost` (hence also their
    // `spill_cost`) should be adjusted downwards by the supplied `u32`.  We
    // can't do this directly in the loop below due to borrowing constraints,
    // hence we collect the required info in this vector and do it in a second
    // loop.
    let mut decVLRcosts = Vec::<(VirtualRangeIx, VirtualRangeIx, u32)>::new();

    for MoveInfoElem {
        dst,
        src,
        iix,
        est_freq,
        ..
    } in move_info.iter()
    {
        trace!(
            "connected by moves: {:?} {:?} <- {:?} (est_freq {})",
            iix,
            dst,
            src,
            est_freq
        );
        match (dst.is_virtual(), src.is_virtual()) {
            (true, true) => {
                // Check for a V <- V hint.
                let srcV = src.to_virtual_reg();
                let dstV = dst.to_virtual_reg();
                let mb_vlrixSrc = doesVRegHaveLastUseAt(srcV, *iix);
                let mb_vlrixDst = doesVRegHaveFirstDefAt(dstV, *iix);
                if mb_vlrixSrc.is_some() && mb_vlrixDst.is_some() {
                    let vlrixSrc = mb_vlrixSrc.unwrap();
                    let vlrixDst = mb_vlrixDst.unwrap();
                    // Per block comment at top of file, make a half-hearted
                    // attempt to avoid creating equivalence classes with
                    // internal overlaps.  Note this can't be completely
                    // effective as presently implemented.
                    if !vlr_env[vlrixSrc].overlaps(&vlr_env[vlrixDst]) {
                        // Add hints for both VLRs, since we don't know which one will
                        // assign first.  Indeed, a VLR may be assigned and un-assigned
                        // arbitrarily many times.
                        hints[vlrixSrc].push(Hint::SameAs(vlrixDst, *est_freq));
                        hints[vlrixDst].push(Hint::SameAs(vlrixSrc, *est_freq));
                        vlrEquivClassesUF.union(vlrixDst, vlrixSrc);
                        is_vv_boundary_move[*iix] = true;
                        // Reduce the total cost, and hence the spill cost, of
                        // both `vlrixSrc` and `vlrixDst`.  This is so as to reduce to
                        // zero, the cost of a VLR whose only instructions are its
                        // v-v boundary copies.
                        trace!("reduce cost of {:?} and {:?}", vlrixSrc, vlrixDst);
                        decVLRcosts.push((vlrixSrc, vlrixDst, 1 * est_freq));
                    }
                }
            }
            (true, false) => {
                // Check for a V <- R hint.
                let srcR = src.to_real_reg();
                let dstV = dst.to_virtual_reg();
                let mb_rlrSrc = doesRRegHaveLastUseAt(srcR, *iix);
                let mb_vlrDst = doesVRegHaveFirstDefAt(dstV, *iix);
                if mb_rlrSrc.is_some() && mb_vlrDst.is_some() {
                    let vlrDst = mb_vlrDst.unwrap();
                    hints[vlrDst].push(Hint::Exactly(srcR, *est_freq));
                }
            }
            (false, true) => {
                // Check for a R <- V hint.
                let srcV = src.to_virtual_reg();
                let dstR = dst.to_real_reg();
                let mb_vlrSrc = doesVRegHaveLastUseAt(srcV, *iix);
                let mb_rlrDst = doesRRegHaveFirstDefAt(dstR, *iix);
                if mb_vlrSrc.is_some() && mb_rlrDst.is_some() {
                    let vlrSrc = mb_vlrSrc.unwrap();
                    hints[vlrSrc].push(Hint::Exactly(dstR, *est_freq));
                }
            }
            (false, false) => {
                // This is a real-to-real move.  There's nothing we can do.  Ignore it.
            }
        }
    }

    // Now decrease the `total_cost` and `spill_cost` fields of selected
    // `VirtualRange`s, as detected by the previous loop.  Don't decrease the
    // `spill_cost` literally to zero; doing that causes various assertion
    // failures and boundary problems later on, in the `CommitmentMap`s.  In
    // such a case, make the `spill_cost` be tiny but nonzero.
    fn decrease_vlr_total_cost_by(vlr: &mut VirtualRange, decrease_total_cost_by: u32) {
        // Adjust `total_cost`.
        if vlr.total_cost < decrease_total_cost_by {
            vlr.total_cost = 0;
        } else {
            vlr.total_cost -= decrease_total_cost_by;
        }
        // And recompute `spill_cost` accordingly.
        if vlr.total_cost == 0 {
            vlr.spill_cost = SpillCost::finite(1.0e-6);
        } else {
            assert!(vlr.size > 0);
            vlr.spill_cost = SpillCost::finite(vlr.total_cost as f32 / vlr.size as f32);
        }
    }

    for (vlrix1, vlrix2, decrease_total_cost_by) in decVLRcosts {
        decrease_vlr_total_cost_by(&mut vlr_env[vlrix1], decrease_total_cost_by);
        decrease_vlr_total_cost_by(&mut vlr_env[vlrix2], decrease_total_cost_by);
    }

    // For the convenience of the allocator core, sort the hints for each VLR so
    // as to move the most preferred to the front.
    for hints_for_one_vlr in hints.iter_mut() {
        hints_for_one_vlr.sort_by(|h1, h2| h2.get_weight().partial_cmp(&h1.get_weight()).unwrap());
    }

    let vlrEquivClasses: UnionFindEquivClasses<VirtualRangeIx> =
        vlrEquivClassesUF.get_equiv_classes();

    if log_enabled!(Level::Trace) {
        trace!("Revised VLRs:");
        let mut n = 0;
        for vlr in vlr_env.iter() {
            trace!("{:<4?}   {:?}", VirtualRangeIx::new(n), vlr);
            n += 1;
        }

        trace!("Coalescing hints:");
        n = 0;
        for hints_for_one_vlr in hints.iter() {
            let mut s = "".to_string();
            for hint in hints_for_one_vlr {
                s = s + &show_hint(hint, &univ) + &" ".to_string();
            }
            trace!("  hintsfor {:<4?} = {}", VirtualRangeIx::new(n), s);
            n += 1;
        }

        for n in 0..vlr_env.len() {
            let vlrix = VirtualRangeIx::new(n);
            let mut tmpvec = vec![];
            for elem in vlrEquivClasses.equiv_class_elems_iter(vlrix) {
                tmpvec.reverse();
                tmpvec.push(elem);
            }
            trace!("  eclassof {:?} = {:?}", vlrix, tmpvec);
        }

        for (b, i) in is_vv_boundary_move.iter().zip(0..) {
            if *b {
                trace!("  vv_boundary_move at {:?}", InstIx::new(i));
            }
        }
    }

    trace!("do_coalescing_analysis: end");
    trace!("");

    (hints, vlrEquivClasses, is_vv_boundary_move)
}
