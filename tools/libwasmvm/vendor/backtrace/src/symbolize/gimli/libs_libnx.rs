use super::{Library, LibrarySegment, Vec};

// DevkitA64 doesn't natively support debug info, but the build system will
// place debug info at the path `romfs:/debug_info.elf`.
pub(super) fn native_libraries() -> Vec<Library> {
    extern "C" {
        static __start__: u8;
    }

    let bias = unsafe { &__start__ } as *const u8 as usize;

    let mut ret = Vec::new();
    let mut segments = Vec::new();
    segments.push(LibrarySegment {
        stated_virtual_memory_address: 0,
        len: usize::max_value() - bias,
    });

    let path = "romfs:/debug_info.elf";
    ret.push(Library {
        name: path.into(),
        segments,
        bias,
    });

    ret
}
