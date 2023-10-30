//! A module to define the FFI definitions we use on Windows for `dbghelp.dll`
//!
//! This module uses a custom macro, `ffi!`, to wrap all definitions to
//! automatically generate tests to assert that our definitions here are the
//! same as `winapi`.
//!
//! This module largely exists to integrate into libstd itself where winapi is
//! not currently available.

#![allow(bad_style, dead_code)]

cfg_if::cfg_if! {
    if #[cfg(feature = "verify-winapi")] {
        pub use self::winapi::c_void;
        pub use self::winapi::HINSTANCE;
        pub use self::winapi::FARPROC;
        pub use self::winapi::LPSECURITY_ATTRIBUTES;
        #[cfg(target_pointer_width = "64")]
        pub use self::winapi::PUNWIND_HISTORY_TABLE;
        #[cfg(target_pointer_width = "64")]
        pub use self::winapi::PRUNTIME_FUNCTION;

        mod winapi {
            pub use winapi::ctypes::*;
            pub use winapi::shared::basetsd::*;
            pub use winapi::shared::minwindef::*;
            pub use winapi::um::dbghelp::*;
            pub use winapi::um::fileapi::*;
            pub use winapi::um::handleapi::*;
            pub use winapi::um::libloaderapi::*;
            pub use winapi::um::memoryapi::*;
            pub use winapi::um::minwinbase::*;
            pub use winapi::um::processthreadsapi::*;
            pub use winapi::um::synchapi::*;
            pub use winapi::um::tlhelp32::*;
            pub use winapi::um::winbase::*;
            pub use winapi::um::winnt::*;
        }
    } else {
        pub use core::ffi::c_void;
        pub type HINSTANCE = *mut c_void;
        pub type FARPROC = *mut c_void;
        pub type LPSECURITY_ATTRIBUTES = *mut c_void;
        #[cfg(target_pointer_width = "64")]
        pub type PRUNTIME_FUNCTION = *mut c_void;
        #[cfg(target_pointer_width = "64")]
        pub type PUNWIND_HISTORY_TABLE = *mut c_void;
    }
}

macro_rules! ffi {
	() => ();

    (#[repr($($r:tt)*)] pub struct $name:ident { $(pub $field:ident: $ty:ty,)* } $($rest:tt)*) => (
        #[repr($($r)*)]
        #[cfg(not(feature = "verify-winapi"))]
        #[derive(Copy, Clone)]
        pub struct $name {
            $(pub $field: $ty,)*
        }

        #[cfg(feature = "verify-winapi")]
        pub use self::winapi::$name;

        #[test]
        #[cfg(feature = "verify-winapi")]
        fn $name() {
            use core::mem;

            #[repr($($r)*)]
            pub struct $name {
                $(pub $field: $ty,)*
            }

            assert_eq!(
                mem::size_of::<$name>(),
                mem::size_of::<winapi::$name>(),
                concat!("size of ", stringify!($name), " is wrong"),
            );
            assert_eq!(
                mem::align_of::<$name>(),
                mem::align_of::<winapi::$name>(),
                concat!("align of ", stringify!($name), " is wrong"),
            );

            type Winapi = winapi::$name;

            fn assert_same<T>(_: T, _: T) {}

            unsafe {
                let a = &*(mem::align_of::<$name>() as *const $name);
                let b = &*(mem::align_of::<Winapi>() as *const Winapi);

                $(
                    ffi!(@test_fields a b $field $ty);
                )*
            }
        }

        ffi!($($rest)*);
    );

    // Handling verification against unions in winapi requires some special care
    (@test_fields $a:ident $b:ident FltSave $ty:ty) => (
        // Skip this field on x86_64 `CONTEXT` since it's a union and a bit funny
    );
    (@test_fields $a:ident $b:ident D $ty:ty) => ({
        let a = &$a.D;
        let b = $b.D();
        assert_same(a, b);
        assert_eq!(a as *const $ty, b as *const $ty, "misplaced field D");
    });
    (@test_fields $a:ident $b:ident s $ty:ty) => ({
        let a = &$a.s;
        let b = $b.s();
        assert_same(a, b);
        assert_eq!(a as *const $ty, b as *const $ty, "misplaced field s");
    });

    // Otherwise test all fields normally.
    (@test_fields $a:ident $b:ident $field:ident $ty:ty) => ({
        let a = &$a.$field;
        let b = &$b.$field;
        assert_same(a, b);
        assert_eq!(a as *const $ty, b as *const $ty,
                   concat!("misplaced field ", stringify!($field)));
    });

    (pub type $name:ident = $ty:ty; $($rest:tt)*) => (
        pub type $name = $ty;

        #[cfg(feature = "verify-winapi")]
        #[allow(dead_code)]
        const $name: () = {
            fn _foo() {
                trait SameType {}
                impl<T> SameType for (T, T) {}
                fn assert_same<T: SameType>() {}

                assert_same::<($name, winapi::$name)>();
            }
        };

        ffi!($($rest)*);
    );

    (pub const $name:ident: $ty:ty = $val:expr; $($rest:tt)*) => (
        pub const $name: $ty = $val;

        #[cfg(feature = "verify-winapi")]
        #[allow(unused_imports)]
        mod $name {
            use super::*;
            #[test]
            fn assert_valid() {
                let x: $ty = winapi::$name;
                assert_eq!(x, $val);
            }
        }


        ffi!($($rest)*);
    );

    ($(#[$meta:meta])* extern "system" { $(pub fn $name:ident($($args:tt)*) -> $ret:ty;)* } $($rest:tt)*) => (
        $(#[$meta])* extern "system" {
            $(pub fn $name($($args)*) -> $ret;)*
        }

        $(
            #[cfg(feature = "verify-winapi")]
            mod $name {
                #[test]
                fn assert_same() {
                    use super::*;

                    assert_eq!($name as usize, winapi::$name as usize);
                    let mut x: unsafe extern "system" fn($($args)*) -> $ret;
                    x = $name;
                    drop(x);
                    x = winapi::$name;
                    drop(x);
                }
            }
        )*

        ffi!($($rest)*);
    );

    (impl $name:ident { $($i:tt)* } $($rest:tt)*) => (
        #[cfg(not(feature = "verify-winapi"))]
        impl $name {
            $($i)*
        }

        ffi!($($rest)*);
    );
}

ffi! {
    #[repr(C)]
    pub struct STACKFRAME64 {
        pub AddrPC: ADDRESS64,
        pub AddrReturn: ADDRESS64,
        pub AddrFrame: ADDRESS64,
        pub AddrStack: ADDRESS64,
        pub AddrBStore: ADDRESS64,
        pub FuncTableEntry: PVOID,
        pub Params: [DWORD64; 4],
        pub Far: BOOL,
        pub Virtual: BOOL,
        pub Reserved: [DWORD64; 3],
        pub KdHelp: KDHELP64,
    }

    pub type LPSTACKFRAME64 = *mut STACKFRAME64;

    #[repr(C)]
    pub struct STACKFRAME_EX {
        pub AddrPC: ADDRESS64,
        pub AddrReturn: ADDRESS64,
        pub AddrFrame: ADDRESS64,
        pub AddrStack: ADDRESS64,
        pub AddrBStore: ADDRESS64,
        pub FuncTableEntry: PVOID,
        pub Params: [DWORD64; 4],
        pub Far: BOOL,
        pub Virtual: BOOL,
        pub Reserved: [DWORD64; 3],
        pub KdHelp: KDHELP64,
        pub StackFrameSize: DWORD,
        pub InlineFrameContext: DWORD,
    }

    pub type LPSTACKFRAME_EX = *mut STACKFRAME_EX;

    #[repr(C)]
    pub struct IMAGEHLP_LINEW64 {
        pub SizeOfStruct: DWORD,
        pub Key: PVOID,
        pub LineNumber: DWORD,
        pub FileName: PWSTR,
        pub Address: DWORD64,
    }

    pub type PIMAGEHLP_LINEW64 = *mut IMAGEHLP_LINEW64;

    #[repr(C)]
    pub struct SYMBOL_INFOW {
        pub SizeOfStruct: ULONG,
        pub TypeIndex: ULONG,
        pub Reserved: [ULONG64; 2],
        pub Index: ULONG,
        pub Size: ULONG,
        pub ModBase: ULONG64,
        pub Flags: ULONG,
        pub Value: ULONG64,
        pub Address: ULONG64,
        pub Register: ULONG,
        pub Scope: ULONG,
        pub Tag: ULONG,
        pub NameLen: ULONG,
        pub MaxNameLen: ULONG,
        pub Name: [WCHAR; 1],
    }

    pub type PSYMBOL_INFOW = *mut SYMBOL_INFOW;

    pub type PTRANSLATE_ADDRESS_ROUTINE64 = Option<
        unsafe extern "system" fn(hProcess: HANDLE, hThread: HANDLE, lpaddr: LPADDRESS64) -> DWORD64,
    >;
    pub type PGET_MODULE_BASE_ROUTINE64 =
        Option<unsafe extern "system" fn(hProcess: HANDLE, Address: DWORD64) -> DWORD64>;
    pub type PFUNCTION_TABLE_ACCESS_ROUTINE64 =
        Option<unsafe extern "system" fn(ahProcess: HANDLE, AddrBase: DWORD64) -> PVOID>;
    pub type PREAD_PROCESS_MEMORY_ROUTINE64 = Option<
        unsafe extern "system" fn(
            hProcess: HANDLE,
            qwBaseAddress: DWORD64,
            lpBuffer: PVOID,
            nSize: DWORD,
            lpNumberOfBytesRead: LPDWORD,
        ) -> BOOL,
    >;

    #[repr(C)]
    pub struct ADDRESS64 {
        pub Offset: DWORD64,
        pub Segment: WORD,
        pub Mode: ADDRESS_MODE,
    }

    pub type LPADDRESS64 = *mut ADDRESS64;

    pub type ADDRESS_MODE = u32;

    #[repr(C)]
    pub struct KDHELP64 {
        pub Thread: DWORD64,
        pub ThCallbackStack: DWORD,
        pub ThCallbackBStore: DWORD,
        pub NextCallback: DWORD,
        pub FramePointer: DWORD,
        pub KiCallUserMode: DWORD64,
        pub KeUserCallbackDispatcher: DWORD64,
        pub SystemRangeStart: DWORD64,
        pub KiUserExceptionDispatcher: DWORD64,
        pub StackBase: DWORD64,
        pub StackLimit: DWORD64,
        pub BuildVersion: DWORD,
        pub Reserved0: DWORD,
        pub Reserved1: [DWORD64; 4],
    }

    #[repr(C)]
    pub struct MODULEENTRY32W {
        pub dwSize: DWORD,
        pub th32ModuleID: DWORD,
        pub th32ProcessID: DWORD,
        pub GlblcntUsage: DWORD,
        pub ProccntUsage: DWORD,
        pub modBaseAddr: *mut u8,
        pub modBaseSize: DWORD,
        pub hModule: HMODULE,
        pub szModule: [WCHAR; MAX_MODULE_NAME32 + 1],
        pub szExePath: [WCHAR; MAX_PATH],
    }

    pub const MAX_SYM_NAME: usize = 2000;
    pub const AddrModeFlat: ADDRESS_MODE = 3;
    pub const TRUE: BOOL = 1;
    pub const FALSE: BOOL = 0;
    pub const PROCESS_QUERY_INFORMATION: DWORD = 0x400;
    pub const IMAGE_FILE_MACHINE_ARM64: u16 = 43620;
    pub const IMAGE_FILE_MACHINE_AMD64: u16 = 34404;
    pub const IMAGE_FILE_MACHINE_I386: u16 = 332;
    pub const IMAGE_FILE_MACHINE_ARMNT: u16 = 452;
    pub const FILE_SHARE_READ: DWORD = 0x1;
    pub const FILE_SHARE_WRITE: DWORD = 0x2;
    pub const OPEN_EXISTING: DWORD = 0x3;
    pub const GENERIC_READ: DWORD = 0x80000000;
    pub const INFINITE: DWORD = !0;
    pub const PAGE_READONLY: DWORD = 2;
    pub const FILE_MAP_READ: DWORD = 4;
    pub const TH32CS_SNAPMODULE: DWORD = 0x00000008;
    pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
    pub const MAX_MODULE_NAME32: usize = 255;
    pub const MAX_PATH: usize = 260;

    pub type DWORD = u32;
    pub type PDWORD = *mut u32;
    pub type BOOL = i32;
    pub type DWORD64 = u64;
    pub type PDWORD64 = *mut u64;
    pub type HANDLE = *mut c_void;
    pub type PVOID = HANDLE;
    pub type PCWSTR = *const u16;
    pub type LPSTR = *mut i8;
    pub type LPCSTR = *const i8;
    pub type PWSTR = *mut u16;
    pub type WORD = u16;
    pub type ULONG = u32;
    pub type ULONG64 = u64;
    pub type WCHAR = u16;
    pub type PCONTEXT = *mut CONTEXT;
    pub type LPDWORD = *mut DWORD;
    pub type DWORDLONG = u64;
    pub type HMODULE = HINSTANCE;
    pub type SIZE_T = usize;
    pub type LPVOID = *mut c_void;
    pub type LPCVOID = *const c_void;
    pub type LPMODULEENTRY32W = *mut MODULEENTRY32W;

    #[link(name = "kernel32")]
    extern "system" {
        pub fn GetCurrentProcess() -> HANDLE;
        pub fn GetCurrentThread() -> HANDLE;
        pub fn RtlCaptureContext(ContextRecord: PCONTEXT) -> ();
        pub fn LoadLibraryA(a: *const i8) -> HMODULE;
        pub fn GetProcAddress(h: HMODULE, name: *const i8) -> FARPROC;
        pub fn GetModuleHandleA(name: *const i8) -> HMODULE;
        pub fn OpenProcess(
            dwDesiredAccess: DWORD,
            bInheitHandle: BOOL,
            dwProcessId: DWORD,
        ) -> HANDLE;
        pub fn GetCurrentProcessId() -> DWORD;
        pub fn CloseHandle(h: HANDLE) -> BOOL;
        pub fn CreateFileA(
            lpFileName: LPCSTR,
            dwDesiredAccess: DWORD,
            dwShareMode: DWORD,
            lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
            dwCreationDisposition: DWORD,
            dwFlagsAndAttributes: DWORD,
            hTemplateFile: HANDLE,
        ) -> HANDLE;
        pub fn CreateMutexA(
            attrs: LPSECURITY_ATTRIBUTES,
            initial: BOOL,
            name: LPCSTR,
        ) -> HANDLE;
        pub fn ReleaseMutex(hMutex: HANDLE) -> BOOL;
        pub fn WaitForSingleObjectEx(
            hHandle: HANDLE,
            dwMilliseconds: DWORD,
            bAlertable: BOOL,
        ) -> DWORD;
        pub fn CreateFileMappingA(
            hFile: HANDLE,
            lpFileMappingAttributes: LPSECURITY_ATTRIBUTES,
            flProtect: DWORD,
            dwMaximumSizeHigh: DWORD,
            dwMaximumSizeLow: DWORD,
            lpName: LPCSTR,
        ) -> HANDLE;
        pub fn MapViewOfFile(
            hFileMappingObject: HANDLE,
            dwDesiredAccess: DWORD,
            dwFileOffsetHigh: DWORD,
            dwFileOffsetLow: DWORD,
            dwNumberOfBytesToMap: SIZE_T,
        ) -> LPVOID;
        pub fn UnmapViewOfFile(lpBaseAddress: LPCVOID) -> BOOL;
        pub fn CreateToolhelp32Snapshot(
            dwFlags: DWORD,
            th32ProcessID: DWORD,
        ) -> HANDLE;
        pub fn Module32FirstW(
            hSnapshot: HANDLE,
            lpme: LPMODULEENTRY32W,
        ) -> BOOL;
        pub fn Module32NextW(
            hSnapshot: HANDLE,
            lpme: LPMODULEENTRY32W,
        ) -> BOOL;
    }
}

#[cfg(target_pointer_width = "64")]
ffi! {
    #[link(name = "kernel32")]
    extern "system" {
        pub fn RtlLookupFunctionEntry(
            ControlPc: DWORD64,
            ImageBase: PDWORD64,
            HistoryTable: PUNWIND_HISTORY_TABLE,
        ) -> PRUNTIME_FUNCTION;
    }
}

#[cfg(target_arch = "aarch64")]
ffi! {
    #[repr(C, align(16))]
    pub struct CONTEXT {
        pub ContextFlags: DWORD,
        pub Cpsr: DWORD,
        pub u: CONTEXT_u,
        pub Sp: u64,
        pub Pc: u64,
        pub V: [ARM64_NT_NEON128; 32],
        pub Fpcr: DWORD,
        pub Fpsr: DWORD,
        pub Bcr: [DWORD; ARM64_MAX_BREAKPOINTS],
        pub Bvr: [DWORD64; ARM64_MAX_BREAKPOINTS],
        pub Wcr: [DWORD; ARM64_MAX_WATCHPOINTS],
        pub Wvr: [DWORD64; ARM64_MAX_WATCHPOINTS],
    }

    #[repr(C)]
    pub struct CONTEXT_u {
        pub s: CONTEXT_u_s,
    }

    impl CONTEXT_u {
        pub unsafe fn s(&self) -> &CONTEXT_u_s {
            &self.s
        }
    }

    #[repr(C)]
    pub struct CONTEXT_u_s {
        pub X0: u64,
        pub X1: u64,
        pub X2: u64,
        pub X3: u64,
        pub X4: u64,
        pub X5: u64,
        pub X6: u64,
        pub X7: u64,
        pub X8: u64,
        pub X9: u64,
        pub X10: u64,
        pub X11: u64,
        pub X12: u64,
        pub X13: u64,
        pub X14: u64,
        pub X15: u64,
        pub X16: u64,
        pub X17: u64,
        pub X18: u64,
        pub X19: u64,
        pub X20: u64,
        pub X21: u64,
        pub X22: u64,
        pub X23: u64,
        pub X24: u64,
        pub X25: u64,
        pub X26: u64,
        pub X27: u64,
        pub X28: u64,
        pub Fp: u64,
        pub Lr: u64,
    }

    pub const ARM64_MAX_BREAKPOINTS: usize = 8;
    pub const ARM64_MAX_WATCHPOINTS: usize = 2;

    #[repr(C)]
    pub struct ARM64_NT_NEON128 {
        pub D: [f64; 2],
    }
}

#[cfg(target_arch = "x86")]
ffi! {
    #[repr(C)]
    pub struct CONTEXT {
        pub ContextFlags: DWORD,
        pub Dr0: DWORD,
        pub Dr1: DWORD,
        pub Dr2: DWORD,
        pub Dr3: DWORD,
        pub Dr6: DWORD,
        pub Dr7: DWORD,
        pub FloatSave: FLOATING_SAVE_AREA,
        pub SegGs: DWORD,
        pub SegFs: DWORD,
        pub SegEs: DWORD,
        pub SegDs: DWORD,
        pub Edi: DWORD,
        pub Esi: DWORD,
        pub Ebx: DWORD,
        pub Edx: DWORD,
        pub Ecx: DWORD,
        pub Eax: DWORD,
        pub Ebp: DWORD,
        pub Eip: DWORD,
        pub SegCs: DWORD,
        pub EFlags: DWORD,
        pub Esp: DWORD,
        pub SegSs: DWORD,
        pub ExtendedRegisters: [u8; 512],
    }

    #[repr(C)]
    pub struct FLOATING_SAVE_AREA {
        pub ControlWord: DWORD,
        pub StatusWord: DWORD,
        pub TagWord: DWORD,
        pub ErrorOffset: DWORD,
        pub ErrorSelector: DWORD,
        pub DataOffset: DWORD,
        pub DataSelector: DWORD,
        pub RegisterArea: [u8; 80],
        pub Spare0: DWORD,
    }
}

#[cfg(target_arch = "x86_64")]
ffi! {
    #[repr(C, align(8))]
    pub struct CONTEXT {
        pub P1Home: DWORDLONG,
        pub P2Home: DWORDLONG,
        pub P3Home: DWORDLONG,
        pub P4Home: DWORDLONG,
        pub P5Home: DWORDLONG,
        pub P6Home: DWORDLONG,

        pub ContextFlags: DWORD,
        pub MxCsr: DWORD,

        pub SegCs: WORD,
        pub SegDs: WORD,
        pub SegEs: WORD,
        pub SegFs: WORD,
        pub SegGs: WORD,
        pub SegSs: WORD,
        pub EFlags: DWORD,

        pub Dr0: DWORDLONG,
        pub Dr1: DWORDLONG,
        pub Dr2: DWORDLONG,
        pub Dr3: DWORDLONG,
        pub Dr6: DWORDLONG,
        pub Dr7: DWORDLONG,

        pub Rax: DWORDLONG,
        pub Rcx: DWORDLONG,
        pub Rdx: DWORDLONG,
        pub Rbx: DWORDLONG,
        pub Rsp: DWORDLONG,
        pub Rbp: DWORDLONG,
        pub Rsi: DWORDLONG,
        pub Rdi: DWORDLONG,
        pub R8:  DWORDLONG,
        pub R9:  DWORDLONG,
        pub R10: DWORDLONG,
        pub R11: DWORDLONG,
        pub R12: DWORDLONG,
        pub R13: DWORDLONG,
        pub R14: DWORDLONG,
        pub R15: DWORDLONG,

        pub Rip: DWORDLONG,

        pub FltSave: FLOATING_SAVE_AREA,

        pub VectorRegister: [M128A; 26],
        pub VectorControl: DWORDLONG,

        pub DebugControl: DWORDLONG,
        pub LastBranchToRip: DWORDLONG,
        pub LastBranchFromRip: DWORDLONG,
        pub LastExceptionToRip: DWORDLONG,
        pub LastExceptionFromRip: DWORDLONG,
    }

    #[repr(C)]
    pub struct M128A {
        pub Low: u64,
        pub High: i64,
    }
}

#[repr(C)]
#[cfg(target_arch = "x86_64")]
#[derive(Copy, Clone)]
pub struct FLOATING_SAVE_AREA {
    _Dummy: [u8; 512],
}

#[cfg(target_arch = "arm")]
ffi! {
    // #[repr(C)]
    // pub struct NEON128 {
    //     pub Low: ULONG64,
    //     pub High: LONG64,
    // }

    // pub type PNEON128 = *mut NEON128;

    #[repr(C)]
    pub struct CONTEXT_u {
        // pub Q: [NEON128; 16],
        pub D: [ULONG64; 32],
        // pub S: [DWORD; 32],
    }

    pub const ARM_MAX_BREAKPOINTS: usize = 8;
    pub const ARM_MAX_WATCHPOINTS: usize = 1;

    #[repr(C)]
    pub struct CONTEXT {
        pub ContextFlags: DWORD,
        pub R0: DWORD,
        pub R1: DWORD,
        pub R2: DWORD,
        pub R3: DWORD,
        pub R4: DWORD,
        pub R5: DWORD,
        pub R6: DWORD,
        pub R7: DWORD,
        pub R8: DWORD,
        pub R9: DWORD,
        pub R10: DWORD,
        pub R11: DWORD,
        pub R12: DWORD,
        pub Sp: DWORD,
        pub Lr: DWORD,
        pub Pc: DWORD,
        pub Cpsr: DWORD,
        pub Fpsrc: DWORD,
        pub Padding: DWORD,
        pub u: CONTEXT_u,
        pub Bvr: [DWORD; ARM_MAX_BREAKPOINTS],
        pub Bcr: [DWORD; ARM_MAX_BREAKPOINTS],
        pub Wvr: [DWORD; ARM_MAX_WATCHPOINTS],
        pub Wcr: [DWORD; ARM_MAX_WATCHPOINTS],
        pub Padding2: [DWORD; 2],
    }
} // IFDEF(arm)
