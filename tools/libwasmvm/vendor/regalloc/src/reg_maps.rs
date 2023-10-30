use crate::{RealReg, RegUsageMapper, VirtualReg};
use smallvec::SmallVec;
use std::mem;

/// This data structure holds the mappings needed to map an instruction's uses, mods and defs from
/// virtual to real registers.
///
/// It remembers the sets of mappings (of a virtual register to a real register) over time, based
/// on precise virtual ranges and their allocations.
///
/// This is the right implementation to use when a register allocation algorithm keeps track of
/// precise virtual ranges, and maintains them over time.
#[derive(Debug)]
pub struct VrangeRegUsageMapper {
    /// Dense vector-map indexed by virtual register number. This is consulted
    /// directly for use-queries and augmented with the overlay for def-queries.
    slots: Vec<RealReg>,

    /// Overlay for def-queries. This is a set of updates that occurs "during"
    /// the instruction in question, and will be applied to the slots array
    /// once we are done processing this instruction (in preparation for
    /// the next one).
    overlay: SmallVec<[(VirtualReg, RealReg); 16]>,
}

impl VrangeRegUsageMapper {
    /// Allocate a reg-usage mapper with the given predicted vreg capacity.
    pub(crate) fn new(vreg_capacity: usize) -> VrangeRegUsageMapper {
        VrangeRegUsageMapper {
            slots: Vec::with_capacity(vreg_capacity),
            overlay: SmallVec::new(),
        }
    }

    /// Is the overlay past the sorted-size threshold?
    fn is_overlay_large_enough_to_sort(&self) -> bool {
        // Use the SmallVec spill-to-heap threshold as a threshold for "large
        // enough to sort"; this has the effect of amortizing the cost of
        // sorting along with the cost of copying out to heap memory, and also
        // ensures that when we access heap (more likely to miss in cache), we
        // do it with O(log N) accesses instead of O(N).
        self.overlay.spilled()
    }

    /// Update the overlay.
    pub(crate) fn set_overlay(&mut self, vreg: VirtualReg, rreg: Option<RealReg>) {
        let rreg = rreg.unwrap_or(RealReg::invalid());
        self.overlay.push((vreg, rreg));
    }

    /// Finish updates to the overlay, sorting if necessary.
    pub(crate) fn finish_overlay(&mut self) {
        if self.overlay.len() == 0 || !self.is_overlay_large_enough_to_sort() {
            return;
        }

        // Sort stably, so that later updates continue to come after earlier
        // ones.
        self.overlay.sort_by_key(|pair| pair.0);
        // Remove duplicates by collapsing runs of same-vreg pairs down to
        // the last one.
        let mut last_vreg = self.overlay[0].0;
        let mut out = 0;
        for i in 1..self.overlay.len() {
            let this_vreg = self.overlay[i].0;
            if this_vreg != last_vreg {
                out += 1;
            }
            if i != out {
                self.overlay[out] = self.overlay[i];
            }
            last_vreg = this_vreg;
        }
        let new_len = out + 1;
        self.overlay.truncate(new_len);
    }

    /// Merge the overlay into the main map.
    pub(crate) fn merge_overlay(&mut self) {
        // Take the SmallVec and swap with empty to allow `&mut self` method
        // call below.
        let mappings = mem::replace(&mut self.overlay, SmallVec::new());
        for (vreg, rreg) in mappings.into_iter() {
            self.set_direct_internal(vreg, rreg);
        }
    }

    /// Make a direct update to the mapping. Only usable when the overlay
    /// is empty.
    pub(crate) fn set_direct(&mut self, vreg: VirtualReg, rreg: Option<RealReg>) {
        debug_assert!(self.overlay.is_empty());
        let rreg = rreg.unwrap_or(RealReg::invalid());
        self.set_direct_internal(vreg, rreg);
    }

    fn set_direct_internal(&mut self, vreg: VirtualReg, rreg: RealReg) {
        let idx = vreg.get_index();
        if idx >= self.slots.len() {
            self.slots.resize(idx + 1, RealReg::invalid());
        }
        self.slots[idx] = rreg;
    }

    /// Perform a lookup directly in the main map. Returns `None` for
    /// not-present.
    fn lookup_direct(&self, vreg: VirtualReg) -> Option<RealReg> {
        let idx = vreg.get_index();
        if idx >= self.slots.len() {
            None
        } else {
            Some(self.slots[idx])
        }
    }

    /// Perform a lookup in the overlay. Returns `None` for not-present. No
    /// fallback to main map (that happens in callers). Returns `Some` even
    /// if mapped to `RealReg::invalid()`, because this is a tombstone
    /// (represents deletion) in the overlay.
    fn lookup_overlay(&self, vreg: VirtualReg) -> Option<RealReg> {
        if self.is_overlay_large_enough_to_sort() {
            // Do a binary search; we are guaranteed to have at most one
            // matching because duplicates were collapsed after sorting.
            if let Ok(idx) = self.overlay.binary_search_by_key(&vreg, |pair| pair.0) {
                return Some(self.overlay[idx].1);
            }
        } else {
            // Search in reverse order to find later updates first.
            for &(this_vreg, this_rreg) in self.overlay.iter().rev() {
                if this_vreg == vreg {
                    return Some(this_rreg);
                }
            }
        }
        None
    }

    /// Sanity check: check that all slots are empty. Typically for use at the
    /// end of processing as a debug-assert.
    pub(crate) fn is_empty(&self) -> bool {
        self.overlay.iter().all(|pair| pair.1.is_invalid())
            && self.slots.iter().all(|rreg| rreg.is_invalid())
    }
}

impl RegUsageMapper for VrangeRegUsageMapper {
    /// Return the `RealReg` if mapped, or `None`, for `vreg` occuring as a use
    /// on the current instruction.
    fn get_use(&self, vreg: VirtualReg) -> Option<RealReg> {
        self.lookup_direct(vreg)
            // Convert Some(RealReg::invalid()) to None.
            .and_then(|reg| reg.maybe_valid())
    }

    /// Return the `RealReg` if mapped, or `None`, for `vreg` occuring as a def
    /// on the current instruction.
    fn get_def(&self, vreg: VirtualReg) -> Option<RealReg> {
        self.lookup_overlay(vreg)
            .or_else(|| self.lookup_direct(vreg))
            // Convert Some(RealReg::invalid()) to None.
            .and_then(|reg| reg.maybe_valid())
    }

    /// Return the `RealReg` if mapped, or `None`, for a `vreg` occuring as a
    /// mod on the current instruction.
    fn get_mod(&self, vreg: VirtualReg) -> Option<RealReg> {
        let result = self.get_use(vreg);
        debug_assert_eq!(result, self.get_def(vreg));
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Reg, RegClass, VirtualReg};

    fn vreg(idx: u32) -> VirtualReg {
        Reg::new_virtual(RegClass::I64, idx).to_virtual_reg()
    }
    fn rreg(idx: u8) -> RealReg {
        Reg::new_real(RegClass::I64, /* enc = */ 0, /* index = */ idx).to_real_reg()
    }

    #[test]
    fn test_reg_use_mapper() {
        let mut mapper = VrangeRegUsageMapper::new(/* estimated vregs = */ 16);
        assert_eq!(None, mapper.get_use(vreg(0)));
        assert_eq!(None, mapper.get_def(vreg(0)));
        assert_eq!(None, mapper.get_mod(vreg(0)));

        mapper.set_direct(vreg(0), Some(rreg(1)));
        mapper.set_direct(vreg(1), Some(rreg(2)));

        assert_eq!(Some(rreg(1)), mapper.get_use(vreg(0)));
        assert_eq!(Some(rreg(1)), mapper.get_def(vreg(0)));
        assert_eq!(Some(rreg(1)), mapper.get_mod(vreg(0)));
        assert_eq!(Some(rreg(2)), mapper.get_use(vreg(1)));
        assert_eq!(Some(rreg(2)), mapper.get_def(vreg(1)));
        assert_eq!(Some(rreg(2)), mapper.get_mod(vreg(1)));

        mapper.set_overlay(vreg(0), Some(rreg(3)));
        mapper.set_overlay(vreg(2), Some(rreg(4)));
        mapper.finish_overlay();

        assert_eq!(Some(rreg(1)), mapper.get_use(vreg(0)));
        assert_eq!(Some(rreg(3)), mapper.get_def(vreg(0)));
        // vreg 0 not valid for mod (use and def differ).
        assert_eq!(Some(rreg(2)), mapper.get_use(vreg(1)));
        assert_eq!(Some(rreg(2)), mapper.get_def(vreg(1)));
        assert_eq!(Some(rreg(2)), mapper.get_mod(vreg(1)));
        assert_eq!(None, mapper.get_use(vreg(2)));
        assert_eq!(Some(rreg(4)), mapper.get_def(vreg(2)));
        // vreg 2 not valid for mod (use and def differ).

        mapper.merge_overlay();
        assert_eq!(Some(rreg(3)), mapper.get_use(vreg(0)));
        assert_eq!(Some(rreg(2)), mapper.get_use(vreg(1)));
        assert_eq!(Some(rreg(4)), mapper.get_use(vreg(2)));
        assert_eq!(None, mapper.get_use(vreg(3)));

        // Check tombstoning behavior.
        mapper.set_overlay(vreg(0), None);
        mapper.finish_overlay();
        assert_eq!(Some(rreg(3)), mapper.get_use(vreg(0)));
        assert_eq!(None, mapper.get_def(vreg(0)));
        mapper.merge_overlay();

        // Check large (sorted) overlay mode.
        for i in (2..50).rev() {
            mapper.set_overlay(vreg(i), Some(rreg((i + 100) as u8)));
        }
        mapper.finish_overlay();
        assert_eq!(None, mapper.get_use(vreg(0)));
        assert_eq!(Some(rreg(2)), mapper.get_use(vreg(1)));
        assert_eq!(Some(rreg(4)), mapper.get_use(vreg(2)));
        for i in 2..50 {
            assert_eq!(Some(rreg((i + 100) as u8)), mapper.get_def(vreg(i)));
        }
        mapper.merge_overlay();

        for i in (0..100).rev() {
            mapper.set_overlay(vreg(i), None);
        }
        mapper.finish_overlay();
        for i in 0..100 {
            assert_eq!(None, mapper.get_def(vreg(i)));
        }
        assert_eq!(false, mapper.is_empty());
        mapper.merge_overlay();
        assert_eq!(true, mapper.is_empty());

        // Check multiple-update behavior in small mode.
        mapper.set_overlay(vreg(1), Some(rreg(1)));
        mapper.set_overlay(vreg(1), Some(rreg(2)));
        mapper.finish_overlay();
        assert_eq!(Some(rreg(2)), mapper.get_def(vreg(1)));
        mapper.merge_overlay();
        assert_eq!(Some(rreg(2)), mapper.get_use(vreg(1)));

        mapper.set_overlay(vreg(1), Some(rreg(2)));
        mapper.set_overlay(vreg(1), None);
        mapper.finish_overlay();
        assert_eq!(None, mapper.get_def(vreg(1)));
        mapper.merge_overlay();
        assert_eq!(None, mapper.get_use(vreg(1)));

        // Check multiple-update behavior in sorted mode.
        for i in 0..100 {
            mapper.set_overlay(vreg(2), Some(rreg(i)));
        }
        for i in 0..100 {
            mapper.set_overlay(vreg(2), Some(rreg(2 * i)));
        }
        mapper.finish_overlay();
        assert_eq!(Some(rreg(198)), mapper.get_def(vreg(2)));
        mapper.merge_overlay();
        assert_eq!(Some(rreg(198)), mapper.get_use(vreg(2)));

        for i in 0..100 {
            mapper.set_overlay(vreg(2), Some(rreg(i)));
        }
        for _ in 0..100 {
            mapper.set_overlay(vreg(2), None);
        }
        mapper.finish_overlay();
        assert_eq!(None, mapper.get_def(vreg(50)));
        mapper.merge_overlay();
        assert_eq!(None, mapper.get_use(vreg(50)));
    }
}

/// This implementation of RegUsageMapper relies on explicit mentions of vregs in instructions. The
/// caller must keep them, and for each instruction:
///
/// - clear the previous mappings, using `clear()`,
/// - feed the mappings from vregs to rregs for uses and defs, with `set_use`/`set_def`,
/// - then call the `Function::map_regs` function with this structure.
///
/// This avoids a lot of resizes, and makes it possible for algorithms that don't have precise live
/// ranges to fill in vreg -> rreg mappings.
#[derive(Debug)]
pub struct MentionRegUsageMapper {
    /// Sparse vector-map indexed by virtual register number. This is consulted for use-queries.
    uses: SmallVec<[(VirtualReg, RealReg); 8]>,

    /// Sparse vector-map indexed by virtual register number. This is consulted for def-queries.
    defs: SmallVec<[(VirtualReg, RealReg); 8]>,
}

impl MentionRegUsageMapper {
    pub(crate) fn new() -> Self {
        Self {
            uses: SmallVec::new(),
            defs: SmallVec::new(),
        }
    }
    pub(crate) fn clear(&mut self) {
        self.uses.clear();
        self.defs.clear();
    }
    pub(crate) fn lookup_use(&self, vreg: VirtualReg) -> Option<RealReg> {
        self.uses.iter().find(|&pair| pair.0 == vreg).map(|x| x.1)
    }
    pub(crate) fn lookup_def(&self, vreg: VirtualReg) -> Option<RealReg> {
        self.defs.iter().find(|&pair| pair.0 == vreg).map(|x| x.1)
    }
    pub(crate) fn set_use(&mut self, vreg: VirtualReg, rreg: RealReg) {
        self.uses.push((vreg, rreg));
    }
    pub(crate) fn set_def(&mut self, vreg: VirtualReg, rreg: RealReg) {
        self.defs.push((vreg, rreg));
    }
}

impl RegUsageMapper for MentionRegUsageMapper {
    fn get_use(&self, vreg: VirtualReg) -> Option<RealReg> {
        return self.lookup_use(vreg);
    }
    fn get_def(&self, vreg: VirtualReg) -> Option<RealReg> {
        return self.lookup_def(vreg);
    }
    fn get_mod(&self, vreg: VirtualReg) -> Option<RealReg> {
        let result = self.lookup_use(vreg);
        debug_assert_eq!(result, self.lookup_def(vreg));
        return result;
    }
}
