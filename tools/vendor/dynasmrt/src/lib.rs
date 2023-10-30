#![warn(missing_docs)]

//! This crate provides runtime support for dynasm-rs. It contains traits that document the interface used by the dynasm proc_macro to generate code,
//! Assemblers that implement these traits, and relocation models for the various supported architectures. Additionally, it also provides the tools
//! to write your own Assemblers using these components.

pub mod mmap;
pub mod components;
pub mod relocations;

/// Helper to implement common traits on register enums.
macro_rules! reg_impls {
    ($r:ty) => {
        impl $crate::Register for $r {
            fn code(&self) -> u8 {
                *self as u8
            }
        }

        impl From<$r> for u8 {
            fn from(rq: $r) -> u8 {
                rq.code()
            }
        }
    }
}

pub mod x64;
pub mod x86;
pub mod aarch64;

pub use crate::mmap::ExecutableBuffer;
pub use dynasm::{dynasm, dynasm_backwards};

use crate::components::{MemoryManager, LabelRegistry, RelocRegistry, ManagedRelocs, PatchLoc};
use crate::relocations::Relocation;

use std::hash::Hash;
use std::iter::Extend;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::io;
use std::error;
use std::fmt::{self, Debug};
use std::mem;

/// This macro takes a *const pointer from the source operand, and then casts it to the desired return type.
/// this allows it to be used as an easy shorthand for passing pointers as dynasm immediate arguments.
#[macro_export]
macro_rules! Pointer {
    ($e:expr) => {$e as *const _ as _};
}

/// Preforms the same action as the `Pointer!` macro, but casts to a *mut pointer.
#[macro_export]
macro_rules! MutPointer {
    ($e:expr) => {$e as *mut _ as _};
}


/// A struct representing an offset into the assembling buffer of a `DynasmLabelApi` struct.
/// The wrapped `usize` is the offset from the start of the assembling buffer in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssemblyOffset(pub usize);

/// A dynamic label
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DynamicLabel(usize);


impl DynamicLabel {
    /// Get the internal ID of this dynamic label. This is only useful for debugging purposes.
    pub fn get_id(self) -> usize {
        self.0
    }
}


/// A read-only shared reference to the executable buffer inside an Assembler. By
/// locking it the internal `ExecutableBuffer` can be accessed and executed.
#[derive(Debug, Clone)]
pub struct Executor {
    execbuffer: Arc<RwLock<ExecutableBuffer>>
}

/// A read-only lockable reference to the internal `ExecutableBuffer` of an Assembler.
/// To gain access to this buffer, it must be locked.
impl Executor {
    /// Gain read-access to the internal `ExecutableBuffer`. While the returned guard
    /// is alive, it can be used to read and execute from the `ExecutableBuffer`.
    /// Any pointers created to the `Executablebuffer` should no longer be used when
    /// the guard is dropped.
    #[inline]
    pub fn lock(&self) -> RwLockReadGuard<ExecutableBuffer> {
        self.execbuffer.read().unwrap()
    }
}


/// A description of a label. Used for error reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LabelKind {
    /// A local label, like `label:`
    Local(&'static str),
    /// A global label, like `->label:`
    Global(&'static str),
    /// A dynamic label, like `=>value:`
    Dynamic(DynamicLabel)
}

impl fmt::Display for LabelKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Local(s) => write!(f, "label {}", s),
            Self::Global(s) => write!(f, "label ->{}", s),
            Self::Dynamic(id) => write!(f, "label =>{}", id.get_id())
        }
    }
}


/// A description of a relocation target. Used for error reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetKind {
    /// This targets a local label with the specified name that still has to be defined.
    Forward(&'static str),
    /// This targets a local label with the specified name that was already previously defined.
    Backward(&'static str),
    /// This targets a global label with the specified name.
    Global(&'static str),
    /// This targets the specified dynamic label.
    Dynamic(DynamicLabel),
    /// This targets the specified address.
    Extern(usize),
    /// An already resolved relocation that needs to be adjusted when the buffer moves in memory.
    Managed,
}

impl fmt::Display for TargetKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Forward(s) => write!(f, "target >{}", s),
            Self::Backward(s) => write!(f, "target <{}", s),
            Self::Global(s) => write!(f, "target ->{}", s),
            Self::Dynamic(id) => write!(f, "target =>{}", id.get_id()),
            Self::Extern(value) => write!(f, "target extern {}", value),
            Self::Managed => write!(f, "while adjusting managed relocation"),
        }
    }
}


/// The various error types generated by dynasm functions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DynasmError {
    /// A check (like `Modifier::check` or `Modifier::check_exact`) that failed
    CheckFailed,
    /// A duplicate label dynamic/global label was defined
    DuplicateLabel(LabelKind),
    /// An unknown label
    UnknownLabel(LabelKind),
    /// The user tried to declare a relocation too far away from the label it targets
    ImpossibleRelocation(TargetKind),
}

impl fmt::Display for DynasmError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DynasmError::CheckFailed => write!(f, "An assembly modification check failed"),
            DynasmError::DuplicateLabel(l) => write!(f, "Duplicate label defined: '{}'", l),
            DynasmError::UnknownLabel(l) => write!(f, "Unknown label: '{}'", l),
            DynasmError::ImpossibleRelocation(s) => write!(f, "Impossible relocation: '{}'", s),
        }
    }
}

impl error::Error for DynasmError {
    fn description(&self) -> &str {
        match self {
            DynasmError::CheckFailed => "An assembly modification offset check failed",
            DynasmError::DuplicateLabel(_) => "Duplicate label defined",
            DynasmError::UnknownLabel(_) => "Unknown label",
            DynasmError::ImpossibleRelocation(_) => "Impossible relocation",
        }
    }
}


/// This trait represents the interface that must be implemented to allow
/// the dynasm preprocessor to assemble into a datastructure.
pub trait DynasmApi: Extend<u8> + for<'a> Extend<&'a u8> {
    /// Report the current offset into the assembling target
    fn offset(&self) -> AssemblyOffset;
    /// Push a byte into the assembling target
    fn push(&mut self, byte: u8);
    /// Push filler until the assembling target end is aligned to the given alignment.
    fn align(&mut self, alignment: usize, with: u8);

    #[inline]
    /// Push a signed byte into the assembling target
    fn push_i8(&mut self, value: i8) {
        self.push(value as u8);
    }
    /// Push a signed word into the assembling target
    #[inline]
    fn push_i16(&mut self, value: i16) {
        self.extend(&value.to_le_bytes());
    }
    /// Push a signed doubleword into the assembling target
    #[inline]
    fn push_i32(&mut self, value: i32) {
        self.extend(&value.to_le_bytes());
    }
    /// Push a signed quadword into the assembling target
    #[inline]
    fn push_i64(&mut self, value: i64) {
        self.extend(&value.to_le_bytes());
    }
    /// Push an usigned word into the assembling target
    #[inline]
    fn push_u16(&mut self, value: u16) {
        self.extend(&value.to_le_bytes());
    }
    /// Push an usigned doubleword into the assembling target
    #[inline]
    fn push_u32(&mut self, value: u32) {
        self.extend(&value.to_le_bytes());
    }
    /// Push an usigned quadword into the assembling target
    #[inline]
    fn push_u64(&mut self, value: u64) {
        self.extend(&value.to_le_bytes());
    }
    /// This function is called in when a runtime error has to be generated. It panics.
    #[inline]
    fn runtime_error(&self, msg: &'static str) -> ! {
        panic!("{}", msg);
    }
}

/// This trait extends DynasmApi to not only allow assembling, but also labels and various directives
pub trait DynasmLabelApi : DynasmApi {
    /// The relocation info type this assembler uses. 
    type Relocation: Relocation;

    /// Record the definition of a local label
    fn local_label(  &mut self, name: &'static str);
    /// Record the definition of a global label
    fn global_label( &mut self, name: &'static str);
    /// Record the definition of a dynamic label
    fn dynamic_label(&mut self, id: DynamicLabel);

    /// Record a relocation spot for a forward reference to a local label
    fn forward_reloc( &mut self, name: &'static str, target_offset: isize, field_offset: u8, ref_offset: u8, kind: <Self::Relocation as Relocation>::Encoding) {
        self.forward_relocation(name, target_offset, field_offset, ref_offset, Self::Relocation::from_encoding(kind))
    }
    /// Record a relocation spot for a backward reference to a local label
    fn backward_reloc(&mut self, name: &'static str, target_offset: isize, field_offset: u8, ref_offset: u8, kind: <Self::Relocation as Relocation>::Encoding) {
        self.backward_relocation(name, target_offset, field_offset, ref_offset, Self::Relocation::from_encoding(kind))
    }
    /// Record a relocation spot for a reference to a global label
    fn global_reloc(  &mut self, name: &'static str, target_offset: isize, field_offset: u8, ref_offset: u8, kind: <Self::Relocation as Relocation>::Encoding) {
        self.global_relocation(name, target_offset, field_offset, ref_offset, Self::Relocation::from_encoding(kind))
    }
    /// Record a relocation spot for a reference to a dynamic label
    fn dynamic_reloc( &mut self, id: DynamicLabel,   target_offset: isize, field_offset: u8, ref_offset: u8, kind: <Self::Relocation as Relocation>::Encoding) {
        self.dynamic_relocation(id, target_offset, field_offset, ref_offset, Self::Relocation::from_encoding(kind))
    }
    /// Record a relocation spot to an arbitrary target.
    fn bare_reloc(&mut self, target: usize, field_offset: u8, ref_offset: u8, kind: <Self::Relocation as Relocation>::Encoding) {
        self.bare_relocation(target, field_offset, ref_offset, Self::Relocation::from_encoding(kind))
    }

    /// Equivalent of forward_reloc, but takes a non-encoded relocation
    fn forward_relocation( &mut self, name: &'static str, target_offset: isize, field_offset: u8, ref_offset: u8, kind: Self::Relocation);
    /// Equivalent of backward_reloc, but takes a non-encoded relocation
    fn backward_relocation(&mut self, name: &'static str, target_offset: isize, field_offset: u8, ref_offset: u8, kind: Self::Relocation);
    /// Equivalent of global_reloc, but takes a non-encoded relocation
    fn global_relocation(  &mut self, name: &'static str, target_offset: isize, field_offset: u8, ref_offset: u8, kind: Self::Relocation);
    /// Equivalent of dynamic_reloc, but takes a non-encoded relocation
    fn dynamic_relocation( &mut self, id: DynamicLabel,   target_offset: isize, field_offset: u8, ref_offset: u8, kind: Self::Relocation);
    /// Equivalent of bare_reloc, but takes a non-encoded relocation
    fn bare_relocation(&mut self, target: usize, field_offset: u8, ref_offset: u8, kind: Self::Relocation);
}


/// An assembler that is purely a `Vec<u8>`. It doesn't support labels or architecture-specific directives,
/// but can be used to easily inspect generated code. It is intended to be used in testcases.
#[derive(Debug, Clone)]
pub struct SimpleAssembler {
    /// The assembling buffer.
    pub ops: Vec<u8>
}

impl SimpleAssembler {
    /// Creates a new `SimpleAssembler`, containing an empty `Vec`.
    pub fn new() -> SimpleAssembler {
        SimpleAssembler {
            ops: Vec::new()
        }
    }

    /// Use an `UncommittedModifier` to alter uncommitted code.
    pub fn alter(&mut self) -> UncommittedModifier {
        UncommittedModifier::new(&mut self.ops, AssemblyOffset(0))
    }

    /// Destroys this assembler, returning the `Vec<u8>` contained within
    pub fn finalize(self) -> Vec<u8> {
        self.ops
    }
}

impl Extend<u8> for SimpleAssembler {
    fn extend<T>(&mut self, iter: T) where T: IntoIterator<Item=u8> {
        self.ops.extend(iter)
    }
}

impl<'a> Extend<&'a u8> for SimpleAssembler {
    fn extend<T>(&mut self, iter: T) where T: IntoIterator<Item=&'a u8> {
        self.ops.extend(iter)
    }
}

impl DynasmApi for SimpleAssembler {
    fn offset(&self) -> AssemblyOffset {
        AssemblyOffset(self.ops.len())
    }
    fn push(&mut self, byte: u8) {
        self.ops.push(byte);
    }
    fn align(&mut self, alignment: usize, with: u8) {
        let offset = self.offset().0 % alignment;
        if offset != 0 {
            for _ in offset .. alignment {
                self.push(with);
            }
        }
    }
}


/// An assembler that assembles into a `Vec<u8>`, while supporting labels. To support the different types of relocations
/// it requires a base address of the to be assembled code to be specified.
#[derive(Debug)]
pub struct VecAssembler<R: Relocation> {
    ops: Vec<u8>,
    baseaddr: usize,
    labels: LabelRegistry,
    relocs: RelocRegistry<R>,
    error: Option<DynasmError>,
}

impl<R: Relocation> VecAssembler<R> {
    /// Creates a new VecAssembler, with the specified base address.
    pub fn new(baseaddr: usize) -> VecAssembler<R> {
        VecAssembler {
            ops: Vec::new(),
            baseaddr,
            labels: LabelRegistry::new(),
            relocs: RelocRegistry::new(),
            error: None
        }
    }

    /// Create a new dynamic label ID
    pub fn new_dynamic_label(&mut self) -> DynamicLabel {
        self.labels.new_dynamic_label()
    }

    /// Resolves any relocations emitted to the assembler before this point.
    /// If an impossible relocation was specified before this point, returns them here.
    pub fn commit(&mut self) -> Result<(), DynasmError> {
        // If we accrued any errors while assembling before, emit them now.
        if let Some(e) = self.error.take() {
            return Err(e);
        }

        // Resolve globals
        for (loc, name) in self.relocs.take_globals() {
            let target = self.labels.resolve_global(name)?;
            let buf = &mut self.ops[loc.range(0)];
            if loc.patch(buf, self.baseaddr, target.0).is_err() {
                return Err(DynasmError::ImpossibleRelocation(TargetKind::Global(name)));
            }
        }

        // Resolve dynamics
        for (loc, id) in self.relocs.take_dynamics() {
            let target = self.labels.resolve_dynamic(id)?;
            let buf = &mut self.ops[loc.range(0)];
            if loc.patch(buf, self.baseaddr, target.0).is_err() {
                return Err(DynasmError::ImpossibleRelocation(TargetKind::Dynamic(id)));
            }
        }

        // Check that there are no unknown local labels
        for (_, name) in self.relocs.take_locals() {
            return Err(DynasmError::UnknownLabel(LabelKind::Local(name)));
        }

        Ok(())
    }

    /// Use an `UncommittedModifier` to alter uncommitted code.
    /// This does not allow the user to change labels/relocations.
    pub fn alter(&mut self) -> UncommittedModifier {
        UncommittedModifier::new(&mut self.ops, AssemblyOffset(0))
    }

    /// Provides access to the assemblers internal labels registry
    pub fn labels(&self) -> &LabelRegistry {
        &self.labels
    }

    /// Provides mutable access to the assemblers internal labels registry
    pub fn labels_mut(&mut self) -> &mut LabelRegistry {
        &mut self.labels
    }

    /// Finalizes the `VecAssembler`, returning the resulting `Vec<u8>` containing all assembled data.
    /// this implicitly commits any relocations beforehand and returns an error if required.
    pub fn finalize(mut self) -> Result<Vec<u8>, DynasmError> {
        self.commit()?;
        Ok(self.ops)
    }
}

impl<R: Relocation> Extend<u8> for VecAssembler<R> {
    fn extend<T>(&mut self, iter: T) where T: IntoIterator<Item=u8> {
        self.ops.extend(iter)
    }
}

impl<'a, R: Relocation> Extend<&'a u8> for VecAssembler<R> {
    fn extend<T>(&mut self, iter: T) where T: IntoIterator<Item=&'a u8> {
        self.ops.extend(iter)
    }
}

impl<R: Relocation> DynasmApi for VecAssembler<R> {
    fn offset(&self) -> AssemblyOffset {
        AssemblyOffset(self.ops.len())
    }
    fn push(&mut self, byte: u8) {
        self.ops.push(byte);
    }
    fn align(&mut self, alignment: usize, with: u8) {
        let offset = self.offset().0 % alignment;
        if offset != 0 {
            for _ in offset .. alignment {
                self.push(with);
            }
        }
    }
}

impl<R: Relocation> DynasmLabelApi for VecAssembler<R> {
    type Relocation = R;

    fn local_label(&mut self, name: &'static str) {
        let offset = self.offset();
        for loc in self.relocs.take_locals_named(name) {
            let buf = &mut self.ops[loc.range(0)];
            if loc.patch(buf, self.baseaddr, offset.0).is_err() {
                self.error = Some(DynasmError::ImpossibleRelocation(TargetKind::Forward(name)))
            }
        }
        self.labels.define_local(name, offset);
    }
    fn global_label( &mut self, name: &'static str) {
        let offset = self.offset();
        if let Err(e) = self.labels.define_global(name, offset) {
            self.error = Some(e)
        }
    }
    fn dynamic_label(&mut self, id: DynamicLabel) {
        let offset = self.offset();
        if let Err(e) = self.labels.define_dynamic(id, offset) {
            self.error = Some(e)
        }
    }
    fn global_relocation(&mut self, name: &'static str, target_offset: isize, field_offset: u8, ref_offset: u8, kind: R) {
        let location = self.offset();
        self.relocs.add_global(name, PatchLoc::new(location, target_offset, field_offset, ref_offset, kind));
    }
    fn dynamic_relocation(&mut self, id: DynamicLabel, target_offset: isize, field_offset: u8, ref_offset: u8, kind: R) {
        let location = self.offset();
        self.relocs.add_dynamic(id, PatchLoc::new(location, target_offset, field_offset, ref_offset, kind));
    }
    fn forward_relocation(&mut self, name: &'static str, target_offset: isize, field_offset: u8, ref_offset: u8, kind: R) {
        let location = self.offset();
        self.relocs.add_local(name, PatchLoc::new(location, target_offset, field_offset, ref_offset, kind));
    }
    fn backward_relocation(&mut self, name: &'static str, target_offset: isize, field_offset: u8, ref_offset: u8, kind: R) {
        let target = match self.labels.resolve_local(name) {
            Ok(target) => target.0,
            Err(e) => {
                self.error = Some(e);
                return;
            }
        };
        let location = self.offset();
        let loc = PatchLoc::new(location, target_offset, field_offset, ref_offset, kind);
        let buf = &mut self.ops[loc.range(0)];
        if loc.patch(buf, self.baseaddr, target).is_err() {
            self.error = Some(DynasmError::ImpossibleRelocation(TargetKind::Backward(name)))
        }
    }
    fn bare_relocation(&mut self, target: usize, field_offset: u8, ref_offset: u8, kind: R) {
        let location = self.offset();
        let loc = PatchLoc::new(location, 0, field_offset, ref_offset, kind);
        let buf = &mut self.ops[loc.range(0)];
        if loc.patch(buf, self.baseaddr, target).is_err() {
            self.error = Some(DynasmError::ImpossibleRelocation(TargetKind::Extern(target)))
        }
    }
}


/// A full assembler implementation. Supports labels, all types of relocations,
/// incremental compilation and multithreaded execution with simultaneous compiltion.
/// Its implementation guarantees no memory is executable and writable at the same time.
#[derive(Debug)]
pub struct Assembler<R: Relocation> {
    ops: Vec<u8>,
    memory: MemoryManager,
    labels: LabelRegistry,
    relocs: RelocRegistry<R>,
    managed: ManagedRelocs<R>,
    error: Option<DynasmError>,
}

impl<R: Relocation> Assembler<R> {
    /// Create a new, empty assembler, with initial allocation size `page_size`.
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            ops: Vec::new(),
            memory: MemoryManager::new(R::page_size())?,
            labels: LabelRegistry::new(),
            relocs: RelocRegistry::new(),
            managed: ManagedRelocs::new(),
            error: None
        })
    }

    /// Create a new dynamic label ID
    pub fn new_dynamic_label(&mut self) -> DynamicLabel {
        self.labels.new_dynamic_label()
    }

    /// Use an `UncommittedModifier` to alter uncommitted code.
    /// This does not allow the user to change labels/relocations.
    pub fn alter_uncommitted(&mut self) -> UncommittedModifier {
        let offset = self.memory.committed();
        UncommittedModifier::new(&mut self.ops, AssemblyOffset(offset))
    }

    /// Use a `Modifier` to alter committed code directly. While this is happening
    /// no code can be executed as the relevant pages are remapped as writable.
    /// This API supports defining new labels/relocations, and overwriting previously defined relocations.
    pub fn alter<F, O>(&mut self, f: F) -> Result<O, DynasmError>
    where F: FnOnce(&mut Modifier<R>) -> O {
        self.commit()?;

        // swap out a buffer from base
        let mut lock = self.memory.write();
        let buffer = mem::replace(&mut *lock, ExecutableBuffer::default());
        let mut buffer = buffer.make_mut().expect("Could not swap buffer protection modes");

        // construct the modifier
        let mut modifier = Modifier {
            asmoffset: 0,
            previous_asmoffset: 0,
            buffer: &mut *buffer,

            labels: &mut self.labels,
            relocs: &mut self.relocs,
            old_managed: &mut self.managed,
            new_managed: ManagedRelocs::new(),

            error: None
        };

        // execute the user code
        let output = f(&mut modifier);

        // flush any changes made by the user code to the buffer
        modifier.encode_relocs()?;

        // repack the buffer
        let buffer = buffer.make_exec().expect("Could not swap buffer protection modes");
        *lock = buffer;

        // call it a day
        Ok(output)
    }

    /// Commit code, flushing the temporary internal assembling buffer to the mapped executable memory.
    /// This makes assembled code available for execution.
    pub fn commit(&mut self) -> Result<(), DynasmError> {
        self.encode_relocs()?;

        let managed = &self.managed;
        let error = &mut self.error;

        self.memory.commit(&mut self.ops, |buffer, old_addr, new_addr| {
            let change = new_addr.wrapping_sub(old_addr) as isize;

            for reloc in managed.iter() {
                let buf = &mut buffer[reloc.range(0)];
                if reloc.adjust(buf, change).is_err() {
                    *error = Some(DynasmError::ImpossibleRelocation(TargetKind::Managed))
                }
            }
        });

        if let Some(e) = self.error.take() {
            return Err(e);
        }
        Ok(())
    }

    /// Finalize this assembler, returning the internal executablebuffer if no Executor instances exist.
    /// This panics if any uncommitted changes caused errors near the end. To handle these, call `commit()` explicitly beforehand.
    pub fn finalize(mut self) -> Result<ExecutableBuffer, Self> {
        self.commit().expect("Errors were encountered when committing before finalization");
        match self.memory.finalize() {
            Ok(execbuffer) => Ok(execbuffer),
            Err(memory) => Err(Self {
                memory,
                ..self
            })
        }
    }

    /// Create an executor which can be used to execute code while still assembling code
    pub fn reader(&self) -> Executor {
        Executor {
            execbuffer: self.memory.reader()
        }
    }

    /// Provides access to the assemblers internal labels registry
    pub fn labels(&self) -> &LabelRegistry {
        &self.labels
    }

    /// Provides mutable access to the assemblers internal labels registry
    pub fn labels_mut(&mut self) -> &mut LabelRegistry {
        &mut self.labels
    }

    // encode uncommited relocations
    fn encode_relocs(&mut self) -> Result<(), DynasmError> {
        let buf_offset = self.memory.committed();
        let buf_addr = self.memory.execbuffer_addr();
        let buf = &mut self.ops;

        // If we accrued any errors while assembling before, emit them now.
        if let Some(e) = self.error.take() {
            return Err(e);
        }

        // Resolve globals
        for (loc, name) in self.relocs.take_globals() {
            let target = self.labels.resolve_global(name)?;
            let buf = &mut buf[loc.range(buf_offset)];
            if loc.patch(buf, buf_addr, target.0).is_err() {
                return Err(DynasmError::ImpossibleRelocation(TargetKind::Global(name)));
            }
            if loc.needs_adjustment() {
                self.managed.add(loc)
            }
        }

        // Resolve dynamics
        for (loc, id) in self.relocs.take_dynamics() {
            let target = self.labels.resolve_dynamic(id)?;
            let buf = &mut buf[loc.range(buf_offset)];
            if loc.patch(buf, buf_addr, target.0).is_err() {
                return Err(DynasmError::ImpossibleRelocation(TargetKind::Dynamic(id)));
            }
            if loc.needs_adjustment() {
                self.managed.add(loc)
            }
        }

        // Check that there are no unknown local labels
        for (_, name) in self.relocs.take_locals() {
            return Err(DynasmError::UnknownLabel(LabelKind::Local(name)));
        }

        Ok(())
    }
}

impl<R: Relocation> Extend<u8> for Assembler<R> {
    fn extend<T>(&mut self, iter: T) where T: IntoIterator<Item=u8> {
        self.ops.extend(iter)
    }
}

impl<'a, R: Relocation> Extend<&'a u8> for Assembler<R> {
    fn extend<T>(&mut self, iter: T) where T: IntoIterator<Item=&'a u8> {
        self.ops.extend(iter)
    }
}

impl<R: Relocation> DynasmApi for Assembler<R> {
    fn offset(&self) -> AssemblyOffset {
        AssemblyOffset(self.memory.committed() + self.ops.len())
    }

    fn push(&mut self, value: u8) {
        self.ops.push(value);
    }

    fn align(&mut self, alignment: usize, with: u8) {
        let misalign = self.offset().0 % alignment;
        if misalign != 0 {
            for _ in misalign .. alignment {
                self.push(with);
            }
        }
    }
}

impl<R: Relocation> DynasmLabelApi for Assembler<R> {
    type Relocation = R;

    fn local_label(&mut self, name: &'static str) {
        let offset = self.offset();
        for loc in self.relocs.take_locals_named(name) {
            let buf = &mut self.ops[loc.range(self.memory.committed())];
            if loc.patch(buf, self.memory.execbuffer_addr(), offset.0).is_err() {
                self.error = Some(DynasmError::ImpossibleRelocation(TargetKind::Forward(name)))
            } else if loc.needs_adjustment() {
                self.managed.add(loc)
            }
        }
        self.labels.define_local(name, offset);
    }
    fn global_label( &mut self, name: &'static str) {
        let offset = self.offset();
        if let Err(e) = self.labels.define_global(name, offset) {
            self.error = Some(e)
        }
    }
    fn dynamic_label(&mut self, id: DynamicLabel) {
        let offset = self.offset();
        if let Err(e) = self.labels.define_dynamic(id, offset) {
            self.error = Some(e)
        }
    }
    fn global_relocation(&mut self, name: &'static str, target_offset: isize, field_offset: u8, ref_offset: u8, kind: R) {
        let location = self.offset();
        self.relocs.add_global(name, PatchLoc::new(location, target_offset, field_offset, ref_offset, kind));
    }
    fn dynamic_relocation(&mut self, id: DynamicLabel, target_offset: isize, field_offset: u8, ref_offset: u8, kind: R) {
        let location = self.offset();
        self.relocs.add_dynamic(id, PatchLoc::new(location, target_offset, field_offset, ref_offset, kind));
    }
    fn forward_relocation(&mut self, name: &'static str, target_offset: isize, field_offset: u8, ref_offset: u8, kind: R) {
        let location = self.offset();
        self.relocs.add_local(name, PatchLoc::new(location, target_offset, field_offset, ref_offset, kind));
    }
    fn backward_relocation(&mut self, name: &'static str, target_offset: isize, field_offset: u8, ref_offset: u8, kind: R) {
        let target = match self.labels.resolve_local(name) {
            Ok(target) => target.0,
            Err(e) => {
                self.error = Some(e);
                return;
            }
        };
        let location = self.offset();
        let loc = PatchLoc::new(location, target_offset, field_offset, ref_offset, kind);
        let buf = &mut self.ops[loc.range(self.memory.committed())];
        if loc.patch(buf, self.memory.execbuffer_addr(), target).is_err() {
            self.error = Some(DynasmError::ImpossibleRelocation(TargetKind::Backward(name)))
        } else if loc.needs_adjustment() {
            self.managed.add(loc)
        }
    }
    fn bare_relocation(&mut self, target: usize, field_offset: u8, ref_offset: u8, kind: R) {
        let location = self.offset();
        let loc = PatchLoc::new(location, 0, field_offset, ref_offset, kind);
        let buf = &mut self.ops[loc.range(self.memory.committed())];
        if loc.patch(buf, self.memory.execbuffer_addr(), target).is_err() {
            self.error = Some(DynasmError::ImpossibleRelocation(TargetKind::Extern(target)))
        } else if loc.needs_adjustment() {
            self.managed.add(loc)
        }
    }
}


/// Allows modification of already committed assembly code. Contains an internal cursor
/// into the emitted assembly, initialized to the start, that can be moved around either with the
/// `goto` function, or just by assembling new code into this `Modifier`.
#[derive(Debug)]
pub struct Modifier<'a, R: Relocation> {
    asmoffset: usize,
    previous_asmoffset: usize,
    buffer: &'a mut [u8],

    labels: &'a mut LabelRegistry,
    relocs: &'a mut RelocRegistry<R>,
    old_managed: &'a mut ManagedRelocs<R>,
    new_managed: ManagedRelocs<R>,

    error: Option<DynasmError>
}

impl<'a, R: Relocation> Modifier<'a, R> {
    /// Move the modifier cursor to the selected location.
    pub fn goto(&mut self, offset: AssemblyOffset) {
        self.old_managed.remove_between(self.previous_asmoffset, self.asmoffset);
        self.asmoffset = offset.0;
        self.previous_asmoffset = offset.0;
    }

    /// Check that the modifier cursor has not moved past the specified location.
    pub fn check(&self, offset: AssemblyOffset) -> Result<(), DynasmError> {
        if self.asmoffset > offset.0 {
            Err(DynasmError::CheckFailed)
        } else {
            Ok(())
        }
    }

    /// Check that the modifier cursor is exactly at the specified location.
    pub fn check_exact(&self, offset: AssemblyOffset) -> Result<(), DynasmError> {
        if self.asmoffset != offset.0 {
            Err(DynasmError::CheckFailed)
        } else {
            Ok(())
        }
    }

    // encode uncommited relocations
    fn encode_relocs(&mut self) -> Result<(), DynasmError> {
        let buf_addr = self.buffer.as_ptr() as usize;

        // If we accrued any errors while assembling before, emit them now.
        if let Some(e) = self.error.take() {
            return Err(e);
        }

        // Resolve globals
        for (loc, name) in self.relocs.take_globals() {
            let target = self.labels.resolve_global(name)?;
            let buf = &mut self.buffer[loc.range(0)];
            if loc.patch(buf, buf_addr, target.0).is_err() {
                return Err(DynasmError::ImpossibleRelocation(TargetKind::Global(name)));
            }
            if loc.needs_adjustment() {
                self.new_managed.add(loc);
            }
        }

        // Resolve dynamics
        for (loc, id) in self.relocs.take_dynamics() {
            let target = self.labels.resolve_dynamic(id)?;
            let buf = &mut self.buffer[loc.range(0)];
            if loc.patch(buf, buf_addr, target.0).is_err() {
                return Err(DynasmError::ImpossibleRelocation(TargetKind::Dynamic(id)));
            }
            if loc.needs_adjustment() {
                self.new_managed.add(loc);
            }
        }

        // Check for unknown locals
        for (_, name) in self.relocs.take_locals() {
            return Err(DynasmError::UnknownLabel(LabelKind::Local(name)));
        }

        self.old_managed.remove_between(self.previous_asmoffset, self.asmoffset);
        self.previous_asmoffset = self.asmoffset;

        self.old_managed.append(&mut self.new_managed);

        Ok(())
    }
}

impl<'a, R: Relocation> Extend<u8> for Modifier<'a,R> {
    fn extend<T>(&mut self, iter: T) where T: IntoIterator<Item=u8> {
        for (src, dst) in iter.into_iter().zip(self.buffer[self.asmoffset ..].iter_mut()) {
            *dst = src;
            self.asmoffset += 1;
        }
    }
}

impl<'a, 'b, R: Relocation> Extend<&'b u8> for Modifier<'a, R> {
    fn extend<T>(&mut self, iter: T) where T: IntoIterator<Item=&'b u8> {
        for (src, dst) in iter.into_iter().zip(self.buffer[self.asmoffset ..].iter_mut()) {
            *dst = *src;
            self.asmoffset += 1;
        }
    }
}

impl<'a, R: Relocation> DynasmApi for Modifier<'a, R> {
    fn offset(&self) -> AssemblyOffset {
        AssemblyOffset(self.asmoffset)
    }

    fn push(&mut self, value: u8) {
        self.buffer[self.asmoffset] = value;
        self.asmoffset += 1
    }

    fn align(&mut self, alignment: usize, with: u8) {
        let mismatch = self.asmoffset % alignment;
        if mismatch != 0 {
            for _ in mismatch .. alignment {
                self.push(with)
            }
        }
    }
}

impl<'a, R: Relocation> DynasmLabelApi for Modifier<'a, R> {
    type Relocation = R;

    fn local_label(&mut self, name: &'static str) {
        let offset = self.offset();
        for loc in self.relocs.take_locals_named(name) {
            let buf_addr = self.buffer.as_ptr() as usize;
            let buf = &mut self.buffer[loc.range(0)];
            if loc.patch(buf, buf_addr, offset.0).is_err()  {
                self.error = Some(DynasmError::ImpossibleRelocation(TargetKind::Forward(name)));
            } else if loc.needs_adjustment() {
                self.new_managed.add(loc);
            }
        }
        self.labels.define_local(name, offset);
    }
    fn global_label( &mut self, name: &'static str) {
        let offset = self.offset();
        if let Err(e) = self.labels.define_global(name, offset) {
            self.error = Some(e);
        }
    }
    fn dynamic_label(&mut self, id: DynamicLabel) {
        let offset = self.offset();
        if let Err(e) = self.labels.define_dynamic(id, offset) {
            self.error = Some(e);
        }
    }
    fn global_relocation(&mut self, name: &'static str, target_offset: isize, field_offset: u8, ref_offset: u8, kind: R) {
        let location = self.offset();
        self.relocs.add_global(name, PatchLoc::new(location, target_offset, field_offset, ref_offset, kind));
    }
    fn dynamic_relocation(&mut self, id: DynamicLabel, target_offset: isize, field_offset: u8, ref_offset: u8, kind: R) {
        let location = self.offset();
        self.relocs.add_dynamic(id, PatchLoc::new(location, target_offset, field_offset, ref_offset, kind));
    }
    fn forward_relocation(&mut self, name: &'static str, target_offset: isize, field_offset: u8, ref_offset: u8, kind: R) {
        let location = self.offset();
        self.relocs.add_local(name, PatchLoc::new(location, target_offset, field_offset, ref_offset, kind));
    }
    fn backward_relocation(&mut self, name: &'static str, target_offset: isize, field_offset: u8, ref_offset: u8, kind: R) {
        let target = match self.labels.resolve_local(name) {
            Ok(target) => target.0,
            Err(e) => {
                self.error = Some(e);
                return;
            }
        };
        let location = self.offset();
        let loc = PatchLoc::new(location, target_offset, field_offset, ref_offset, kind);
            let buf_addr = self.buffer.as_ptr() as usize;
        let buf = &mut self.buffer[loc.range(0)];
        if loc.patch(buf, buf_addr, target).is_err() {
            self.error = Some(DynasmError::ImpossibleRelocation(TargetKind::Backward(name)));
        } else if loc.needs_adjustment() {
            self.new_managed.add(loc)
        }
    }
    fn bare_relocation(&mut self, target: usize, field_offset: u8, ref_offset: u8, kind: R) {
        let location = self.offset();
        let loc = PatchLoc::new(location, 0, field_offset, ref_offset, kind);
            let buf_addr = self.buffer.as_ptr() as usize;
        let buf = &mut self.buffer[loc.range(0)];
        if loc.patch(buf, buf_addr, target).is_err() {
            self.error = Some(DynasmError::ImpossibleRelocation(TargetKind::Extern(target)));
        } else if loc.needs_adjustment() {
            self.new_managed.add(loc)
        }
    }
}


/// This struct is a wrapper around an `Assembler` normally created using the
/// `Assembler.alter_uncommitted` method. It allows the user to edit parts
/// of the assembling buffer that cannot be determined easily or efficiently
/// in advance. Due to limitations of the label resolution algorithms, this
/// assembler does not allow labels to be used.
#[derive(Debug)]
pub struct UncommittedModifier<'a> {
    buffer: &'a mut Vec<u8>,
    base_offset: usize,
    offset: usize
}

impl<'a> UncommittedModifier<'a> {
    /// create a new uncommittedmodifier
    pub fn new(buffer: &mut Vec<u8>, base_offset: AssemblyOffset) -> UncommittedModifier {
        UncommittedModifier {
            buffer,
            base_offset: base_offset.0,
            offset: base_offset.0
        }
    }

    /// Sets the current modification offset to the given value
    pub fn goto(&mut self, offset: AssemblyOffset) {
        self.offset = offset.0;
    }

    /// Checks that the current modification offset is not larger than the specified offset.
    pub fn check(&mut self, offset: AssemblyOffset) -> Result<(), DynasmError> {
        if self.offset > offset.0 {
            Err(DynasmError::CheckFailed)
        } else {
            Ok(())
        }
    }

    /// Checks that the current modification offset is exactly the specified offset.
    pub fn check_exact(&mut self, offset: AssemblyOffset) -> Result<(), DynasmError> {
        if self.offset != offset.0 {
            Err(DynasmError::CheckFailed)
        } else {
            Ok(())
        }
    }
}

impl<'a> DynasmApi for UncommittedModifier<'a> {
    fn offset(&self) -> AssemblyOffset {
        AssemblyOffset(self.offset)
    }

    fn push(&mut self, value: u8) {
        self.buffer[self.offset - self.base_offset] = value;
        self.offset += 1;
    }

    fn align(&mut self, alignment: usize, with: u8) {
        let mismatch = self.offset % alignment;
        if mismatch != 0 {
            for _ in mismatch .. alignment {
                self.push(with)
            }
        }
    }
}

impl<'a> Extend<u8> for UncommittedModifier<'a> {
    fn extend<T>(&mut self, iter: T) where T: IntoIterator<Item=u8> {
        for i in iter {
            self.push(i)
        }
    }
}

impl<'a, 'b> Extend<&'b u8> for UncommittedModifier<'a> {
    fn extend<T>(&mut self, iter: T) where T: IntoIterator<Item=&'b u8> {
        self.extend(iter.into_iter().cloned())
    }
}

/// A trait abstracting over architectural register families. This is usually implemented
/// over an enum of all available registers in each family. This allows for code that is generic
/// over register families.
pub trait Register: Debug + Clone + Copy + PartialEq + Eq + Hash {
    /// Returns the integer ID of the register. Usually equivalent to casting
    /// the enum to an u8, but allows you to be generic over the register family.
    fn code(&self) -> u8;
}
