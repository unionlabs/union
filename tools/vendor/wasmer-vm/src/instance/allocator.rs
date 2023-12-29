use super::{Instance, VMInstance};
use crate::vmcontext::VMTableDefinition;
use crate::VMMemoryDefinition;
use std::alloc::{self, Layout};
use std::convert::TryFrom;
use std::mem;
use std::ptr::{self, NonNull};
use wasmer_types::entity::EntityRef;
use wasmer_types::VMOffsets;
use wasmer_types::{LocalMemoryIndex, LocalTableIndex, ModuleInfo};

/// This is an intermediate type that manages the raw allocation and
/// metadata when creating an [`Instance`].
///
/// This type will free the allocated memory if it's dropped before
/// being used.
///
/// It is important to remind that [`Instance`] is dynamically-sized
/// based on `VMOffsets`: The `Instance.vmctx` field represents a
/// dynamically-sized array that extends beyond the nominal end of the
/// type. So in order to create an instance of it, we must:
///
/// 1. Define the correct layout for `Instance` (size and alignment),
/// 2. Allocate it properly.
///
/// The [`InstanceAllocator::instance_layout`] computes the correct
/// layout to represent the wanted [`Instance`].
///
/// Then we use this layout to allocate an empty `Instance` properly.
pub struct InstanceAllocator {
    /// The buffer that will contain the [`Instance`] and dynamic fields.
    instance_ptr: NonNull<Instance>,

    /// The layout of the `instance_ptr` buffer.
    instance_layout: Layout,

    /// Information about the offsets into the `instance_ptr` buffer for
    /// the dynamic fields.
    offsets: VMOffsets,

    /// Whether or not this type has transferred ownership of the
    /// `instance_ptr` buffer. If it has not when being dropped,
    /// the buffer should be freed.
    consumed: bool,
}

impl Drop for InstanceAllocator {
    fn drop(&mut self) {
        if !self.consumed {
            // If `consumed` has not been set, then we still have ownership
            // over the buffer and must free it.
            let instance_ptr = self.instance_ptr.as_ptr();

            unsafe {
                std::alloc::dealloc(instance_ptr as *mut u8, self.instance_layout);
            }
        }
    }
}

impl InstanceAllocator {
    /// Allocates instance data for use with [`VMInstance::new`].
    ///
    /// Returns a wrapper type around the allocation and 2 vectors of
    /// pointers into the allocated buffer. These lists of pointers
    /// correspond to the location in memory for the local memories and
    /// tables respectively. These pointers should be written to before
    /// calling [`VMInstance::new`].
    ///
    /// [`VMInstance::new`]: super::VMInstance::new
    pub fn new(
        module: &ModuleInfo,
    ) -> (
        Self,
        Vec<NonNull<VMMemoryDefinition>>,
        Vec<NonNull<VMTableDefinition>>,
    ) {
        let offsets = VMOffsets::new(mem::size_of::<usize>() as u8, module);
        let instance_layout = Self::instance_layout(&offsets);

        #[allow(clippy::cast_ptr_alignment)]
        let instance_ptr = unsafe { alloc::alloc(instance_layout) as *mut Instance };

        let instance_ptr = if let Some(ptr) = NonNull::new(instance_ptr) {
            ptr
        } else {
            alloc::handle_alloc_error(instance_layout);
        };

        let allocator = Self {
            instance_ptr,
            instance_layout,
            offsets,
            consumed: false,
        };

        // # Safety
        // Both of these calls are safe because we allocate the pointer
        // above with the same `offsets` that these functions use.
        // Thus there will be enough valid memory for both of them.
        let memories = unsafe { allocator.memory_definition_locations() };
        let tables = unsafe { allocator.table_definition_locations() };

        (allocator, memories, tables)
    }

    /// Calculate the appropriate layout for the [`Instance`].
    fn instance_layout(offsets: &VMOffsets) -> Layout {
        let vmctx_size = usize::try_from(offsets.size_of_vmctx())
            .expect("Failed to convert the size of `vmctx` to a `usize`");

        let instance_vmctx_layout =
            Layout::array::<u8>(vmctx_size).expect("Failed to create a layout for `VMContext`");

        let (instance_layout, _offset) = Layout::new::<Instance>()
            .extend(instance_vmctx_layout)
            .expect("Failed to extend to `Instance` layout to include `VMContext`");

        instance_layout.pad_to_align()
    }

    /// Get the locations of where the local [`VMMemoryDefinition`]s should be stored.
    ///
    /// This function lets us create `Memory` objects on the host with backing
    /// memory in the VM.
    ///
    /// # Safety
    ///
    /// - `Self.instance_ptr` must point to enough memory that all of
    ///   the offsets in `Self.offsets` point to valid locations in
    ///   memory, i.e. `Self.instance_ptr` must have been allocated by
    ///   `Self::new`.
    unsafe fn memory_definition_locations(&self) -> Vec<NonNull<VMMemoryDefinition>> {
        let num_memories = self.offsets.num_local_memories();
        let num_memories = usize::try_from(num_memories).unwrap();
        let mut out = Vec::with_capacity(num_memories);

        // We need to do some pointer arithmetic now. The unit is `u8`.
        let ptr = self.instance_ptr.cast::<u8>().as_ptr();
        let base_ptr = ptr.add(mem::size_of::<Instance>());

        for i in 0..num_memories {
            let mem_offset = self
                .offsets
                .vmctx_vmmemory_definition(LocalMemoryIndex::new(i));
            let mem_offset = usize::try_from(mem_offset).unwrap();

            let new_ptr = NonNull::new_unchecked(base_ptr.add(mem_offset));

            out.push(new_ptr.cast());
        }

        out
    }

    /// Get the locations of where the [`VMTableDefinition`]s should be stored.
    ///
    /// This function lets us create [`Table`] objects on the host with backing
    /// memory in the VM.
    ///
    /// # Safety
    ///
    /// - `Self.instance_ptr` must point to enough memory that all of
    ///   the offsets in `Self.offsets` point to valid locations in
    ///   memory, i.e. `Self.instance_ptr` must have been allocated by
    ///   `Self::new`.
    unsafe fn table_definition_locations(&self) -> Vec<NonNull<VMTableDefinition>> {
        let num_tables = self.offsets.num_local_tables();
        let num_tables = usize::try_from(num_tables).unwrap();
        let mut out = Vec::with_capacity(num_tables);

        // We need to do some pointer arithmetic now. The unit is `u8`.
        let ptr = self.instance_ptr.cast::<u8>().as_ptr();
        let base_ptr = ptr.add(std::mem::size_of::<Instance>());

        for i in 0..num_tables {
            let table_offset = self
                .offsets
                .vmctx_vmtable_definition(LocalTableIndex::new(i));
            let table_offset = usize::try_from(table_offset).unwrap();

            let new_ptr = NonNull::new_unchecked(base_ptr.add(table_offset));

            out.push(new_ptr.cast());
        }
        out
    }

    /// Finish preparing by writing the [`Instance`] into memory, and
    /// consume this `InstanceAllocator`.
    pub(crate) fn into_vminstance(mut self, instance: Instance) -> VMInstance {
        // Prevent the old state's drop logic from being called as we
        // transition into the new state.
        self.consumed = true;

        unsafe {
            // `instance` is moved at `Self.instance_ptr`. This
            // pointer has been allocated by `Self::allocate_instance`
            // (so by `VMInstance::allocate_instance`).
            ptr::write(self.instance_ptr.as_ptr(), instance);
            // Now `instance_ptr` is correctly initialized!
        }
        let instance = self.instance_ptr;
        let instance_layout = self.instance_layout;

        // This is correct because of the invariants of `Self` and
        // because we write `Instance` to the pointer in this function.
        VMInstance {
            instance,
            instance_layout,
        }
    }

    /// Get the [`VMOffsets`] for the allocated buffer.
    pub(crate) fn offsets(&self) -> &VMOffsets {
        &self.offsets
    }
}
