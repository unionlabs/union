use crate::sys::{ExportError, Instance};
use thiserror::Error;

/// An error while initializing the user supplied host env with the `WasmerEnv` trait.
#[derive(Error, Debug)]
#[error("Host env initialization error: {0}")]
pub enum HostEnvInitError {
    /// An error occurred when accessing an export
    Export(ExportError),
}

impl From<ExportError> for HostEnvInitError {
    fn from(other: ExportError) -> Self {
        Self::Export(other)
    }
}

/// Trait for initializing the environments passed to host functions after
/// instantiation but before execution.
///
/// This is useful for filling an environment with data that can only be accesed
/// after instantiation. For example, exported items such as memories and
/// functions which don't exist prior to instantiation can be accessed here so
/// that host functions can use them.
///
/// # Examples
///
/// This trait can be derived like so:
///
/// ```
/// use wasmer::{WasmerEnv, LazyInit, Memory, NativeFunc};
///
/// #[derive(WasmerEnv, Clone)]
/// pub struct MyEnvWithNoInstanceData {
///     non_instance_data: u8,
/// }
///
/// #[derive(WasmerEnv, Clone)]
/// pub struct MyEnvWithInstanceData {
///     non_instance_data: u8,
///     #[wasmer(export)]
///     memory: LazyInit<Memory>,
///     #[wasmer(export(name = "real_name"))]
///     func: LazyInit<NativeFunc<(i32, i32), i32>>,
///     #[wasmer(export(optional = true, alias = "memory2", alias = "_memory2"))]
///     optional_memory: LazyInit<Memory>,
/// }
///
/// ```
///
/// When deriving `WasmerEnv`, you must wrap your types to be initialized in
/// [`LazyInit`]. The derive macro will also generate helper methods of the form
/// `<field_name>_ref` and `<field_name>_ref_unchecked` for easy access to the
/// data.
///
/// The valid arguments to `export` are:
/// - `name = "string"`: specify the name of this item in the Wasm module. If this is not specified, it will default to the name of the field.
/// - `optional = true`: specify whether this export is optional. Defaults to
/// `false`. Being optional means that if the export can't be found, the
/// [`LazyInit`] will be left uninitialized.
/// - `alias = "string"`: specify additional names to look for in the Wasm module.
/// `alias` may be specified multiple times to search for multiple aliases.
/// -------
///
/// This trait may also be implemented manually:
/// ```
/// # use wasmer::{WasmerEnv, LazyInit, Memory, Instance, HostEnvInitError};
/// #[derive(Clone)]
/// pub struct MyEnv {
///    memory: LazyInit<Memory>,
/// }
///
/// impl WasmerEnv for MyEnv {
///     fn init_with_instance(&mut self, instance: &Instance) -> Result<(), HostEnvInitError> {
///         let memory: Memory = instance.exports.get_with_generics_weak("memory").unwrap();
///         self.memory.initialize(memory.clone());
///         Ok(())
///     }
/// }
/// ```
///
/// When implementing the trait manually, it's important to get a "weak" export to
/// prevent a cyclic reference leaking memory. You can access a "weak" export with
/// a method like `get_with_generics_weak`.
pub trait WasmerEnv: Clone + Send + Sync {
    /// The function that Wasmer will call on your type to let it finish
    /// setting up the environment with data from the `Instance`.
    ///
    /// This function is called after `Instance` is created but before it is
    /// returned to the user via `Instance::new`.
    fn init_with_instance(&mut self, _instance: &Instance) -> Result<(), HostEnvInitError> {
        Ok(())
    }
}

impl WasmerEnv for u8 {}
impl WasmerEnv for i8 {}
impl WasmerEnv for u16 {}
impl WasmerEnv for i16 {}
impl WasmerEnv for u32 {}
impl WasmerEnv for i32 {}
impl WasmerEnv for u64 {}
impl WasmerEnv for i64 {}
impl WasmerEnv for u128 {}
impl WasmerEnv for i128 {}
impl WasmerEnv for f32 {}
impl WasmerEnv for f64 {}
impl WasmerEnv for usize {}
impl WasmerEnv for isize {}
impl WasmerEnv for char {}
impl WasmerEnv for bool {}
impl WasmerEnv for String {}
impl<'a> WasmerEnv for &'a ::std::sync::atomic::AtomicBool {}
impl<'a> WasmerEnv for &'a ::std::sync::atomic::AtomicI8 {}
impl<'a> WasmerEnv for &'a ::std::sync::atomic::AtomicU8 {}
impl<'a> WasmerEnv for &'a ::std::sync::atomic::AtomicI16 {}
impl<'a> WasmerEnv for &'a ::std::sync::atomic::AtomicU16 {}
impl<'a> WasmerEnv for &'a ::std::sync::atomic::AtomicI32 {}
impl<'a> WasmerEnv for &'a ::std::sync::atomic::AtomicU32 {}
impl<'a> WasmerEnv for &'a ::std::sync::atomic::AtomicI64 {}
impl<'a> WasmerEnv for &'a ::std::sync::atomic::AtomicUsize {}
impl<'a> WasmerEnv for &'a ::std::sync::atomic::AtomicIsize {}
impl<T: WasmerEnv> WasmerEnv for Box<T> {
    fn init_with_instance(&mut self, instance: &Instance) -> Result<(), HostEnvInitError> {
        (&mut **self).init_with_instance(instance)
    }
}

impl<T: WasmerEnv> WasmerEnv for ::std::sync::Arc<::std::sync::Mutex<T>> {
    fn init_with_instance(&mut self, instance: &Instance) -> Result<(), HostEnvInitError> {
        let mut guard = self.lock().unwrap();
        guard.init_with_instance(instance)
    }
}

/// Lazily init an item
pub struct LazyInit<T: Sized> {
    /// The data to be initialized
    data: std::mem::MaybeUninit<T>,
    /// Whether or not the data has been initialized
    initialized: bool,
}

impl<T> LazyInit<T> {
    /// Creates an unitialized value.
    pub fn new() -> Self {
        Self {
            data: std::mem::MaybeUninit::uninit(),
            initialized: false,
        }
    }

    /// # Safety
    /// - The data must be initialized first
    pub unsafe fn get_unchecked(&self) -> &T {
        &*self.data.as_ptr()
    }

    /// Get the inner data.
    pub fn get_ref(&self) -> Option<&T> {
        if !self.initialized {
            None
        } else {
            Some(unsafe { self.get_unchecked() })
        }
    }

    /// Sets a value and marks the data as initialized.
    pub fn initialize(&mut self, value: T) -> bool {
        if self.initialized {
            return false;
        }
        unsafe {
            self.data.as_mut_ptr().write(value);
        }
        self.initialized = true;
        true
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for LazyInit<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("LazyInit")
            .field("data", &self.get_ref())
            .finish()
    }
}

impl<T: Clone> Clone for LazyInit<T> {
    fn clone(&self) -> Self {
        if let Some(inner) = self.get_ref() {
            Self {
                data: std::mem::MaybeUninit::new(inner.clone()),
                initialized: true,
            }
        } else {
            Self {
                data: std::mem::MaybeUninit::uninit(),
                initialized: false,
            }
        }
    }
}

impl<T> Drop for LazyInit<T> {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                let ptr = self.data.as_mut_ptr();
                std::ptr::drop_in_place(ptr);
            };
        }
    }
}

impl<T> Default for LazyInit<T> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<T: Send> Send for LazyInit<T> {}
// I thought we could opt out of sync..., look into this
// unsafe impl<T> !Sync for InitWithInstance<T> {}
