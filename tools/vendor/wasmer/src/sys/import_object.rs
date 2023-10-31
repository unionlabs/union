//! The import module contains the implementation data structures and helper functions used to
//! manipulate and access a wasm module's imports including memories, tables, globals, and
//! functions.
use crate::Exports;
use crate::Extern;
use std::borrow::{Borrow, BorrowMut};
use std::collections::VecDeque;
use std::collections::{hash_map::Entry, HashMap};
use std::fmt;
use std::sync::{Arc, Mutex};
use wasmer_engine::{Export, NamedResolver};

/// The `LikeNamespace` trait represents objects that act as a namespace for imports.
/// For example, an `Instance` or `Namespace` could be
/// considered namespaces that could provide imports to an instance.
pub trait LikeNamespace {
    /// Gets an export by name.
    fn get_namespace_export(&self, name: &str) -> Option<Export>;
    /// Gets all exports in the namespace.
    fn get_namespace_exports(&self) -> Vec<(String, Export)>;
    /// Returns the contents of this namespace as an `Exports`.
    ///
    /// This is used by `ImportObject::get_namespace_exports`.
    fn as_exports(&self) -> Option<Exports> {
        None
    }
}

/// All of the import data used when instantiating.
///
/// It's suggested that you use the [`imports!`] macro
/// instead of creating an `ImportObject` by hand.
///
/// [`imports!`]: macro.imports.html
///
/// # Usage:
/// ```ignore
/// use wasmer::{Exports, ImportObject, Function};
///
/// let mut import_object = ImportObject::new();
/// let mut env = Exports::new();
///
/// env.insert("foo", Function::new_native(foo));
/// import_object.register("env", env);
///
/// fn foo(n: i32) -> i32 {
///     n
/// }
/// ```
#[derive(Clone, Default)]
pub struct ImportObject {
    map: Arc<Mutex<HashMap<String, Box<dyn LikeNamespace + Send + Sync>>>>,
}

impl ImportObject {
    /// Create a new `ImportObject`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Gets an export given a module and a name
    ///
    /// # Usage
    /// ```ignore
    /// # use wasmer_vm::{ImportObject, Instance, Namespace};
    /// let mut import_object = ImportObject::new();
    /// import_object.get_export("module", "name");
    /// ```
    pub fn get_export(&self, module: &str, name: &str) -> Option<Export> {
        let guard = self.map.lock().unwrap();
        let map_ref = guard.borrow();
        if map_ref.contains_key(module) {
            let namespace = map_ref[module].as_ref();
            return namespace.get_namespace_export(name);
        }
        None
    }

    /// Returns true if the ImportObject contains namespace with the provided name.
    pub fn contains_namespace(&self, name: &str) -> bool {
        self.map.lock().unwrap().borrow().contains_key(name)
    }

    /// Register anything that implements `LikeNamespace` as a namespace.
    ///
    /// # Usage:
    /// ```ignore
    /// # use wasmer_vm::{ImportObject, Instance, Namespace};
    /// let mut import_object = ImportObject::new();
    ///
    /// import_object.register("namespace0", instance);
    /// import_object.register("namespace1", namespace);
    /// // ...
    /// ```
    pub fn register<S, N>(&mut self, name: S, namespace: N) -> Option<Box<dyn LikeNamespace>>
    where
        S: Into<String>,
        N: LikeNamespace + Send + Sync + 'static,
    {
        let mut guard = self.map.lock().unwrap();
        let map = guard.borrow_mut();

        match map.entry(name.into()) {
            Entry::Vacant(empty) => {
                empty.insert(Box::new(namespace));
                None
            }
            Entry::Occupied(mut occupied) => Some(occupied.insert(Box::new(namespace))),
        }
    }

    /// Returns the contents of a namespace as an `Exports`.
    ///
    /// Returns `None` if the namespace doesn't exist or doesn't implement the
    /// `as_exports` method.
    pub fn get_namespace_exports(&self, name: &str) -> Option<Exports> {
        let map = self.map.lock().unwrap();
        map.get(name).and_then(|ns| ns.as_exports())
    }

    /// Returns a list of all externs defined in all namespaces.
    pub fn externs_vec(&self) -> Vec<(String, String, Extern)> {
        let mut out = Vec::new();
        let guard = self.map.lock().unwrap();
        let map = guard.borrow();
        for (namespace, ns) in map.iter() {
            match ns.as_exports() {
                Some(exports) => {
                    for (name, extern_) in exports.iter() {
                        out.push((namespace.clone(), name.clone(), extern_.clone()));
                    }
                }
                None => {}
            }
        }
        out
    }

    fn get_objects(&self) -> VecDeque<((String, String), Export)> {
        let mut out = VecDeque::new();
        let guard = self.map.lock().unwrap();
        let map = guard.borrow();
        for (name, ns) in map.iter() {
            for (id, exp) in ns.get_namespace_exports() {
                out.push_back(((name.clone(), id), exp));
            }
        }
        out
    }
}

impl NamedResolver for ImportObject {
    fn resolve_by_name(&self, module: &str, name: &str) -> Option<Export> {
        self.get_export(module, name)
    }
}

/// Iterator for an `ImportObject`'s exports.
pub struct ImportObjectIterator {
    elements: VecDeque<((String, String), Export)>,
}

impl Iterator for ImportObjectIterator {
    type Item = ((String, String), Export);
    fn next(&mut self) -> Option<Self::Item> {
        self.elements.pop_front()
    }
}

impl IntoIterator for ImportObject {
    type IntoIter = ImportObjectIterator;
    type Item = ((String, String), Export);

    fn into_iter(self) -> Self::IntoIter {
        ImportObjectIterator {
            elements: self.get_objects(),
        }
    }
}

impl fmt::Debug for ImportObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        enum SecretMap {
            Empty,
            Some(usize),
        }

        impl SecretMap {
            fn new(len: usize) -> Self {
                if len == 0 {
                    Self::Empty
                } else {
                    Self::Some(len)
                }
            }
        }

        impl fmt::Debug for SecretMap {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    Self::Empty => write!(f, "(empty)"),
                    Self::Some(len) => write!(f, "(... {} item(s) ...)", len),
                }
            }
        }

        f.debug_struct("ImportObject")
            .field(
                "map",
                &SecretMap::new(self.map.lock().unwrap().borrow().len()),
            )
            .finish()
    }
}

// The import! macro for ImportObject

/// Generate an [`ImportObject`] easily with the `imports!` macro.
///
/// [`ImportObject`]: struct.ImportObject.html
///
/// # Usage
///
/// ```
/// # use wasmer::{Function, Store};
/// # let store = Store::default();
/// use wasmer::imports;
///
/// let import_object = imports! {
///     "env" => {
///         "foo" => Function::new_native(&store, foo)
///     },
/// };
///
/// fn foo(n: i32) -> i32 {
///     n
/// }
/// ```
#[macro_export]
macro_rules! imports {
    ( $( $ns_name:expr => $ns:tt ),* $(,)? ) => {
        {
            let mut import_object = $crate::ImportObject::new();

            $({
                let namespace = $crate::import_namespace!($ns);

                import_object.register($ns_name, namespace);
            })*

            import_object
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! namespace {
    ($( $import_name:expr => $import_item:expr ),* $(,)? ) => {
        $crate::import_namespace!( { $( $import_name => $import_item, )* } )
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! import_namespace {
    ( { $( $import_name:expr => $import_item:expr ),* $(,)? } ) => {{
        let mut namespace = $crate::Exports::new();

        $(
            namespace.insert($import_name, $import_item);
        )*

        namespace
    }};

    ( $namespace:ident ) => {
        $namespace
    };
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::sys::{Global, Store, Val};
    use wasmer_engine::ChainableNamedResolver;
    use wasmer_types::Type;

    #[test]
    fn chaining_works() {
        let store = Store::default();
        let g = Global::new(&store, Val::I32(0));

        let imports1 = imports! {
            "dog" => {
                "happy" => g.clone()
            }
        };

        let imports2 = imports! {
            "dog" => {
                "small" => g.clone()
            },
            "cat" => {
                "small" => g.clone()
            }
        };

        let resolver = imports1.chain_front(imports2);

        let small_cat_export = resolver.resolve_by_name("cat", "small");
        assert!(small_cat_export.is_some());

        let happy = resolver.resolve_by_name("dog", "happy");
        let small = resolver.resolve_by_name("dog", "small");
        assert!(happy.is_some());
        assert!(small.is_some());
    }

    #[test]
    fn extending_conflict_overwrites() {
        let store = Store::default();
        let g1 = Global::new(&store, Val::I32(0));
        let g2 = Global::new(&store, Val::I64(0));

        let imports1 = imports! {
            "dog" => {
                "happy" => g1,
            },
        };

        let imports2 = imports! {
            "dog" => {
                "happy" => g2,
            },
        };

        let resolver = imports1.chain_front(imports2);
        let happy_dog_entry = resolver.resolve_by_name("dog", "happy").unwrap();

        assert!(if let Export::Global(happy_dog_global) = happy_dog_entry {
            happy_dog_global.from.ty().ty == Type::I64
        } else {
            false
        });

        // now test it in reverse
        let store = Store::default();
        let g1 = Global::new(&store, Val::I32(0));
        let g2 = Global::new(&store, Val::I64(0));

        let imports1 = imports! {
            "dog" => {
                "happy" => g1,
            },
        };

        let imports2 = imports! {
            "dog" => {
                "happy" => g2,
            },
        };

        let resolver = imports1.chain_back(imports2);
        let happy_dog_entry = resolver.resolve_by_name("dog", "happy").unwrap();

        assert!(if let Export::Global(happy_dog_global) = happy_dog_entry {
            happy_dog_global.from.ty().ty == Type::I32
        } else {
            false
        });
    }

    #[test]
    fn namespace() {
        let store = Store::default();
        let g1 = Global::new(&store, Val::I32(0));
        let namespace = namespace! {
            "happy" => g1
        };
        let imports1 = imports! {
            "dog" => namespace
        };

        let happy_dog_entry = imports1.resolve_by_name("dog", "happy").unwrap();

        assert!(if let Export::Global(happy_dog_global) = happy_dog_entry {
            happy_dog_global.from.ty().ty == Type::I32
        } else {
            false
        });
    }

    #[test]
    fn imports_macro_allows_trailing_comma_and_none() {
        use crate::sys::Function;

        let store = Default::default();

        fn func(arg: i32) -> i32 {
            arg + 1
        }

        let _ = imports! {
            "env" => {
                "func" => Function::new_native(&store, func),
            },
        };
        let _ = imports! {
            "env" => {
                "func" => Function::new_native(&store, func),
            }
        };
        let _ = imports! {
            "env" => {
                "func" => Function::new_native(&store, func),
            },
            "abc" => {
                "def" => Function::new_native(&store, func),
            }
        };
        let _ = imports! {
            "env" => {
                "func" => Function::new_native(&store, func)
            },
        };
        let _ = imports! {
            "env" => {
                "func" => Function::new_native(&store, func)
            }
        };
        let _ = imports! {
            "env" => {
                "func1" => Function::new_native(&store, func),
                "func2" => Function::new_native(&store, func)
            }
        };
        let _ = imports! {
            "env" => {
                "func1" => Function::new_native(&store, func),
                "func2" => Function::new_native(&store, func),
            }
        };
    }
}
