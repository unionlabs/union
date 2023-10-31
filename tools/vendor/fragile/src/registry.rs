pub struct Entry {
    /// The pointer to the object stored in the registry. This is a type-erased
    /// `Box<T>`.
    pub ptr: *mut (),
    /// The function that can be called on the above pointer to drop the object
    /// and free its allocation.
    pub drop: unsafe fn(*mut ()),
}

#[cfg(feature = "slab")]
mod slab_impl {
    use std::cell::UnsafeCell;
    use std::num::NonZeroUsize;

    use super::Entry;

    pub struct Registry(pub slab::Slab<Entry>);

    thread_local!(static REGISTRY: UnsafeCell<Registry> = UnsafeCell::new(Registry(slab::Slab::new())));

    pub use usize as ItemId;

    pub fn insert(thread_id: NonZeroUsize, entry: Entry) -> ItemId {
        let _ = thread_id;
        REGISTRY.with(|registry| unsafe { (*registry.get()).0.insert(entry) })
    }

    pub fn with<R, F: FnOnce(&Entry) -> R>(item_id: ItemId, thread_id: NonZeroUsize, f: F) -> R {
        let _ = thread_id;
        REGISTRY.with(|registry| f(unsafe { &*registry.get() }.0.get(item_id).unwrap()))
    }

    pub fn remove(item_id: ItemId, thread_id: NonZeroUsize) -> Entry {
        let _ = thread_id;
        REGISTRY.with(|registry| unsafe { (*registry.get()).0.remove(item_id) })
    }

    pub fn try_remove(item_id: ItemId, thread_id: NonZeroUsize) -> Option<Entry> {
        let _ = thread_id;
        REGISTRY.with(|registry| unsafe { (*registry.get()).0.try_remove(item_id) })
    }
}

#[cfg(not(feature = "slab"))]
mod map_impl {
    use std::cell::UnsafeCell;
    use std::num::NonZeroUsize;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::Entry;

    pub struct Registry(pub std::collections::HashMap<(NonZeroUsize, NonZeroUsize), Entry>);

    thread_local!(static REGISTRY: UnsafeCell<Registry> = UnsafeCell::new(Registry(Default::default())));

    pub type ItemId = NonZeroUsize;

    fn next_item_id() -> NonZeroUsize {
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        NonZeroUsize::new(COUNTER.fetch_add(1, Ordering::SeqCst))
            .expect("more than usize::MAX items")
    }

    pub fn insert(thread_id: NonZeroUsize, entry: Entry) -> ItemId {
        let item_id = next_item_id();
        REGISTRY
            .with(|registry| unsafe { (*registry.get()).0.insert((thread_id, item_id), entry) });
        item_id
    }

    pub fn with<R, F: FnOnce(&Entry) -> R>(item_id: ItemId, thread_id: NonZeroUsize, f: F) -> R {
        REGISTRY.with(|registry| {
            f(unsafe { &*registry.get() }
                .0
                .get(&(thread_id, item_id))
                .unwrap())
        })
    }

    pub fn remove(item_id: ItemId, thread_id: NonZeroUsize) -> Entry {
        REGISTRY
            .with(|registry| unsafe { (*registry.get()).0.remove(&(thread_id, item_id)).unwrap() })
    }

    pub fn try_remove(item_id: ItemId, thread_id: NonZeroUsize) -> Option<Entry> {
        REGISTRY.with(|registry| unsafe { (*registry.get()).0.remove(&(thread_id, item_id)) })
    }
}

#[cfg(feature = "slab")]
pub use self::slab_impl::*;

#[cfg(not(feature = "slab"))]
pub use self::map_impl::*;

impl Drop for Registry {
    fn drop(&mut self) {
        for (_, value) in self.0.iter() {
            // SAFETY: This function is only called once, and is called with the
            // pointer it was created with.
            unsafe { (value.drop)(value.ptr) };
        }
    }
}
