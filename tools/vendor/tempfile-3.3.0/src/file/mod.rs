use std::env;
use std::error;
use std::ffi::OsStr;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::mem;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use crate::error::IoResultExt;
use crate::Builder;

mod imp;

/// Create a new temporary file.
///
/// The file will be created in the location returned by [`std::env::temp_dir()`].
///
/// # Security
///
/// This variant is secure/reliable in the presence of a pathological temporary file cleaner.
///
/// # Resource Leaking
///
/// The temporary file will be automatically removed by the OS when the last handle to it is closed.
/// This doesn't rely on Rust destructors being run, so will (almost) never fail to clean up the temporary file.
///
/// # Errors
///
/// If the file can not be created, `Err` is returned.
///
/// # Examples
///
/// ```
/// use tempfile::tempfile;
/// use std::io::{self, Write};
///
/// # fn main() {
/// #     if let Err(_) = run() {
/// #         ::std::process::exit(1);
/// #     }
/// # }
/// # fn run() -> Result<(), io::Error> {
/// // Create a file inside of `std::env::temp_dir()`.
/// let mut file = tempfile()?;
///
/// writeln!(file, "Brian was here. Briefly.")?;
/// # Ok(())
/// # }
/// ```
///
/// [`std::env::temp_dir()`]: https://doc.rust-lang.org/std/env/fn.temp_dir.html
pub fn tempfile() -> io::Result<File> {
    tempfile_in(&env::temp_dir())
}

/// Create a new temporary file in the specified directory.
///
/// # Security
///
/// This variant is secure/reliable in the presence of a pathological temporary file cleaner.
/// If the temporary file isn't created in [`std::env::temp_dir()`] then temporary file cleaners aren't an issue.
///
/// # Resource Leaking
///
/// The temporary file will be automatically removed by the OS when the last handle to it is closed.
/// This doesn't rely on Rust destructors being run, so will (almost) never fail to clean up the temporary file.
///
/// # Errors
///
/// If the file can not be created, `Err` is returned.
///
/// # Examples
///
/// ```
/// use tempfile::tempfile_in;
/// use std::io::{self, Write};
///
/// # fn main() {
/// #     if let Err(_) = run() {
/// #         ::std::process::exit(1);
/// #     }
/// # }
/// # fn run() -> Result<(), io::Error> {
/// // Create a file inside of the current working directory
/// let mut file = tempfile_in("./")?;
///
/// writeln!(file, "Brian was here. Briefly.")?;
/// # Ok(())
/// # }
/// ```
///
/// [`std::env::temp_dir()`]: https://doc.rust-lang.org/std/env/fn.temp_dir.html
pub fn tempfile_in<P: AsRef<Path>>(dir: P) -> io::Result<File> {
    imp::create(dir.as_ref())
}

/// Error returned when persisting a temporary file path fails.
#[derive(Debug)]
pub struct PathPersistError {
    /// The underlying IO error.
    pub error: io::Error,
    /// The temporary file path that couldn't be persisted.
    pub path: TempPath,
}

impl From<PathPersistError> for io::Error {
    #[inline]
    fn from(error: PathPersistError) -> io::Error {
        error.error
    }
}

impl From<PathPersistError> for TempPath {
    #[inline]
    fn from(error: PathPersistError) -> TempPath {
        error.path
    }
}

impl fmt::Display for PathPersistError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to persist temporary file path: {}", self.error)
    }
}

impl error::Error for PathPersistError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&self.error)
    }
}

/// A path to a named temporary file without an open file handle.
///
/// This is useful when the temporary file needs to be used by a child process,
/// for example.
///
/// When dropped, the temporary file is deleted.
pub struct TempPath {
    path: Box<Path>,
}

impl TempPath {
    /// Close and remove the temporary file.
    ///
    /// Use this if you want to detect errors in deleting the file.
    ///
    /// # Errors
    ///
    /// If the file cannot be deleted, `Err` is returned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::io;
    /// use tempfile::NamedTempFile;
    ///
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), io::Error> {
    /// let file = NamedTempFile::new()?;
    ///
    /// // Close the file, but keep the path to it around.
    /// let path = file.into_temp_path();
    ///
    /// // By closing the `TempPath` explicitly, we can check that it has
    /// // been deleted successfully. If we don't close it explicitly, the
    /// // file will still be deleted when `file` goes out of scope, but we
    /// // won't know whether deleting the file succeeded.
    /// path.close()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn close(mut self) -> io::Result<()> {
        let result = fs::remove_file(&self.path).with_err_path(|| &*self.path);
        self.path = PathBuf::new().into_boxed_path();
        mem::forget(self);
        result
    }

    /// Persist the temporary file at the target path.
    ///
    /// If a file exists at the target path, persist will atomically replace it.
    /// If this method fails, it will return `self` in the resulting
    /// [`PathPersistError`].
    ///
    /// Note: Temporary files cannot be persisted across filesystems. Also
    /// neither the file contents nor the containing directory are
    /// synchronized, so the update may not yet have reached the disk when
    /// `persist` returns.
    ///
    /// # Security
    ///
    /// Only use this method if you're positive that a temporary file cleaner
    /// won't have deleted your file. Otherwise, you might end up persisting an
    /// attacker controlled file.
    ///
    /// # Errors
    ///
    /// If the file cannot be moved to the new location, `Err` is returned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::io::{self, Write};
    /// use tempfile::NamedTempFile;
    ///
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), io::Error> {
    /// let mut file = NamedTempFile::new()?;
    /// writeln!(file, "Brian was here. Briefly.")?;
    ///
    /// let path = file.into_temp_path();
    /// path.persist("./saved_file.txt")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`PathPersistError`]: struct.PathPersistError.html
    pub fn persist<P: AsRef<Path>>(mut self, new_path: P) -> Result<(), PathPersistError> {
        match imp::persist(&self.path, new_path.as_ref(), true) {
            Ok(_) => {
                // Don't drop `self`. We don't want to try deleting the old
                // temporary file path. (It'll fail, but the failure is never
                // seen.)
                self.path = PathBuf::new().into_boxed_path();
                mem::forget(self);
                Ok(())
            }
            Err(e) => Err(PathPersistError {
                error: e,
                path: self,
            }),
        }
    }

    /// Persist the temporary file at the target path if and only if no file exists there.
    ///
    /// If a file exists at the target path, fail. If this method fails, it will
    /// return `self` in the resulting [`PathPersistError`].
    ///
    /// Note: Temporary files cannot be persisted across filesystems. Also Note:
    /// This method is not atomic. It can leave the original link to the
    /// temporary file behind.
    ///
    /// # Security
    ///
    /// Only use this method if you're positive that a temporary file cleaner
    /// won't have deleted your file. Otherwise, you might end up persisting an
    /// attacker controlled file.
    ///
    /// # Errors
    ///
    /// If the file cannot be moved to the new location or a file already exists
    /// there, `Err` is returned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::io::{self, Write};
    /// use tempfile::NamedTempFile;
    ///
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), io::Error> {
    /// let mut file = NamedTempFile::new()?;
    /// writeln!(file, "Brian was here. Briefly.")?;
    ///
    /// let path = file.into_temp_path();
    /// path.persist_noclobber("./saved_file.txt")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`PathPersistError`]: struct.PathPersistError.html
    pub fn persist_noclobber<P: AsRef<Path>>(
        mut self,
        new_path: P,
    ) -> Result<(), PathPersistError> {
        match imp::persist(&self.path, new_path.as_ref(), false) {
            Ok(_) => {
                // Don't drop `self`. We don't want to try deleting the old
                // temporary file path. (It'll fail, but the failure is never
                // seen.)
                self.path = PathBuf::new().into_boxed_path();
                mem::forget(self);
                Ok(())
            }
            Err(e) => Err(PathPersistError {
                error: e,
                path: self,
            }),
        }
    }

    /// Keep the temporary file from being deleted. This function will turn the
    /// temporary file into a non-temporary file without moving it.
    ///
    ///
    /// # Errors
    ///
    /// On some platforms (e.g., Windows), we need to mark the file as
    /// non-temporary. This operation could fail.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::io::{self, Write};
    /// use tempfile::NamedTempFile;
    ///
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), io::Error> {
    /// let mut file = NamedTempFile::new()?;
    /// writeln!(file, "Brian was here. Briefly.")?;
    ///
    /// let path = file.into_temp_path();
    /// let path = path.keep()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`PathPersistError`]: struct.PathPersistError.html
    pub fn keep(mut self) -> Result<PathBuf, PathPersistError> {
        match imp::keep(&self.path) {
            Ok(_) => {
                // Don't drop `self`. We don't want to try deleting the old
                // temporary file path. (It'll fail, but the failure is never
                // seen.)
                let path = mem::replace(&mut self.path, PathBuf::new().into_boxed_path());
                mem::forget(self);
                Ok(path.into())
            }
            Err(e) => Err(PathPersistError {
                error: e,
                path: self,
            }),
        }
    }

    /// Create a new TempPath from an existing path. This can be done even if no
    /// file exists at the given path.
    ///
    /// This is mostly useful for interacting with libraries and external
    /// components that provide files to be consumed or expect a path with no
    /// existing file to be given.
    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into().into_boxed_path(),
        }
    }
}

impl fmt::Debug for TempPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.path.fmt(f)
    }
}

impl Drop for TempPath {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

impl Deref for TempPath {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.path
    }
}

impl AsRef<Path> for TempPath {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl AsRef<OsStr> for TempPath {
    fn as_ref(&self) -> &OsStr {
        self.path.as_os_str()
    }
}

/// A named temporary file.
///
/// The default constructor, [`NamedTempFile::new()`], creates files in
/// the location returned by [`std::env::temp_dir()`], but `NamedTempFile`
/// can be configured to manage a temporary file in any location
/// by constructing with [`NamedTempFile::new_in()`].
///
/// # Security
///
/// Most operating systems employ temporary file cleaners to delete old
/// temporary files. Unfortunately these temporary file cleaners don't always
/// reliably _detect_ whether the temporary file is still being used.
///
/// Specifically, the following sequence of events can happen:
///
/// 1. A user creates a temporary file with `NamedTempFile::new()`.
/// 2. Time passes.
/// 3. The temporary file cleaner deletes (unlinks) the temporary file from the
///    filesystem.
/// 4. Some other program creates a new file to replace this deleted temporary
///    file.
/// 5. The user tries to re-open the temporary file (in the same program or in a
///    different program) by path. Unfortunately, they'll end up opening the
///    file created by the other program, not the original file.
///
/// ## Operating System Specific Concerns
///
/// The behavior of temporary files and temporary file cleaners differ by
/// operating system.
///
/// ### Windows
///
/// On Windows, open files _can't_ be deleted. This removes most of the concerns
/// around temporary file cleaners.
///
/// Furthermore, temporary files are, by default, created in per-user temporary
/// file directories so only an application running as the same user would be
/// able to interfere (which they could do anyways). However, an application
/// running as the same user can still _accidentally_ re-create deleted
/// temporary files if the number of random bytes in the temporary file name is
/// too small.
///
/// So, the only real concern on Windows is:
///
/// 1. Opening a named temporary file in a world-writable directory.
/// 2. Using the `into_temp_path()` and/or `into_parts()` APIs to close the file
///    handle without deleting the underlying file.
/// 3. Continuing to use the file by path.
///
/// ### UNIX
///
/// Unlike on Windows, UNIX (and UNIX like) systems allow open files to be
/// "unlinked" (deleted).
///
/// #### MacOS
///
/// Like on Windows, temporary files are created in per-user temporary file
/// directories by default so calling `NamedTempFile::new()` should be
/// relatively safe.
///
/// #### Linux
///
/// Unfortunately, most _Linux_ distributions don't create per-user temporary
/// file directories. Worse, systemd's tmpfiles daemon (a common temporary file
/// cleaner) will happily remove open temporary files if they haven't been
/// modified within the last 10 days.
///
/// # Resource Leaking
///
/// If the program exits before the `NamedTempFile` destructor is
/// run, such as via [`std::process::exit()`], by segfaulting, or by
/// receiving a signal like `SIGINT`, then the temporary file
/// will not be deleted.
///
/// Use the [`tempfile()`] function unless you absolutely need a named file.
///
/// [`tempfile()`]: fn.tempfile.html
/// [`NamedTempFile::new()`]: #method.new
/// [`NamedTempFile::new_in()`]: #method.new_in
/// [`std::env::temp_dir()`]: https://doc.rust-lang.org/std/env/fn.temp_dir.html
/// [`std::process::exit()`]: http://doc.rust-lang.org/std/process/fn.exit.html
pub struct NamedTempFile {
    path: TempPath,
    file: File,
}

impl fmt::Debug for NamedTempFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NamedTempFile({:?})", self.path)
    }
}

impl AsRef<Path> for NamedTempFile {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.path()
    }
}

/// Error returned when persisting a temporary file fails.
#[derive(Debug)]
pub struct PersistError {
    /// The underlying IO error.
    pub error: io::Error,
    /// The temporary file that couldn't be persisted.
    pub file: NamedTempFile,
}

impl From<PersistError> for io::Error {
    #[inline]
    fn from(error: PersistError) -> io::Error {
        error.error
    }
}

impl From<PersistError> for NamedTempFile {
    #[inline]
    fn from(error: PersistError) -> NamedTempFile {
        error.file
    }
}

impl fmt::Display for PersistError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to persist temporary file: {}", self.error)
    }
}

impl error::Error for PersistError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&self.error)
    }
}

impl NamedTempFile {
    /// Create a new named temporary file.
    ///
    /// See [`Builder`] for more configuration.
    ///
    /// # Security
    ///
    /// This will create a temporary file in the default temporary file
    /// directory (platform dependent). This has security implications on many
    /// platforms so please read the security section of this type's
    /// documentation.
    ///
    /// Reasons to use this method:
    ///
    ///   1. The file has a short lifetime and your temporary file cleaner is
    ///      sane (doesn't delete recently accessed files).
    ///
    ///   2. You trust every user on your system (i.e. you are the only user).
    ///
    ///   3. You have disabled your system's temporary file cleaner or verified
    ///      that your system doesn't have a temporary file cleaner.
    ///
    /// Reasons not to use this method:
    ///
    ///   1. You'll fix it later. No you won't.
    ///
    ///   2. You don't care about the security of the temporary file. If none of
    ///      the "reasons to use this method" apply, referring to a temporary
    ///      file by name may allow an attacker to create/overwrite your
    ///      non-temporary files. There are exceptions but if you don't already
    ///      know them, don't use this method.
    ///
    /// # Errors
    ///
    /// If the file can not be created, `Err` is returned.
    ///
    /// # Examples
    ///
    /// Create a named temporary file and write some data to it:
    ///
    /// ```no_run
    /// # use std::io::{self, Write};
    /// use tempfile::NamedTempFile;
    ///
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), ::std::io::Error> {
    /// let mut file = NamedTempFile::new()?;
    ///
    /// writeln!(file, "Brian was here. Briefly.")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Builder`]: struct.Builder.html
    pub fn new() -> io::Result<NamedTempFile> {
        Builder::new().tempfile()
    }

    /// Create a new named temporary file in the specified directory.
    ///
    /// See [`NamedTempFile::new()`] for details.
    ///
    /// [`NamedTempFile::new()`]: #method.new
    pub fn new_in<P: AsRef<Path>>(dir: P) -> io::Result<NamedTempFile> {
        Builder::new().tempfile_in(dir)
    }

    /// Get the temporary file's path.
    ///
    /// # Security
    ///
    /// Referring to a temporary file's path may not be secure in all cases.
    /// Please read the security section on the top level documentation of this
    /// type for details.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::io::{self, Write};
    /// use tempfile::NamedTempFile;
    ///
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), ::std::io::Error> {
    /// let file = NamedTempFile::new()?;
    ///
    /// println!("{:?}", file.path());
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Close and remove the temporary file.
    ///
    /// Use this if you want to detect errors in deleting the file.
    ///
    /// # Errors
    ///
    /// If the file cannot be deleted, `Err` is returned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::io;
    /// use tempfile::NamedTempFile;
    ///
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), io::Error> {
    /// let file = NamedTempFile::new()?;
    ///
    /// // By closing the `NamedTempFile` explicitly, we can check that it has
    /// // been deleted successfully. If we don't close it explicitly,
    /// // the file will still be deleted when `file` goes out
    /// // of scope, but we won't know whether deleting the file
    /// // succeeded.
    /// file.close()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn close(self) -> io::Result<()> {
        let NamedTempFile { path, .. } = self;
        path.close()
    }

    /// Persist the temporary file at the target path.
    ///
    /// If a file exists at the target path, persist will atomically replace it.
    /// If this method fails, it will return `self` in the resulting
    /// [`PersistError`].
    ///
    /// Note: Temporary files cannot be persisted across filesystems. Also
    /// neither the file contents nor the containing directory are
    /// synchronized, so the update may not yet have reached the disk when
    /// `persist` returns.
    ///
    /// # Security
    ///
    /// This method persists the temporary file using its path and may not be
    /// secure in the in all cases. Please read the security section on the top
    /// level documentation of this type for details.
    ///
    /// # Errors
    ///
    /// If the file cannot be moved to the new location, `Err` is returned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::io::{self, Write};
    /// use tempfile::NamedTempFile;
    ///
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), io::Error> {
    /// let file = NamedTempFile::new()?;
    ///
    /// let mut persisted_file = file.persist("./saved_file.txt")?;
    /// writeln!(persisted_file, "Brian was here. Briefly.")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`PersistError`]: struct.PersistError.html
    pub fn persist<P: AsRef<Path>>(self, new_path: P) -> Result<File, PersistError> {
        let NamedTempFile { path, file } = self;
        match path.persist(new_path) {
            Ok(_) => Ok(file),
            Err(err) => {
                let PathPersistError { error, path } = err;
                Err(PersistError {
                    file: NamedTempFile { path, file },
                    error,
                })
            }
        }
    }

    /// Persist the temporary file at the target path if and only if no file exists there.
    ///
    /// If a file exists at the target path, fail. If this method fails, it will
    /// return `self` in the resulting PersistError.
    ///
    /// Note: Temporary files cannot be persisted across filesystems. Also Note:
    /// This method is not atomic. It can leave the original link to the
    /// temporary file behind.
    ///
    /// # Security
    ///
    /// This method persists the temporary file using its path and may not be
    /// secure in the in all cases. Please read the security section on the top
    /// level documentation of this type for details.
    ///
    /// # Errors
    ///
    /// If the file cannot be moved to the new location or a file already exists there,
    /// `Err` is returned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::io::{self, Write};
    /// use tempfile::NamedTempFile;
    ///
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), io::Error> {
    /// let file = NamedTempFile::new()?;
    ///
    /// let mut persisted_file = file.persist_noclobber("./saved_file.txt")?;
    /// writeln!(persisted_file, "Brian was here. Briefly.")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn persist_noclobber<P: AsRef<Path>>(self, new_path: P) -> Result<File, PersistError> {
        let NamedTempFile { path, file } = self;
        match path.persist_noclobber(new_path) {
            Ok(_) => Ok(file),
            Err(err) => {
                let PathPersistError { error, path } = err;
                Err(PersistError {
                    file: NamedTempFile { path, file },
                    error,
                })
            }
        }
    }

    /// Keep the temporary file from being deleted. This function will turn the
    /// temporary file into a non-temporary file without moving it.
    ///
    ///
    /// # Errors
    ///
    /// On some platforms (e.g., Windows), we need to mark the file as
    /// non-temporary. This operation could fail.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::io::{self, Write};
    /// use tempfile::NamedTempFile;
    ///
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), io::Error> {
    /// let mut file = NamedTempFile::new()?;
    /// writeln!(file, "Brian was here. Briefly.")?;
    ///
    /// let (file, path) = file.keep()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`PathPersistError`]: struct.PathPersistError.html
    pub fn keep(self) -> Result<(File, PathBuf), PersistError> {
        let (file, path) = (self.file, self.path);
        match path.keep() {
            Ok(path) => Ok((file, path)),
            Err(PathPersistError { error, path }) => Err(PersistError {
                file: NamedTempFile { path, file },
                error,
            }),
        }
    }

    /// Securely reopen the temporary file.
    ///
    /// This function is useful when you need multiple independent handles to
    /// the same file. It's perfectly fine to drop the original `NamedTempFile`
    /// while holding on to `File`s returned by this function; the `File`s will
    /// remain usable. However, they may not be nameable.
    ///
    /// # Errors
    ///
    /// If the file cannot be reopened, `Err` is returned.
    ///
    /// # Security
    ///
    /// Unlike `File::open(my_temp_file.path())`, `NamedTempFile::reopen()`
    /// guarantees that the re-opened file is the _same_ file, even in the
    /// presence of pathological temporary file cleaners.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::io;
    /// use tempfile::NamedTempFile;
    ///
    /// # fn main() {
    /// #     if let Err(_) = run() {
    /// #         ::std::process::exit(1);
    /// #     }
    /// # }
    /// # fn run() -> Result<(), io::Error> {
    /// let file = NamedTempFile::new()?;
    ///
    /// let another_handle = file.reopen()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn reopen(&self) -> io::Result<File> {
        imp::reopen(self.as_file(), NamedTempFile::path(self))
            .with_err_path(|| NamedTempFile::path(self))
    }

    /// Get a reference to the underlying file.
    pub fn as_file(&self) -> &File {
        &self.file
    }

    /// Get a mutable reference to the underlying file.
    pub fn as_file_mut(&mut self) -> &mut File {
        &mut self.file
    }

    /// Convert the temporary file into a `std::fs::File`.
    ///
    /// The inner file will be deleted.
    pub fn into_file(self) -> File {
        self.file
    }

    /// Closes the file, leaving only the temporary file path.
    ///
    /// This is useful when another process must be able to open the temporary
    /// file.
    pub fn into_temp_path(self) -> TempPath {
        self.path
    }

    /// Converts the named temporary file into its constituent parts.
    ///
    /// Note: When the path is dropped, the file is deleted but the file handle
    /// is still usable.
    pub fn into_parts(self) -> (File, TempPath) {
        (self.file, self.path)
    }
}

impl Read for NamedTempFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.as_file_mut().read(buf).with_err_path(|| self.path())
    }
}

impl<'a> Read for &'a NamedTempFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.as_file().read(buf).with_err_path(|| self.path())
    }
}

impl Write for NamedTempFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.as_file_mut().write(buf).with_err_path(|| self.path())
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.as_file_mut().flush().with_err_path(|| self.path())
    }
}

impl<'a> Write for &'a NamedTempFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.as_file().write(buf).with_err_path(|| self.path())
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.as_file().flush().with_err_path(|| self.path())
    }
}

impl Seek for NamedTempFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.as_file_mut().seek(pos).with_err_path(|| self.path())
    }
}

impl<'a> Seek for &'a NamedTempFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.as_file().seek(pos).with_err_path(|| self.path())
    }
}

#[cfg(unix)]
impl std::os::unix::io::AsRawFd for NamedTempFile {
    #[inline]
    fn as_raw_fd(&self) -> std::os::unix::io::RawFd {
        self.as_file().as_raw_fd()
    }
}

#[cfg(windows)]
impl std::os::windows::io::AsRawHandle for NamedTempFile {
    #[inline]
    fn as_raw_handle(&self) -> std::os::windows::io::RawHandle {
        self.as_file().as_raw_handle()
    }
}

pub(crate) fn create_named(
    mut path: PathBuf,
    open_options: &mut OpenOptions,
) -> io::Result<NamedTempFile> {
    // Make the path absolute. Otherwise, changing directories could cause us to
    // delete the wrong file.
    if !path.is_absolute() {
        path = env::current_dir()?.join(path)
    }
    imp::create_named(&path, open_options)
        .with_err_path(|| path.clone())
        .map(|file| NamedTempFile {
            path: TempPath {
                path: path.into_boxed_path(),
            },
            file,
        })
}
