pub(crate) use imp::*;

#[cfg(not(feature = "std"))]
mod imp {
    use alloc::sync::Arc;
    use core::sync::atomic::{AtomicPtr, Ordering};
    use core::{mem, ptr};

    #[derive(Debug, Default)]
    pub(crate) struct LazyArc<T> {
        // Only written once with a value obtained from `Arc<T>::into_raw`.
        // This holds a ref count for the `Arc`, so it is always safe to
        // clone the `Arc` given a reference to the `LazyArc`.
        value: AtomicPtr<T>,
    }

    impl<T> Drop for LazyArc<T> {
        fn drop(&mut self) {
            let value_ptr = self.value.load(Ordering::Acquire);
            if !value_ptr.is_null() {
                // SAFETY: all writes to `self.value` are pointers obtained from `Arc::into_raw`.
                drop(unsafe { Arc::from_raw(value_ptr) });
            }
        }
    }

    impl<T> LazyArc<T> {
        pub(crate) fn get<E, F: FnOnce() -> Result<T, E>>(&self, f: F) -> Result<Arc<T>, E> {
            // Clone an `Arc` given a pointer obtained from `Arc::into_raw`.
            // SAFETY: `value_ptr` must be a valid pointer obtained from `Arc<T>::into_raw`.
            unsafe fn clone_arc_ptr<T>(value_ptr: *const T) -> Arc<T> {
                let value = Arc::from_raw(value_ptr);
                let clone = Arc::clone(&value);
                mem::forget(value);
                clone
            }

            // Return the existing value if already computed.
            // `Ordering::Acquire` is needed so that the content of the loaded `Arc` is
            // visible to this thread.
            let value_ptr = self.value.load(Ordering::Acquire);
            if !value_ptr.is_null() {
                // SAFETY: all writes to `self.value` are pointers obtained from `Arc::into_raw`.
                return Ok(unsafe { clone_arc_ptr(value_ptr) });
            }

            // Race to compute and set the value.
            let value = f().map(Arc::new)?;
            let value_ptr = Arc::into_raw(value);
            match self.value.compare_exchange(
                ptr::null_mut(),
                value_ptr as *mut T,
                // Success: `Ordering::Release` is needed so that the content of the stored `Arc`
                // is visible to other threads. No ordering is required for the null ptr that is
                // loaded, but older rust versions (< 1.64) require that its ordering must not
                // be weaker than the failure ordering, so we use `Ordering::AcqRel`.
                Ordering::AcqRel,
                // Failure: `Ordering::Acquire` is needed so that the content of the loaded `Arc`
                // is visible to this thread.
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    // Return the value we computed.
                    // SAFETY: `value_ptr` was obtained from `Arc::into_raw`.
                    Ok(unsafe { clone_arc_ptr(value_ptr) })
                }
                Err(existing_value_ptr) => {
                    // We lost the race, drop unneeded `value_ptr`.
                    // SAFETY: `value_ptr` was obtained from `Arc::into_raw`.
                    drop(unsafe { Arc::from_raw(value_ptr) });
                    // Return the existing value.
                    // SAFETY: all writes to `self.value` are pointers obtained from `Arc::into_raw`.
                    Ok(unsafe { clone_arc_ptr(existing_value_ptr) })
                }
            }
        }
    }
}

#[cfg(feature = "std")]
mod imp {
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Default)]
    pub(crate) struct LazyArc<T> {
        value: Mutex<Option<Arc<T>>>,
    }

    impl<T> LazyArc<T> {
        pub(crate) fn get<E, F: FnOnce() -> Result<T, E>>(&self, f: F) -> Result<Arc<T>, E> {
            let mut lock = self.value.lock().unwrap();
            if let Some(value) = &*lock {
                return Ok(value.clone());
            }
            let value = f().map(Arc::new)?;
            *lock = Some(value.clone());
            Ok(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lazy_arc() {
        let lazy = LazyArc::default();
        let value = lazy.get(|| Err(()));
        assert_eq!(value, Err(()));
        let value = lazy.get(|| Ok::<i32, ()>(3)).unwrap();
        assert_eq!(*value, 3);
        let value = lazy.get(|| Err(())).unwrap();
        assert_eq!(*value, 3);
    }
}
