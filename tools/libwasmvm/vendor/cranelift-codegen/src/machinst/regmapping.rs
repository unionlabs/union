use crate::ir::Type;
use regalloc::{Reg, RegUsageMapper, Writable};
use smallvec::SmallVec;
use std::cell::Cell;

// Define our own register-mapping trait so we can do arbitrary register
// renaming that are more free form than what `regalloc` constrains us to with
// its `RegUsageMapper` trait definition.
pub trait RegMapper {
    fn get_use(&self, reg: Reg) -> Option<Reg>;
    fn get_def(&self, reg: Reg) -> Option<Reg>;
    fn get_mod(&self, reg: Reg) -> Option<Reg>;

    fn map_use(&self, r: &mut Reg) {
        if let Some(new) = self.get_use(*r) {
            *r = new;
        }
    }

    fn map_def(&self, r: &mut Writable<Reg>) {
        if let Some(new) = self.get_def(r.to_reg()) {
            *r = Writable::from_reg(new);
        }
    }

    fn map_mod(&self, r: &mut Writable<Reg>) {
        if let Some(new) = self.get_mod(r.to_reg()) {
            *r = Writable::from_reg(new);
        }
    }
}

impl<T> RegMapper for T
where
    T: RegUsageMapper,
{
    fn get_use(&self, reg: Reg) -> Option<Reg> {
        let v = reg.as_virtual_reg()?;
        self.get_use(v).map(|r| r.to_reg())
    }

    fn get_def(&self, reg: Reg) -> Option<Reg> {
        let v = reg.as_virtual_reg()?;
        self.get_def(v).map(|r| r.to_reg())
    }

    fn get_mod(&self, reg: Reg) -> Option<Reg> {
        let v = reg.as_virtual_reg()?;
        self.get_mod(v).map(|r| r.to_reg())
    }
}

#[derive(Debug, Default)]
pub struct RegRenamer {
    // Map of `(old, new, used, ty)` register names. Use a `SmallVec` because
    // we typically only have one or two renamings.
    //
    // The `used` flag indicates whether the mapping has been used for
    // `get_def`, later used afterwards during `unmapped_defs` to know what
    // moves need to be generated.
    renames: SmallVec<[(Reg, Reg, Cell<bool>, Type); 2]>,
}

impl RegRenamer {
    /// Adds a new mapping which means that `old` reg should now be called
    /// `new`. The type of `old` is `ty` as specified.
    pub fn add_rename(&mut self, old: Reg, new: Reg, ty: Type) {
        self.renames.push((old, new, Cell::new(false), ty));
    }

    fn get_rename(&self, reg: Reg, set_used_def: bool) -> Option<Reg> {
        let (_, new, used_def, _) = self.renames.iter().find(|(old, _, _, _)| reg == *old)?;
        used_def.set(used_def.get() || set_used_def);
        Some(*new)
    }

    /// Returns the list of register mappings, with their type, which were not
    /// actually mapped.
    ///
    /// This list is used because it means that the `old` name for the register
    /// was never actually defined, so to correctly rename this register the
    /// caller needs to move `old` into `new`.
    ///
    /// This yields tuples of `(old, new, ty)`.
    pub fn unmapped_defs(&self) -> impl Iterator<Item = (Reg, Reg, Type)> + '_ {
        self.renames.iter().filter_map(|(old, new, used_def, ty)| {
            if used_def.get() {
                None
            } else {
                Some((*old, *new, *ty))
            }
        })
    }
}

impl RegMapper for RegRenamer {
    fn get_use(&self, reg: Reg) -> Option<Reg> {
        self.get_rename(reg, false)
    }

    fn get_def(&self, reg: Reg) -> Option<Reg> {
        self.get_rename(reg, true)
    }

    fn get_mod(&self, reg: Reg) -> Option<Reg> {
        self.get_rename(reg, false)
    }
}
