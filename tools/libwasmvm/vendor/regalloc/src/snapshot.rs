//! Snapshotting facilities.
//!
//! This makes it possible to save one entire IR input in a generic form that encapsulates all the
//! constraints, so as to be replayed only in the regalloc.rs environment. The main structure,
//! `GenericFunction`, can be created from any type implementing `Function`, acting as a generic
//! Function wrapper. Its layout is simple enough that it can be optionally serialized and
//! deserialized, making it easy to transfer test cases from regalloc.rs users to the crate's
//! maintainers.

use crate::data_structures::RegVecs;
use crate::*;
use std::borrow::Cow;

#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
enum IRInstKind {
    Spill { vreg: Option<VirtualReg> },
    Reload { vreg: Option<VirtualReg> },
    Move { vreg: VirtualReg },
    ZeroLenNop,
    UserReturn,
    UserMove,
    UserOther,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct IRInst {
    reg_uses: Vec<Reg>,
    reg_mods: Vec<Writable<Reg>>,
    reg_defs: Vec<Writable<Reg>>,
    kind: IRInstKind,
}

#[derive(Clone)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct IRFunction {
    instructions: Vec<IRInst>,
    block_ranges: Vec<Range<InstIx>>,
    block_succs: Vec<Vec<BlockIx>>,
    entry_block: BlockIx,
    liveins: Set<RealReg>,
    liveouts: Set<RealReg>,
    vreg_spill_slot_sizes: Vec<Option<(u32, RegClass)>>,
    num_vregs: usize,
}

#[derive(Clone)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct IRSnapshot {
    reg_universe: RealRegUniverse,
    func: IRFunction,
}

impl IRSnapshot {
    fn new_inst<F: Function>(func: &F, ix: InstIx, inst: &F::Inst) -> IRInst {
        let mut reg_vecs = RegVecs::new(/* sanitized */ false);

        let mut collector = RegUsageCollector::new(&mut reg_vecs);
        F::get_regs(inst, &mut collector);

        let kind = if let Some((_wreg, _reg)) = func.is_move(inst) {
            IRInstKind::UserMove
        } else if func.is_ret(ix) {
            IRInstKind::UserReturn
        } else {
            IRInstKind::UserOther
        };

        IRInst {
            reg_uses: reg_vecs.uses,
            reg_mods: reg_vecs
                .mods
                .into_iter()
                .map(|reg| Writable::from_reg(reg))
                .collect(),
            reg_defs: reg_vecs
                .defs
                .into_iter()
                .map(|reg| Writable::from_reg(reg))
                .collect(),
            kind,
        }
    }

    pub fn from_function<F: Function>(func: &F, reg_universe: &RealRegUniverse) -> Self {
        let instructions: Vec<IRInst> = func
            .insns()
            .iter()
            .enumerate()
            .map(|(ix, inst)| IRSnapshot::new_inst(func, InstIx::new(ix as u32), inst))
            .collect();

        let mut block_ranges = Vec::new();
        let mut block_succs = Vec::new();
        for block in func.blocks() {
            block_ranges.push(func.block_insns(block));
            block_succs.push(func.block_succs(block).into());
        }

        let vreg_spill_slot_sizes = {
            let mut array: Vec<Option<(u32, RegClass)>> = Vec::new();

            let mut handle_reg = |reg: &Reg| {
                if let Some(vreg) = reg.as_virtual_reg() {
                    let rc = vreg.get_class();
                    let spill_slot_size = func.get_spillslot_size(rc, vreg);
                    let index = vreg.get_index();
                    if index >= array.len() {
                        array.resize(index + 1, None);
                    }
                    let entry = &mut array[vreg.get_index()];
                    match entry {
                        None => *entry = Some((spill_slot_size, rc)),
                        Some((prev_size, prev_rc)) => {
                            assert_eq!(*prev_rc, rc);
                            assert_eq!(*prev_size, spill_slot_size);
                        }
                    }
                }
            };

            for inst in &instructions {
                for reg in &inst.reg_uses {
                    handle_reg(reg);
                }
                for reg in &inst.reg_mods {
                    handle_reg(&reg.to_reg());
                }
                for reg in &inst.reg_defs {
                    handle_reg(&reg.to_reg());
                }
            }

            array
        };

        let entry_block = func.entry_block();
        let liveins = func.func_liveins();
        let liveouts = func.func_liveouts();

        Self {
            reg_universe: reg_universe.clone(),
            func: IRFunction {
                instructions,
                block_ranges,
                block_succs,
                entry_block,
                liveins,
                liveouts,
                vreg_spill_slot_sizes,
                num_vregs: func.get_num_vregs(),
            },
        }
    }

    pub fn allocate(&mut self, opts: Options) -> Result<RegAllocResult<IRFunction>, RegAllocError> {
        allocate_registers_with_opts(
            &mut self.func,
            &self.reg_universe,
            None, /*no stackmap request*/
            opts,
        )
    }
}

impl Function for IRFunction {
    type Inst = IRInst;

    // Liveins, liveouts.
    fn func_liveins(&self) -> Set<RealReg> {
        self.liveins.clone()
    }
    fn func_liveouts(&self) -> Set<RealReg> {
        self.liveouts.clone()
    }
    fn get_num_vregs(&self) -> usize {
        self.num_vregs
    }

    // Instructions.
    fn insns(&self) -> &[Self::Inst] {
        &self.instructions
    }
    fn insns_mut(&mut self) -> &mut [Self::Inst] {
        &mut self.instructions
    }
    fn get_insn(&self, insn: InstIx) -> &Self::Inst {
        &self.instructions[insn.get() as usize]
    }
    fn get_insn_mut(&mut self, insn: InstIx) -> &mut Self::Inst {
        &mut self.instructions[insn.get() as usize]
    }

    fn is_ret(&self, insn: InstIx) -> bool {
        let inst = &self.instructions[insn.get() as usize];
        if let IRInstKind::UserReturn = inst.kind {
            true
        } else {
            false
        }
    }

    fn is_move(&self, insn: &Self::Inst) -> Option<(Writable<Reg>, Reg)> {
        if let IRInstKind::UserMove = insn.kind {
            let from = insn.reg_uses[0];
            let to = insn.reg_defs[0];
            Some((to, from))
        } else {
            None
        }
    }

    // Blocks.
    fn blocks(&self) -> Range<BlockIx> {
        Range::new(BlockIx::new(0), self.block_ranges.len())
    }
    fn entry_block(&self) -> BlockIx {
        self.entry_block
    }
    fn block_insns(&self, block: BlockIx) -> Range<InstIx> {
        self.block_ranges[block.get() as usize]
    }
    fn block_succs(&self, block: BlockIx) -> Cow<[BlockIx]> {
        Cow::Borrowed(&self.block_succs[block.get() as usize])
    }

    fn get_regs(insn: &Self::Inst, collector: &mut RegUsageCollector) {
        collector.add_uses(&insn.reg_uses);
        collector.add_mods(&insn.reg_mods);
        collector.add_defs(&insn.reg_defs);
    }

    fn map_regs<RUM: RegUsageMapper>(insn: &mut Self::Inst, maps: &RUM) {
        for reg_use in insn.reg_uses.iter_mut() {
            if let Some(vreg) = reg_use.as_virtual_reg() {
                *reg_use = maps.get_use(vreg).expect("missing alloc for use").to_reg();
            }
        }
        for reg_mod in insn.reg_mods.iter_mut() {
            if let Some(vreg) = reg_mod.to_reg().as_virtual_reg() {
                *reg_mod =
                    Writable::from_reg(maps.get_mod(vreg).expect("missing alloc for mod").to_reg());
            }
        }
        for reg_def in insn.reg_defs.iter_mut() {
            if let Some(vreg) = reg_def.to_reg().as_virtual_reg() {
                *reg_def =
                    Writable::from_reg(maps.get_def(vreg).expect("missing alloc for def").to_reg());
            }
        }
    }

    fn gen_spill(
        &self,
        _to_slot: SpillSlot,
        from_reg: RealReg,
        for_vreg: Option<VirtualReg>,
    ) -> Self::Inst {
        IRInst {
            reg_uses: vec![from_reg.to_reg()],
            reg_mods: vec![],
            reg_defs: vec![],
            kind: IRInstKind::Spill { vreg: for_vreg },
        }
    }
    fn gen_reload(
        &self,
        to_reg: Writable<RealReg>,
        _from_slot: SpillSlot,
        for_vreg: Option<VirtualReg>,
    ) -> Self::Inst {
        IRInst {
            reg_uses: vec![],
            reg_mods: vec![],
            reg_defs: vec![Writable::from_reg(to_reg.to_reg().to_reg())],
            kind: IRInstKind::Reload { vreg: for_vreg },
        }
    }
    fn gen_move(
        &self,
        to_reg: Writable<RealReg>,
        from_reg: RealReg,
        for_vreg: VirtualReg,
    ) -> Self::Inst {
        IRInst {
            reg_uses: vec![from_reg.to_reg()],
            reg_mods: vec![],
            reg_defs: vec![Writable::from_reg(to_reg.to_reg().to_reg())],
            kind: IRInstKind::Move { vreg: for_vreg },
        }
    }
    fn gen_zero_len_nop(&self) -> Self::Inst {
        IRInst {
            reg_uses: vec![],
            reg_mods: vec![],
            reg_defs: vec![],
            kind: IRInstKind::ZeroLenNop,
        }
    }

    fn get_spillslot_size(&self, regclass: RegClass, for_vreg: VirtualReg) -> u32 {
        let entry =
            self.vreg_spill_slot_sizes[for_vreg.get_index()].expect("missing spillslot info");
        assert_eq!(entry.1, regclass);
        return entry.0;
    }

    fn maybe_direct_reload(
        &self,
        _insn: &Self::Inst,
        _reg: VirtualReg,
        _slot: SpillSlot,
    ) -> Option<Self::Inst> {
        unimplemented!();
    }
}
