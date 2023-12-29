pub(crate) use objects::{InternalStoreHandle, StoreObject};
pub use objects::{StoreHandle, StoreObjects};

mod objects {
    use rusty_jsc::{JSContext, JSObject, JSValue};

    use crate::jsc::vm::{VMFunctionEnvironment, VMGlobal};
    use std::{fmt, marker::PhantomData, num::NonZeroUsize};
    pub use wasmer_types::StoreId;

    /// Trait to represent an object managed by a context. This is implemented on
    /// the VM types managed by the context.
    pub trait StoreObject: Sized {
        fn list(store: &StoreObjects) -> &Vec<Self>;
        fn list_mut(store: &mut StoreObjects) -> &mut Vec<Self>;
    }

    macro_rules! impl_store_object {
    ($($field:ident => $ty:ty,)*) => {
        $(
            impl StoreObject for $ty {
                fn list(store: &StoreObjects) -> &Vec<Self> {
                    &store.$field
                }
                fn list_mut(store: &mut StoreObjects) -> &mut Vec<Self> {
                    &mut store.$field
                }
            }
        )*
    };
}

    impl_store_object! {
        // Note: we store the globals in order to be able to access them later via
        // `StoreObjects::iter_globals`.
        globals => VMGlobal,
        // functions => VMFunction,
        // tables => VMTable,
        // memories => VMMemory,
        // The function environments are the only things attached to a store,
        // since the other JS objects (table, globals, memory and functions)
        // live in the JS VM Store by default
        function_environments => VMFunctionEnvironment,
    }

    /// Set of objects managed by a context.
    #[derive(Default, Debug)]
    pub struct StoreObjects {
        id: StoreId,
        globals: Vec<VMGlobal>,
        function_environments: Vec<VMFunctionEnvironment>,
    }

    impl StoreObjects {
        /// Returns the ID of this context.
        pub fn id(&self) -> StoreId {
            self.id
        }

        /// Sets the ID of this store
        pub fn set_id(&mut self, id: StoreId) {
            self.id = id;
        }

        /// Returns a pair of mutable references from two handles.
        ///
        /// Panics if both handles point to the same object.
        pub fn get_2_mut<T: StoreObject>(
            &mut self,
            a: InternalStoreHandle<T>,
            b: InternalStoreHandle<T>,
        ) -> (&mut T, &mut T) {
            assert_ne!(a.index(), b.index());
            let list = T::list_mut(self);
            if a.index() < b.index() {
                let (low, high) = list.split_at_mut(b.index());
                (&mut low[a.index()], &mut high[0])
            } else {
                let (low, high) = list.split_at_mut(a.index());
                (&mut high[0], &mut low[a.index()])
            }
        }

        /// Return an immutable iterator over all globals
        pub fn iter_globals(&self) -> core::slice::Iter<VMGlobal> {
            self.globals.iter()
        }

        /// Return an vector of all globals and converted to u128
        pub fn as_u128_globals(&self) -> Vec<u128> {
            vec![]
            // self.iter_globals()
            //     .map(|v| v.global.value().as_f64().unwrap() as u128)
            //     .collect()
        }

        /// Set a global, at index idx. Will panic if idx is out of range
        /// Safety: the caller should check taht the raw value is compatible
        /// with destination VMGlobal type
        pub fn set_global_unchecked(&self, idx: usize, new_val: u128) {
            assert!(idx < self.globals.len());
            // let g = &self.globals[idx].global;
            // let cur_val = g.value().as_f64().unwrap();
            // let new_val = new_val as f64;
            // if cur_val != new_val {
            //     let new_value = JSValue::from(new_val);
            //     g.set_value(&new_value);
            // }
        }
    }

    /// Handle to an object managed by a context.
    ///
    /// Internally this is just an integer index into a context. A reference to the
    /// context must be passed in separately to access the actual object.
    pub struct StoreHandle<T> {
        id: StoreId,
        internal: InternalStoreHandle<T>,
    }

    impl<T> core::cmp::PartialEq for StoreHandle<T> {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }

    impl<T> std::hash::Hash for StoreHandle<T> {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.id.hash(state);
            self.internal.idx.hash(state);
        }
    }

    impl<T> Clone for StoreHandle<T> {
        fn clone(&self) -> Self {
            Self {
                id: self.id,
                internal: self.internal,
            }
        }
    }

    impl<T: StoreObject> fmt::Debug for StoreHandle<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("StoreHandle")
                .field("id", &self.id)
                .field("internal", &self.internal.index())
                .finish()
        }
    }

    impl<T: StoreObject> StoreHandle<T> {
        /// Moves the given object into a context and returns a handle to it.
        pub fn new(store: &mut StoreObjects, val: T) -> Self {
            Self {
                id: store.id,
                internal: InternalStoreHandle::new(store, val),
            }
        }

        /// Returns a reference to the object that this handle points to.
        pub fn get<'a>(&self, store: &'a StoreObjects) -> &'a T {
            assert_eq!(self.id, store.id, "object used with the wrong context");
            self.internal.get(store)
        }

        /// Returns a mutable reference to the object that this handle points to.
        pub fn get_mut<'a>(&self, store: &'a mut StoreObjects) -> &'a mut T {
            assert_eq!(self.id, store.id, "object used with the wrong context");
            self.internal.get_mut(store)
        }

        /// Returns the internal handle contains within this handle.
        pub fn internal_handle(&self) -> InternalStoreHandle<T> {
            self.internal
        }

        /// Returns the ID of the context associated with the handle.
        #[allow(unused)]
        pub fn store_id(&self) -> StoreId {
            self.id
        }

        /// Overrides the store id with a new ID
        #[allow(unused)]
        pub fn set_store_id(&mut self, id: StoreId) {
            self.id = id;
        }

        /// Constructs a `StoreHandle` from a `StoreId` and an `InternalStoreHandle`.
        ///
        /// # Safety
        /// Handling `InternalStoreHandle` values is unsafe because they do not track context ID.
        pub unsafe fn from_internal(id: StoreId, internal: InternalStoreHandle<T>) -> Self {
            Self { id, internal }
        }
    }

    /// Internal handle to an object owned by the current context.
    ///
    /// Unlike `StoreHandle` this does not track the context ID: it is only
    /// intended to be used within objects already owned by a context.
    #[repr(transparent)]
    pub struct InternalStoreHandle<T> {
        // Use a NonZero here to reduce the size of Option<InternalStoreHandle>.
        idx: NonZeroUsize,
        marker: PhantomData<fn() -> T>,
    }

    impl<T> Clone for InternalStoreHandle<T> {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<T> Copy for InternalStoreHandle<T> {}

    impl<T> fmt::Debug for InternalStoreHandle<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("InternalStoreHandle")
                .field("idx", &self.idx)
                .finish()
        }
    }
    impl<T> PartialEq for InternalStoreHandle<T> {
        fn eq(&self, other: &Self) -> bool {
            self.idx == other.idx
        }
    }
    impl<T> Eq for InternalStoreHandle<T> {}

    impl<T: StoreObject> InternalStoreHandle<T> {
        /// Moves the given object into a context and returns a handle to it.
        pub fn new(store: &mut StoreObjects, val: T) -> Self {
            let list = T::list_mut(store);
            let idx = NonZeroUsize::new(list.len() + 1).unwrap();
            list.push(val);
            Self {
                idx,
                marker: PhantomData,
            }
        }

        /// Returns a reference to the object that this handle points to.
        pub fn get<'a>(&self, store: &'a StoreObjects) -> &'a T {
            &T::list(store)[self.idx.get() - 1]
        }

        /// Returns a mutable reference to the object that this handle points to.
        pub fn get_mut<'a>(&self, store: &'a mut StoreObjects) -> &'a mut T {
            &mut T::list_mut(store)[self.idx.get() - 1]
        }

        pub(crate) fn index(&self) -> usize {
            self.idx.get()
        }

        pub(crate) fn from_index(idx: usize) -> Option<Self> {
            NonZeroUsize::new(idx).map(|idx| Self {
                idx,
                marker: PhantomData,
            })
        }
    }
}
