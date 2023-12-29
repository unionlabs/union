// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

//! Memory management for executable code.
use super::unwind::UnwindRegistry;
use crate::GlobalFrameInfoRegistration;
use wasmer_types::{
    compilation::unwind::CompiledFunctionUnwindInfoLike, CompiledFunctionUnwindInfoReference,
    CustomSectionLike, FunctionBodyLike,
};
use wasmer_vm::{Mmap, VMFunctionBody};

/// The optimal alignment for functions.
///
/// On x86-64, this is 16 since it's what the optimizations assume.
/// When we add support for other architectures, we should also figure out their
/// optimal alignment values.
const ARCH_FUNCTION_ALIGNMENT: usize = 16;

/// The optimal alignment for data.
///
const DATA_SECTION_ALIGNMENT: usize = 64;

/// Memory manager for executable code.
pub struct CodeMemory {
    // frame info is placed first, to ensure it's dropped before the mmap
    frame_info_registration: Option<GlobalFrameInfoRegistration>,
    unwind_registry: UnwindRegistry,
    mmap: Mmap,
    start_of_nonexecutable_pages: usize,
}

impl CodeMemory {
    /// Create a new `CodeMemory` instance.
    pub fn new() -> Self {
        Self {
            unwind_registry: UnwindRegistry::new(),
            mmap: Mmap::new(),
            start_of_nonexecutable_pages: 0,
            frame_info_registration: None,
        }
    }

    /// Mutably get the UnwindRegistry.
    pub fn unwind_registry_mut(&mut self) -> &mut UnwindRegistry {
        &mut self.unwind_registry
    }

    /// Allocate a single contiguous block of memory at a fixed virtual address for the functions and custom sections, and copy the data in place.
    #[allow(clippy::type_complexity)]
    pub fn allocate<'module, 'memory, FunctionBody, CustomSection>(
        &'memory mut self,
        functions: &'memory [&'module FunctionBody],
        executable_sections: &'memory [&'module CustomSection],
        data_sections: &'memory [&'module CustomSection],
    ) -> Result<
        (
            Vec<&'memory mut [VMFunctionBody]>,
            Vec<&'memory mut [u8]>,
            Vec<&'memory mut [u8]>,
        ),
        String,
    >
    where
        FunctionBody: FunctionBodyLike<'module> + 'module,
        CustomSection: CustomSectionLike<'module> + 'module,
    {
        let mut function_result = vec![];
        let mut data_section_result = vec![];
        let mut executable_section_result = vec![];

        let page_size = region::page::size();

        // 1. Calculate the total size, that is:
        // - function body size, including all trampolines
        // -- windows unwind info
        // -- padding between functions
        // - executable section body
        // -- padding between executable sections
        // - padding until a new page to change page permissions
        // - data section body size
        // -- padding between data sections

        let total_len = round_up(
            functions.iter().fold(0, |acc, func| {
                round_up(
                    acc + Self::function_allocation_size(*func),
                    ARCH_FUNCTION_ALIGNMENT,
                )
            }) + executable_sections.iter().fold(0, |acc, exec| {
                round_up(acc + exec.bytes().len(), ARCH_FUNCTION_ALIGNMENT)
            }),
            page_size,
        ) + data_sections.iter().fold(0, |acc, data| {
            round_up(acc + data.bytes().len(), DATA_SECTION_ALIGNMENT)
        });

        // 2. Allocate the pages. Mark them all read-write.

        self.mmap = Mmap::with_at_least(total_len)?;

        // 3. Determine where the pointers to each function, executable section
        // or data section are. Copy the functions. Collect the addresses of each and return them.

        let mut bytes = 0;
        let mut buf = self.mmap.as_mut_slice();
        for func in functions {
            let len = round_up(
                Self::function_allocation_size(*func),
                ARCH_FUNCTION_ALIGNMENT,
            );
            let (func_buf, next_buf) = buf.split_at_mut(len);
            buf = next_buf;
            bytes += len;

            let vmfunc = Self::copy_function(&mut self.unwind_registry, *func, func_buf);
            assert_eq!(vmfunc.as_ptr() as usize % ARCH_FUNCTION_ALIGNMENT, 0);
            function_result.push(vmfunc);
        }
        for section in executable_sections {
            let section = section.bytes();
            assert_eq!(buf.as_mut_ptr() as usize % ARCH_FUNCTION_ALIGNMENT, 0);
            let len = round_up(section.len(), ARCH_FUNCTION_ALIGNMENT);
            let (s, next_buf) = buf.split_at_mut(len);
            buf = next_buf;
            bytes += len;
            s[..section.len()].copy_from_slice(section);
            executable_section_result.push(s);
        }

        self.start_of_nonexecutable_pages = bytes;

        if !data_sections.is_empty() {
            // Data sections have different page permissions from the executable
            // code that came before it, so they need to be on different pages.
            let padding = round_up(bytes, page_size) - bytes;
            buf = buf.split_at_mut(padding).1;

            for section in data_sections {
                let section = section.bytes();
                assert_eq!(buf.as_mut_ptr() as usize % DATA_SECTION_ALIGNMENT, 0);
                let len = round_up(section.len(), DATA_SECTION_ALIGNMENT);
                let (s, next_buf) = buf.split_at_mut(len);
                buf = next_buf;
                s[..section.len()].copy_from_slice(section);
                data_section_result.push(s);
            }
        }

        Ok((
            function_result,
            executable_section_result,
            data_section_result,
        ))
    }

    /// Apply the page permissions.
    pub fn publish(&mut self) {
        if self.mmap.is_empty() || self.start_of_nonexecutable_pages == 0 {
            return;
        }
        assert!(self.mmap.len() >= self.start_of_nonexecutable_pages);
        unsafe {
            region::protect(
                self.mmap.as_mut_ptr(),
                self.start_of_nonexecutable_pages,
                region::Protection::READ_EXECUTE,
            )
        }
        .expect("unable to make memory readonly and executable");
    }

    /// Calculates the allocation size of the given compiled function.
    fn function_allocation_size<'a>(func: &'a impl FunctionBodyLike<'a>) -> usize {
        match &func.unwind_info().map(|o| o.get()) {
            Some(CompiledFunctionUnwindInfoReference::WindowsX64(info)) => {
                // Windows unwind information is required to be emitted into code memory
                // This is because it must be a positive relative offset from the start of the memory
                // Account for necessary unwind information alignment padding (32-bit alignment)
                ((func.body().len() + 3) & !3) + info.len()
            }
            _ => func.body().len(),
        }
    }

    /// Copies the data of the compiled function to the given buffer.
    ///
    /// This will also add the function to the current function table.
    fn copy_function<'module, 'memory>(
        registry: &mut UnwindRegistry,
        func: &'module impl FunctionBodyLike<'module>,
        buf: &'memory mut [u8],
    ) -> &'memory mut [VMFunctionBody] {
        assert_eq!(buf.as_ptr() as usize % ARCH_FUNCTION_ALIGNMENT, 0);

        let func_len = func.body().len();

        let (body, remainder) = buf.split_at_mut(func_len);
        body.copy_from_slice(func.body());
        let vmfunc = Self::view_as_mut_vmfunc_slice(body);

        let unwind_info = func.unwind_info().map(|o| o.get());
        if let Some(CompiledFunctionUnwindInfoReference::WindowsX64(info)) = unwind_info {
            // Windows unwind information is written following the function body
            // Keep unwind information 32-bit aligned (round up to the nearest 4 byte boundary)
            let unwind_start = (func_len + 3) & !3;
            let unwind_size = info.len();
            let padding = unwind_start - func_len;
            assert_eq!((func_len + padding) % 4, 0);
            let slice = remainder.split_at_mut(padding + unwind_size).0;
            slice[padding..].copy_from_slice(info);
        }

        if let Some(ref info) = unwind_info {
            registry
                .register(vmfunc.as_ptr() as usize, 0, func_len as u32, info)
                .expect("failed to register unwind information");
        }

        vmfunc
    }

    /// Convert mut a slice from u8 to VMFunctionBody.
    fn view_as_mut_vmfunc_slice(slice: &mut [u8]) -> &mut [VMFunctionBody] {
        let byte_ptr: *mut [u8] = slice;
        let body_ptr = byte_ptr as *mut [VMFunctionBody];
        unsafe { &mut *body_ptr }
    }

    /// Register the frame info, so it's free when the mememory gets freed
    pub fn register_frame_info(&mut self, frame_info: GlobalFrameInfoRegistration) {
        self.frame_info_registration = Some(frame_info);
    }
}

fn round_up(size: usize, multiple: usize) -> usize {
    debug_assert!(multiple.is_power_of_two());
    (size + (multiple - 1)) & !(multiple - 1)
}

#[cfg(test)]
mod tests {
    use super::CodeMemory;
    fn _assert() {
        fn _assert_send_sync<T: Send + Sync>() {}
        _assert_send_sync::<CodeMemory>();
    }
}
