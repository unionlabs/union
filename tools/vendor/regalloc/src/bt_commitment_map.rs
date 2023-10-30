#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

//! Backtracking allocator: per-real-register commitment maps

use std::cmp::Ordering;
use std::fmt;

use crate::avl_tree::{AVLTree, AVL_NULL};
use crate::data_structures::{
    cmp_range_frags, InstPoint, RangeFrag, RangeFragIx, RangeId, SortedRangeFragIxs,
    SortedRangeFrags, TypedIxVec,
};

//=============================================================================
// Per-real-register commitment maps
//

// Something that pairs a fragment index with the identity of the virtual or real range to which
// this fragment conceptually "belongs", at least for the purposes of this commitment map.  If
// the `lr_id` field denotes a real range, the associated fragment belongs to a real-reg live
// range and is therefore non-evictable.  The identity of the range is necessary because:
//
// * for VirtualRanges, (1) we may need to evict the mapping, so we will need to get hold of the
//   VirtualRange, so that we have all fragments of the VirtualRange to hand, and (2) if the
//   client requires stackmaps, we need to look at the VirtualRange to see if it is reftyped.
//
// * for RealRanges, only (2) applies; (1) is irrelevant since RealRange assignments are
//   non-evictable.
//
// (A fragment merely denotes a sequence of instruction (points), but within the context of a
// commitment map for a real register, obviously any particular fragment can't be part of two
// different virtual live ranges.)
//
// Note that we don't intend to actually use the PartialOrd methods for RangeFragAndRangeId.
// However, they need to exist since we want to construct an AVLTree<RangeFragAndRangeId>, and
// that requires PartialOrd for its element type.  For working with such trees we will supply
// our own comparison function; hence PartialOrd here serves only to placate the typechecker.
// It should never actually be used.
#[derive(Clone)]
pub struct RangeFragAndRangeId {
    pub frag: RangeFrag,
    pub id: RangeId,
}
impl RangeFragAndRangeId {
    fn new(frag: RangeFrag, id: RangeId) -> Self {
        Self { frag, id }
    }
}
impl PartialEq for RangeFragAndRangeId {
    fn eq(&self, _other: &Self) -> bool {
        // See comments above.
        panic!("impl PartialEq for RangeFragAndRangeId: should never be used");
    }
}
impl PartialOrd for RangeFragAndRangeId {
    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
        // See comments above.
        panic!("impl PartialOrd for RangeFragAndRangeId: should never be used");
    }
}
impl fmt::Debug for RangeFragAndRangeId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "(FnV {:?} {:?})", self.frag, self.id)
    }
}

//=============================================================================
// Per-real-register commitment maps
//

// This indicates the current set of fragments to which some real register is
// currently "committed".  The fragments *must* be non-overlapping.  Hence
// they form a total order, and so we may validly build an AVL tree of them.

pub(crate) struct CommitmentMap {
    pub tree: AVLTree<RangeFragAndRangeId>,
}

impl fmt::Debug for CommitmentMap {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let as_vec = self.tree.to_vec();
        as_vec.fmt(fmt)
    }
}

impl CommitmentMap {
    pub(crate) fn new() -> Self {
        // The AVL tree constructor needs a default value for the elements.  It
        // will never be used.  The RangeId index value will show as
        // obviously bogus if we ever try to "dereference" any part of it.
        let dflt = RangeFragAndRangeId::new(RangeFrag::invalid_value(), RangeId::invalid_value());
        Self {
            tree: AVLTree::<RangeFragAndRangeId>::new(dflt),
        }
    }

    pub(crate) fn add(&mut self, to_add_frags: &SortedRangeFrags, to_add_lr_id: RangeId) {
        for frag in to_add_frags.iter() {
            let to_add = RangeFragAndRangeId::new(frag.clone(), to_add_lr_id);
            let added = self.tree.insert(
                to_add,
                Some(&|pair1: RangeFragAndRangeId, pair2: RangeFragAndRangeId| {
                    cmp_range_frags(&pair1.frag, &pair2.frag)
                }),
            );
            // If this fails, it means the fragment overlaps one that has already
            // been committed to.  That's a serious error.
            assert!(added);
        }
    }

    pub(crate) fn add_indirect(
        &mut self,
        to_add_frags: &SortedRangeFragIxs,
        to_add_lr_id: RangeId,
        frag_env: &TypedIxVec<RangeFragIx, RangeFrag>,
    ) {
        for fix in to_add_frags.iter() {
            let to_add = RangeFragAndRangeId::new(frag_env[*fix].clone(), to_add_lr_id);
            let added = self.tree.insert(
                to_add,
                Some(&|pair1: RangeFragAndRangeId, pair2: RangeFragAndRangeId| {
                    cmp_range_frags(&pair1.frag, &pair2.frag)
                }),
            );
            // If this fails, it means the fragment overlaps one that has already
            // been committed to.  That's a serious error.
            assert!(added);
        }
    }

    pub(crate) fn del(&mut self, to_del_frags: &SortedRangeFrags) {
        for frag in to_del_frags.iter() {
            // re RangeId::invalid_value(): we don't care what the RangeId is, since we're
            // deleting by RangeFrags alone.
            let to_del = RangeFragAndRangeId::new(frag.clone(), RangeId::invalid_value());
            let deleted = self.tree.delete(
                to_del,
                Some(&|pair1: RangeFragAndRangeId, pair2: RangeFragAndRangeId| {
                    cmp_range_frags(&pair1.frag, &pair2.frag)
                }),
            );
            // If this fails, it means the fragment wasn't already committed to.
            // That's also a serious error.
            assert!(deleted);
        }
    }

    // Find the RangeId for the RangeFrag that overlaps `pt`, if one exists.
    // This is conceptually equivalent to LogicalSpillSlot::get_refness_at_inst_point.
    pub(crate) fn lookup_inst_point(&self, pt: InstPoint) -> Option<RangeId> {
        let mut root = self.tree.root;
        while root != AVL_NULL {
            let root_node = &self.tree.pool[root as usize];
            let root_item = &root_node.item;
            if pt < root_item.frag.first {
                // `pt` is to the left of the `root`.  So there's no
                // overlap with `root`.  Continue by inspecting the left subtree.
                root = root_node.left;
            } else if root_item.frag.last < pt {
                // Ditto for the right subtree.
                root = root_node.right;
            } else {
                // `pt` overlaps the `root`, so we have what we want.
                return Some(root_item.id);
            }
        }
        None
    }
}
