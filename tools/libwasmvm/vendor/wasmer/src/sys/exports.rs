use crate::sys::externals::{Extern, Function, Global, Memory, Table};
use crate::sys::import_object::LikeNamespace;
use crate::sys::native::NativeFunc;
use crate::sys::WasmTypeList;
use indexmap::IndexMap;
use loupe::MemoryUsage;
use std::fmt;
use std::iter::{ExactSizeIterator, FromIterator};
use thiserror::Error;
use wasmer_engine::Export;

/// The `ExportError` can happen when trying to get a specific
/// export [`Extern`] from the [`Instance`] exports.
///
/// [`Instance`]: crate::Instance
///
/// # Examples
///
/// ## Incompatible export type
///
/// ```should_panic
/// # use wasmer::{imports, wat2wasm, Function, Instance, Module, Store, Type, Value, ExportError};
/// # let store = Store::default();
/// # let wasm_bytes = wat2wasm(r#"
/// # (module
/// #   (global $one (export "glob") f32 (f32.const 1)))
/// # "#.as_bytes()).unwrap();
/// # let module = Module::new(&store, wasm_bytes).unwrap();
/// # let import_object = imports! {};
/// # let instance = Instance::new(&module, &import_object).unwrap();
/// #
/// // This results with an error: `ExportError::IncompatibleType`.
/// let export = instance.exports.get_function("glob").unwrap();
/// ```
///
/// ## Missing export
///
/// ```should_panic
/// # use wasmer::{imports, wat2wasm, Function, Instance, Module, Store, Type, Value, ExportError};
/// # let store = Store::default();
/// # let wasm_bytes = wat2wasm("(module)".as_bytes()).unwrap();
/// # let module = Module::new(&store, wasm_bytes).unwrap();
/// # let import_object = imports! {};
/// # let instance = Instance::new(&module, &import_object).unwrap();
/// #
/// // This results with an error: `ExportError::Missing`.
/// let export = instance.exports.get_function("unknown").unwrap();
/// ```
#[derive(Error, Debug)]
pub enum ExportError {
    /// An error than occurs when the exported type and the expected type
    /// are incompatible.
    #[error("Incompatible Export Type")]
    IncompatibleType,
    /// This error arises when an export is missing
    #[error("Missing export {0}")]
    Missing(String),
}

/// Exports is a special kind of map that allows easily unwrapping
/// the types of instances.
///
/// TODO: add examples of using exports
#[derive(Clone, Default, MemoryUsage)]
pub struct Exports {
    map: IndexMap<String, Extern>,
}

impl Exports {
    /// Creates a new `Exports`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Creates a new `Exports` with capacity `n`.
    pub fn with_capacity(n: usize) -> Self {
        Self {
            map: IndexMap::with_capacity(n),
        }
    }

    /// Return the number of exports in the `Exports` map.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Return whether or not there are no exports
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Insert a new export into this `Exports` map.
    pub fn insert<S, E>(&mut self, name: S, value: E)
    where
        S: Into<String>,
        E: Into<Extern>,
    {
        self.map.insert(name.into(), value.into());
    }

    /// Get an export given a `name`.
    ///
    /// The `get` method is specifically made for usage inside of
    /// Rust APIs, as we can detect what's the desired type easily.
    ///
    /// If you want to get an export dynamically with type checking
    /// please use the following functions: `get_func`, `get_memory`,
    /// `get_table` or `get_global` instead.
    ///
    /// If you want to get an export dynamically handling manually
    /// type checking manually, please use `get_extern`.
    pub fn get<'a, T: Exportable<'a>>(&'a self, name: &str) -> Result<&'a T, ExportError> {
        match self.map.get(name) {
            None => Err(ExportError::Missing(name.to_string())),
            Some(extern_) => T::get_self_from_extern(extern_),
        }
    }

    /// Get an export as a `Global`.
    pub fn get_global(&self, name: &str) -> Result<&Global, ExportError> {
        self.get(name)
    }

    /// Get an export as a `Memory`.
    pub fn get_memory(&self, name: &str) -> Result<&Memory, ExportError> {
        self.get(name)
    }

    /// Get an export as a `Table`.
    pub fn get_table(&self, name: &str) -> Result<&Table, ExportError> {
        self.get(name)
    }

    /// Get an export as a `Func`.
    pub fn get_function(&self, name: &str) -> Result<&Function, ExportError> {
        self.get(name)
    }

    /// Get an export as a `NativeFunc`.
    pub fn get_native_function<Args, Rets>(
        &self,
        name: &str,
    ) -> Result<NativeFunc<Args, Rets>, ExportError>
    where
        Args: WasmTypeList,
        Rets: WasmTypeList,
    {
        self.get_function(name)?
            .native()
            .map_err(|_| ExportError::IncompatibleType)
    }

    /// Hack to get this working with nativefunc too
    pub fn get_with_generics<'a, T, Args, Rets>(&'a self, name: &str) -> Result<T, ExportError>
    where
        Args: WasmTypeList,
        Rets: WasmTypeList,
        T: ExportableWithGenerics<'a, Args, Rets>,
    {
        match self.map.get(name) {
            None => Err(ExportError::Missing(name.to_string())),
            Some(extern_) => T::get_self_from_extern_with_generics(extern_),
        }
    }

    /// Like `get_with_generics` but with a WeakReference to the `InstanceRef` internally.
    /// This is useful for passing data into `WasmerEnv`, for example.
    pub fn get_with_generics_weak<'a, T, Args, Rets>(&'a self, name: &str) -> Result<T, ExportError>
    where
        Args: WasmTypeList,
        Rets: WasmTypeList,
        T: ExportableWithGenerics<'a, Args, Rets>,
    {
        let mut out: T = self.get_with_generics(name)?;
        out.into_weak_instance_ref();
        Ok(out)
    }

    /// Get an export as an `Extern`.
    pub fn get_extern(&self, name: &str) -> Option<&Extern> {
        self.map.get(name)
    }

    /// Returns true if the `Exports` contains the given export name.
    pub fn contains<S>(&self, name: S) -> bool
    where
        S: Into<String>,
    {
        self.map.contains_key(&name.into())
    }

    /// Get an iterator over the exports.
    pub fn iter(&self) -> ExportsIterator<impl Iterator<Item = (&String, &Extern)>> {
        ExportsIterator {
            iter: self.map.iter(),
        }
    }
}

impl fmt::Debug for Exports {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

/// An iterator over exports.
pub struct ExportsIterator<'a, I>
where
    I: Iterator<Item = (&'a String, &'a Extern)> + Sized,
{
    iter: I,
}

impl<'a, I> Iterator for ExportsIterator<'a, I>
where
    I: Iterator<Item = (&'a String, &'a Extern)> + Sized,
{
    type Item = (&'a String, &'a Extern);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a, I> ExactSizeIterator for ExportsIterator<'a, I>
where
    I: Iterator<Item = (&'a String, &'a Extern)> + ExactSizeIterator + Sized,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, I> ExportsIterator<'a, I>
where
    I: Iterator<Item = (&'a String, &'a Extern)> + Sized,
{
    /// Get only the functions.
    pub fn functions(self) -> impl Iterator<Item = (&'a String, &'a Function)> + Sized {
        self.iter.filter_map(|(name, export)| match export {
            Extern::Function(function) => Some((name, function)),
            _ => None,
        })
    }

    /// Get only the memories.
    pub fn memories(self) -> impl Iterator<Item = (&'a String, &'a Memory)> + Sized {
        self.iter.filter_map(|(name, export)| match export {
            Extern::Memory(memory) => Some((name, memory)),
            _ => None,
        })
    }

    /// Get only the globals.
    pub fn globals(self) -> impl Iterator<Item = (&'a String, &'a Global)> + Sized {
        self.iter.filter_map(|(name, export)| match export {
            Extern::Global(global) => Some((name, global)),
            _ => None,
        })
    }

    /// Get only the tables.
    pub fn tables(self) -> impl Iterator<Item = (&'a String, &'a Table)> + Sized {
        self.iter.filter_map(|(name, export)| match export {
            Extern::Table(table) => Some((name, table)),
            _ => None,
        })
    }
}

impl FromIterator<(String, Extern)> for Exports {
    fn from_iter<I: IntoIterator<Item = (String, Extern)>>(iter: I) -> Self {
        Self {
            map: IndexMap::from_iter(iter),
        }
    }
}

impl LikeNamespace for Exports {
    fn get_namespace_export(&self, name: &str) -> Option<Export> {
        self.map.get(name).map(|is_export| is_export.to_export())
    }

    fn get_namespace_exports(&self) -> Vec<(String, Export)> {
        self.map
            .iter()
            .map(|(k, v)| (k.clone(), v.to_export()))
            .collect()
    }

    fn as_exports(&self) -> Option<Exports> {
        Some(self.clone())
    }
}

/// This trait is used to mark types as gettable from an [`Instance`].
///
/// [`Instance`]: crate::Instance
pub trait Exportable<'a>: Sized {
    /// This function is used when providedd the [`Extern`] as exportable, so it
    /// can be used while instantiating the [`Module`].
    ///
    /// [`Module`]: crate::Module
    fn to_export(&self) -> Export;

    /// Implementation of how to get the export corresponding to the implementing type
    /// from an [`Instance`] by name.
    ///
    /// [`Instance`]: crate::Instance
    fn get_self_from_extern(_extern: &'a Extern) -> Result<&'a Self, ExportError>;

    /// Convert the extern internally to hold a weak reference to the `InstanceRef`.
    /// This is useful for preventing cycles, for example for data stored in a
    /// type implementing `WasmerEnv`.
    fn into_weak_instance_ref(&mut self);
}

/// A trait for accessing exports (like [`Exportable`]) but it takes generic
/// `Args` and `Rets` parameters so that `NativeFunc` can be accessed directly
/// as well.
pub trait ExportableWithGenerics<'a, Args: WasmTypeList, Rets: WasmTypeList>: Sized {
    /// Get an export with the given generics.
    fn get_self_from_extern_with_generics(_extern: &'a Extern) -> Result<Self, ExportError>;
    /// Convert the extern internally to hold a weak reference to the `InstanceRef`.
    /// This is useful for preventing cycles, for example for data stored in a
    /// type implementing `WasmerEnv`.
    fn into_weak_instance_ref(&mut self);
}

/// We implement it for all concrete [`Exportable`] types (that are `Clone`)
/// with empty `Args` and `Rets`.
impl<'a, T: Exportable<'a> + Clone + 'static> ExportableWithGenerics<'a, (), ()> for T {
    fn get_self_from_extern_with_generics(_extern: &'a Extern) -> Result<Self, ExportError> {
        T::get_self_from_extern(_extern).map(|i| i.clone())
    }

    fn into_weak_instance_ref(&mut self) {
        <Self as Exportable>::into_weak_instance_ref(self);
    }
}
