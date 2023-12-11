// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

//! Memory management for tables.
//!
//! `Table` is to WebAssembly tables what `Memory` is to WebAssembly linear memories.

use crate::store::MaybeInstanceOwned;
use crate::vmcontext::VMTableDefinition;
use crate::Trap;
use crate::VMExternRef;
use crate::VMFuncRef;
use derivative::Derivative;
use std::cell::UnsafeCell;
use std::convert::TryFrom;
use std::fmt;
use std::ptr::NonNull;
use wasmer_types::TableStyle;
use wasmer_types::{TableType, TrapCode, Type as ValType};

/// A reference stored in a table. Can be either an externref or a funcref.
#[derive(Debug, Clone)]
pub enum TableElement {
    /// Opaque pointer to arbitrary host data.
    ExternRef(Option<VMExternRef>),
    /// Pointer to function: contains enough information to call it.
    FuncRef(Option<VMFuncRef>),
}

impl From<TableElement> for RawTableElement {
    fn from(other: TableElement) -> Self {
        match other {
            TableElement::ExternRef(extern_ref) => Self { extern_ref },
            TableElement::FuncRef(func_ref) => Self { func_ref },
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union RawTableElement {
    pub(crate) extern_ref: Option<VMExternRef>,
    pub(crate) func_ref: Option<VMFuncRef>,
}

#[cfg(test)]
#[test]
fn table_element_size_test() {
    use std::mem::size_of;
    assert_eq!(size_of::<RawTableElement>(), size_of::<VMExternRef>());
    assert_eq!(size_of::<RawTableElement>(), size_of::<VMFuncRef>());
}

impl fmt::Debug for RawTableElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RawTableElement").finish()
    }
}

impl Default for RawTableElement {
    fn default() -> Self {
        Self { func_ref: None }
    }
}

impl Default for TableElement {
    fn default() -> Self {
        Self::FuncRef(None)
    }
}

/// A table instance.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct VMTable {
    #[derivative(Debug = "ignore")]
    vec: Vec<RawTableElement>,
    maximum: Option<u32>,
    /// The WebAssembly table description.
    table: TableType,
    /// Our chosen implementation style.
    style: TableStyle,
    #[derivative(Debug = "ignore")]
    vm_table_definition: MaybeInstanceOwned<VMTableDefinition>,
}

impl VMTable {
    /// Create a new linear table instance with specified minimum and maximum number of elements.
    ///
    /// This creates a `Table` with metadata owned by a VM, pointed to by
    /// `vm_table_location`: this can be used to create a local table.
    pub fn new(table: &TableType, style: &TableStyle) -> Result<Self, String> {
        unsafe { Self::new_inner(table, style, None) }
    }

    /// Returns the size of the table
    pub fn get_runtime_size(&self) -> u32 {
        self.vec.len() as u32
    }

    /// Create a new linear table instance with specified minimum and maximum number of elements.
    ///
    /// This creates a `Table` with metadata owned by a VM, pointed to by
    /// `vm_table_location`: this can be used to create a local table.
    ///
    /// # Safety
    /// - `vm_table_location` must point to a valid location in VM memory.
    pub unsafe fn from_definition(
        table: &TableType,
        style: &TableStyle,
        vm_table_location: NonNull<VMTableDefinition>,
    ) -> Result<Self, String> {
        Self::new_inner(table, style, Some(vm_table_location))
    }

    /// Create a new `Table` with either self-owned or VM owned metadata.
    unsafe fn new_inner(
        table: &TableType,
        style: &TableStyle,
        vm_table_location: Option<NonNull<VMTableDefinition>>,
    ) -> Result<Self, String> {
        match table.ty {
            ValType::FuncRef | ValType::ExternRef => (),
            ty => {
                return Err(format!(
                    "tables of types other than funcref or externref ({})",
                    ty
                ))
            }
        };
        if let Some(max) = table.maximum {
            if max < table.minimum {
                return Err(format!(
                    "Table minimum ({}) is larger than maximum ({})!",
                    table.minimum, max
                ));
            }
        }
        let table_minimum = usize::try_from(table.minimum)
            .map_err(|_| "Table minimum is bigger than usize".to_string())?;
        let mut vec = vec![RawTableElement::default(); table_minimum];
        let base = vec.as_mut_ptr();
        match style {
            TableStyle::CallerChecksSignature => Ok(Self {
                vec,
                maximum: table.maximum,
                table: *table,
                style: style.clone(),
                vm_table_definition: if let Some(table_loc) = vm_table_location {
                    {
                        let mut ptr = table_loc;
                        let td = ptr.as_mut();
                        td.base = base as _;
                        td.current_elements = table_minimum as _;
                    }
                    MaybeInstanceOwned::Instance(table_loc)
                } else {
                    MaybeInstanceOwned::Host(Box::new(UnsafeCell::new(VMTableDefinition {
                        base: base as _,
                        current_elements: table_minimum as _,
                    })))
                },
            }),
        }
    }

    /// Get the `VMTableDefinition`.
    fn get_vm_table_definition(&self) -> NonNull<VMTableDefinition> {
        self.vm_table_definition.as_ptr()
    }

    /// Returns the type for this Table.
    pub fn ty(&self) -> &TableType {
        &self.table
    }

    /// Returns the style for this Table.
    pub fn style(&self) -> &TableStyle {
        &self.style
    }

    /// Returns the number of allocated elements.
    pub fn size(&self) -> u32 {
        // TODO: investigate this function for race conditions
        unsafe {
            let td_ptr = self.get_vm_table_definition();
            let td = td_ptr.as_ref();
            td.current_elements
        }
    }

    /// Grow table by the specified amount of elements.
    ///
    /// Returns `None` if table can't be grown by the specified amount
    /// of elements, otherwise returns the previous size of the table.
    pub fn grow(&mut self, delta: u32, init_value: TableElement) -> Option<u32> {
        let size = self.size();
        let new_len = size.checked_add(delta)?;
        if self.maximum.map_or(false, |max| new_len > max) {
            return None;
        }
        if new_len == size {
            debug_assert_eq!(delta, 0);
            return Some(size);
        }

        self.vec
            .resize(usize::try_from(new_len).unwrap(), init_value.into());

        // update table definition
        unsafe {
            let mut td_ptr = self.get_vm_table_definition();
            let td = td_ptr.as_mut();
            td.current_elements = new_len;
            td.base = self.vec.as_mut_ptr() as _;
        }
        Some(size)
    }

    /// Get reference to the specified element.
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn get(&self, index: u32) -> Option<TableElement> {
        let raw_data = self.vec.get(index as usize).cloned()?;
        Some(match self.table.ty {
            ValType::ExternRef => TableElement::ExternRef(unsafe { raw_data.extern_ref }),
            ValType::FuncRef => TableElement::FuncRef(unsafe { raw_data.func_ref }),
            _ => todo!("getting invalid type from table, handle this error"),
        })
    }

    /// Set reference to the specified element.
    ///
    /// # Errors
    ///
    /// Returns an error if the index is out of bounds.
    pub fn set(&mut self, index: u32, reference: TableElement) -> Result<(), Trap> {
        match self.vec.get_mut(index as usize) {
            Some(slot) => {
                match (self.table.ty, reference) {
                    (ValType::ExternRef, r @ TableElement::ExternRef(_)) => {
                        *slot = r.into();
                    }
                    (ValType::FuncRef, r @ TableElement::FuncRef(_)) => {
                        *slot = r.into();
                    }
                    // This path should never be hit by the generated code due to Wasm
                    // validation.
                    (ty, v) => {
                        panic!(
                            "Attempted to set a table of type {} with the value {:?}",
                            ty, v
                        )
                    }
                };

                Ok(())
            }
            None => Err(Trap::lib(TrapCode::TableAccessOutOfBounds)),
        }
    }

    /// Return a `VMTableDefinition` for exposing the table to compiled wasm code.
    pub fn vmtable(&self) -> NonNull<VMTableDefinition> {
        self.get_vm_table_definition()
    }

    /// Copy `len` elements from `src_table[src_index..]` into `dst_table[dst_index..]`.
    ///
    /// # Errors
    ///
    /// Returns an error if the range is out of bounds of either the source or
    /// destination tables.
    pub fn copy(
        &mut self,
        src_table: &Self,
        dst_index: u32,
        src_index: u32,
        len: u32,
    ) -> Result<(), Trap> {
        // https://webassembly.github.io/bulk-memory-operations/core/exec/instructions.html#exec-table-copy

        if src_index
            .checked_add(len)
            .map_or(true, |n| n > src_table.size())
        {
            return Err(Trap::lib(TrapCode::TableAccessOutOfBounds));
        }

        if dst_index.checked_add(len).map_or(true, |m| m > self.size()) {
            return Err(Trap::lib(TrapCode::TableAccessOutOfBounds));
        }

        let srcs = src_index..src_index + len;
        let dsts = dst_index..dst_index + len;

        // Note on the unwraps: the bounds check above means that these will
        // never panic.
        //
        // TODO: investigate replacing this get/set loop with a `memcpy`.
        if dst_index <= src_index {
            for (s, d) in (srcs).zip(dsts) {
                self.set(d, src_table.get(s).unwrap())?;
            }
        } else {
            for (s, d) in srcs.rev().zip(dsts.rev()) {
                self.set(d, src_table.get(s).unwrap())?;
            }
        }

        Ok(())
    }

    /// Copies the table into a new table
    pub fn copy_on_write(&self) -> Result<Self, String> {
        let mut ret = Self::new(&self.table, &self.style)?;
        ret.copy(self, 0, 0, self.size())
            .map_err(|trap| format!("failed to copy the table - {:?}", trap))?;
        Ok(ret)
    }

    /// Copy `len` elements from `table[src_index..]` to `table[dst_index..]`.
    ///
    /// # Errors
    ///
    /// Returns an error if the range is out of bounds of either the source or
    /// destination tables.
    pub fn copy_within(&mut self, dst_index: u32, src_index: u32, len: u32) -> Result<(), Trap> {
        // https://webassembly.github.io/bulk-memory-operations/core/exec/instructions.html#exec-table-copy

        if src_index.checked_add(len).map_or(true, |n| n > self.size()) {
            return Err(Trap::lib(TrapCode::TableAccessOutOfBounds));
        }

        if dst_index.checked_add(len).map_or(true, |m| m > self.size()) {
            return Err(Trap::lib(TrapCode::TableAccessOutOfBounds));
        }

        let srcs = src_index..src_index + len;
        let dsts = dst_index..dst_index + len;

        // Note on the unwraps: the bounds check above means that these will
        // never panic.
        //
        // TODO: investigate replacing this get/set loop with a `memcpy`.
        if dst_index <= src_index {
            for (s, d) in (srcs).zip(dsts) {
                self.set(d, self.get(s).unwrap())?;
            }
        } else {
            for (s, d) in srcs.rev().zip(dsts.rev()) {
                self.set(d, self.get(s).unwrap())?;
            }
        }

        Ok(())
    }
}
