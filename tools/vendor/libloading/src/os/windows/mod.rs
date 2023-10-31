// A hack for docs.rs to build documentation that has both windows and linux documentation in the
// same rustdoc build visible.
#[cfg(all(libloading_docs, not(windows)))]
mod windows_imports {
    pub(super) enum WORD {}
    pub(super) struct DWORD;
    pub(super) enum HMODULE {}
    pub(super) enum FARPROC {}

    pub(super) mod consts {
        use super::DWORD;
        pub(crate) const LOAD_IGNORE_CODE_AUTHZ_LEVEL: DWORD = DWORD;
        pub(crate) const LOAD_LIBRARY_AS_DATAFILE: DWORD = DWORD;
        pub(crate) const LOAD_LIBRARY_AS_DATAFILE_EXCLUSIVE: DWORD = DWORD;
        pub(crate) const LOAD_LIBRARY_AS_IMAGE_RESOURCE: DWORD = DWORD;
        pub(crate) const LOAD_LIBRARY_SEARCH_APPLICATION_DIR: DWORD = DWORD;
        pub(crate) const LOAD_LIBRARY_SEARCH_DEFAULT_DIRS: DWORD = DWORD;
        pub(crate) const LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR: DWORD = DWORD;
        pub(crate) const LOAD_LIBRARY_SEARCH_SYSTEM32: DWORD = DWORD;
        pub(crate) const LOAD_LIBRARY_SEARCH_USER_DIRS: DWORD = DWORD;
        pub(crate) const LOAD_WITH_ALTERED_SEARCH_PATH: DWORD = DWORD;
        pub(crate) const LOAD_LIBRARY_REQUIRE_SIGNED_TARGET: DWORD = DWORD;
        pub(crate) const LOAD_LIBRARY_SAFE_CURRENT_DIRS: DWORD = DWORD;
    }
}
#[cfg(any(not(libloading_docs), windows))]
mod windows_imports {
    extern crate winapi;
    pub(super) use self::winapi::shared::minwindef::{WORD, DWORD, HMODULE, FARPROC};
    pub(super) use self::winapi::shared::ntdef::WCHAR;
    pub(super) use self::winapi::um::{errhandlingapi, libloaderapi};
    pub(super) use std::os::windows::ffi::{OsStrExt, OsStringExt};
    pub(super) const SEM_FAILCE: DWORD = 1;

    pub(super) mod consts {
        pub(crate) use super::winapi::um::libloaderapi::{
            LOAD_IGNORE_CODE_AUTHZ_LEVEL,
            LOAD_LIBRARY_AS_DATAFILE,
            LOAD_LIBRARY_AS_DATAFILE_EXCLUSIVE,
            LOAD_LIBRARY_AS_IMAGE_RESOURCE,
            LOAD_LIBRARY_SEARCH_APPLICATION_DIR,
            LOAD_LIBRARY_SEARCH_DEFAULT_DIRS,
            LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR,
            LOAD_LIBRARY_SEARCH_SYSTEM32,
            LOAD_LIBRARY_SEARCH_USER_DIRS,
            LOAD_WITH_ALTERED_SEARCH_PATH,
            LOAD_LIBRARY_REQUIRE_SIGNED_TARGET,
            LOAD_LIBRARY_SAFE_CURRENT_DIRS,
        };
    }
}

use self::windows_imports::*;
use util::{ensure_compatible_types, cstr_cow_from_bytes};
use std::ffi::{OsStr, OsString};
use std::{fmt, io, marker, mem, ptr};

/// The platform-specific counterpart of the cross-platform [`Library`](crate::Library).
pub struct Library(HMODULE);

unsafe impl Send for Library {}
// Now, this is sort-of-tricky. MSDN documentation does not really make any claims as to safety of
// the Win32 APIs. Sadly, whomever I asked, even current and former Microsoft employees, couldn’t
// say for sure whether the Win32 APIs used to implement `Library` are thread-safe or not.
//
// My investigation ended up with a question about thread-safety properties of the API involved
// being sent to an internal (to MS) general question mailing-list. The conclusion of the mail is
// as such:
//
// * Nobody inside MS (at least out of all of the people who have seen the question) knows for
//   sure either;
// * However, the general consensus between MS developers is that one can rely on the API being
//   thread-safe. In case it is not thread-safe it should be considered a bug on the Windows
//   part. (NB: bugs filed at https://connect.microsoft.com/ against Windows Server)
unsafe impl Sync for Library {}

impl Library {
    /// Find and load a module.
    ///
    /// If the `filename` specifies a full path, the function only searches that path for the
    /// module. Otherwise, if the `filename` specifies a relative path or a module name without a
    /// path, the function uses a Windows-specific search strategy to find the module. For more
    /// information, see the [Remarks on MSDN][msdn].
    ///
    /// If the `filename` specifies a library filename without a path and with the extension omitted,
    /// the `.dll` extension is implicitly added. This behaviour may be suppressed by appending a
    /// trailing `.` to the `filename`.
    ///
    /// This is equivalent to <code>[Library::load_with_flags](filename, 0)</code>.
    ///
    /// [msdn]: https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryw#remarks
    ///
    /// # Safety
    ///
    /// When a library is loaded, initialisation routines contained within the library are executed.
    /// For the purposes of safety, the execution of these routines is conceptually the same calling an
    /// unknown foreign function and may impose arbitrary requirements on the caller for the call
    /// to be sound.
    ///
    /// Additionally, the callers of this function must also ensure that execution of the
    /// termination routines contained within the library is safe as well. These routines may be
    /// executed when the library is unloaded.
    #[inline]
    pub unsafe fn new<P: AsRef<OsStr>>(filename: P) -> Result<Library, crate::Error> {
        Library::load_with_flags(filename, 0)
    }

    /// Get the `Library` representing the original program executable.
    ///
    /// Note that the behaviour of the `Library` loaded with this method is different from
    /// Libraries loaded with [`os::unix::Library::this`]. For more information refer to [MSDN].
    ///
    /// Corresponds to `GetModuleHandleExW(0, NULL, _)`.
    ///
    /// [`os::unix::Library::this`]: crate::os::unix::Library::this
    /// [MSDN]: https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getmodulehandleexw
    pub fn this() -> Result<Library, crate::Error> {
        unsafe {
            let mut handle: HMODULE = std::ptr::null_mut();
            with_get_last_error(|source| crate::Error::GetModuleHandleExW { source }, || {
                let result = libloaderapi::GetModuleHandleExW(0, std::ptr::null_mut(), &mut handle);
                if result == 0 {
                    None
                } else {
                    Some(Library(handle))
                }
            }).map_err(|e| e.unwrap_or(crate::Error::GetModuleHandleExWUnknown))
        }
    }

    /// Get a module that is already loaded by the program.
    ///
    /// This function returns a `Library` corresponding to a module with the given name that is
    /// already mapped into the address space of the process. If the module isn't found, an error is
    /// returned.
    ///
    /// If the `filename` does not include a full path and there are multiple different loaded
    /// modules corresponding to the `filename`, it is impossible to predict which module handle
    /// will be returned. For more information refer to [MSDN].
    ///
    /// If the `filename` specifies a library filename without a path and with the extension omitted,
    /// the `.dll` extension is implicitly added. This behaviour may be suppressed by appending a
    /// trailing `.` to the `filename`.
    ///
    /// This is equivalent to `GetModuleHandleExW(0, filename, _)`.
    ///
    /// [MSDN]: https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getmodulehandleexw
    pub fn open_already_loaded<P: AsRef<OsStr>>(filename: P) -> Result<Library, crate::Error> {
        let wide_filename: Vec<u16> = filename.as_ref().encode_wide().chain(Some(0)).collect();

        let ret = unsafe {
            let mut handle: HMODULE = std::ptr::null_mut();
            with_get_last_error(|source| crate::Error::GetModuleHandleExW { source }, || {
                // Make sure no winapi calls as a result of drop happen inside this closure, because
                // otherwise that might change the return value of the GetLastError.
                let result = libloaderapi::GetModuleHandleExW(0, wide_filename.as_ptr(), &mut handle);
                if result == 0 {
                    None
                } else {
                    Some(Library(handle))
                }
            }).map_err(|e| e.unwrap_or(crate::Error::GetModuleHandleExWUnknown))
        };

        drop(wide_filename); // Drop wide_filename here to ensure it doesn’t get moved and dropped
                             // inside the closure by mistake. See comment inside the closure.
        ret
    }

    /// Find and load a module, additionally adjusting behaviour with flags.
    ///
    /// See [`Library::new`] for documentation on the handling of the `filename` argument. See the
    /// [flag table on MSDN][flags] for information on applicable values for the `flags` argument.
    ///
    /// Corresponds to `LoadLibraryExW(filename, reserved: NULL, flags)`.
    ///
    /// [flags]: https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw#parameters
    ///
    /// # Safety
    ///
    /// When a library is loaded, initialisation routines contained within the library are executed.
    /// For the purposes of safety, the execution of these routines is conceptually the same calling an
    /// unknown foreign function and may impose arbitrary requirements on the caller for the call
    /// to be sound.
    ///
    /// Additionally, the callers of this function must also ensure that execution of the
    /// termination routines contained within the library is safe as well. These routines may be
    /// executed when the library is unloaded.
    pub unsafe fn load_with_flags<P: AsRef<OsStr>>(filename: P, flags: DWORD) -> Result<Library, crate::Error> {
        let wide_filename: Vec<u16> = filename.as_ref().encode_wide().chain(Some(0)).collect();
        let _guard = ErrorModeGuard::new();

        let ret = with_get_last_error(|source| crate::Error::LoadLibraryExW { source }, || {
            // Make sure no winapi calls as a result of drop happen inside this closure, because
            // otherwise that might change the return value of the GetLastError.
            let handle =
                libloaderapi::LoadLibraryExW(wide_filename.as_ptr(), std::ptr::null_mut(), flags);
            if handle.is_null()  {
                None
            } else {
                Some(Library(handle))
            }
        }).map_err(|e| e.unwrap_or(crate::Error::LoadLibraryExWUnknown));
        drop(wide_filename); // Drop wide_filename here to ensure it doesn’t get moved and dropped
                             // inside the closure by mistake. See comment inside the closure.
        ret
    }

    /// Get a pointer to a function or static variable by symbol name.
    ///
    /// The `symbol` may not contain any null bytes, with the exception of the last byte. A null
    /// terminated `symbol` may avoid a string allocation in some cases.
    ///
    /// Symbol is interpreted as-is; no mangling is done. This means that symbols like `x::y` are
    /// most likely invalid.
    ///
    /// # Safety
    ///
    /// Users of this API must specify the correct type of the function or variable loaded.
    pub unsafe fn get<T>(&self, symbol: &[u8]) -> Result<Symbol<T>, crate::Error> {
        ensure_compatible_types::<T, FARPROC>()?;
        let symbol = cstr_cow_from_bytes(symbol)?;
        with_get_last_error(|source| crate::Error::GetProcAddress { source }, || {
            let symbol = libloaderapi::GetProcAddress(self.0, symbol.as_ptr());
            if symbol.is_null() {
                None
            } else {
                Some(Symbol {
                    pointer: symbol,
                    pd: marker::PhantomData
                })
            }
        }).map_err(|e| e.unwrap_or(crate::Error::GetProcAddressUnknown))
    }

    /// Get a pointer to a function or static variable by ordinal number.
    ///
    /// # Safety
    ///
    /// Users of this API must specify the correct type of the function or variable loaded.
    pub unsafe fn get_ordinal<T>(&self, ordinal: WORD) -> Result<Symbol<T>, crate::Error> {
        ensure_compatible_types::<T, FARPROC>()?;
        with_get_last_error(|source| crate::Error::GetProcAddress { source }, || {
            let ordinal = ordinal as usize as *mut _;
            let symbol = libloaderapi::GetProcAddress(self.0, ordinal);
            if symbol.is_null() {
                None
            } else {
                Some(Symbol {
                    pointer: symbol,
                    pd: marker::PhantomData
                })
            }
        }).map_err(|e| e.unwrap_or(crate::Error::GetProcAddressUnknown))
    }

    /// Convert the `Library` to a raw handle.
    pub fn into_raw(self) -> HMODULE {
        let handle = self.0;
        mem::forget(self);
        handle
    }

    /// Convert a raw handle to a `Library`.
    ///
    /// # Safety
    ///
    /// The handle must be the result of a successful call of `LoadLibraryA`, `LoadLibraryW`,
    /// `LoadLibraryExW`, or `LoadLibraryExA`, or a handle previously returned by the
    /// `Library::into_raw` call.
    pub unsafe fn from_raw(handle: HMODULE) -> Library {
        Library(handle)
    }

    /// Unload the library.
    ///
    /// You only need to call this if you are interested in handling any errors that may arise when
    /// library is unloaded. Otherwise this will be done when `Library` is dropped.
    ///
    /// The underlying data structures may still get leaked if an error does occur.
    pub fn close(self) -> Result<(), crate::Error> {
        let result = with_get_last_error(|source| crate::Error::FreeLibrary { source }, || {
            if unsafe { libloaderapi::FreeLibrary(self.0) == 0 } {
                None
            } else {
                Some(())
            }
        }).map_err(|e| e.unwrap_or(crate::Error::FreeLibraryUnknown));
        // While the library is not free'd yet in case of an error, there is no reason to try
        // dropping it again, because all that will do is try calling `FreeLibrary` again. only
        // this time it would ignore the return result, which we already seen failing...
        std::mem::forget(self);
        result
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        unsafe { libloaderapi::FreeLibrary(self.0); }
    }
}

impl fmt::Debug for Library {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            // FIXME: use Maybeuninit::uninit_array when stable
            let mut buf =
                mem::MaybeUninit::<[mem::MaybeUninit::<WCHAR>; 1024]>::uninit().assume_init();
            let len = libloaderapi::GetModuleFileNameW(self.0,
                buf[..].as_mut_ptr().cast(), 1024) as usize;
            if len == 0 {
                f.write_str(&format!("Library@{:p}", self.0))
            } else {
                let string: OsString = OsString::from_wide(
                    // FIXME: use Maybeuninit::slice_get_ref when stable
                    &*(&buf[..len] as *const [_] as *const [WCHAR])
                );
                f.write_str(&format!("Library@{:p} from {:?}", self.0, string))
            }
        }
    }
}

/// A symbol from a library.
///
/// A major difference compared to the cross-platform `Symbol` is that this does not ensure that the
/// `Symbol` does not outlive the `Library` that it comes from.
pub struct Symbol<T> {
    pointer: FARPROC,
    pd: marker::PhantomData<T>
}

impl<T> Symbol<T> {
    /// Convert the loaded `Symbol` into a handle.
    pub fn into_raw(self) -> FARPROC {
        self.pointer
    }
}

impl<T> Symbol<Option<T>> {
    /// Lift Option out of the symbol.
    pub fn lift_option(self) -> Option<Symbol<T>> {
        if self.pointer.is_null() {
            None
        } else {
            Some(Symbol {
                pointer: self.pointer,
                pd: marker::PhantomData,
            })
        }
    }
}

unsafe impl<T: Send> Send for Symbol<T> {}
unsafe impl<T: Sync> Sync for Symbol<T> {}

impl<T> Clone for Symbol<T> {
    fn clone(&self) -> Symbol<T> {
        Symbol { ..*self }
    }
}

impl<T> ::std::ops::Deref for Symbol<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe {
            // Additional reference level for a dereference on `deref` return value.
            &*(&self.pointer as *const *mut _ as *const T)
        }
    }
}

impl<T> fmt::Debug for Symbol<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&format!("Symbol@{:p}", self.pointer))
    }
}

struct ErrorModeGuard(DWORD);

impl ErrorModeGuard {
    #[allow(clippy::if_same_then_else)]
    fn new() -> Option<ErrorModeGuard> {
        unsafe {
            let mut previous_mode = 0;
            if errhandlingapi::SetThreadErrorMode(SEM_FAILCE, &mut previous_mode) == 0 {
                // How in the world is it possible for what is essentially a simple variable swap
                // to fail?  For now we just ignore the error -- the worst that can happen here is
                // the previous mode staying on and user seeing a dialog error on older Windows
                // machines.
                None
            } else if previous_mode == SEM_FAILCE {
                None
            } else {
                Some(ErrorModeGuard(previous_mode))
            }
        }
    }
}

impl Drop for ErrorModeGuard {
    fn drop(&mut self) {
        unsafe {
            errhandlingapi::SetThreadErrorMode(self.0, ptr::null_mut());
        }
    }
}

fn with_get_last_error<T, F>(wrap: fn(crate::error::WindowsError) -> crate::Error, closure: F)
-> Result<T, Option<crate::Error>>
where F: FnOnce() -> Option<T> {
    closure().ok_or_else(|| {
        let error = unsafe { errhandlingapi::GetLastError() };
        if error == 0 {
            None
        } else {
            Some(wrap(crate::error::WindowsError(io::Error::from_raw_os_error(error as i32))))
        }
    })
}

/// Do not check AppLocker rules or apply Software Restriction Policies for the DLL.
///
/// This action applies only to the DLL being loaded and not to its dependencies. This value is
/// recommended for use in setup programs that must run extracted DLLs during installation.
///
/// See [flag documentation on MSDN](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw#parameters).
pub const LOAD_IGNORE_CODE_AUTHZ_LEVEL: DWORD = consts::LOAD_IGNORE_CODE_AUTHZ_LEVEL;

/// Map the file into the calling process’ virtual address space as if it were a data file.
///
/// Nothing is done to execute or prepare to execute the mapped file. Therefore, you cannot call
/// functions like [`Library::get`] with this DLL. Using this value causes writes to read-only
/// memory to raise an access violation. Use this flag when you want to load a DLL only to extract
/// messages or resources from it.
///
/// See [flag documentation on MSDN](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw#parameters).
pub const LOAD_LIBRARY_AS_DATAFILE: DWORD = consts::LOAD_LIBRARY_AS_DATAFILE;

/// Map the file into the calling process’ virtual address space as if it were a data file.
///
/// Similar to [`LOAD_LIBRARY_AS_DATAFILE`], except that the DLL file is opened with exclusive
/// write access for the calling process. Other processes cannot open the DLL file for write access
/// while it is in use. However, the DLL can still be opened by other processes.
///
/// See [flag documentation on MSDN](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw#parameters).
pub const LOAD_LIBRARY_AS_DATAFILE_EXCLUSIVE: DWORD = consts::LOAD_LIBRARY_AS_DATAFILE_EXCLUSIVE;

/// Map the file into the process’ virtual address space as an image file.
///
/// The loader does not load the static imports or perform the other usual initialisation steps.
/// Use this flag when you want to load a DLL only to extract messages or resources from it.
///
/// Unless the application depends on the file having the in-memory layout of an image, this value
/// should be used with either [`LOAD_LIBRARY_AS_DATAFILE_EXCLUSIVE`] or
/// [`LOAD_LIBRARY_AS_DATAFILE`].
///
/// See [flag documentation on MSDN](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw#parameters).
pub const LOAD_LIBRARY_AS_IMAGE_RESOURCE: DWORD = consts::LOAD_LIBRARY_AS_IMAGE_RESOURCE;

/// Search the application's installation directory for the DLL and its dependencies.
///
/// Directories in the standard search path are not searched. This value cannot be combined with
/// [`LOAD_WITH_ALTERED_SEARCH_PATH`].
///
/// See [flag documentation on MSDN](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw#parameters).
pub const LOAD_LIBRARY_SEARCH_APPLICATION_DIR: DWORD = consts::LOAD_LIBRARY_SEARCH_APPLICATION_DIR;

/// Search default directories when looking for the DLL and its dependencies.
///
/// This value is a combination of [`LOAD_LIBRARY_SEARCH_APPLICATION_DIR`],
/// [`LOAD_LIBRARY_SEARCH_SYSTEM32`], and [`LOAD_LIBRARY_SEARCH_USER_DIRS`]. Directories in the
/// standard search path are not searched. This value cannot be combined with
/// [`LOAD_WITH_ALTERED_SEARCH_PATH`].
///
/// See [flag documentation on MSDN](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw#parameters).
pub const LOAD_LIBRARY_SEARCH_DEFAULT_DIRS: DWORD = consts::LOAD_LIBRARY_SEARCH_DEFAULT_DIRS;

/// Directory that contains the DLL is temporarily added to the beginning of the list of
/// directories that are searched for the DLL’s dependencies.
///
/// Directories in the standard search path are not searched.
///
/// The `filename` parameter must specify a fully qualified path. This value cannot be combined
/// with [`LOAD_WITH_ALTERED_SEARCH_PATH`].
///
/// See [flag documentation on MSDN](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw#parameters).
pub const LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR: DWORD = consts::LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR;

/// Search `%windows%\system32` for the DLL and its dependencies.
///
/// Directories in the standard search path are not searched. This value cannot be combined with
/// [`LOAD_WITH_ALTERED_SEARCH_PATH`].
///
/// See [flag documentation on MSDN](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw#parameters).
pub const LOAD_LIBRARY_SEARCH_SYSTEM32: DWORD = consts::LOAD_LIBRARY_SEARCH_SYSTEM32;

///  Directories added using the `AddDllDirectory` or the `SetDllDirectory` function are searched
///  for the DLL and its dependencies.
///
///  If more than one directory has been added, the order in which the directories are searched is
///  unspecified. Directories in the standard search path are not searched. This value cannot be
///  combined with [`LOAD_WITH_ALTERED_SEARCH_PATH`].
///
/// See [flag documentation on MSDN](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw#parameters).
pub const LOAD_LIBRARY_SEARCH_USER_DIRS: DWORD = consts::LOAD_LIBRARY_SEARCH_USER_DIRS;

/// If `filename` specifies an absolute path, the system uses the alternate file search strategy
/// discussed in the [Remarks section] to find associated executable modules that the specified
/// module causes to be loaded.
///
/// If this value is used and `filename` specifies a relative path, the behaviour is undefined.
///
/// If this value is not used, or if `filename` does not specify a path, the system uses the
/// standard search strategy discussed in the [Remarks section] to find associated executable
/// modules that the specified module causes to be loaded.
///
/// See [flag documentation on MSDN](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw#parameters).
///
/// [Remarks]: https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw#remarks
pub const LOAD_WITH_ALTERED_SEARCH_PATH: DWORD = consts::LOAD_WITH_ALTERED_SEARCH_PATH;

/// Specifies that the digital signature of the binary image must be checked at load time.
///
/// See [flag documentation on MSDN](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw#parameters).
pub const LOAD_LIBRARY_REQUIRE_SIGNED_TARGET: DWORD = consts::LOAD_LIBRARY_REQUIRE_SIGNED_TARGET;

/// Allow loading a DLL for execution from the current directory only if it is under a directory in
/// the Safe load list.
///
/// See [flag documentation on MSDN](https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryexw#parameters).
pub const LOAD_LIBRARY_SAFE_CURRENT_DIRS: DWORD = consts::LOAD_LIBRARY_SAFE_CURRENT_DIRS;
